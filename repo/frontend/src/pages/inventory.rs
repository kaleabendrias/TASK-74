use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::route_guard::RouteGuard;
use crate::context::{AuthContext, ToastAction, ToastContext};
use crate::models::*;
use crate::router::Route;
use crate::services::api;

// ── Inventory Dashboard ──

#[function_component(InventoryPage)]
pub fn inventory_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let toasts = use_context::<ToastContext>().unwrap();
    let lots = use_state(|| Vec::<LotResponse>::new());
    let loading = use_state(|| true);
    let show_near_expiry = use_state(|| false);

    // Reserve modal
    let reserve_lot_id = use_state(|| Option::<String>::None);
    let reserve_qty = use_state(|| "1".to_string());

    {
        let lots = lots.clone();
        let loading = loading.clone();
        let near = *show_near_expiry;
        let facility = auth.user.as_ref().and_then(|u| u.facility_id.clone());
        use_effect_with(near, move |_| {
            let lots = lots.clone();
            let loading = loading.clone();
            spawn_local(async move {
                loading.set(true);
                if let Ok(list) = api::list_lots(facility.as_deref(), near).await {
                    lots.set(list);
                }
                loading.set(false);
            });
            || {}
        });
    }

    let toggle_expiry = {
        let show_near_expiry = show_near_expiry.clone();
        Callback::from(move |_: MouseEvent| show_near_expiry.set(!*show_near_expiry))
    };

    let open_reserve = {
        let reserve_lot_id = reserve_lot_id.clone();
        Callback::from(move |id: String| reserve_lot_id.set(Some(id)))
    };

    let close_reserve = {
        let reserve_lot_id = reserve_lot_id.clone();
        Callback::from(move |_: MouseEvent| reserve_lot_id.set(None))
    };

    let on_reserve = {
        let reserve_lot_id = reserve_lot_id.clone();
        let reserve_qty = reserve_qty.clone();
        let lots = lots.clone();
        let toasts = toasts.clone();
        Callback::from(move |_: MouseEvent| {
            let lid = (*reserve_lot_id).clone();
            let qty_s = (*reserve_qty).clone();
            let lots = lots.clone();
            let toasts = toasts.clone();
            let reserve_lot_id = reserve_lot_id.clone();
            if let (Some(lid), Ok(qty)) = (lid, qty_s.parse::<i32>()) {
                spawn_local(async move {
                    let req = ReserveRequest { quantity: qty };
                    match api::reserve_lot(&lid, &req).await {
                        Ok(updated) => {
                            let mut l = (*lots).clone();
                            if let Some(pos) = l.iter().position(|x| x.id == updated.id) {
                                l[pos] = updated;
                            }
                            lots.set(l);
                            reserve_lot_id.set(None);
                            toasts.dispatch(ToastAction::Add(ToastKind::Success, "Stock reserved".into()));
                        }
                        Err(e) => toasts.dispatch(ToastAction::Add(ToastKind::Error, e)),
                    }
                });
            }
        })
    };

    // For reserve modal: compute remaining stock
    let reserve_lot = reserve_lot_id.as_ref().and_then(|id| lots.iter().find(|l| &l.id == id));
    let reserve_qty_val: i32 = reserve_qty.parse().unwrap_or(0);
    let remaining = reserve_lot.map(|l| l.quantity_on_hand - reserve_qty_val).unwrap_or(0);

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::Clinician, UserRole::InventoryClerk]}>
        <>
        <div class="page-header">
            <h1>{ "Inventory" }</h1>
            <div class="actions">
                <button id="toggle-near-expiry" class={if *show_near_expiry { "btn btn-primary btn-sm" } else { "btn btn-secondary btn-sm" }}
                    onclick={toggle_expiry}>
                    { if *show_near_expiry { "Show All" } else { "Near Expiry Only" } }
                </button>
            </div>
        </div>

        <div class="card">
            <div class="table-wrapper">
                <table>
                    <thead>
                        <tr>
                            <th>{ "Item" }</th>
                            <th>{ "Lot #" }</th>
                            <th>{ "Warehouse" }</th>
                            <th>{ "Bin" }</th>
                            <th>{ "On Hand" }</th>
                            <th>{ "Reserved" }</th>
                            <th>{ "Expiration" }</th>
                            <th>{ "Actions" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { if *loading {
                            html! { <tr><td colspan="8" class="text-center text-secondary">{ "Loading..." }</td></tr> }
                        } else if lots.is_empty() {
                            html! { <tr><td colspan="8" class="text-center text-secondary">{ "No lots found" }</td></tr> }
                        } else {
                            html! { for lots.iter().map(|l| {
                                let row_class = if l.near_expiry { "near-expiry" } else { "" };
                                let id = l.id.clone();
                                let open = open_reserve.clone();
                                html! {
                                    <tr key={l.id.clone()} class={row_class}>
                                        <td>{ &l.item_name }</td>
                                        <td class="text-sm">{ &l.lot_number }</td>
                                        <td class="text-sm text-secondary">{ &l.warehouse_id }</td>
                                        <td class="text-sm text-secondary">{ &l.bin_id }</td>
                                        <td>{ l.quantity_on_hand }</td>
                                        <td>{ l.quantity_reserved }</td>
                                        <td>
                                            { if let Some(ref d) = l.expiration_date {
                                                html! { <span class={if l.near_expiry { "badge badge-in-review" } else { "" }}>{ d }</span> }
                                            } else {
                                                html! { "—" }
                                            }}
                                        </td>
                                        <td>
                                            <button id={format!("reserve-{}", l.id)} class="btn btn-sm btn-primary"
                                                onclick={Callback::from(move |_: MouseEvent| open.emit(id.clone()))}>
                                                { "Reserve" }
                                            </button>
                                        </td>
                                    </tr>
                                }
                            })}
                        }}
                    </tbody>
                </table>
            </div>
        </div>

        // Reserve modal
        { if reserve_lot_id.is_some() {
            html! {
                <div class="modal-overlay">
                    <div class="modal">
                        <div class="modal-header">
                            <h2>{ "Reserve Stock" }</h2>
                            <button class="modal-close" onclick={close_reserve}>{ "\u{2715}" }</button>
                        </div>
                        { if let Some(lot) = reserve_lot {
                            html! {
                                <>
                                <p class="mb-4">
                                    { format!("{} — Lot {}", lot.item_name, lot.lot_number) }
                                </p>
                                <p class="text-sm text-secondary mb-2">
                                    { format!("Available: {}", lot.quantity_on_hand) }
                                </p>
                                <div class="form-group">
                                    <label for="reserve-quantity">{ "Quantity to Reserve" }</label>
                                    <input id="reserve-quantity" type="number" min="1"
                                        max={lot.quantity_on_hand.to_string()}
                                        value={(*reserve_qty).clone()}
                                        oninput={{
                                            let reserve_qty = reserve_qty.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                reserve_qty.set(input.value());
                                            })
                                        }} />
                                </div>
                                <p class={if remaining < 0 { "text-sm" } else { "text-sm text-secondary" }}
                                    style={if remaining < 0 { "color:var(--color-error)" } else { "" }}>
                                    { format!("Remaining after reservation: {}", remaining) }
                                </p>
                                <div class="modal-footer">
                                    <button class="btn btn-secondary" onclick={close_reserve.clone()}>{ "Cancel" }</button>
                                    <button id="confirm-reserve" class="btn btn-primary"
                                        disabled={reserve_qty_val <= 0 || remaining < 0}
                                        onclick={on_reserve}>
                                        { "Confirm Reserve" }
                                    </button>
                                </div>
                                </>
                            }
                        } else { html!{} }}
                    </div>
                </div>
            }
        } else { html!{} }}
        </>
        </RouteGuard>
    }
}

// ── Transactions Page ──

#[function_component(TransactionsPage)]
pub fn transactions_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let transactions = use_state(|| Vec::<TransactionResponse>::new());
    let loading = use_state(|| true);
    let filter_lot = use_state(String::new);
    let filter_dir = use_state(String::new);
    let filter_from = use_state(String::new);
    let filter_to = use_state(String::new);

    {
        let transactions = transactions.clone();
        let loading = loading.clone();
        let lot = (*filter_lot).clone();
        let dir = (*filter_dir).clone();
        let from = (*filter_from).clone();
        let to = (*filter_to).clone();
        use_effect_with((lot.clone(), dir.clone(), from.clone(), to.clone()), move |_| {
            let transactions = transactions.clone();
            let loading = loading.clone();
            spawn_local(async move {
                loading.set(true);
                let lot_opt = if lot.is_empty() { None } else { Some(lot.as_str()) };
                let dir_opt = if dir.is_empty() { None } else { Some(dir.as_str()) };
                let from_opt = if from.is_empty() { None } else { Some(from.as_str()) };
                let to_opt = if to.is_empty() { None } else { Some(to.as_str()) };
                if let Ok(list) = api::list_transactions(lot_opt, dir_opt, from_opt, to_opt).await {
                    transactions.set(list);
                }
                loading.set(false);
            });
            || {}
        });
    }

    let on_input = |setter: UseStateHandle<String>| {
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            setter.set(input.value());
        })
    };

    let on_print = {
        let filter_lot = filter_lot.clone();
        Callback::from(move |_: MouseEvent| {
            let lot = (*filter_lot).clone();
            if !lot.is_empty() {
                let url = api::audit_print_url(&lot);
                let _ = web_sys::window().unwrap().open_with_url_and_target(&url, "_blank");
            }
        })
    };

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::Clinician, UserRole::InventoryClerk]}>
        <>
        <div class="page-header">
            <h1>{ "Transactions" }</h1>
            <div class="actions">
                <button id="print-audit-btn" class="btn btn-secondary btn-sm"
                    disabled={filter_lot.is_empty()}
                    onclick={on_print}>
                    { "Print Audit View" }
                </button>
            </div>
        </div>

        <div class="filter-bar">
            <div class="form-group">
                <label>{ "Lot ID" }</label>
                <input id="txn-filter-lot" type="text" placeholder="Filter by lot ID"
                    value={(*filter_lot).clone()} oninput={on_input(filter_lot.clone())} />
            </div>
            <div class="form-group">
                <label>{ "Direction" }</label>
                <select id="txn-filter-dir" onchange={{
                    let filter_dir = filter_dir.clone();
                    Callback::from(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        filter_dir.set(input.value());
                    })
                }}>
                    <option value="">{ "All" }</option>
                    <option value="inbound">{ "Inbound" }</option>
                    <option value="outbound">{ "Outbound" }</option>
                </select>
            </div>
            <div class="form-group">
                <label>{ "From" }</label>
                <input id="txn-filter-from" type="date" value={(*filter_from).clone()}
                    oninput={on_input(filter_from.clone())} />
            </div>
            <div class="form-group">
                <label>{ "To" }</label>
                <input id="txn-filter-to" type="date" value={(*filter_to).clone()}
                    oninput={on_input(filter_to.clone())} />
            </div>
        </div>

        <div class="card">
            <div class="table-wrapper">
                <table>
                    <thead>
                        <tr>
                            <th>{ "Date" }</th>
                            <th>{ "Lot" }</th>
                            <th>{ "Direction" }</th>
                            <th>{ "Qty" }</th>
                            <th>{ "Reason" }</th>
                            <th>{ "User" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { if *loading {
                            html! { <tr><td colspan="6" class="text-center text-secondary">{ "Loading..." }</td></tr> }
                        } else if transactions.is_empty() {
                            html! { <tr><td colspan="6" class="text-center text-secondary">{ "No transactions found" }</td></tr> }
                        } else {
                            html! { for transactions.iter().map(|t| {
                                let dir_badge = if t.direction == "inbound" { "badge badge-published" } else { "badge badge-offline" };
                                html! {
                                    <tr key={t.id.clone()}>
                                        <td class="text-sm">{ &t.created_at }</td>
                                        <td class="text-sm">{ &t.lot_id }</td>
                                        <td><span class={dir_badge}>{ &t.direction }</span></td>
                                        <td>{ t.quantity }</td>
                                        <td class="text-sm text-secondary">{ t.reason.as_deref().unwrap_or("—") }</td>
                                        <td class="text-sm">{ &t.performed_by }</td>
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
