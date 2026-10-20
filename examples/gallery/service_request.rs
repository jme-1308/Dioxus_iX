//! Controlled maintenance service request form with host validation and native payload preview.
use dioxus::html::FormValue;
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceRequestStatus {
    Draft,
    Invalid,
    Ready,
}

impl ServiceRequestStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Invalid => "invalid",
            Self::Ready => "ready",
        }
    }

    pub const fn pill_variant(self) -> StatusVariant {
        match self {
            Self::Draft => StatusVariant::Neutral,
            Self::Invalid => StatusVariant::Warning,
            Self::Ready => StatusVariant::Success,
        }
    }

    pub const fn pill_icon(self) -> &'static str {
        match self {
            Self::Draft => "info",
            Self::Invalid => "warning",
            Self::Ready => "success",
        }
    }

    pub const fn pill_label(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Invalid => "Needs attention",
            Self::Ready => "Preview ready",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceRequestPreview {
    pub asset: String,
    pub priority: String,
    pub technicians: u32,
    pub notes: String,
    pub shutdown: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ServiceRequestErrors {
    pub asset_invalid: bool,
    pub technicians_invalid: bool,
}

impl ServiceRequestErrors {
    pub const fn has_errors(self) -> bool {
        self.asset_invalid || self.technicians_invalid
    }
}

fn priority_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("low", "Low"),
        SelectOption::new("normal", "Normal"),
        SelectOption::new("high", "High"),
    ]
}

fn priority_label(value: &str) -> &str {
    match value {
        "low" => "Low",
        "normal" => "Normal",
        "high" => "High",
        other => other,
    }
}

pub fn is_valid_asset(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn parse_technicians(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(count) = trimmed.parse::<u32>() {
        return (1..=6).contains(&count).then_some(count);
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    if !parsed.is_finite() || parsed.fract() != 0.0 || !(1.0..=6.0).contains(&parsed) {
        return None;
    }
    Some(parsed as u32)
}

fn validate_fields(asset: &str, technicians: &str) -> ServiceRequestErrors {
    ServiceRequestErrors {
        asset_invalid: !is_valid_asset(asset),
        technicians_invalid: parse_technicians(technicians).is_none(),
    }
}

pub fn validate_service_request(
    values: impl IntoIterator<Item = (String, FormValue)>,
) -> Result<ServiceRequestPreview, ServiceRequestErrors> {
    let mut text_values = BTreeMap::new();
    for (name, value) in values {
        if let FormValue::Text(text) = value {
            text_values.insert(name, text);
        }
    }

    let raw_asset = text_values.get("asset").map(String::as_str).unwrap_or("");
    let raw_technicians = text_values
        .get("technicians")
        .map(String::as_str)
        .unwrap_or("");
    let errors = validate_fields(raw_asset, raw_technicians);
    if errors.has_errors() {
        return Err(errors);
    }

    let priority = text_values.get("priority").cloned().unwrap_or_default();
    let technicians = parse_technicians(raw_technicians).expect("validated technicians count");
    let notes = text_values.get("notes").cloned().unwrap_or_default();
    let shutdown = matches!(
        text_values.get("shutdown").map(String::as_str),
        Some("on" | "true")
    );

    Ok(ServiceRequestPreview {
        asset: raw_asset.trim().to_string(),
        priority,
        technicians,
        notes,
        shutdown,
    })
}

fn status_summary(status: ServiceRequestStatus) -> &'static str {
    match status {
        ServiceRequestStatus::Draft => "Complete the fields to preview a request. Nothing is sent.",
        ServiceRequestStatus::Invalid => "Check the highlighted fields and try again.",
        ServiceRequestStatus::Ready => "Review the request details below. Nothing has been sent.",
    }
}

#[component]
fn ServicePreviewList(preview: ServiceRequestPreview) -> Element {
    let shutdown_text = if preview.shutdown {
        "Yes (shutdown required)"
    } else {
        "No"
    };
    let notes_text = if preview.notes.trim().is_empty() {
        "None".to_string()
    } else {
        preview.notes.clone()
    };
    rsx! {
        dl {
            id: "service-preview",
            class: "service-request-preview",
            "data-asset": "{preview.asset}",
            "data-priority": "{preview.priority}",
            "data-technicians": "{preview.technicians}",
            "data-notes": "{preview.notes}",
            "data-shutdown": "{preview.shutdown}",
            div { class: "service-request-preview-item",
                dt { "Asset tag" }
                dd { "{preview.asset}" }
            }
            div { class: "service-request-preview-item",
                dt { "Priority" }
                dd { "{priority_label(&preview.priority)}" }
            }
            div { class: "service-request-preview-item",
                dt { "Technicians" }
                dd { "{preview.technicians}" }
            }
            div { class: "service-request-preview-item",
                dt { "Requires shutdown" }
                dd { "{shutdown_text}" }
            }
            div { class: "service-request-preview-item service-request-preview-notes",
                dt { "Work notes" }
                dd { "{notes_text}" }
            }
        }
    }
}

/// Interactive service request form with Rust validation and native FormData preview.
#[component]
pub fn ServiceRequestExample() -> Element {
    let asset_controller = use_input();
    let technicians_controller = use_number_input();
    let mut asset = use_signal(String::new);
    let mut priority = use_signal(|| "normal".to_string());
    let mut technicians = use_signal(|| "2".to_string());
    let mut notes = use_signal(String::new);
    let mut shutdown = use_signal(|| false);
    let mut locked = use_signal(|| false);
    let mut attempted = use_signal(|| false);
    let mut preview = use_signal(|| None::<ServiceRequestPreview>);
    let mut submissions = use_signal(|| 0usize);
    let mut epoch = use_signal(|| 0u64);

    let current_asset = asset();
    let current_priority = priority();
    let current_technicians = technicians();
    let current_notes = notes();
    let current_shutdown = shutdown();
    let current_locked = locked();
    let current_attempted = attempted();
    let current_preview = preview();
    let current_submissions = submissions();
    let current_epoch = epoch();

    let errors = if current_attempted {
        validate_fields(&current_asset, &current_technicians)
    } else {
        ServiceRequestErrors::default()
    };
    let status = if current_preview.is_some() {
        ServiceRequestStatus::Ready
    } else if errors.has_errors() {
        ServiceRequestStatus::Invalid
    } else {
        ServiceRequestStatus::Draft
    };
    let asset_state = if errors.asset_invalid {
        FieldState::Invalid
    } else {
        FieldState::Default
    };
    let technicians_state = if errors.technicians_invalid {
        FieldState::Invalid
    } else {
        FieldState::Default
    };

    rsx! {
        section {
            id: "service-request",
            class: "demo-section service-request-example",
            h2 { "Service request" }
            p {
                "Prepare an equipment maintenance request and preview its details locally. Nothing is sent."
            }
            div { class: "service-request-toolbar",
                Checkbox {
                    id: "service-lock",
                    label: "Read-only request",
                    checked: current_locked,
                    oncheckedchange: move |next| locked.set(next),
                }
                Button {
                    id: "service-reset",
                    variant: ButtonVariant::Secondary,
                    button_type: ButtonType::Button,
                    onclick: move |_| {
                        asset.set(String::new());
                        priority.set("normal".to_string());
                        technicians.set("2".to_string());
                        notes.set(String::new());
                        shutdown.set(false);
                        locked.set(false);
                        attempted.set(false);
                        preview.set(None);
                        submissions.set(0);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Reset request"
                }
            }
            for generation in [current_epoch] {
                form {
                    key: "service-form-{generation}",
                    id: "service-form",
                    class: "service-request-form",
                    novalidate: true,
                    onsubmit: {
                        let asset_controller = asset_controller.clone();
                        let technicians_controller = technicians_controller.clone();
                        move |event: FormEvent| {
                        event.prevent_default();
                        if *locked.peek() {
                            return;
                        }
                        match validate_service_request(event.values()) {
                            Ok(accepted) => {
                                attempted.set(false);
                                let next_count = *submissions.peek() + 1;
                                submissions.set(next_count);
                                preview.set(Some(accepted));
                            }
                            Err(submit_errors) => {
                                attempted.set(true);
                                preview.set(None);
                                let focus_target = if submit_errors.asset_invalid {
                                    Some(asset_controller.clone())
                                } else if submit_errors.technicians_invalid {
                                    Some(technicians_controller.clone())
                                } else {
                                    None
                                };
                                if let Some(controller) = focus_target {
                                    spawn(async move {
                                        let _ = controller.focus_input().await;
                                    });
                                }
                            }
                        }
                        }
                    },
                    div { class: "service-request-grid",
                        Input {
                            id: "service-asset",
                            name: "asset",
                            label: "Asset tag",
                            placeholder: "e.g. MTR-42",
                            value: current_asset.clone(),
                            required: true,
                            readonly: current_locked,
                            controller: asset_controller.clone(),
                            state: asset_state,
                            helper_text: "Enter the target equipment or line identifier.",
                            invalid_text: "Enter a non-empty asset tag.",
                            onvaluechange: move |next| {
                                if *locked.peek() {
                                    return;
                                }
                                asset.set(next);
                                preview.set(None);
                            },
                        }
                        Select {
                            id: "service-priority",
                            name: "priority",
                            label: "Priority",
                            options: priority_options(),
                            value: current_priority.clone(),
                            readonly: current_locked,
                            helper_text: "Choose the scheduling priority.",
                            onvaluechange: move |next| {
                                if *locked.peek() {
                                    return;
                                }
                                priority.set(next);
                                preview.set(None);
                            },
                        }
                        NumberInput {
                            id: "service-technicians",
                            name: "technicians",
                            label: "Technicians",
                            value: current_technicians.clone(),
                            min: 1.0,
                            max: 6.0,
                            step: 1.0,
                            required: true,
                            readonly: current_locked,
                            allow_empty_value_change: true,
                            show_stepper_buttons: true,
                            controller: technicians_controller.clone(),
                            state: technicians_state,
                            helper_text: "Assign 1 to 6 technicians.",
                            invalid_text: "Enter a whole number of technicians from 1 to 6.",
                            onvaluechange: move |next| {
                                if *locked.peek() {
                                    return;
                                }
                                technicians.set(next);
                                preview.set(None);
                            },
                        }
                        div { class: "service-request-notes-cell",
                            Textarea {
                                id: "service-notes",
                                name: "notes",
                                label: "Work notes",
                                placeholder: "Scope, lockout notes, or parts needed…",
                                value: current_notes.clone(),
                                max_length: 200,
                                rows: 3,
                                width: "100%",
                                resize: "vertical",
                                readonly: current_locked,
                                helper_text: "Optional maintenance scope and preparation notes.",
                                onvaluechange: move |next| {
                                    if *locked.peek() {
                                        return;
                                    }
                                    notes.set(next);
                                    preview.set(None);
                                },
                            }
                        }
                    }
                    div { class: "service-request-actions",
                        Checkbox {
                            id: "service-shutdown",
                            name: "shutdown",
                            label: "Requires shutdown",
                            value: "on",
                            checked: current_shutdown,
                            disabled: current_locked,
                            oncheckedchange: move |next| {
                                if *locked.peek() {
                                    return;
                                }
                                shutdown.set(next);
                                preview.set(None);
                            },
                        }
                        Button {
                            id: "service-submit",
                            variant: ButtonVariant::Primary,
                            button_type: ButtonType::Submit,
                            disabled: current_locked,
                            "Preview request"
                        }
                    }
                }
            }
            output {
                id: "service-result",
                class: "service-request-result",
                role: "status",
                "aria-live": "polite",
                "data-status": status.as_str(),
                "data-submissions": current_submissions.to_string(),
                div { class: "service-request-status-bar",
                    Pill {
                        variant: status.pill_variant(),
                        outline: true,
                        icon: status.pill_icon(),
                        "{status.pill_label()}"
                    }
                    span { class: "service-request-status-text",
                        "{status_summary(status)}"
                    }
                }
                if let Some(accepted) = current_preview {
                    ServicePreviewList { preview: accepted }
                }
            }
        }
    }
}

pub const STYLES: &str = "\
.service-request-example{min-width:0;max-width:100%}\
.service-request-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}\
.service-request-toolbar{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px 16px;max-width:880px;margin:16px 0}\
.service-request-form{display:flex;flex-direction:column;gap:20px;max-width:880px;min-width:0}\
.service-request-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,15rem),1fr));gap:20px 24px;align-items:start;min-width:0}\
.service-request-notes-cell{grid-column:1/-1;min-width:0}\
.service-request-notes-cell .ix-textarea{box-sizing:border-box;max-width:100%}\
.service-request-actions{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:12px 16px;padding-top:4px;min-width:0}\
.service-request-result{display:block;max-width:880px;margin-top:20px;padding-top:16px;border-top:1px solid var(--ix-color-border);min-width:0}\
.service-request-status-bar{display:flex;align-items:center;flex-wrap:wrap;gap:10px 12px;min-width:0}\
.service-request-status-text{font-size:14px;color:var(--ix-color-text-secondary);font-variant-numeric:tabular-nums;overflow-wrap:anywhere}\
.service-request-preview{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,11rem),1fr));gap:12px 20px;margin:16px 0 0;padding:12px 0 0;border-top:1px solid var(--ix-color-border);min-width:0}\
.service-request-preview-item{margin:0;min-width:0}\
.service-request-preview-notes{grid-column:1/-1}\
.service-request-preview dt{font-size:12px;color:var(--ix-color-text-secondary);margin-bottom:4px}\
.service-request-preview dd{margin:0;font-size:14px;font-weight:600;color:var(--ix-color-text);overflow-wrap:anywhere;white-space:pre-wrap}\
@media(max-width:480px){.service-request-toolbar,.service-request-actions{align-items:flex-start}.service-request-toolbar .ix-button,.service-request-actions .ix-button{max-width:100%}}\
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_request_validates_trimmed_asset_and_bounded_integer_technicians_from_form_values() {
        let missing = validate_service_request([
            ("asset".to_string(), FormValue::Text("   ".to_string())),
            (
                "priority".to_string(),
                FormValue::Text("normal".to_string()),
            ),
            ("technicians".to_string(), FormValue::Text("2".to_string())),
        ]);
        assert_eq!(
            missing,
            Err(ServiceRequestErrors {
                asset_invalid: true,
                technicians_invalid: false,
            })
        );

        for invalid_count in ["", "0", "7", "-1", "2.5", "NaN", "inf", "two"] {
            let result = validate_service_request([
                ("asset".to_string(), FormValue::Text("MTR-42".to_string())),
                (
                    "technicians".to_string(),
                    FormValue::Text(invalid_count.to_string()),
                ),
            ]);
            assert_eq!(
                result,
                Err(ServiceRequestErrors {
                    asset_invalid: false,
                    technicians_invalid: true,
                }),
                "rejected technicians input: {invalid_count}"
            );
        }

        let accepted = validate_service_request([
            (
                "asset".to_string(),
                FormValue::Text("  MTR-42  ".to_string()),
            ),
            ("priority".to_string(), FormValue::Text("high".to_string())),
            ("technicians".to_string(), FormValue::Text("4".to_string())),
            (
                "notes".to_string(),
                FormValue::Text("Inspect <bearings> & seals.".to_string()),
            ),
            ("shutdown".to_string(), FormValue::Text("on".to_string())),
        ])
        .expect("valid service request payload");
        assert_eq!(
            accepted,
            ServiceRequestPreview {
                asset: "MTR-42".to_string(),
                priority: "high".to_string(),
                technicians: 4,
                notes: "Inspect <bearings> & seals.".to_string(),
                shutdown: true,
            }
        );

        let without_shutdown = validate_service_request([
            ("asset".to_string(), FormValue::Text("PUMP-7".to_string())),
            ("priority".to_string(), FormValue::Text("low".to_string())),
            ("technicians".to_string(), FormValue::Text("1".to_string())),
        ])
        .expect("valid request without optional notes or shutdown");
        assert!(!without_shutdown.shutdown);
        assert!(without_shutdown.notes.is_empty());
    }

    #[test]
    fn service_request_renders_initial_draft_contract_and_escapes_preview_fields() {
        let mut dom = VirtualDom::new(ServiceRequestExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"service-request\""));
        assert!(html.contains("id=\"service-form\""));
        assert!(html.contains("novalidate"));
        assert!(html.contains("id=\"service-asset\""));
        assert!(html.contains("name=\"asset\""));
        assert!(html.contains("id=\"service-priority\""));
        assert!(html.contains("name=\"priority\""));
        assert!(html.contains("id=\"service-technicians\""));
        assert!(html.contains("name=\"technicians\""));
        assert!(html.contains("id=\"service-notes\""));
        assert!(html.contains("name=\"notes\""));
        assert!(html.contains("id=\"service-shutdown\""));
        assert!(html.contains("name=\"shutdown\""));
        assert!(html.contains("id=\"service-lock\""));
        assert!(html.contains("id=\"service-submit\""));
        assert!(html.contains("id=\"service-reset\""));
        assert!(html.contains("id=\"service-result\""));
        assert!(html.contains("data-status=\"draft\""));
        assert!(html.contains("data-submissions=\"0\""));
        assert!(html.contains("Draft"));
        assert!(!html.contains("id=\"service-preview\""));

        fn preview_fixture() -> Element {
            rsx! {
                ServicePreviewList {
                    preview: ServiceRequestPreview {
                        asset: "MTR <42> & \"A\"".to_string(),
                        priority: "high".to_string(),
                        technicians: 3,
                        notes: "Use tag <LOTO-1> & verify.".to_string(),
                        shutdown: true,
                    }
                }
            }
        }
        let mut preview_dom = VirtualDom::new(preview_fixture);
        preview_dom.rebuild_in_place();
        let preview_html = dioxus_ssr::render(&preview_dom);
        assert!(preview_html.contains("id=\"service-preview\""));
        assert!(preview_html.contains("data-priority=\"high\""));
        assert!(preview_html.contains("data-technicians=\"3\""));
        assert!(preview_html.contains("data-shutdown=\"true\""));
        assert!(
            preview_html.contains("MTR &#60;42&#62; &#38;"),
            "{preview_html}"
        );
        assert!(preview_html.contains("Use tag &#60;LOTO-1&#62; &#38; verify."));
        assert!(!preview_html.contains("<LOTO-1>"));
    }
}
