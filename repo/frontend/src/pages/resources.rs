//! Resource library pages: paginated list with search/filter, create/edit form with validation,
//! version history panel, and state transition buttons.

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::route_guard::RouteGuard;
use crate::context::{AuthContext, ToastAction, ToastContext};
use crate::models::*;
use crate::router::Route;
use crate::services::api;

// ── Resource List ──

#[function_component(ResourceListPage)]
pub fn resource_list_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let resources = use_state(|| Vec::<ResourceResponse>::new());
    let total = use_state(|| 0i64);
    let page = use_state(|| 1i64);
    let search = use_state(String::new);
    let filter_state = use_state(String::new);
    let filter_cat = use_state(String::new);
    let sort_by = use_state(|| "created_at".to_string());
    let loading = use_state(|| false);

    let per_page = 20i64;

    // Fetch resources
    {
        let resources = resources.clone();
        let total = total.clone();
        let loading = loading.clone();
        let page = page.clone();
        let search = search.clone();
        let filter_state = filter_state.clone();
        let filter_cat = filter_cat.clone();
        let sort_by = sort_by.clone();
        use_effect_with(
            ((*page), (*search).clone(), (*filter_state).clone(), (*filter_cat).clone(), (*sort_by).clone()),
            move |_| {
                loading.set(true);
                let resources = resources.clone();
                let total = total.clone();
                let loading = loading.clone();
                let pg = *page;
                let s = (*search).clone();
                let fs = (*filter_state).clone();
                let fc = (*filter_cat).clone();
                let sb = (*sort_by).clone();
                spawn_local(async move {
                    match api::list_resources(pg, per_page, &fs, &fc, &s, &sb).await {
                        Ok(resp) => {
                            total.set(resp.total);
                            resources.set(resp.data);
                        }
                        Err(_) => {}
                    }
                    loading.set(false);
                });
                || {}
            },
        );
    }

    let on_search = {
        let search = search.clone();
        let page = page.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search.set(input.value());
            page.set(1);
        })
    };

    let on_state_filter = {
        let filter_state = filter_state.clone();
        let page = page.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            filter_state.set(input.value());
            page.set(1);
        })
    };

    let total_pages = ((*total) as f64 / per_page as f64).ceil() as i64;
    let role = auth.user.as_ref().map(|u| &u.role);

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer, UserRole::Clinician]}>
        <>
        <div class="page-header">
            <h1>{ "Resources" }</h1>
            <div class="actions">
                { if matches!(role, Some(UserRole::Administrator) | Some(UserRole::Publisher)) {
                    html! {
                        <Link<Route> to={Route::ResourceNew} classes="btn btn-primary">
                            { "+ New Resource" }
                        </Link<Route>>
                    }
                } else { html!{} }}
            </div>
        </div>

        <div class="filter-bar">
            <div class="form-group">
                <label>{ "Search" }</label>
                <input id="resource-search" type="text" placeholder="Search title..."
                    value={(*search).clone()} oninput={on_search} />
            </div>
            <div class="form-group">
                <label>{ "State" }</label>
                <select id="resource-state-filter" onchange={on_state_filter}>
                    <option value="">{ "All states" }</option>
                    <option value="draft">{ "Draft" }</option>
                    <option value="in_review">{ "In Review" }</option>
                    <option value="published">{ "Published" }</option>
                    <option value="offline">{ "Offline" }</option>
                </select>
            </div>
        </div>

        <div class="card">
            <div class="table-wrapper">
                <table>
                    <thead>
                        <tr>
                            <th>{ "Title" }</th>
                            <th>{ "Category" }</th>
                            <th>{ "State" }</th>
                            <th>{ "Scheduled" }</th>
                            <th>{ "Updated" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { if *loading {
                            html! { <tr><td colspan="5" class="text-center text-secondary">{ "Loading..." }</td></tr> }
                        } else if resources.is_empty() {
                            html! { <tr><td colspan="5" class="text-center text-secondary">{ "No resources found" }</td></tr> }
                        } else {
                            html! { for resources.iter().map(|r| {
                                let badge_class = format!("badge badge-{}", r.state.replace('_', "-"));
                                let id = r.id.clone();
                                html! {
                                    <tr key={r.id.clone()}>
                                        <td>
                                            <Link<Route> to={Route::ResourceDetail { id }}>
                                                { &r.title }
                                            </Link<Route>>
                                        </td>
                                        <td>{ r.category.as_deref().unwrap_or("—") }</td>
                                        <td><span class={badge_class}>{ &r.state }</span></td>
                                        <td class="text-secondary text-sm">
                                            { r.scheduled_publish_at.as_deref().unwrap_or("—") }
                                        </td>
                                        <td class="text-secondary text-sm">{ &r.updated_at }</td>
                                    </tr>
                                }
                            })}
                        }}
                    </tbody>
                </table>
            </div>

            { if total_pages > 1 {
                let current = *page;
                let page_prev = page.clone();
                let page_next = page.clone();
                html! {
                    <div class="pagination">
                        <button disabled={current <= 1}
                            onclick={Callback::from(move |_| page_prev.set(current - 1))}>
                            { "Prev" }
                        </button>
                        <span class="pagination-info">
                            { format!("Page {} of {}", current, total_pages) }
                        </span>
                        <button disabled={current >= total_pages}
                            onclick={Callback::from(move |_| page_next.set(current + 1))}>
                            { "Next" }
                        </button>
                    </div>
                }
            } else { html!{} }}
        </div>
        </>
        </RouteGuard>
    }
}

// ── Resource Form (Create / Edit) ──

#[derive(Properties, PartialEq)]
pub struct ResourceFormProps {
    #[prop_or_default]
    pub id: Option<String>,
}

#[function_component(ResourceFormPage)]
pub fn resource_form_page(props: &ResourceFormProps) -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let toasts = use_context::<ToastContext>().unwrap();
    let nav = use_navigator().unwrap();

    let title = use_state(String::new);
    let category = use_state(String::new);
    let tags = use_state(|| Vec::<String>::new());
    let tag_input = use_state(String::new);
    let address = use_state(String::new);
    let latitude = use_state(String::new);
    let longitude = use_state(String::new);
    let hours = use_state(|| serde_json::json!({}));
    let pricing = use_state(|| serde_json::json!({}));
    let scheduled = use_state(String::new);
    let contact_info = use_state(String::new);
    let state = use_state(|| "draft".to_string());
    let current_version = use_state(|| 1i32);
    let media_previews = use_state(|| Vec::<(String, String, String)>::new()); // (preview_url, filename, media_id)
    let media_drag = use_state(|| false);
    let media_error = use_state(|| Option::<String>::None);
    let error = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);

    let is_edit = props.id.is_some();

    // Load existing resource
    {
        let id = props.id.clone();
        let title = title.clone();
        let category = category.clone();
        let tags = tags.clone();
        let address = address.clone();
        let latitude = latitude.clone();
        let longitude = longitude.clone();
        let scheduled = scheduled.clone();
        let state = state.clone();
        let current_version = current_version.clone();
        use_effect_with(id.clone(), move |id| {
            if let Some(rid) = id.clone() {
                spawn_local(async move {
                    if let Ok(r) = api::get_resource(&rid).await {
                        title.set(r.title);
                        category.set(r.category.unwrap_or_default());
                        let t: Vec<String> = r.tags.as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default();
                        tags.set(t);
                        address.set(r.address.unwrap_or_default());
                        latitude.set(r.latitude.map(|v| v.to_string()).unwrap_or_default());
                        longitude.set(r.longitude.map(|v| v.to_string()).unwrap_or_default());
                        scheduled.set(r.scheduled_publish_at.unwrap_or_default());
                        state.set(r.state);
                        current_version.set(r.current_version);
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

    // Tag chip handling
    let on_tag_key = {
        let tag_input = tag_input.clone();
        let tags = tags.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                let val = (*tag_input).trim().to_string();
                if !val.is_empty() && tags.len() < 20 && !tags.contains(&val) {
                    let mut t = (*tags).clone();
                    t.push(val);
                    tags.set(t);
                    tag_input.set(String::new());
                }
            }
        })
    };

    let remove_tag = {
        let tags = tags.clone();
        Callback::from(move |idx: usize| {
            let mut t = (*tags).clone();
            t.remove(idx);
            tags.set(t);
        })
    };

    // Media file validation + upload
    let validate_media_file = |file: &web_sys::File| -> Result<(), String> {
        let name = file.name().to_lowercase();
        let valid_ext = name.ends_with(".jpg") || name.ends_with(".jpeg")
            || name.ends_with(".png") || name.ends_with(".mp4");
        if !valid_ext {
            return Err("Only JPG, PNG, and MP4 files are allowed".into());
        }
        if file.size() > 52_428_800.0 {
            return Err("File exceeds 50 MB limit".into());
        }
        Ok(())
    };

    let upload_media_file = {
        let media_previews = media_previews.clone();
        let media_error = media_error.clone();
        let toasts = toasts.clone();
        move |file: web_sys::File| {
            let media_previews = media_previews.clone();
            let media_error = media_error.clone();
            let toasts = toasts.clone();
            let name = file.name();
            let preview_url = web_sys::Url::create_object_url_with_blob(&file).unwrap_or_default();
            spawn_local(async move {
                match crate::services::api::upload_media(file).await {
                    Ok(resp) => {
                        let mut items = (*media_previews).clone();
                        items.push((preview_url, name, resp.id.to_string()));
                        media_previews.set(items);
                        media_error.set(None);
                    }
                    Err(e) => {
                        media_error.set(Some(e.clone()));
                        toasts.dispatch(crate::context::ToastAction::Add(
                            crate::models::ToastKind::Error, e));
                    }
                }
            });
        }
    };

    let on_media_select = {
        let upload = upload_media_file.clone();
        let media_error = media_error.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                for i in 0..files.length() {
                    if let Some(file) = files.get(i) {
                        match validate_media_file(&file) {
                            Ok(()) => upload(file),
                            Err(msg) => media_error.set(Some(msg)),
                        }
                    }
                }
            }
        })
    };

    let on_media_drop = {
        let upload = upload_media_file.clone();
        let media_error = media_error.clone();
        let media_drag = media_drag.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            media_drag.set(false);
            if let Some(dt) = e.data_transfer() {
                if let Some(files) = dt.files() {
                    for i in 0..files.length() {
                        if let Some(file) = files.get(i) {
                            match validate_media_file(&file) {
                                Ok(()) => upload(file),
                                Err(msg) => media_error.set(Some(msg)),
                            }
                        }
                    }
                }
            }
        })
    };

    let remove_media = {
        let media_previews = media_previews.clone();
        Callback::from(move |idx: usize| {
            let mut items = (*media_previews).clone();
            // Revoke the preview URL
            if let Some((url, _, _)) = items.get(idx) {
                web_sys::Url::revoke_object_url(url).ok();
            }
            items.remove(idx);
            media_previews.set(items);
        })
    };

    // Submit
    let on_submit = {
        let props_id = props.id.clone();
        let title = title.clone();
        let category = category.clone();
        let tags = tags.clone();
        let address = address.clone();
        let latitude = latitude.clone();
        let longitude = longitude.clone();
        let hours = hours.clone();
        let pricing = pricing.clone();
        let scheduled = scheduled.clone();
        let contact_info = contact_info.clone();
        let state_h = state.clone();
        let error = error.clone();
        let loading = loading.clone();
        let toasts = toasts.clone();
        let nav = nav.clone();
        let media_previews_c = media_previews.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let props_id = props_id.clone();
            let title_v = (*title).clone();
            let category_v = (*category).clone();
            let tags_v = (*tags).clone();
            let address_v = (*address).clone();
            let lat = (*latitude).clone();
            let lng = (*longitude).clone();
            let hours_v = (*hours).clone();
            let pricing_v = (*pricing).clone();
            let sched_v = (*scheduled).clone();
            let contact_info_v = (*contact_info).clone();
            let error = error.clone();
            let loading = loading.clone();
            let toasts = toasts.clone();
            let nav = nav.clone();
            let media_previews_c = media_previews_c.clone();

            loading.set(true);
            error.set(None);

            spawn_local(async move {
                let lat_f = lat.parse::<f64>().ok();
                let lng_f = lng.parse::<f64>().ok();
                let sched_opt = if sched_v.is_empty() {
                    None
                } else if sched_v.len() == 16 {
                    // datetime-local gives "YYYY-MM-DDTHH:MM", backend needs seconds
                    Some(format!("{}:00", sched_v))
                } else {
                    Some(sched_v)
                };

                // Capture the browser's local UTC offset so the backend can correctly
                // interpret the naive datetime entered in the scheduler field.
                // JS getTimezoneOffset() returns minutes WEST of UTC (opposite sign),
                // so we negate: UTC+5 → JS returns -300 → we send +300.
                let tz_offset = sched_opt.as_ref().map(|_| {
                    let js_offset = js_sys::Date::new_0().get_timezone_offset() as i32;
                    -js_offset
                });

                let media_ids: Vec<String> = (*media_previews_c).iter().map(|(_, _, id)| id.clone()).collect();

                let contact_opt = if contact_info_v.is_empty() { None } else { Some(contact_info_v) };

                let result = if let Some(rid) = props_id {
                    let req = UpdateResourceRequest {
                        title: Some(title_v),
                        category: Some(category_v),
                        tags: Some(tags_v),
                        hours: Some(hours_v),
                        pricing: Some(pricing_v),
                        address: Some(address_v),
                        latitude: lat_f,
                        longitude: lng_f,
                        media_refs: Some(media_ids.clone()),
                        state: None,
                        scheduled_publish_at: sched_opt,
                        tz_offset_minutes: tz_offset,
                        contact_info: contact_opt,
                    };
                    api::update_resource(&rid, &req).await
                } else {
                    let req = CreateResourceRequest {
                        title: title_v,
                        category: if category_v.is_empty() { None } else { Some(category_v) },
                        tags: tags_v,
                        hours: hours_v,
                        pricing: pricing_v,
                        address: address_v,
                        latitude: lat_f,
                        longitude: lng_f,
                        media_refs: media_ids.clone(),
                        scheduled_publish_at: sched_opt,
                        tz_offset_minutes: tz_offset,
                        contact_info: contact_opt,
                    };
                    api::create_resource(&req).await
                };

                match result {
                    Ok(_) => {
                        toasts.dispatch(ToastAction::Add(ToastKind::Success, "Resource saved".into()));
                        nav.push(&Route::ResourceList);
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        })
    };

    // State transition
    let on_state_change = {
        let props_id = props.id.clone();
        let toasts = toasts.clone();
        let state_h = state.clone();
        let nav = nav.clone();
        move |new_state: String| {
            let pid = props_id.clone();
            let toasts = toasts.clone();
            let state_h = state_h.clone();
            let nav = nav.clone();
            Callback::from(move |_: MouseEvent| {
                let pid = pid.clone();
                let new_st = new_state.clone();
                let toasts = toasts.clone();
                let state_h = state_h.clone();
                let nav = nav.clone();
                if let Some(rid) = pid {
                    spawn_local(async move {
                        let req = UpdateResourceRequest {
                            title: None, category: None, tags: None, hours: None,
                            pricing: None, address: None, latitude: None, longitude: None,
                            media_refs: None, state: Some(new_st.clone()),
                            scheduled_publish_at: None,
                            tz_offset_minutes: None,
                            contact_info: None,
                        };
                        match api::update_resource(&rid, &req).await {
                            Ok(_) => {
                                state_h.set(new_st);
                                toasts.dispatch(ToastAction::Add(ToastKind::Success, "State updated".into()));
                            }
                            Err(e) => {
                                toasts.dispatch(ToastAction::Add(ToastKind::Error, e));
                            }
                        }
                    });
                }
            })
        }
    };

    let title_len = title.len();
    let role = auth.user.as_ref().map(|u| u.role.clone());
    let cur_state = &*state;

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer, UserRole::Clinician]}>
        <>
        <div class="page-header">
            <h1>{ if is_edit { "Edit Resource" } else { "New Resource" } }</h1>
            <div class="actions">
                { if is_edit { html! {
                    <>
                    { if cur_state == "draft" && matches!(role, Some(UserRole::Publisher) | Some(UserRole::Administrator)) {
                        html! { <button id="btn-submit-review" class="btn btn-primary btn-sm"
                            onclick={on_state_change("in_review".into())}>{ "Submit for Review" }</button> }
                    } else { html!{} }}
                    { if cur_state == "in_review" && matches!(role, Some(UserRole::Reviewer) | Some(UserRole::Administrator)) {
                        html! { <button id="btn-publish" class="btn btn-success btn-sm"
                            onclick={on_state_change("published".into())}>{ "Publish" }</button> }
                    } else { html!{} }}
                    { if cur_state == "published" && matches!(role, Some(UserRole::Publisher) | Some(UserRole::Administrator)) {
                        html! { <button id="btn-take-offline" class="btn btn-danger btn-sm"
                            onclick={on_state_change("offline".into())}>{ "Take Offline" }</button> }
                    } else { html!{} }}
                    { if cur_state == "offline" && matches!(role, Some(UserRole::Publisher) | Some(UserRole::Administrator)) {
                        html! { <button id="btn-to-draft" class="btn btn-secondary btn-sm"
                            onclick={on_state_change("draft".into())}>{ "Return to Draft" }</button> }
                    } else { html!{} }}
                    { if let Some(ref rid) = props.id {
                        html! { <Link<Route> to={Route::ResourceHistory { id: rid.clone() }}
                            classes="btn btn-secondary btn-sm">{ "Version History" }</Link<Route>> }
                    } else { html!{} }}
                    </>
                }} else { html!{} }}
            </div>
        </div>

        { if let Some(ref e) = *error {
            html! { <div class="error-banner">{ e }</div> }
        } else { html!{} }}

        { if is_edit {
            let badge_class = format!("badge badge-{}", cur_state.replace('_', "-"));
            html! {
                <div class="mb-4">
                    <span class={badge_class}>{ cur_state }</span>
                    <span class="text-secondary text-sm" style="margin-left:8px">
                        { format!("v{}", *current_version) }
                    </span>
                </div>
            }
        } else { html!{} }}

        <form onsubmit={on_submit}>
            <div class="card">
                <div class="card-header"><h2>{ "Details" }</h2></div>

                <div class="form-group">
                    <label for="res-title">{ "Title" }</label>
                    <input id="res-title" type="text" value={(*title).clone()}
                        oninput={on_input(title.clone())} maxlength="200"
                        class={if title_len > 200 { "error" } else { "" }} />
                    <div class={if title_len > 200 { "char-counter over" } else { "char-counter" }}>
                        { format!("{}/200", title_len) }
                    </div>
                </div>

                <div class="form-group">
                    <label for="res-category">{ "Category" }</label>
                    <select id="res-category" onchange={
                        let category = category.clone();
                        Callback::from(move |e: Event| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            category.set(input.value());
                        })
                    }>
                        <option value="">{ "Select category" }</option>
                        <option value="attraction">{ "Attraction" }</option>
                        <option value="restaurant">{ "Restaurant" }</option>
                        <option value="hotel">{ "Hotel" }</option>
                        <option value="activity">{ "Activity" }</option>
                        <option value="transportation">{ "Transportation" }</option>
                        <option value="shopping">{ "Shopping" }</option>
                        <option value="service">{ "Service" }</option>
                    </select>
                </div>

                <div class="form-group">
                    <label>{ "Tags" }</label>
                    <div class="chip-input-container">
                        { for tags.iter().enumerate().map(|(i, tag)| {
                            let remove = remove_tag.clone();
                            html! {
                                <span class="chip" key={tag.clone()}>
                                    { tag }
                                    <button type="button"
                                        onclick={Callback::from(move |_: MouseEvent| remove.emit(i))}>
                                        { "\u{2715}" }
                                    </button>
                                </span>
                            }
                        })}
                        <input id="res-tag-input" type="text" placeholder="Type and press Enter"
                            value={(*tag_input).clone()}
                            oninput={on_input(tag_input.clone())}
                            onkeydown={on_tag_key} />
                    </div>
                    <div class={if tags.len() > 20 { "char-counter over" } else { "char-counter" }}>
                        { format!("{}/20 tags", tags.len()) }
                    </div>
                </div>

                <div class="form-group">
                    <label for="res-contact-info">{ "Contact Info " }<span class="text-secondary text-sm">{ "(encrypted at rest)" }</span></label>
                    <textarea id="res-contact-info" rows="2"
                        placeholder="Phone, email, or other contact details (stored encrypted)"
                        value={(*contact_info).clone()}
                        oninput={{
                            let ci = contact_info.clone();
                            Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                ci.set(input.value());
                            })
                        }} />
                    <div class="text-secondary text-sm mt-1">
                        { "Leave blank when editing to keep the existing value unchanged." }
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="card-header"><h2>{ "Location" }</h2></div>
                <div class="form-group">
                    <label for="res-address">{ "Address" }</label>
                    <input id="res-address" type="text" value={(*address).clone()}
                        oninput={on_input(address.clone())} placeholder="123 Main St, City, ST 12345" />
                </div>
                <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
                    <div class="form-group">
                        <label for="res-lat">{ "Latitude" }</label>
                        <input id="res-lat" type="number" step="any" min="-90" max="90"
                            value={(*latitude).clone()} oninput={on_input(latitude.clone())}
                            placeholder="-90 to 90" />
                    </div>
                    <div class="form-group">
                        <label for="res-lng">{ "Longitude" }</label>
                        <input id="res-lng" type="number" step="any" min="-180" max="180"
                            value={(*longitude).clone()} oninput={on_input(longitude.clone())}
                            placeholder="-180 to 180" />
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="card-header"><h2>{ "Scheduling" }</h2></div>
                <div class="form-group">
                    <label for="res-scheduled">{ "Scheduled Publish Date" }</label>
                    <input id="res-scheduled" type="datetime-local"
                        value={(*scheduled).clone()} oninput={on_input(scheduled.clone())} />
                </div>
            </div>

            <div class="card">
                <div class="card-header"><h2>{ "Media Attachments" }</h2></div>
                <div class={if *media_drag { "upload-area dragover" } else { "upload-area" }}
                    id="res-media-upload"
                    ondragover={{
                        let d = media_drag.clone();
                        Callback::from(move |e: DragEvent| { e.prevent_default(); d.set(true); })
                    }}
                    ondragleave={{
                        let d = media_drag.clone();
                        Callback::from(move |_: DragEvent| d.set(false))
                    }}
                    ondrop={on_media_drop.clone()}
                    onclick={{
                        Callback::from(move |_: MouseEvent| {
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(el) = doc.get_element_by_id("res-media-file-input") {
                                    let input: web_sys::HtmlInputElement = el.unchecked_into();
                                    input.click();
                                }
                            }
                        })
                    }}
                >
                    <p><strong>{ "Drop files here or click to browse" }</strong></p>
                    <p>{ "JPG, PNG, MP4 — max 50 MB per file" }</p>
                </div>
                <input id="res-media-file-input" type="file" accept=".jpg,.jpeg,.png,.mp4"
                    multiple=true
                    style="display:none"
                    onchange={on_media_select.clone()} />
                { if !media_previews.is_empty() {
                    html! {
                        <div class="upload-preview mt-4">
                            { for media_previews.iter().enumerate().map(|(i, (url, name, _id))| {
                                let remove = remove_media.clone();
                                let is_video = name.ends_with(".mp4");
                                html! {
                                    <div class="upload-preview-item" key={name.clone()}>
                                        { if is_video {
                                            html! { <video src={url.clone()} /> }
                                        } else {
                                            html! { <img src={url.clone()} alt={name.clone()} /> }
                                        }}
                                        <button type="button" class="remove-btn"
                                            onclick={Callback::from(move |_: MouseEvent| remove.emit(i))}>
                                            { "\u{2715}" }
                                        </button>
                                        <div class="text-sm text-center" style="padding:2px;font-size:0.65rem;overflow:hidden;">
                                            { name }
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    }
                } else { html!{} }}
                { if let Some(ref e) = *media_error {
                    html! { <div class="field-error mt-2">{ e }</div> }
                } else { html!{} }}
            </div>

            <div style="display:flex;gap:12px;justify-content:flex-end;margin-top:16px;">
                <Link<Route> to={Route::ResourceList} classes="btn btn-secondary">
                    { "Cancel" }
                </Link<Route>>
                <button id="res-submit" type="submit" class="btn btn-primary" disabled={*loading}>
                    { if *loading { "Saving..." } else if is_edit { "Update" } else { "Create" } }
                </button>
            </div>
        </form>
        </>
        </RouteGuard>
    }
}

// ── Resource History ──

#[derive(Properties, PartialEq)]
pub struct HistoryProps {
    pub id: String,
}

#[function_component(ResourceHistoryPage)]
pub fn resource_history_page(props: &HistoryProps) -> Html {
    let versions = use_state(|| Vec::<crate::models::ResourceVersionResponse>::new());
    let loading = use_state(|| true);

    {
        let id = props.id.clone();
        let versions = versions.clone();
        let loading = loading.clone();
        use_effect_with(id.clone(), move |id| {
            let id = id.clone();
            let versions = versions.clone();
            let loading = loading.clone();
            spawn_local(async move {
                if let Ok(v) = api::list_resource_versions(&id).await {
                    versions.set(v);
                }
                loading.set(false);
            });
            || {}
        });
    }

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::Publisher, UserRole::Reviewer, UserRole::Clinician]}>
        <>
        <div class="page-header">
            <h1>{ "Version History" }</h1>
            <div class="actions">
                <Link<Route> to={Route::ResourceDetail { id: props.id.clone() }}
                    classes="btn btn-secondary btn-sm">{ "Back to Resource" }</Link<Route>>
            </div>
        </div>
        <div class="card">
            { if *loading {
                html! { <p class="text-secondary">{ "Loading version history..." }</p> }
            } else if versions.is_empty() {
                html! { <p class="text-secondary">{ "No version history yet." }</p> }
            } else {
                html! {
                    <div class="mt-4">
                        { for versions.iter().map(|v| {
                            let snapshot = serde_json::to_string_pretty(&v.snapshot).unwrap_or_default();
                            html! {
                                <div key={v.id.clone()} style="border-left:3px solid var(--color-primary);padding-left:16px;margin-bottom:16px;">
                                    <div class="flex items-center gap-3 mb-2">
                                        <strong>{ format!("v{}", v.version_number) }</strong>
                                        <span class="text-sm text-secondary">{ &v.created_at }</span>
                                        <span class="text-sm text-secondary">{ format!("by {}", v.changed_by) }</span>
                                    </div>
                                    <pre class="text-sm" style="background:var(--color-bg);padding:8px;border-radius:4px;overflow-x:auto;max-height:300px;">
                                        { snapshot }
                                    </pre>
                                </div>
                            }
                        })}
                    </div>
                }
            }}
        </div>
        </>
        </RouteGuard>
    }
}
