//! Validated local service requests and explicit message-dialog decisions.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ServiceDraft {
    name: String,
    details: String,
    priority: String,
}

fn initial_draft() -> ServiceDraft {
    ServiceDraft {
        name: "Motor 001 inspection".into(),
        details: "Inspect bearing housing".into(),
        priority: "normal".into(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ServiceRequest {
    draft: ServiceDraft,
    cancelled: bool,
}

fn parse_variant(value: &str) -> MessageDialogVariant {
    match value {
        "info" => MessageDialogVariant::Info,
        "warning" => MessageDialogVariant::Warning,
        "error" => MessageDialogVariant::Error,
        "success" => MessageDialogVariant::Success,
        _ => MessageDialogVariant::Question,
    }
}

fn variant_value(variant: MessageDialogVariant) -> &'static str {
    match variant {
        MessageDialogVariant::Info => "info",
        MessageDialogVariant::Warning => "warning",
        MessageDialogVariant::Error => "error",
        MessageDialogVariant::Success => "success",
        MessageDialogVariant::Question => "question",
    }
}

#[component]
fn ServicePreparation() -> Element {
    let mut note = use_signal(|| "Prepare Motor 001 maintenance".to_string());
    rsx! {
        label { class: "service-preparation", r#for: "service-preparation",
            span { "Inspection preparation" }
            input { id: "service-preparation", r#type: "text", maxlength: "160", value: note(),
                oninput: move |event: FormEvent| note.set(event.value()),
            }
        }
    }
}

#[component]
fn ServiceWorkspace(
    saved: Option<ServiceRequest>,
    variant: MessageDialogVariant,
    german: bool,
    hide_close: bool,
    single_action: bool,
    saves: usize,
    cancellations: usize,
    onsave: EventHandler<ServiceDraft>,
    oncancel: EventHandler<()>,
) -> Element {
    let create = use_modal();
    let confirm = use_modal();
    let mut draft = use_signal(initial_draft);
    let mut pending_save = use_signal(|| None::<ServiceDraft>);
    let mut pending = use_signal(|| None::<ModalDismissRequest>);
    let mut name_field = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut validation = use_signal(|| None::<String>);
    let mut create_open = use_signal(|| false);
    let mut confirm_open = use_signal(|| false);
    let mut last_result = use_signal(|| "none".to_string());
    let current = draft();
    let saved_status = saved.as_ref().map_or("none", |request| {
        if request.cancelled {
            "Cancelled"
        } else {
            "Open"
        }
    });
    let close_label = if german {
        "Modal schließen"
    } else {
        "Close modal"
    };
    let can_cancel = saved.as_ref().is_some_and(|request| !request.cancelled);
    let message = saved.as_ref().map_or_else(
        || "No local request has been saved.".to_string(),
        |request| {
            if request.cancelled {
                format!(
                    "{} is already cancelled in this local example.",
                    request.draft.name
                )
            } else if single_action {
                format!(
                    "Review {} without changing the saved request.",
                    request.draft.name
                )
            } else {
                format!(
                    "Cancel {}? This updates only the saved request in this local example.",
                    request.draft.name
                )
            }
        },
    );

    rsx! {
        div { class: "service-dialog-triggers",
            span { id: "service-create-trigger", ModalTrigger { modal: create, "Create service request" } }
            // Retain a usable invoker after cancellation for ordinary native focus return.
            span { id: "service-cancel-trigger", ModalTrigger { modal: confirm, variant: ButtonVariant::Secondary, disabled: saved.is_none(), "Review service request" } }
        }
        div { id: "service-create-dialog",
            Modal { modal: create, label: "Create service request", size: ModalSize::Sm, close_on_backdrop_click: true,
                onopenchange: move |open| {
                    create_open.set(open);
                    if open { draft.set(initial_draft()); pending.set(None); pending_save.set(None); validation.set(None); }
                },
                ondismissrequest: move |request: ModalDismissRequest| {
                    if *draft.peek() != initial_draft() { pending.set(Some(request)); }
                    else { request.allow(); }
                },
                onresult: move |result: ModalResult| {
                    if let ModalResult::Closed(value) = &result {
                        if value == "save-service" {
                            let submitted = pending_save.peek().clone();
                            if let Some(submitted) = submitted { onsave.call(submitted); }
                        }
                        last_result.set(value.clone());
                    } else { last_result.set("draft-dismissed".into()); }
                    draft.set(initial_draft()); pending_save.set(None); pending.set(None); validation.set(None);
                },
                ModalHeader { hide_close, close_label, "Create service request" }
                ModalContent {
                    p { id: "service-form-help", "Create an illustrative Motor 001 request stored in this example. Name and details are required." }
                    form { id: "service-request-form", class: "service-request-form", novalidate: true,
                        onsubmit: move |event: FormEvent| {
                            event.prevent_default();
                            let mut submitted = draft.peek().clone();
                            submitted.name = submitted.name.trim().to_string(); submitted.details = submitted.details.trim().to_string();
                            if submitted.name.is_empty() || submitted.details.is_empty() {
                                validation.set(Some("Enter a service name and details containing more than spaces.".into())); return;
                            }
                            if submitted.priority != "normal" && submitted.priority != "urgent" {
                                validation.set(Some("Choose a service priority.".into())); return;
                            }
                            validation.set(None); pending_save.set(Some(submitted)); create.close("save-service");
                        },
                        label { r#for: "service-name", span { "Service name" }
                            input { id: "service-name", name: "name", r#type: "text", required: true, maxlength: "100", autofocus: true, value: current.name,
                                onmounted: move |event: MountedEvent| name_field.set(Some(event.data())),
                                "aria-describedby": if validation().is_some() { "service-validation" } else { "service-form-help" },
                                "aria-invalid": (validation().is_some() && draft().name.trim().is_empty()).to_string(),
                                oninput: move |event: FormEvent| draft.write().name = event.value(),
                            }
                        }
                        label { r#for: "service-details", span { "Service details" }
                            textarea { id: "service-details", name: "details", required: true, maxlength: "400", rows: "3", value: current.details,
                                "aria-describedby": if validation().is_some() { "service-validation" } else { "service-form-help" },
                                "aria-invalid": (validation().is_some() && draft().details.trim().is_empty()).to_string(),
                                oninput: move |event: FormEvent| draft.write().details = event.value(),
                            }
                        }
                        label { r#for: "service-priority", span { "Priority" }
                            select { id: "service-priority", name: "priority", value: current.priority.clone(),
                                onchange: move |event: FormEvent| draft.write().priority = event.value(),
                                option { value: "normal", selected: current.priority == "normal", "Normal" }
                                option { value: "urgent", selected: current.priority == "urgent", "Urgent" }
                            }
                        }
                    }
                    if let Some(error) = validation() { p { id: "service-validation", class: "service-validation", role: "alert", "{error}" } }
                    if let Some(request) = pending() {
                        div { id: "service-discard-decision", class: "service-discard-decision", role: "status",
                            p { "Discard the changes to this draft?" }
                            div { class: "service-decision-actions",
                                Button { id: "service-keep-editing", variant: ButtonVariant::Secondary,
                                    onclick: { let request = request.clone(); move |_| {
                                        request.clone().deny(); pending.set(None);
                                        // The decision button disappears; return to the retained native field.
                                        if let Some(field) = name_field.peek().clone() {
                                            spawn(async move { if create.is_open() { let _ = field.set_focus(true).await; } });
                                        }
                                    } }, "Keep editing"
                                }
                                Button { id: "service-discard", variant: ButtonVariant::DangerPrimary,
                                    onclick: move |_| { request.clone().allow(); pending.set(None); }, "Discard draft"
                                }
                            }
                        }
                    }
                }
                ModalFooter {
                    span { id: "service-cancel-draft", ModalClose { modal: create, dismiss: true, value: "discard-draft", "Cancel" } }
                    Button { id: "service-save", button_type: ButtonType::Submit, form: "service-request-form", "Save service request" }
                }
            }
        }
        div { id: "service-confirm-dialog",
            MessageDialog { modal: confirm, title: if single_action { "Service request summary" } else { "Cancel service request?" }, message, variant,
                hide_close, close_label,
                onopenchange: move |open| confirm_open.set(open),
                onresult: move |result: ModalResult| {
                    match result {
                        ModalResult::Closed(value) => { if value == "cancel-service" && can_cancel { oncancel.call(()); } last_result.set(value); }
                        ModalResult::Dismissed(_) => last_result.set("request-kept".into()),
                        ModalResult::NativeClosed => last_result.set("native-closed".into()),
                    }
                },
                actions: rsx! {
                    span { id: "service-keep-request", ModalClose { modal: confirm, dismiss: true, value: "keep-service", "Keep request" } }
                    if !single_action {
                        span { id: "service-confirm-cancel", ModalClose { modal: confirm, value: "cancel-service", disabled: !can_cancel, variant: ButtonVariant::DangerPrimary, "Cancel request" } }
                    }
                },
            }
        }
        article { id: "service-request", class: "service-saved-request", "aria-labelledby": "service-saved-heading",
            h3 { id: "service-saved-heading", "Saved local request" }
            if let Some(request) = saved {
                dl {
                    div { dt { "Service name" } dd { "data-field": "name", "{request.draft.name}" } }
                    div { dt { "Service details" } dd { "data-field": "details", "{request.draft.details}" } }
                    div { dt { "Priority" } dd { "data-field": "priority", "{request.draft.priority}" } }
                    div { dt { "Status" } dd { "data-field": "status", if request.cancelled { "Cancelled" } else { "Open" } } }
                }
            } else { p { "No service request" } }
        }
        output { id: "service-result", class: "service-result", role: "status",
            "data-saved": (saved_status != "none").to_string(), "data-status": saved_status,
            "data-create-open": create_open().to_string(), "data-confirm-open": confirm_open().to_string(), "data-pending-discard": pending().is_some().to_string(),
            "data-saves": saves.to_string(), "data-cancellations": cancellations.to_string(), "data-last-result": last_result(),
            "Saved requests: {saves} · Cancellations: {cancellations} · Current status: {saved_status}"
        }
    }
}

/// Native form and message dialogs backed by one illustrative local request.
#[component]
pub fn ServiceDialogsExample() -> Element {
    let mut saved = use_signal(|| None::<ServiceRequest>);
    let mut variant = use_signal(|| MessageDialogVariant::Question);
    let mut german = use_signal(|| false);
    let mut hide_close = use_signal(|| false);
    let mut single_action = use_signal(|| false);
    let mut saves = use_signal(|| 0_usize);
    let mut cancellations = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    rsx! {
        section { id: "service-dialogs", class: "service-dialogs-example",
            h2 { "Service requests" }
            p { class: "service-intro", "Create an illustrative local Motor 001 service request. Unsaved changes ask for a decision before dismissal; cancelling a saved request requires a separate confirmation. No maintenance service is contacted." }
            div { class: "service-toolbar",
                Button { id: "service-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        saved.set(None); variant.set(MessageDialogVariant::Question); german.set(false); hide_close.set(false); single_action.set(false);
                        saves.set(0); cancellations.set(0); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore service dialogs"
                }
            }
            details { id: "service-options", class: "service-options",
                summary { "Dialog options" }
                div { class: "service-option-fields",
                    label { r#for: "service-message-variant", span { "Message style" }
                        select { id: "service-message-variant", value: variant_value(variant()), onchange: move |event: FormEvent| variant.set(parse_variant(&event.value())),
                            for option in [MessageDialogVariant::Info, MessageDialogVariant::Warning, MessageDialogVariant::Error, MessageDialogVariant::Success, MessageDialogVariant::Question] {
                                option { key: "{variant_value(option)}", value: variant_value(option), selected: variant() == option, "{variant_value(option)}" }
                            }
                        }
                    }
                    label { input { id: "service-german", r#type: "checkbox", checked: german(), onchange: move |event: FormEvent| german.set(event.checked()) } "German close label" }
                    label { input { id: "service-hide-close", r#type: "checkbox", checked: hide_close(), onchange: move |event: FormEvent| hide_close.set(event.checked()) } "Hide header close" }
                    label { input { id: "service-single-action", r#type: "checkbox", checked: single_action(), onchange: move |event: FormEvent| single_action.set(event.checked()) } "Review without cancellation" }
                }
            }
            for generation in [epoch()] { ServicePreparation { key: "service-preparation-{generation}" } }
            for generation in [epoch()] {
                ServiceWorkspace { key: "service-workspace-{generation}", saved: saved(), variant: variant(), german: german(), hide_close: hide_close(), single_action: single_action(), saves: saves(), cancellations: cancellations(),
                    onsave: move |draft| { saved.set(Some(ServiceRequest { draft, cancelled: false })); let count = saves.peek().saturating_add(1); saves.set(count); },
                    oncancel: move |_| {
                        let current = saved.peek().clone();
                        if let Some(mut request) = current.filter(|request| !request.cancelled) {
                            request.cancelled = true; saved.set(Some(request)); let count = cancellations.peek().saturating_add(1); cancellations.set(count);
                        }
                    },
                }
            }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.service-dialogs-example{min-width:0;max-width:100%;margin-bottom:48px}.service-intro{max-width:72ch;line-height:1.55}.service-toolbar,.service-dialog-triggers,.service-decision-actions{display:flex;align-items:center;flex-wrap:wrap;gap:12px;min-width:0}.service-toolbar{margin:20px 0 16px}.service-dialog-triggers{margin-bottom:24px}.service-dialog-triggers>span{min-width:0;max-width:100%}.service-toolbar .ix-button,.service-dialog-triggers .ix-button,.service-decision-actions .ix-button{max-width:100%;white-space:normal}.service-options{margin:0 0 24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.service-options>summary{padding:10px 12px;font-size:14px;font-weight:600;cursor:pointer}.service-option-fields{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;padding:12px;min-width:0}.service-option-fields>label{display:inline-flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;max-width:100%;font-size:14px;line-height:20px}.service-option-fields input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.service-option-fields select,.service-request-form select{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.service-preparation{display:flex;flex-direction:column;gap:8px;min-width:0;width:100%;max-width:480px;margin-bottom:24px;font-size:14px;line-height:20px;color:var(--ix-color-text)}.service-preparation input,.service-request-form input,.service-request-form textarea{box-sizing:border-box;width:100%;min-width:0;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.service-preparation input,.service-request-form input{height:32px}.service-request-form{display:flex;flex-direction:column;gap:16px;min-width:0}.service-request-form>label{display:flex;flex-direction:column;gap:8px;min-width:0;font-size:14px;line-height:20px}.service-request-form textarea{min-height:80px;resize:vertical}.service-request-form select{align-self:flex-start}.service-validation{margin:16px 0 0;font-size:14px;line-height:20px;color:var(--ix-color-danger-text)}.service-discard-decision{margin-top:20px;padding:16px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);color:var(--ix-color-text)}.service-discard-decision>p{margin:0 0 12px}.service-saved-request{max-width:72ch;overflow-wrap:anywhere}.service-saved-request h3{margin:0 0 12px;font-size:20px;line-height:28px;font-weight:600}.service-saved-request dl{display:flex;flex-direction:column;gap:12px;margin:0}.service-saved-request dl>div{display:flex;align-items:baseline;flex-wrap:wrap;gap:8px 16px}.service-saved-request dt{min-width:100px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary)}.service-saved-request dd{margin:0;min-width:0;overflow-wrap:anywhere}.service-result{display:block;margin-top:24px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}.service-preparation input:focus-visible,.service-request-form input:focus-visible,.service-request-form textarea:focus-visible,.service-request-form select:focus-visible,.service-option-fields select:focus-visible,.service-options>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.service-option-fields input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_dialogs_start_without_a_request_and_render_validated_native_form() {
        let mut dom = VirtualDom::new(ServiceDialogsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "service-dialogs",
            "service-create-trigger",
            "service-cancel-trigger",
            "service-request-form",
            "service-name",
            "service-details",
            "service-priority",
            "service-save",
            "service-preparation",
            "service-request",
            "service-result",
            "service-keep-request",
            "service-confirm-cancel",
            "service-reset",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("No service request") && html.contains("Prepare Motor 001 maintenance")
        );
        assert!(html.contains("Motor 001 inspection") && html.contains("Inspect bearing housing"));
        assert!(html.contains("for=\"service-name\"") && html.contains("for=\"service-details\""));
        assert!(html.contains("form=\"service-request-form\"") && html.contains("type=\"submit\""));
        assert!(html.contains("aria-describedby=") && html.contains("Cancel service request?"));
        assert!(html.contains("data-status=\"none\"") && html.contains("data-saves=\"0\""));
    }
}
