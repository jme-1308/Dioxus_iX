//! Local maintenance review and archival through MessageBar dismissal events.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MaintenanceNotice {
    asset: &'static str,
    detail: &'static str,
    reviewed: bool,
}

fn initial_notice() -> MaintenanceNotice {
    MaintenanceNotice {
        asset: "Motor 001",
        detail: "Bearing inspection awaits local review.",
        reviewed: false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArchivedNotice {
    notice: MaintenanceNotice,
    severity: MessageType,
}

fn parse_severity(value: &str) -> MessageType {
    match value {
        "alarm" => MessageType::Alarm,
        "critical" => MessageType::Critical,
        "warning" => MessageType::Warning,
        "success" => MessageType::Success,
        "neutral" => MessageType::Neutral,
        "primary" => MessageType::Primary,
        _ => MessageType::Info,
    }
}

#[component]
fn NoticePreparation() -> Element {
    let mut note = use_signal(|| "Prepare Motor 001 inspection".to_string());
    rsx! {
        label { class: "notices-note-field", r#for: "notices-note",
            span { "Inspection preparation" }
            input { id: "notices-note", r#type: "text", maxlength: "160", value: note(),
                oninput: move |event: FormEvent| note.set(event.value()),
            }
        }
    }
}

/// Review an illustrative local notice before dismissing it into the archive.
#[component]
pub fn MaintenanceNoticesExample() -> Element {
    let mut notice = use_signal(initial_notice);
    let mut archived = use_signal(|| None::<ArchivedNotice>);
    let mut pending_archive = use_signal(|| None::<ArchivedNotice>);
    let mut require_review = use_signal(|| true);
    let mut persistent = use_signal(|| false);
    let mut severity = use_signal(|| MessageType::Warning);
    let mut german = use_signal(|| false);
    let mut close_blocked = use_signal(|| false);
    let mut close_requests = use_signal(|| 0_usize);
    let mut accepted = use_signal(|| 0_usize);
    let mut completions = use_signal(|| 0_usize);
    let mut blocked = use_signal(|| 0_usize);
    let mut reviews = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let current = notice();
    let archive = archived();
    let status = if archive.is_some() {
        "Archived"
    } else if current.reviewed {
        "Reviewed"
    } else {
        "Awaiting review"
    };

    rsx! {
        section { id: "maintenance-notices", class: "maintenance-notices-example",
            h2 { "Maintenance notices" }
            p { class: "notices-intro", "Review an illustrative Motor 001 inspection notice, then dismiss it to the local archive. Required review keeps an unfinished notice open. This example does not connect to equipment or a maintenance service." }
            div { class: "notices-toolbar",
                Button { id: "notices-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        notice.set(initial_notice()); archived.set(None); pending_archive.set(None); require_review.set(true);
                        persistent.set(false); severity.set(MessageType::Warning); german.set(false); close_blocked.set(false);
                        close_requests.set(0); accepted.set(0); completions.set(0); blocked.set(0); reviews.set(0);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore maintenance notice"
                }
            }
            details { id: "notices-options", class: "notices-options",
                summary { "Notice options" }
                div { class: "notices-option-fields",
                    label { input { id: "notices-require-review", r#type: "checkbox", checked: require_review(),
                        onchange: move |event: FormEvent| { require_review.set(event.checked()); close_blocked.set(false); },
                    } "Require review before dismissal" }
                    label { input { id: "notices-persistent", r#type: "checkbox", checked: persistent(), onchange: move |event: FormEvent| persistent.set(event.checked()) } "Persistent notice" }
                    label { input { id: "notices-german", r#type: "checkbox", checked: german(), onchange: move |event: FormEvent| german.set(event.checked()) } "German close label" }
                    label { class: "notices-select-field", r#for: "notices-severity", span { "Severity" }
                        select { id: "notices-severity", value: severity().as_str(), onchange: move |event: FormEvent| severity.set(parse_severity(&event.value())),
                            for variant in [MessageType::Info, MessageType::Alarm, MessageType::Critical, MessageType::Warning, MessageType::Success, MessageType::Neutral, MessageType::Primary] {
                                option { key: "{variant.as_str()}", value: variant.as_str(), selected: severity() == variant, "{variant.as_str()}" }
                            }
                        }
                    }
                }
            }
            div { id: "notices-bar", class: "notices-bar",
                for generation in [epoch()] {
                    MessageBar { key: "maintenance-notice-{generation}", message_type: severity(), persistent: persistent(),
                        close_label: if german() { "Meldung schließen" } else { "Close message" },
                        oncloserequest: move |request: MessageBarCloseRequest| {
                            let count = close_requests.peek().saturating_add(1); close_requests.set(count);
                            if *require_review.peek() && !notice.peek().reviewed {
                                request.prevent_default(); close_blocked.set(true);
                                let count = blocked.peek().saturating_add(1); blocked.set(count);
                            }
                        },
                        onclose: move |_| {
                            pending_archive.set(Some(ArchivedNotice { notice: notice.peek().clone(), severity: *severity.peek() }));
                            close_blocked.set(false); let count = accepted.peek().saturating_add(1); accepted.set(count);
                        },
                        onclosecomplete: move |_| {
                            if archived.peek().is_none() {
                                let entry = pending_archive.peek().clone();
                                if let Some(entry) = entry {
                                    archived.set(Some(entry)); pending_archive.set(None);
                                    let count = completions.peek().saturating_add(1); completions.set(count);
                                }
                            }
                        },
                        div { class: "notices-content",
                            div { class: "notices-copy",
                                strong { "{current.asset} inspection notice" }
                                span { id: "notices-detail", "{current.detail}" }
                            }
                            // Keep native focus so Tab can reach Close after review.
                            button { id: "notices-review", class: "notices-review", r#type: "button", "aria-disabled": current.reviewed.to_string(),
                                onclick: move |_| {
                                    if !notice.peek().reviewed && archived.peek().is_none() {
                                        let mut next = notice.peek().clone(); next.reviewed = true; next.detail = "Bearing inspection review recorded locally.";
                                        notice.set(next); close_blocked.set(false);
                                        let count = reviews.peek().saturating_add(1); reviews.set(count);
                                    }
                                },
                                if current.reviewed { "Review recorded" } else { "Review Motor 001" }
                            }
                        }
                    }
                }
            }
            if close_blocked() {
                p { id: "notices-blocked", class: "notices-blocked", "Review Motor 001 before dismissing this notice." }
            }
            p { id: "notices-status", class: "notices-status", "{status}" }
            for generation in [epoch()] { NoticePreparation { key: "notice-preparation-{generation}" } }
            section { class: "notices-archive-section", "aria-labelledby": "notices-archive-heading",
                h3 { id: "notices-archive-heading", "Archived notices" }
                ul { id: "notices-archive", class: "notices-archive",
                    if let Some(entry) = archive {
                        li { id: "notices-archived-motor", "data-reviewed": entry.notice.reviewed.to_string(), "data-variant": entry.severity.as_str(),
                            span { "data-field": "asset", "{entry.notice.asset}" }
                            " · "
                            span { "data-field": "review", if entry.notice.reviewed { "Reviewed" } else { "Unreviewed" } }
                            " · "
                            span { "data-field": "severity", "{entry.severity.as_str()}" }
                            p { "{entry.notice.detail}" }
                        }
                    } else { li { "No archived notices" } }
                }
            }
            output { id: "notices-result", class: "notices-result", role: "status",
                "data-reviewed": current.reviewed.to_string(), "data-require-review": require_review().to_string(), "data-persistent": persistent().to_string(),
                "data-variant": severity().as_str(), "data-archived": archived().is_some().to_string(), "data-german": german().to_string(),
                "data-close-requests": close_requests().to_string(), "data-accepted": accepted().to_string(), "data-completions": completions().to_string(),
                "data-blocked": blocked().to_string(), "data-reviews": reviews().to_string(),
                "Motor 001 notice: {status}"
            }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.maintenance-notices-example{min-width:0;max-width:100%;margin-bottom:48px}.notices-intro{max-width:72ch;line-height:1.55}.notices-toolbar{display:flex;flex-wrap:wrap;gap:12px;margin:20px 0 16px}.notices-toolbar .ix-button{max-width:100%;white-space:normal}.notices-options{margin:0 0 24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.notices-options>summary{padding:10px 12px;font-size:14px;font-weight:600;cursor:pointer}.notices-option-fields{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;padding:12px;min-width:0}.notices-option-fields>label{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;font-size:14px;line-height:20px}.notices-option-fields input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.notices-select-field{flex-wrap:wrap}.notices-select-field select{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.notices-bar{min-width:0;max-width:100%}.notices-content{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;min-width:0}.notices-copy{display:flex;flex:1 1 240px;flex-direction:column;gap:4px;min-width:0;overflow-wrap:anywhere;color:inherit}.notices-copy strong{font-weight:600}.notices-review{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:transparent;color:inherit;font:14px/20px var(--ix-font);cursor:pointer}.notices-review:hover:not([aria-disabled=true]){background:var(--ix-color-surface-2)}.notices-review[aria-disabled=true]{opacity:.4;cursor:default}.notices-status{margin:16px 0;line-height:24px;color:var(--ix-color-text)}.notices-blocked{margin:12px 0 0;max-width:72ch;font-size:14px;line-height:20px;color:var(--ix-color-text)}.notices-note-field{display:flex;flex-direction:column;gap:8px;min-width:0;width:100%;max-width:480px;margin-bottom:32px;font-size:14px;line-height:20px;color:var(--ix-color-text)}.notices-note-field input{box-sizing:border-box;width:100%;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.notices-archive-section{max-width:72ch;line-height:1.55}.notices-archive-section h3{margin:0 0 12px;font-size:20px;line-height:28px;font-weight:600}.notices-archive{margin:0;padding-inline-start:24px;overflow-wrap:anywhere}.notices-archive p{margin:8px 0 0}.notices-result{display:block;margin-top:24px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}.notices-note-field input:focus-visible,.notices-review:focus-visible,.notices-select-field select:focus-visible,.notices-options>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.notices-option-fields input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintenance_notices_renders_real_pending_review_and_independent_preparation() {
        let mut dom = VirtualDom::new(MaintenanceNoticesExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "maintenance-notices",
            "notices-bar",
            "notices-detail",
            "notices-review",
            "notices-status",
            "notices-note",
            "notices-archive",
            "notices-options",
            "notices-reset",
            "notices-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("Motor 001 inspection notice") && html.contains(initial_notice().detail)
        );
        assert!(
            html.contains("Review Motor 001")
                && html.contains("Awaiting review")
                && html.contains("No archived notices")
        );
        assert!(
            html.contains("Prepare Motor 001 inspection") && html.contains("for=\"notices-note\"")
        );
        assert!(html.contains("role=\"alert\"") && html.contains("aria-label=\"Close message\""));
        assert!(
            html.contains("data-type=\"warning\"")
                && html.contains("data-reviewed=\"false\"")
                && html.contains("data-archived=\"false\"")
        );
        assert!(
            html.contains("data-close-requests=\"0\"") && html.contains("data-completions=\"0\"")
        );
    }
}
