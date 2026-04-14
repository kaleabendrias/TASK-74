//! Lodging management pages: list view, create/edit form with amenity checkboxes,
//! deposit cap warning, vacancy periods, and rent change requests.

use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::route_guard::RouteGuard;
use crate::context::{AuthContext, ToastAction, ToastContext};
use crate::models::*;
use crate::router::Route;
use crate::services::api;

const AMENITY_OPTIONS: &[&str] = &[
    "wifi", "parking", "pool", "gym", "air_conditioning", "heating",
    "kitchen", "laundry", "elevator", "wheelchair_accessible",
    "pet_friendly", "balcony", "garden", "security", "cctv",
    "reception_24h", "room_service", "restaurant", "bar", "spa",
];

// ── Lodging List ──

#[function_component(LodgingListPage)]
pub fn lodging_list_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let lodgings = use_state(|| Vec::<LodgingResponse>::new());
    let loading = use_state(|| true);

    {
        let lodgings = lodgings.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(list) = api::list_lodgings().await {
                    lodgings.set(list);
                }
                loading.set(false);
            });
            || {}
        });
    }

    let role = auth.user.as_ref().map(|u| &u.role);

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer]}>
        <>
        <div class="page-header">
            <h1>{ "Lodgings" }</h1>
            <div class="actions">
                { if matches!(role, Some(UserRole::Administrator) | Some(UserRole::Publisher)) {
                    html! {
                        <Link<Route> to={Route::LodgingNew} classes="btn btn-primary">
                            { "+ New Lodging" }
                        </Link<Route>>
                    }
                } else { html!{} }}
            </div>
        </div>

        <div class="card">
            <div class="table-wrapper">
                <table>
                    <thead>
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "State" }</th>
                            <th>{ "Rent" }</th>
                            <th>{ "Deposit" }</th>
                            <th>{ "Facility" }</th>
                            <th>{ "Updated" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { if *loading {
                            html! { <tr><td colspan="6" class="text-center text-secondary">{ "Loading..." }</td></tr> }
                        } else if lodgings.is_empty() {
                            html! { <tr><td colspan="6" class="text-center text-secondary">{ "No lodgings found" }</td></tr> }
                        } else {
                            html! { for lodgings.iter().map(|l| {
                                let badge = format!("badge badge-{}", l.state.replace('_', "-"));
                                let id = l.id.clone();
                                html! {
                                    <tr key={l.id.clone()}>
                                        <td>
                                            <Link<Route> to={Route::LodgingDetail { id }}>
                                                { &l.name }
                                            </Link<Route>>
                                        </td>
                                        <td><span class={badge}>{ &l.state }</span></td>
                                        <td>{ l.monthly_rent.map(|r| format!("${:.2}", r)).unwrap_or_else(|| "—".into()) }</td>
                                        <td>{ l.deposit_amount.map(|d| format!("${:.2}", d)).unwrap_or_else(|| "—".into()) }</td>
                                        <td class="text-secondary text-sm">{ l.facility_id.as_deref().unwrap_or("—") }</td>
                                        <td class="text-secondary text-sm">{ &l.updated_at }</td>
                                    </tr>
                                }
                            })}
                        }}
                    </tbody>
                </table>
            </div>
        </div>
        </>
        </RouteGuard>
    }
}

// ── Lodging Form ──

#[derive(Properties, PartialEq)]
pub struct LodgingFormProps {
    #[prop_or_default]
    pub id: Option<String>,
}

#[function_component(LodgingFormPage)]
pub fn lodging_form_page(props: &LodgingFormProps) -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let toasts = use_context::<ToastContext>().unwrap();
    let nav = use_navigator().unwrap();

    let name = use_state(String::new);
    let description = use_state(String::new);
    let amenities = use_state(|| Vec::<String>::new());
    let deposit = use_state(String::new);
    let rent = use_state(String::new);
    let state = use_state(|| "draft".to_string());
    let error = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);

    // Rent changes state
    let rent_changes = use_state(|| Vec::<RentChangeResponse>::new());
    let proposed_rent = use_state(String::new);
    let proposed_deposit = use_state(String::new);

    // Fetch pending rent changes
    {
        let rent_changes = rent_changes.clone();
        let id = props.id.clone();
        use_effect_with(id.clone(), move |id| {
            if let Some(_lid) = id.clone() {
                let rent_changes = rent_changes.clone();
                spawn_local(async move {
                    if let Ok(pending) = api::list_pending_rent_changes().await {
                        rent_changes.set(pending);
                    }
                });
            }
            || {}
        });
    }

    // Periods
    let periods = use_state(|| Vec::<LodgingPeriodResponse>::new());
    let period_start = use_state(String::new);
    let period_end = use_state(String::new);

    let is_edit = props.id.is_some();

    // Load existing lodging
    {
        let id = props.id.clone();
        let name = name.clone();
        let description = description.clone();
        let amenities = amenities.clone();
        let deposit = deposit.clone();
        let rent = rent.clone();
        let state = state.clone();
        let periods = periods.clone();
        use_effect_with(id.clone(), move |id| {
            if let Some(lid) = id.clone() {
                let lid2 = lid.clone();
                spawn_local(async move {
                    if let Ok(l) = api::get_lodging(&lid).await {
                        name.set(l.name);
                        description.set(l.description.unwrap_or_default());
                        let a: Vec<String> = l.amenities.as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.into())).collect())
                            .unwrap_or_default();
                        amenities.set(a);
                        deposit.set(l.deposit_amount.map(|d| format!("{:.2}", d)).unwrap_or_default());
                        rent.set(l.monthly_rent.map(|r| format!("{:.2}", r)).unwrap_or_default());
                        state.set(l.state);
                    }
                    if let Ok(p) = api::get_periods(&lid2).await {
                        periods.set(p);
                    }
                });
            }
            || {}
        });
    }

    let on_input = |setter: UseStateHandle<String>| {
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            setter.set(input.value());
        })
    };

    let toggle_amenity = {
        let amenities = amenities.clone();
        Callback::from(move |name: String| {
            let mut a = (*amenities).clone();
            if a.contains(&name) {
                a.retain(|x| x != &name);
            } else {
                a.push(name);
            }
            amenities.set(a);
        })
    };

    // Deposit cap warning
    let deposit_f: f64 = deposit.parse().unwrap_or(0.0);
    let rent_f: f64 = rent.parse().unwrap_or(0.0);
    let cap = rent_f * 1.5;
    let over_cap = rent_f > 0.0 && deposit_f > cap;

    // Submit
    let on_submit = {
        let props_id = props.id.clone();
        let name = name.clone();
        let description = description.clone();
        let amenities = amenities.clone();
        let deposit = deposit.clone();
        let rent = rent.clone();
        let error = error.clone();
        let loading = loading.clone();
        let toasts = toasts.clone();
        let nav = nav.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let props_id = props_id.clone();
            let name_v = (*name).clone();
            let desc_v = (*description).clone();
            let amen_v = (*amenities).clone();
            let dep_v = (*deposit).clone();
            let rent_v = (*rent).clone();
            let error = error.clone();
            let loading = loading.clone();
            let toasts = toasts.clone();
            let nav = nav.clone();

            loading.set(true);
            error.set(None);

            spawn_local(async move {
                let dep_f = dep_v.parse::<f64>().ok();
                let rent_f = rent_v.parse::<f64>().ok();

                let result = if let Some(lid) = props_id {
                    let req = UpdateLodgingRequest {
                        name: Some(name_v),
                        description: Some(desc_v),
                        amenities: Some(amen_v),
                        facility_id: None,
                        deposit_amount: dep_f,
                        monthly_rent: rent_f,
                        state: None,
                    };
                    api::update_lodging(&lid, &req).await
                } else {
                    let req = CreateLodgingRequest {
                        name: name_v,
                        description: if desc_v.is_empty() { None } else { Some(desc_v) },
                        amenities: amen_v,
                        facility_id: None,
                        deposit_amount: dep_f,
                        monthly_rent: rent_f,
                    };
                    api::create_lodging(&req).await
                };

                match result {
                    Ok(_) => {
                        toasts.dispatch(ToastAction::Add(ToastKind::Success, "Lodging saved".into()));
                        nav.push(&Route::LodgingList);
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        })
    };

    // Add period
    let on_add_period = {
        let props_id = props.id.clone();
        let period_start = period_start.clone();
        let period_end = period_end.clone();
        let periods = periods.clone();
        let toasts = toasts.clone();
        Callback::from(move |_: MouseEvent| {
            let lid = props_id.clone();
            let start = (*period_start).clone();
            let end = (*period_end).clone();
            let periods = periods.clone();
            let toasts = toasts.clone();
            if let Some(lid) = lid {
                if !start.is_empty() && !end.is_empty() {
                    spawn_local(async move {
                        let req = LodgingPeriodRequest {
                            start_date: start,
                            end_date: end,
                            min_nights: Some(7),
                            max_nights: Some(365),
                            vacancy: Some(true),
                        };
                        match api::upsert_period(&lid, &req).await {
                            Ok(p) => {
                                let mut ps = (*periods).clone();
                                ps.push(p);
                                periods.set(ps);
                                toasts.dispatch(ToastAction::Add(ToastKind::Success, "Period added".into()));
                            }
                            Err(e) => {
                                toasts.dispatch(ToastAction::Add(ToastKind::Error, e));
                            }
                        }
                    });
                }
            }
        })
    };

    // Request rent change
    let on_rent_change = {
        let props_id = props.id.clone();
        let proposed_rent = proposed_rent.clone();
        let proposed_deposit = proposed_deposit.clone();
        let toasts = toasts.clone();
        Callback::from(move |_: MouseEvent| {
            let lid = props_id.clone();
            let pr = (*proposed_rent).clone();
            let pd = (*proposed_deposit).clone();
            let toasts = toasts.clone();
            if let (Some(lid), Ok(r), Ok(d)) = (lid, pr.parse::<f64>(), pd.parse::<f64>()) {
                spawn_local(async move {
                    let req = RentChangeRequest { proposed_rent: r, proposed_deposit: d };
                    match api::request_rent_change(&lid, &req).await {
                        Ok(_) => toasts.dispatch(ToastAction::Add(ToastKind::Success, "Rent change requested".into())),
                        Err(e) => toasts.dispatch(ToastAction::Add(ToastKind::Error, e)),
                    }
                });
            }
        })
    };

    let role = auth.user.as_ref().map(|u| u.role.clone());

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer]}>
        <>
        <div class="page-header">
            <h1>{ if is_edit { "Edit Lodging" } else { "New Lodging" } }</h1>
        </div>

        { if let Some(ref e) = *error {
            html! { <div class="error-banner">{ e }</div> }
        } else { html!{} }}

        { if over_cap {
            html! {
                <div class="warning-banner" id="deposit-cap-warning">
                    { format!("Deposit (${:.2}) exceeds 1.5x monthly rent. Maximum allowed: ${:.2}", deposit_f, cap) }
                </div>
            }
        } else { html!{} }}

        <form onsubmit={on_submit}>
            <div class="card">
                <div class="card-header"><h2>{ "Details" }</h2></div>
                <div class="form-group">
                    <label for="lodging-name">{ "Name" }</label>
                    <input id="lodging-name" type="text" value={(*name).clone()}
                        oninput={on_input(name.clone())} />
                </div>
                <div class="form-group">
                    <label for="lodging-desc">{ "Description" }</label>
                    <textarea id="lodging-desc" rows="3" value={(*description).clone()}
                        oninput={on_input(description.clone())} />
                </div>
            </div>

            <div class="card">
                <div class="card-header"><h2>{ "Pricing" }</h2></div>
                <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
                    <div class="form-group">
                        <label for="lodging-rent">{ "Monthly Rent ($)" }</label>
                        <input id="lodging-rent" type="number" step="0.01" min="0"
                            value={(*rent).clone()} oninput={on_input(rent.clone())} />
                    </div>
                    <div class="form-group">
                        <label for="lodging-deposit">{ "Deposit ($)" }</label>
                        <input id="lodging-deposit" type="number" step="0.01" min="0"
                            class={if over_cap { "error" } else { "" }}
                            value={(*deposit).clone()} oninput={on_input(deposit.clone())} />
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="card-header"><h2>{ "Amenities" }</h2></div>
                <div class="checkbox-grid">
                    { for AMENITY_OPTIONS.iter().map(|&a| {
                        let checked = amenities.contains(&a.to_string());
                        let toggle = toggle_amenity.clone();
                        let name = a.to_string();
                        html! {
                            <label class="checkbox-item" key={a}>
                                <input
                                    id={format!("amenity-{}", a)}
                                    type="checkbox"
                                    checked={checked}
                                    onchange={Callback::from(move |_| toggle.emit(name.clone()))}
                                />
                                { a.replace('_', " ") }
                            </label>
                        }
                    })}
                </div>
            </div>

            <div style="display:flex;gap:12px;justify-content:flex-end;margin-top:16px;">
                <Link<Route> to={Route::LodgingList} classes="btn btn-secondary">{ "Cancel" }</Link<Route>>
                <button id="lodging-submit" type="submit" class="btn btn-primary" disabled={*loading}>
                    { if *loading { "Saving..." } else if is_edit { "Update" } else { "Create" } }
                </button>
            </div>
        </form>

        // Periods section (edit mode only)
        { if is_edit { html! {
            <div class="card mt-6">
                <div class="card-header"><h2>{ "Vacancy Periods" }</h2></div>
                <div style="display:grid;grid-template-columns:1fr 1fr auto;gap:12px;align-items:flex-end;margin-bottom:16px;">
                    <div class="form-group">
                        <label for="period-start">{ "Start Date" }</label>
                        <input id="period-start" type="date" value={(*period_start).clone()}
                            oninput={on_input(period_start.clone())} />
                    </div>
                    <div class="form-group">
                        <label for="period-end">{ "End Date" }</label>
                        <input id="period-end" type="date" value={(*period_end).clone()}
                            oninput={on_input(period_end.clone())} />
                    </div>
                    <button id="add-period-btn" type="button" class="btn btn-primary btn-sm"
                        onclick={on_add_period}>{ "Add Period" }</button>
                </div>
                <div class="text-secondary text-sm mb-4">{ "Min 7 nights, max 365 nights. Overlapping periods are rejected." }</div>
                <table>
                    <thead><tr><th>{ "Start" }</th><th>{ "End" }</th><th>{ "Min" }</th><th>{ "Max" }</th><th>{ "Vacancy" }</th></tr></thead>
                    <tbody>
                        { for periods.iter().map(|p| html! {
                            <tr key={p.id.clone()}>
                                <td>{ &p.start_date }</td>
                                <td>{ &p.end_date }</td>
                                <td>{ p.min_nights }</td>
                                <td>{ p.max_nights }</td>
                                <td>
                                    <span class={if p.vacancy { "badge badge-published" } else { "badge badge-offline" }}>
                                        { if p.vacancy { "Available" } else { "Occupied" } }
                                    </span>
                                </td>
                            </tr>
                        })}
                    </tbody>
                </table>
            </div>
        }} else { html!{} }}

        // Rent change section (edit mode only)
        { if is_edit { html! {
            <div class="card mt-4">
                <div class="card-header"><h2>{ "Rent Change Requests" }</h2></div>
                { if matches!(role, Some(UserRole::Administrator) | Some(UserRole::Publisher)) {
                    html! {
                        <div style="display:grid;grid-template-columns:1fr 1fr auto;gap:12px;align-items:flex-end;margin-bottom:16px;">
                            <div class="form-group">
                                <label for="proposed-rent">{ "Proposed Rent ($)" }</label>
                                <input id="proposed-rent" type="number" step="0.01" min="0"
                                    value={(*proposed_rent).clone()} oninput={on_input(proposed_rent.clone())} />
                            </div>
                            <div class="form-group">
                                <label for="proposed-deposit">{ "Proposed Deposit ($)" }</label>
                                <input id="proposed-deposit" type="number" step="0.01" min="0"
                                    value={(*proposed_deposit).clone()} oninput={on_input(proposed_deposit.clone())} />
                            </div>
                            <button id="request-rent-change" type="button" class="btn btn-primary btn-sm"
                                onclick={on_rent_change}>{ "Request Change" }</button>
                        </div>
                    }
                } else { html!{} }}
                { if rent_changes.is_empty() {
                    html! { <p class="text-secondary text-sm">{ "No pending rent change requests." }</p> }
                } else {
                    html! {
                        <table class="mt-4">
                            <thead>
                                <tr>
                                    <th>{ "Proposed Rent" }</th>
                                    <th>{ "Proposed Deposit" }</th>
                                    <th>{ "Status" }</th>
                                    <th>{ "Requested" }</th>
                                    <th>{ "Actions" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for rent_changes.iter().filter(|rc| {
                                    // Show changes for this lodging
                                    props.id.as_ref().map(|lid| rc.lodging_id == *lid).unwrap_or(false)
                                }).map(|rc| {
                                    let badge = format!("badge badge-{}", rc.status);
                                    let is_reviewer = matches!(role, Some(UserRole::Administrator) | Some(UserRole::Reviewer));
                                    let is_pending = rc.status == "pending";
                                    let lid = props.id.clone().unwrap_or_default();
                                    let cid = rc.id.clone();
                                    let lid2 = lid.clone();
                                    let cid2 = rc.id.clone();
                                    let toasts_a = toasts.clone();
                                    let toasts_r = toasts.clone();
                                    let rent_changes_a = rent_changes.clone();
                                    let rent_changes_r = rent_changes.clone();
                                    html! {
                                        <tr key={rc.id.clone()}>
                                            <td>{ format!("${:.2}", rc.proposed_rent) }</td>
                                            <td>{ format!("${:.2}", rc.proposed_deposit) }</td>
                                            <td><span class={badge}>{ &rc.status }</span></td>
                                            <td class="text-sm text-secondary">{ &rc.created_at }</td>
                                            <td>
                                                { if is_reviewer && is_pending {
                                                    html! {
                                                        <div class="flex gap-2">
                                                            <button class="btn btn-success btn-sm"
                                                                onclick={Callback::from(move |_: MouseEvent| {
                                                                    let lid = lid.clone();
                                                                    let cid = cid.clone();
                                                                    let toasts = toasts_a.clone();
                                                                    let rent_changes = rent_changes_a.clone();
                                                                    spawn_local(async move {
                                                                        match api::approve_rent_change(&lid, &cid).await {
                                                                            Ok(updated) => {
                                                                                let mut rcs = (*rent_changes).clone();
                                                                                if let Some(pos) = rcs.iter().position(|r| r.id == updated.id) {
                                                                                    rcs[pos] = updated;
                                                                                }
                                                                                rent_changes.set(rcs);
                                                                                toasts.dispatch(ToastAction::Add(ToastKind::Success, "Rent change approved".into()));
                                                                            }
                                                                            Err(e) => toasts.dispatch(ToastAction::Add(ToastKind::Error, e)),
                                                                        }
                                                                    });
                                                                })}>{ "Approve" }</button>
                                                            <button class="btn btn-danger btn-sm"
                                                                onclick={Callback::from(move |_: MouseEvent| {
                                                                    let lid = lid2.clone();
                                                                    let cid = cid2.clone();
                                                                    let toasts = toasts_r.clone();
                                                                    let rent_changes = rent_changes_r.clone();
                                                                    spawn_local(async move {
                                                                        match api::reject_rent_change(&lid, &cid).await {
                                                                            Ok(updated) => {
                                                                                let mut rcs = (*rent_changes).clone();
                                                                                if let Some(pos) = rcs.iter().position(|r| r.id == updated.id) {
                                                                                    rcs[pos] = updated;
                                                                                }
                                                                                rent_changes.set(rcs);
                                                                                toasts.dispatch(ToastAction::Add(ToastKind::Success, "Rent change rejected".into()));
                                                                            }
                                                                            Err(e) => toasts.dispatch(ToastAction::Add(ToastKind::Error, e)),
                                                                        }
                                                                    });
                                                                })}>{ "Reject" }</button>
                                                        </div>
                                                    }
                                                } else if !is_pending {
                                                    html! { <span class="text-sm text-secondary">{ &rc.status }</span> }
                                                } else {
                                                    html! { <span class="text-sm text-secondary">{ "Awaiting review" }</span> }
                                                }}
                                            </td>
                                        </tr>
                                    }
                                })}
                            </tbody>
                        </table>
                    }
                }}
            </div>
        }} else { html!{} }}
        </>
        </RouteGuard>
    }
}
