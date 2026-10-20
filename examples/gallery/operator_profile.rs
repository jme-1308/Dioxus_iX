//! Illustrative operator preferences stored locally, with a retained menu draft.
use dioxus::prelude::*;
use dioxus_ix::*;

const INITIAL_NOTE: &str = "Inspect Motor 001";
const INVALID_IMAGE: &str = "data:image/png;base64,not-a-valid-image";
// An authored geometric badge, rather than a remote portrait or licensed asset.
const BADGE_IMAGE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' fill='%23006673'/%3E%3Ccircle cx='16' cy='16' r='9' fill='%2300cccc'/%3E%3Cpath d='M11 16h10M16 11v10' stroke='%23000028' stroke-width='2'/%3E%3C/svg%3E";

#[derive(Clone, Debug, PartialEq, Eq)]
struct OperatorRecord {
    id: &'static str,
    name: &'static str,
    initials: &'static str,
    role: &'static str,
    available: bool,
    notifications: bool,
    note: String,
}

fn initial_operators() -> Vec<OperatorRecord> {
    vec![
        OperatorRecord {
            id: "alex",
            name: "Alex Taylor",
            initials: "AT",
            role: "Line 1 operator",
            available: true,
            notifications: true,
            note: "No preparation note".into(),
        },
        OperatorRecord {
            id: "jo",
            name: "Jo Rivera",
            initials: "JR",
            role: "Shift coordinator",
            available: false,
            notifications: false,
            note: "No preparation note".into(),
        },
    ]
}

fn availability(available: bool) -> &'static str {
    if available { "Available" } else { "Away" }
}

#[component]
fn OperatorMenuFields(
    notifications: bool,
    available: bool,
    action_disabled: bool,
    onnotifications: EventHandler<bool>,
    onapply: EventHandler<String>,
    onstatus: EventHandler<()>,
) -> Element {
    // This private draft belongs to the menu, independently of the selected operator.
    let mut draft = use_signal(|| INITIAL_NOTE.to_string());
    let note = use_dropdown_item_with_options(
        "profile-note",
        DropdownItemOptions {
            order: Some(0),
            tabindex: Some(0),
            ..Default::default()
        },
    );
    let preference = use_dropdown_item_with_options(
        "profile-notifications",
        DropdownItemOptions {
            order: Some(1),
            tabindex: Some(0),
            checked: notifications,
            ..Default::default()
        },
    );
    rsx! {
        div { class: "profile-menu-fields", onclick: move |event: MouseEvent| event.stop_propagation(),
            label { class: "profile-note-field", r#for: "profile-note",
                span { "Preparation note" }
                input { id: "profile-note", r#type: "text", maxlength: "80", value: draft(),
                    onmounted: note.onmounted, tabindex: note.tabindex.or(Some(0)),
                    oninput: move |event: FormEvent| draft.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if !matches!(event.key(), Key::Escape | Key::Tab) { event.stop_propagation(); }
                    },
                    ..note.attributes(),
                }
            }
            label { class: "profile-notifications-field",
                input { id: "profile-notifications", r#type: "checkbox", checked: notifications,
                    onmounted: preference.onmounted, tabindex: preference.tabindex.or(Some(0)),
                    onchange: move |event: FormEvent| onnotifications.call(event.checked()),
                    onkeydown: move |event: KeyboardEvent| {
                        if !matches!(event.key(), Key::Escape | Key::Tab) { event.stop_propagation(); }
                    },
                    ..preference.attributes(),
                }
                span { "Receive shift notifications" }
            }
        }
        DropdownItem { id: "profile-apply", tabindex: "0", label: "Apply preparation note", order: 2,
            onitemclick: move |_| onapply.call(draft.peek().clone()),
        }
        DropdownItem { id: "profile-status", tabindex: if action_disabled { "-1" } else { "0" },
            label: if available { "Mark as away" } else { "Mark as available" }, order: 3, disabled: action_disabled,
            onitemclick: move |_| { if !action_disabled { onstatus.call(()); } },
        }
    }
}

/// A local profile menu with independent operator settings and a shared preparation draft.
#[component]
pub fn OperatorProfileExample() -> Element {
    let mut operators = use_signal(initial_operators);
    let mut selected = use_signal(|| "alex".to_string());
    let mut disabled = use_signal(|| false);
    let mut action_disabled = use_signal(|| false);
    let mut veto = use_signal(|| false);
    let mut top_layer = use_signal(|| false);
    let mut image_mode = use_signal(|| "initials".to_string());
    let mut menu_open = use_signal(|| false);
    let mut updates = use_signal(|| 0_usize);
    let mut status_actions = use_signal(|| 0_usize);
    let mut preference_changes = use_signal(|| 0_usize);
    let mut change_requests = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);

    let records = operators();
    let operator = records
        .iter()
        .find(|record| record.id == selected())
        .expect("known local operator")
        .clone();
    let image = match image_mode().as_str() {
        "invalid" => Some(INVALID_IMAGE.to_string()),
        "valid" => Some(BADGE_IMAGE.to_string()),
        _ => None,
    };
    let apply_note = use_callback(move |note: String| {
        let key = selected.peek().clone();
        let mut next = operators.peek().clone();
        if let Some(record) = next.iter_mut().find(|record| record.id == key) {
            record.note = note;
            operators.set(next);
            let count = updates.peek().saturating_add(1);
            updates.set(count);
        }
    });
    let set_notifications = use_callback(move |enabled| {
        let key = selected.peek().clone();
        let mut next = operators.peek().clone();
        if let Some(record) = next.iter_mut().find(|record| record.id == key) {
            record.notifications = enabled;
            operators.set(next);
            let count = preference_changes.peek().saturating_add(1);
            preference_changes.set(count);
        }
    });
    let toggle_status = use_callback(move |_| {
        if *action_disabled.peek() {
            return;
        }
        let key = selected.peek().clone();
        let mut next = operators.peek().clone();
        if let Some(record) = next.iter_mut().find(|record| record.id == key) {
            record.available = !record.available;
            operators.set(next);
            let count = status_actions.peek().saturating_add(1);
            status_actions.set(count);
        }
    });

    rsx! {
        section { id: "operator-profile", class: "operator-profile-example",
            h2 { "Operator profile" }
            p { class: "profile-intro", "Manage illustrative local operator settings. Shift notifications save immediately; Apply preparation note saves the menu draft for the current operator. The draft stays when you close the menu or switch operators." }
            div { class: "profile-toolbar",
                Button { id: "profile-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        operators.set(initial_operators()); selected.set("alex".into()); disabled.set(false); action_disabled.set(false);
                        veto.set(false); top_layer.set(false); image_mode.set("initials".into()); menu_open.set(false);
                        updates.set(0); status_actions.set(0); preference_changes.set(0); change_requests.set(0);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore operator profiles"
                }
                label { class: "profile-select-field", r#for: "profile-operator", span { "Operator" }
                    select { id: "profile-operator", value: selected(), onchange: move |event: FormEvent| {
                        let value = event.value(); if value == "alex" || value == "jo" { selected.set(value); }
                    },
                        option { value: "alex", selected: selected() == "alex", "Alex Taylor" }
                        option { value: "jo", selected: selected() == "jo", "Jo Rivera" }
                    }
                }
            }
            details { id: "profile-options", class: "profile-options",
                summary { "Profile options" }
                div { class: "profile-option-fields",
                    label { input { id: "profile-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable profile menu" }
                    label { input { id: "profile-action-disabled", r#type: "checkbox", checked: action_disabled(), onchange: move |event: FormEvent| action_disabled.set(event.checked()) } "Disable status action" }
                    label { input { id: "profile-veto", r#type: "checkbox", checked: veto(), onchange: move |event: FormEvent| veto.set(event.checked()) } "Veto menu changes" }
                    label { input { id: "profile-top-layer", r#type: "checkbox", checked: top_layer(), onchange: move |event: FormEvent| top_layer.set(event.checked()) } "Native top layer" }
                    label { class: "profile-select-field", r#for: "profile-image", span { "Profile image" }
                        select { id: "profile-image", value: image_mode(), onchange: move |event: FormEvent| image_mode.set(event.value()),
                            option { value: "initials", selected: image_mode() == "initials", "Initials" }
                            option { value: "invalid", selected: image_mode() == "invalid", "Unavailable image" }
                            option { value: "valid", selected: image_mode() == "valid", "Geometric badge" }
                        }
                    }
                }
            }
            div { class: "profile-identity-bar",
                for generation in [epoch()] {
                    Avatar { key: "operator-profile-{generation}", id: "profile-avatar", class: "operator-profile-avatar",
                        username: operator.name, initials: operator.initials, extra: operator.role, image: image.clone(),
                        menu: true, disabled: disabled(), aria_label: "Open operator profile", menu_label: "Operator profile actions",
                        enable_top_layer: top_layer(), navigation_mode: DropdownNavigationMode::ActiveDescendant,
                        onbeforechange: move |event: DropdownChangeEvent| {
                            let count = change_requests.peek().saturating_add(1); change_requests.set(count);
                            if *veto.peek() { event.prevent_default(); }
                        },
                        onchange: move |open| menu_open.set(open),
                        OperatorMenuFields { notifications: operator.notifications, available: operator.available, action_disabled: action_disabled(),
                            onnotifications: set_notifications, onapply: apply_note, onstatus: toggle_status,
                        }
                    }
                }
                div { class: "profile-current-identity",
                    strong { id: "profile-current-name", "{operator.name}" }
                    span { id: "profile-current-role", "{operator.role}" }
                    span { id: "profile-current-status", "{availability(operator.available)}" }
                }
            }
            table { id: "profile-records", class: "profile-records",
                caption { "Saved operator settings" }
                thead { tr { th { scope: "col", "Operator" } th { scope: "col", "Saved settings" } } }
                tbody {
                    for record in records {
                        tr { key: "{record.id}", id: format!("profile-record-{}", record.id), "data-operator": record.id,
                            "data-status": availability(record.available), "data-notifications": record.notifications.to_string(),
                            th { scope: "row", "{record.name}" }
                            td {
                                span { "data-field": "status", "{availability(record.available)}" }
                                span { "Notifications: " span { "data-field": "notifications", if record.notifications { "On" } else { "Off" } } }
                                span { "data-field": "note", "{record.note}" }
                            }
                        }
                    }
                }
            }
            output { id: "profile-result", class: "profile-result", role: "status",
                "data-operator": selected(), "data-menu-open": menu_open().to_string(), "data-updates": updates().to_string(),
                "data-status-actions": status_actions().to_string(), "data-preference-changes": preference_changes().to_string(),
                "data-change-requests": change_requests().to_string(), "data-image-mode": image_mode(),
                "Preparation notes saved: {updates} · Status changes: {status_actions} · Notification changes: {preference_changes}"
            }
        }
    }
}

pub const STYLES: &str = r#"
.operator-profile-example{min-width:0;max-width:100%;margin-bottom:48px}.profile-intro{max-width:72ch;line-height:1.55}.profile-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;margin:20px 0 16px;min-width:0}.profile-toolbar .ix-button{max-width:100%;white-space:normal}.profile-select-field{display:flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;max-width:100%;font-size:14px;line-height:20px}.profile-select-field select{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.profile-options{margin:0 0 24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.profile-options>summary{padding:10px 12px;font-size:14px;font-weight:600;cursor:pointer}.profile-option-fields{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;padding:12px;min-width:0}.profile-option-fields>label,.profile-notifications-field{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;font-size:14px;line-height:20px}.profile-option-fields input,.profile-notifications-field input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.profile-identity-bar{display:flex;align-items:center;gap:16px;flex-wrap:wrap;min-width:0;margin:0 0 24px}.profile-current-identity{display:flex;flex-direction:column;gap:4px;min-width:0}.profile-current-identity strong{font-size:18px;line-height:24px}.profile-current-identity>span{font-size:14px;line-height:20px;color:var(--ix-color-text)}.profile-menu-fields{box-sizing:border-box;width:288px;max-width:100%;display:flex;flex-direction:column;gap:12px;padding:8px 16px 12px;color:var(--ix-color-text)}.profile-note-field{display:flex;flex-direction:column;gap:8px;min-width:0;font-size:14px;line-height:20px}.profile-note-field input{box-sizing:border-box;width:100%;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.profile-notifications-field span{min-width:0;overflow-wrap:anywhere}.profile-records{width:100%;max-width:640px;border-collapse:collapse;table-layout:fixed;font-size:14px;line-height:20px;color:var(--ix-color-text)}.profile-records caption{text-align:start;font-weight:600;padding-bottom:8px}.profile-records th,.profile-records td{padding:8px;border-bottom:1px solid var(--ix-color-border);text-align:start;vertical-align:top;overflow-wrap:anywhere}.profile-records thead th{background:var(--ix-color-surface-1)}.profile-records th:first-child{width:34%}.profile-records td>span{display:block;margin-bottom:4px}.profile-result{display:block;margin-top:16px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}.profile-note-field input:focus-visible,.profile-select-field select:focus-visible,.profile-options>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}.profile-option-fields input:focus-visible,.profile-notifications-field input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_profile_renders_actual_records_and_registered_menu_controls() {
        let mut dom = VirtualDom::new(OperatorProfileExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "operator-profile",
            "profile-avatar",
            "profile-operator",
            "profile-note",
            "profile-notifications",
            "profile-apply",
            "profile-status",
            "profile-record-alex",
            "profile-record-jo",
            "profile-result",
            "profile-image",
            "profile-reset",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("Alex Taylor")
                && html.contains("Jo Rivera")
                && html.contains("Line 1 operator")
        );
        assert!(
            html.contains("Preparation note")
                && html.contains(INITIAL_NOTE)
                && html.contains("Receive shift notifications")
        );
        assert!(
            html.contains("data-status=\"Available\"") && html.contains("data-status=\"Away\"")
        );
        assert!(
            html.contains("data-notifications=\"true\"")
                && html.contains("data-notifications=\"false\"")
        );
        assert!(html.matches("No preparation note").count() == 2);
        assert!(
            html.contains("aria-haspopup=\"menu\"")
                && html.contains("Open operator profile")
                && html.contains("Operator profile actions")
        );
        assert!(html.contains("data-ix-item-key=") && html.contains("role=\"separator\""));
        assert!(html.contains("data-menu-open=\"false\"") && html.contains("data-updates=\"0\""));
    }
}
