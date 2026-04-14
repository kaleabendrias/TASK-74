//! Import/Export page: drag-and-drop .xlsx upload with real-time SSE job tracking,
//! falling back to 2-second polling if the EventSource connection fails.
//! Export request flow with approval and watermarked download.

use gloo_timers::callback::Interval;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, File, MessageEvent};
use yew::prelude::*;

use crate::components::route_guard::RouteGuard;
use crate::context::{AuthContext, ToastAction, ToastContext};
use crate::models::*;
use crate::services::api;

#[function_component(ImportExportPage)]
pub fn import_export_page() -> Html {
    let auth = use_context::<AuthContext>().unwrap();
    let toasts = use_context::<ToastContext>().unwrap();

    // Import state
    let import_file = use_state(|| Option::<String>::None);
    let import_file_obj = use_state(|| Option::<File>::None);
    let import_job = use_state(|| Option::<ImportJobResponse>::None);
    let import_error = use_state(|| Option::<String>::None);
    let uploading = use_state(|| false);
    let drag_over = use_state(|| false);

    // Export state
    let export_type = use_state(|| "resources".to_string());
    let export_approvals = use_state(|| Vec::<ExportApprovalResponse>::new());

    // Fetch pending exports from server
    {
        let export_approvals = export_approvals.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(pending) = api::list_pending_exports().await {
                    export_approvals.set(pending);
                }
            });
            || {}
        });
    }

    // Real-time job progress: SSE primary, polling fallback.
    //
    // We always produce `cleanup: Box<dyn FnOnce()>` so that every branch of
    // the conditional returns a closure of the same concrete type, satisfying
    // Rust's type-checker inside `use_effect_with`.
    {
        let import_job = import_job.clone();
        use_effect_with((*import_job).clone(), move |job| {
            let cleanup: Box<dyn FnOnce()> = if let Some(j) = job.clone() {
                if j.status == "queued" || j.status == "running" {
                    let sse_url = format!("/api/import/jobs/{}/stream", j.id);

                    match web_sys::EventSource::new(&sse_url) {
                        Ok(es) => {
                            // ── onmessage: update job state from push events ──
                            let import_job_msg = import_job.clone();
                            let es_for_close = es.clone();
                            let onmessage = Closure::<dyn Fn(MessageEvent)>::new(
                                move |e: MessageEvent| {
                                    let Some(raw) = e.data().as_string() else { return };
                                    let Ok(updated) =
                                        serde_json::from_str::<ImportJobResponse>(&raw)
                                    else {
                                        return;
                                    };
                                    let terminal = updated.status == "completed"
                                        || updated.status == "failed";
                                    import_job_msg.set(Some(updated));
                                    if terminal {
                                        es_for_close.close();
                                    }
                                },
                            );
                            es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                            onmessage.forget();

                            // ── onerror: SSE failed, fall back to 2-second polling ──
                            let import_job_err = import_job.clone();
                            let jid_err = j.id.clone();
                            let es_for_err = es.clone();
                            let onerror = Closure::<dyn Fn(web_sys::Event)>::new(
                                move |_: web_sys::Event| {
                                    es_for_err.close();
                                    let imj = import_job_err.clone();
                                    let jid = jid_err.clone();
                                    // Leak the interval intentionally: the polling stops itself
                                    // when it observes a terminal status on the next tick.
                                    Interval::new(2_000, move || {
                                        let imj2 = imj.clone();
                                        let jid2 = jid.clone();
                                        spawn_local(async move {
                                            if let Ok(upd) = api::get_import_job(&jid2).await {
                                                imj2.set(Some(upd));
                                            }
                                        });
                                    })
                                    .forget();
                                },
                            );
                            es.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                            onerror.forget();

                            Box::new(move || {
                                es.close();
                            }) as Box<dyn FnOnce()>
                        }
                        Err(_) => {
                            // EventSource API unavailable (e.g. during local dev without TLS) —
                            // fall back directly to 2-second HTTP polling.
                            let import_job_poll = import_job.clone();
                            let jid = j.id.clone();
                            let interval = Interval::new(2_000, move || {
                                let imj = import_job_poll.clone();
                                let jid2 = jid.clone();
                                spawn_local(async move {
                                    if let Ok(updated) = api::get_import_job(&jid2).await {
                                        imj.set(Some(updated));
                                    }
                                });
                            });
                            Box::new(move || {
                                drop(interval);
                            }) as Box<dyn FnOnce()>
                        }
                    }
                } else {
                    Box::new(|| {}) as Box<dyn FnOnce()>
                }
            } else {
                Box::new(|| {}) as Box<dyn FnOnce()>
            };

            // The outer closure always has the same concrete type regardless of
            // which branch above ran — `cleanup` is always `Box<dyn FnOnce()>`.
            move || {
                (cleanup)();
            }
        });
    }

    let on_file_select = {
        let import_file = import_file.clone();
        let import_file_obj = import_file_obj.clone();
        let import_error = import_error.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let name = file.name();
                    if !name.ends_with(".xlsx") {
                        import_error.set(Some("Only .xlsx files are accepted".into()));
                        import_file.set(None);
                        import_file_obj.set(None);
                    } else {
                        import_error.set(None);
                        import_file.set(Some(name));
                        import_file_obj.set(Some(file));
                    }
                }
            }
        })
    };

    let on_drag_over = {
        let drag_over = drag_over.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            drag_over.set(true);
        })
    };

    let on_drag_leave = {
        let drag_over = drag_over.clone();
        Callback::from(move |_: DragEvent| drag_over.set(false))
    };

    let on_drop = {
        let import_file = import_file.clone();
        let import_file_obj = import_file_obj.clone();
        let import_error = import_error.clone();
        let drag_over = drag_over.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            drag_over.set(false);
            if let Some(dt) = e.data_transfer() {
                if let Some(files) = dt.files() {
                    if let Some(file) = files.get(0) {
                        let name = file.name();
                        if !name.ends_with(".xlsx") {
                            import_error.set(Some("Only .xlsx files are accepted".into()));
                            import_file.set(None);
                            import_file_obj.set(None);
                        } else {
                            import_error.set(None);
                            import_file.set(Some(name));
                            import_file_obj.set(Some(file));
                        }
                    }
                }
            }
        })
    };

    let on_upload = {
        let toasts = toasts.clone();
        let uploading = uploading.clone();
        let import_job = import_job.clone();
        let import_file_obj = import_file_obj.clone();
        Callback::from(move |_: MouseEvent| {
            let toasts = toasts.clone();
            let uploading = uploading.clone();
            let import_job = import_job.clone();
            let file_obj = (*import_file_obj).clone();
            uploading.set(true);

            spawn_local(async move {
                match file_obj {
                    Some(file) => {
                        match api::upload_import_file(file).await {
                            Ok(job) => {
                                toasts.dispatch(ToastAction::Add(
                                    ToastKind::Success, "File uploaded, processing started".into()
                                ));
                                import_job.set(Some(job));
                            }
                            Err(e) => {
                                toasts.dispatch(ToastAction::Add(ToastKind::Error, e));
                            }
                        }
                    }
                    None => {
                        toasts.dispatch(ToastAction::Add(ToastKind::Error, "No file selected".into()));
                    }
                }
                uploading.set(false);
            });
        })
    };

    // Export request
    let on_request_export = {
        let export_type = export_type.clone();
        let export_approvals = export_approvals.clone();
        let toasts = toasts.clone();
        Callback::from(move |_: MouseEvent| {
            let et = (*export_type).clone();
            let approvals = export_approvals.clone();
            let toasts = toasts.clone();
            spawn_local(async move {
                let req = ExportRequestBody { export_type: et };
                match api::request_export(&req).await {
                    Ok(a) => {
                        let mut list = (*approvals).clone();
                        list.push(a);
                        approvals.set(list);
                        toasts.dispatch(ToastAction::Add(ToastKind::Success, "Export requested — awaiting approval".into()));
                    }
                    Err(e) => toasts.dispatch(ToastAction::Add(ToastKind::Error, e)),
                }
            });
        })
    };

    let on_approve_export = {
        let export_approvals = export_approvals.clone();
        let toasts = toasts.clone();
        Callback::from(move |id: String| {
            let approvals = export_approvals.clone();
            let toasts = toasts.clone();
            spawn_local(async move {
                match api::approve_export(&id).await {
                    Ok(updated) => {
                        let mut list = (*approvals).clone();
                        if let Some(pos) = list.iter().position(|a| a.id == updated.id) {
                            list[pos] = updated;
                        }
                        approvals.set(list);
                        toasts.dispatch(ToastAction::Add(ToastKind::Success, "Export approved".into()));
                    }
                    Err(e) => toasts.dispatch(ToastAction::Add(ToastKind::Error, e)),
                }
            });
        })
    };

    let upload_area_class = if *drag_over { "upload-area dragover" } else { "upload-area" };
    let role = auth.user.as_ref().map(|u| &u.role);

    html! {
        <RouteGuard allowed_roles={vec![UserRole::Administrator, UserRole::InventoryClerk, UserRole::Reviewer]}>
        <>
        <div class="page-header">
            <h1>{ "Import / Export" }</h1>
        </div>

        // ── Import section ──
        <div class="card">
            <div class="card-header"><h2>{ "Import Data" }</h2></div>

            <div class={upload_area_class} id="import-dropzone"
                ondragover={on_drag_over} ondragleave={on_drag_leave} ondrop={on_drop}>
                <p><strong>{ "Drop .xlsx file here" }</strong></p>
                <p>{ "or click to select a file" }</p>
                <input id="import-file-input" type="file" accept=".xlsx"
                    onchange={on_file_select}
                    style="position:absolute;opacity:0;width:100%;height:100%;top:0;left:0;cursor:pointer;" />
            </div>

            { if let Some(ref name) = *import_file {
                html! {
                    <div class="mt-4 flex items-center gap-3">
                        <span class="text-sm">{ format!("Selected: {}", name) }</span>
                        <button id="import-upload-btn" class="btn btn-primary btn-sm"
                            disabled={*uploading}
                            onclick={on_upload}>
                            { if *uploading { "Uploading..." } else { "Upload & Process" } }
                        </button>
                    </div>
                }
            } else { html!{} }}

            { if let Some(ref e) = *import_error {
                html! { <div class="field-error mt-2">{ e }</div> }
            } else { html!{} }}

            // Job progress
            { if let Some(ref job) = *import_job {
                let status_badge = format!("badge badge-{}", job.status);
                html! {
                    <div class="mt-4 card" style="background:var(--color-bg);">
                        <div class="flex items-center justify-between mb-2">
                            <span class="text-sm"><strong>{ "Job: " }</strong>{ &job.id }</span>
                            <span class={status_badge}>{ &job.status }</span>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-bar-fill" style={format!("width:{}%", job.progress_percent)} />
                        </div>
                        <div class="text-sm text-secondary mt-2">
                            { format!("{}/{} rows ({}%)", job.processed_rows, job.total_rows, job.progress_percent) }
                            { if job.retries > 0 { format!(" — {} retries", job.retries) } else { String::new() } }
                        </div>
                        { if let Some(ref log) = job.failure_log {
                            html! {
                                <details class="mt-2">
                                    <summary class="text-sm" style="cursor:pointer;color:var(--color-error);">
                                        { "View failure log" }
                                    </summary>
                                    <pre class="text-sm mt-2" style="white-space:pre-wrap;background:#fef2f2;padding:8px;border-radius:4px;">
                                        { log }
                                    </pre>
                                </details>
                            }
                        } else { html!{} }}
                    </div>
                }
            } else { html!{} }}
        </div>

        // ── Export section — only Administrator and Reviewer may request exports ──
        { if matches!(role, Some(UserRole::Administrator) | Some(UserRole::Reviewer)) { html! {
        <div class="card">
            <div class="card-header"><h2>{ "Export Data" }</h2></div>

            <div class="flex gap-3 items-center mb-4">
                <div class="form-group" style="margin-bottom:0;flex:1;max-width:240px;">
                    <select id="export-type-select" onchange={{
                        let export_type = export_type.clone();
                        Callback::from(move |e: Event| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            export_type.set(input.value());
                        })
                    }}>
                        <option value="resources">{ "Resources" }</option>
                        <option value="lodgings">{ "Lodgings" }</option>
                        <option value="inventory">{ "Inventory" }</option>
                        <option value="transactions">{ "Transactions" }</option>
                    </select>
                </div>
                <button id="request-export-btn" class="btn btn-primary btn-sm"
                    onclick={on_request_export}>
                    { "Request Export" }
                </button>
            </div>

            { if !export_approvals.is_empty() {
                html! {
                    <table>
                        <thead>
                            <tr>
                                <th>{ "Type" }</th>
                                <th>{ "Status" }</th>
                                <th>{ "Requested" }</th>
                                <th>{ "Watermark" }</th>
                                <th>{ "Actions" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for export_approvals.iter().map(|a| {
                                let status_badge = format!("badge badge-{}", a.status);
                                let id = a.id.clone();
                                let approve_cb = on_approve_export.clone();
                                let is_reviewer = matches!(role, Some(UserRole::Administrator) | Some(UserRole::Reviewer));
                                html! {
                                    <tr key={a.id.clone()}>
                                        <td>{ &a.export_type }</td>
                                        <td><span class={status_badge}>{ &a.status }</span></td>
                                        <td class="text-sm">{ &a.created_at }</td>
                                        <td class="text-sm text-secondary">{ a.watermark_text.as_deref().unwrap_or("—") }</td>
                                        <td>
                                            { if a.status == "pending" && is_reviewer {
                                                let aid = id.clone();
                                                html! {
                                                    <button id={format!("approve-export-{}", a.id)}
                                                        class="btn btn-success btn-sm"
                                                        onclick={Callback::from(move |_: MouseEvent| approve_cb.emit(aid.clone()))}>
                                                        { "Approve" }
                                                    </button>
                                                }
                                            } else if a.status == "approved" {
                                                let url = api::export_download_url(&a.id);
                                                html! {
                                                    <a id={format!("download-export-{}", a.id)}
                                                        href={url} class="btn btn-primary btn-sm"
                                                        target="_blank">
                                                        { "Download" }
                                                    </a>
                                                }
                                            } else {
                                                html! { "—" }
                                            }}
                                        </td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                }
            } else {
                html! { <p class="text-secondary text-sm">{ "No export requests yet." }</p> }
            }}
        </div>
        }} else { html! {
            <div class="card">
                <div class="card-header"><h2>{ "Export Data" }</h2></div>
                <p class="text-secondary text-sm">
                    { "Export requests require Administrator or Reviewer role." }
                </p>
            </div>
        }}}
        </>
        </RouteGuard>
    }
}
