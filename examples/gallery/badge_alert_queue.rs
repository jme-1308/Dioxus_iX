//! Local equipment-alert reviews with accessible attached and standalone badges.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;

pub const STYLES: &str = include_str!("styles/badge_alert_queue.css");

#[derive(Clone, Copy, PartialEq)]
struct AlertDefinition {
    id: &'static str,
    title: &'static str,
    detail: &'static str,
}
const ALERTS: [AlertDefinition; 3] = [
    AlertDefinition {
        id: "pump",
        title: "Pump 002 inspection",
        detail: "Review the illustrative inspection reminder and record the next step.",
    },
    AlertDefinition {
        id: "compressor",
        title: "Compressor 003 handover",
        detail: "Review the illustrative handover note before planning follow-up work.",
    },
    AlertDefinition {
        id: "sensor",
        title: "Sensor 004 maintenance",
        detail: "Review the illustrative maintenance reminder and note any preparation.",
    },
];

#[derive(Clone)]
struct AlertReview {
    id: String,
    note: String,
    payload: BTreeMap<String, String>,
}

#[component]
fn AlertPreparation() -> Element {
    let mut preparation = use_signal(|| String::from("Check access with the shift lead"));
    rsx! {
        Input { id: "badge-alert-preparation", label: "Review preparation", value: preparation(),
            max_length: 160, onvaluechange: move |value| preparation.set(value), }
    }
}

#[component]
fn AlertReviewForm(
    alert: AlertDefinition,
    reviewed: bool,
    onsave: EventHandler<AlertReview>,
) -> Element {
    let mut note = use_signal(String::new);
    let mut acknowledged = use_signal(|| false);
    let mut submissions = use_signal(|| 0_usize);
    rsx! {
        form { id: "badge-alert-form-{alert.id}", class: "badge-alert-form",
            "data-draft-note": note(), "data-ack": acknowledged(), "data-reviewed": reviewed,
            "data-submissions": submissions(),
            onsubmit: move |event| {
                event.prevent_default();
                submissions += 1;
                if reviewed { return; }
                let values = event.values().into_iter().filter_map(|(name, value)| match value {
                    dioxus::html::FormValue::Text(text) => Some((name, text)),
                    _ => None,
                }).collect::<BTreeMap<_, _>>();
                if values.get("review-ack").map(String::as_str) != Some("acknowledged") { return; }
                let Some(submitted_note) = values.get("review-note").cloned() else { return; };
                onsave.call(AlertReview { id: alert.id.into(), note: submitted_note, payload: values });
            },
            h3 { "{alert.title}" }
            p { "{alert.detail}" }
            CustomField { label: "Review note", html_for: format!("badge-alert-note-{}", alert.id),
                helper_id: format!("badge-alert-note-help-{}", alert.id), helper_text: "Optional. Saved only in this example.",
                textarea { id: "badge-alert-note-{alert.id}", class: "ix-input ix-textarea",
                    name: "review-note", rows: "2", maxlength: "240", disabled: reviewed,
                    value: note(), "aria-describedby": "badge-alert-note-help-{alert.id}",
                    oninput: move |event| note.set(event.value()),
                }
            }
            label { class: "badge-alert-ack", r#for: "badge-alert-ack-{alert.id}",
                input { id: "badge-alert-ack-{alert.id}", r#type: "checkbox", name: "review-ack",
                    value: "acknowledged", required: true, disabled: reviewed, checked: acknowledged(),
                    onchange: move |event| acknowledged.set(event.checked()),
                }
                "I have reviewed this alert"
            }
            Button { id: format!("badge-alert-review-{}", alert.id), button_type: ButtonType::Submit,
                disabled: reviewed, if reviewed { "Review saved" } else { "Save review" }
            }
        }
    }
}

#[component]
pub fn BadgeAlertQueueExample() -> Element {
    let mut open = use_signal(|| false);
    let mut detailed = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut records = use_signal(BTreeMap::<String, String>::new);
    let mut payloads = use_signal(BTreeMap::<String, BTreeMap<String, String>>::new);
    let saved = records();
    let pending = ALERTS.len() - saved.len();
    let status_label = if pending == 0 {
        String::from("All equipment alerts reviewed")
    } else {
        format!("{pending} equipment alerts await review")
    };
    let tooltip = detailed().then(|| {
        if pending == 0 {
            String::from("All three local alerts have saved reviews. Reset the queue to review them again.")
        } else {
            format!("Review the local pump, compressor and sensor alerts. {pending} still await acknowledgement.")
        }
    });
    let serialized = serde_json::to_string(&payloads()).unwrap_or_default();
    rsx! {
        style { {STYLES} }
        section { id: "badge-alert-workflow", class: "badge-alert-workflow",
            h2 { "Review equipment alerts" }
            p { id: "badge-alert-overview-help", class: "badge-alert-intro", "Open the queue to review three illustrative equipment reminders. Notes and acknowledgements stay here; no device or backend is contacted." }
            div { class: "badge-alert-preparation", AlertPreparation {} }
            label { class: "badge-alert-option",
                input { id: "badge-alert-details", r#type: "checkbox", checked: detailed(),
                    onchange: move |event| detailed.set(event.checked()), }
                "Show detailed status help"
            }
            div { class: "badge-alert-toolbar",
                Badge { label: pending.to_string(), variant: if pending == 0 { StatusVariant::Success } else { StatusVariant::Warning },
                    description_id: "badge-alert-count-help", description: format!("{pending} equipment alerts await review"),
                    Button { id: "badge-alert-open", button_type: ButtonType::Button,
                        aria_describedby: "badge-alert-overview-help badge-alert-count-help",
                        onclick: move |_| open.toggle(), if open() { "Close alerts" } else { "Open alerts" }
                    }
                }
                div { class: "badge-alert-state",
                    Badge { class: "badge-alert-status-badge", badge_type: BadgeType::StatusIcon,
                        variant: if pending == 0 { StatusVariant::Success } else { StatusVariant::Warning },
                        role: "img", aria_label: status_label.clone(), tooltip_from_label: true, tooltip,
                    }
                    span { "{status_label}" }
                }
            }
            p { id: "badge-alert-status", class: "badge-alert-status", role: "status", "aria-live": "polite",
                "data-pending": pending, "data-reviewed": saved.len(), "data-open": open(), "data-detailed": detailed(),
                "{pending} awaiting review; {saved.len()} saved locally."
            }
            div { id: "badge-alert-panel", class: "badge-alert-panel", hidden: !open(),
                for generation in [epoch()] {
                    div { key: "badge-alert-forms-{generation}",
                        for alert in ALERTS {
                            AlertReviewForm { key: "{alert.id}", alert, reviewed: saved.contains_key(alert.id),
                                onsave: move |review: AlertReview| {
                                    if !open() || records.peek().contains_key(&review.id) { return; }
                                    records.write().insert(review.id.clone(), review.note);
                                    payloads.write().insert(review.id, review.payload);
                                }
                            }
                        }
                    }
                }
            }
            table { id: "badge-alert-records", class: "badge-alert-records",
                caption { "Saved alert reviews" }
                thead { tr { th { scope: "col", "Equipment alert" } th { scope: "col", "Review note" } } }
                tbody {
                    if saved.is_empty() { tr { td { colspan: "2", "No reviews saved yet. Open alerts to start." } } }
                    for (id, note) in saved.iter() {
                        tr { key: "{id}", id: "badge-alert-record-{id}", "data-alert": id.clone(), "data-note": note.clone(),
                            td { {ALERTS.iter().find(|alert| alert.id == id.as_str()).map_or("Equipment alert", |alert| alert.title)} }
                            td { if note.is_empty() { "No note" } else { "{note}" } }
                        }
                    }
                }
            }
            Button { id: "badge-alert-reset", class: "badge-alert-reset", variant: ButtonVariant::Secondary,
                button_type: ButtonType::Button, onclick: move |_| {
                    records.write().clear(); payloads.write().clear(); epoch += 1;
                }, "Reset alert queue"
            }
            output { id: "badge-alert-payload", hidden: true, "{serialized}" }
        }
    }
}
