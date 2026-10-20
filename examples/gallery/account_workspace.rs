//! Local account settings composed through the application's shared account menu.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct AccountRecord {
    id: &'static str,
    name: &'static str,
    role: &'static str,
    initials: &'static str,
    notifications: bool,
    available: bool,
    preparation: String,
}

fn initial_accounts() -> Vec<AccountRecord> {
    vec![
        AccountRecord {
            id: "alex",
            name: "Alex Taylor",
            role: "Line 1 operator",
            initials: "AT",
            notifications: true,
            available: true,
            preparation: "No applied preparation".into(),
        },
        AccountRecord {
            id: "jo",
            name: "Jo Rivera",
            role: "Shift coordinator",
            initials: "JR",
            notifications: false,
            available: false,
            preparation: "No applied preparation".into(),
        },
    ]
}

fn status(available: bool) -> &'static str {
    if available { "Available" } else { "Away" }
}

#[component]
fn AccountMenuFields(
    notifications: bool,
    available: bool,
    action_disabled: bool,
    onnotifications: EventHandler<bool>,
    onapply: EventHandler<String>,
    onstatus: EventHandler<()>,
) -> Element {
    let mut preparation = use_signal(|| "Prepare Motor 001 review".to_string());
    let note = use_dropdown_item_with_options(
        "account-note",
        DropdownItemOptions {
            order: Some(0),
            ..Default::default()
        },
    );
    let preference = use_dropdown_item_with_options(
        "account-notifications",
        DropdownItemOptions {
            order: Some(1),
            checked: notifications,
            ..Default::default()
        },
    );
    rsx! {
        div { class: "account-menu-fields", onclick: move |event: MouseEvent| event.stop_propagation(),
            label { r#for: "account-note", span { "Account preparation" }
                input { id: "account-note", r#type: "text", maxlength: "80", value: preparation(),
                    onmounted: note.onmounted, tabindex: note.tabindex,
                    oninput: move |event: FormEvent| preparation.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| { if !matches!(event.key(), Key::Escape | Key::Tab) { event.stop_propagation(); } },
                    ..note.attributes(),
                }
            }
            label { class: "account-notifications-field",
                input { id: "account-notifications", r#type: "checkbox", checked: notifications,
                    onmounted: preference.onmounted, tabindex: preference.tabindex, onkeydown: preference.onkeydown,
                    onchange: move |event: FormEvent| onnotifications.call(event.checked()), ..preference.attributes(),
                }
                span { "Receive account notifications" }
            }
        }
        div { id: "account-apply", MenuAvatarItem { label: "Apply preparation", icon: "check", onclick: move |_| onapply.call(preparation.peek().clone()) } }
        div { id: "account-status", MenuAvatarItem { label: if available { "Mark as away" } else { "Mark as available" }, disabled: action_disabled,
            onclick: move |_| { if !action_disabled { onstatus.call(()); } },
        } }
    }
}

#[component]
fn AccountPreparation() -> Element {
    let mut preparation = use_signal(|| "Prepare account settings review".to_string());
    rsx! {
        label { class: "account-preparation", r#for: "account-preparation",
            span { "Page preparation" }
            input { id: "account-preparation", r#type: "text", maxlength: "160", value: preparation(), oninput: move |event: FormEvent| preparation.set(event.value()) }
        }
    }
}

/// An illustrative account menu whose preferences and sign-in state stay local.
#[component]
pub fn AccountWorkspaceExample() -> Element {
    let mut accounts = use_signal(initial_accounts);
    let mut selected = use_signal(|| "alex".to_string());
    let mut signed_in = use_signal(|| true);
    let mut german = use_signal(|| false);
    let mut hide_logout = use_signal(|| false);
    let mut hide_actions = use_signal(|| false);
    let mut disable_action = use_signal(|| false);
    let mut top_layer = use_signal(|| false);
    let mut menu_open = use_signal(|| false);
    let mut applies = use_signal(|| 0_usize);
    let mut status_actions = use_signal(|| 0_usize);
    let mut notification_changes = use_signal(|| 0_usize);
    let mut logouts = use_signal(|| 0_usize);
    let mut change_requests = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let mut user_field = use_signal(|| None::<Rc<MountedData>>);
    let mut sign_in_button = use_signal(|| None::<Rc<MountedData>>);
    let mut focus_recovery = use_signal(|| false);
    // Only the two local authentication transitions request focus; user changes do not.
    use_effect(move || {
        let recover = focus_recovery();
        let active = signed_in();
        let user = user_field();
        let sign_in = sign_in_button();
        if recover && let Some(node) = if active { user } else { sign_in } {
            focus_recovery.set(false);
            spawn(async move {
                if *signed_in.peek() == active {
                    let _ = node.set_focus(true).await;
                }
            });
        }
    });
    let records = accounts();
    let current = records
        .iter()
        .find(|account| account.id == selected())
        .expect("known sample account")
        .clone();
    let apply = use_callback(move |preparation: String| {
        if !*signed_in.peek() {
            return;
        }
        let mut next = accounts.peek().clone();
        let key = selected.peek().clone();
        if let Some(account) = next.iter_mut().find(|account| account.id == key) {
            account.preparation = preparation;
            accounts.set(next);
            let count = applies.peek().saturating_add(1);
            applies.set(count);
        }
    });
    let notifications = use_callback(move |enabled| {
        if !*signed_in.peek() {
            return;
        }
        let mut next = accounts.peek().clone();
        let key = selected.peek().clone();
        if let Some(account) = next.iter_mut().find(|account| account.id == key) {
            account.notifications = enabled;
            accounts.set(next);
            let count = notification_changes.peek().saturating_add(1);
            notification_changes.set(count);
        }
    });
    let toggle_status = use_callback(move |_| {
        if !*signed_in.peek() || *disable_action.peek() {
            return;
        }
        let mut next = accounts.peek().clone();
        let key = selected.peek().clone();
        if let Some(account) = next.iter_mut().find(|account| account.id == key) {
            account.available = !account.available;
            accounts.set(next);
            let count = status_actions.peek().saturating_add(1);
            status_actions.set(count);
        }
    });
    let custom_actions = (!hide_actions()).then(|| rsx! {
        AccountMenuFields { notifications: current.notifications, available: current.available, action_disabled: disable_action(),
            onnotifications: notifications, onapply: apply, onstatus: toggle_status,
        }
    });
    rsx! {
        section { id: "account-workspace", class: "account-workspace-example",
            style { {STYLES} }
            h2 { "Account workspace" }
            p { class: "account-intro", "Manage two illustrative local accounts. Notifications and availability update the selected account; Apply preparation saves the menu draft. Logout only signs out of this example, and saved settings remain available when you sign in again." }
            div { class: "account-toolbar",
                Button { id: "account-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        accounts.set(initial_accounts()); selected.set("alex".into()); signed_in.set(true); german.set(false); hide_logout.set(false);
                        hide_actions.set(false); disable_action.set(false); top_layer.set(false); menu_open.set(false); applies.set(0);
                        status_actions.set(0); notification_changes.set(0); logouts.set(0); change_requests.set(0);
                        focus_recovery.set(false); sign_in_button.set(None); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore account workspace"
                }
                label { class: "account-select", r#for: "account-user", span { "Sample account" }
                    select { id: "account-user", value: selected(), disabled: !signed_in(), onmounted: move |event: MountedEvent| user_field.set(Some(event.data())),
                        onchange: move |event: FormEvent| {
                            let value = event.value(); if *signed_in.peek() && (value == "alex" || value == "jo") { selected.set(value); }
                        },
                        option { value: "alex", selected: selected() == "alex", "Alex Taylor" }
                        option { value: "jo", selected: selected() == "jo", "Jo Rivera" }
                    }
                }
            }
            details { id: "account-options", class: "account-options",
                summary { "Account menu options" }
                div { class: "account-option-fields",
                    label { input { id: "account-german", r#type: "checkbox", checked: german(), onchange: move |event: FormEvent| german.set(event.checked()) } "German logout and tooltip labels" }
                    label { input { id: "account-hide-logout", r#type: "checkbox", checked: hide_logout(), onchange: move |event: FormEvent| hide_logout.set(event.checked()) } "Hide logout" }
                    label { input { id: "account-hide-actions", r#type: "checkbox", checked: hide_actions(), onchange: move |event: FormEvent| hide_actions.set(event.checked()) } "Hide custom actions" }
                    label { input { id: "account-disable-action", r#type: "checkbox", checked: disable_action(), onchange: move |event: FormEvent| disable_action.set(event.checked()) } "Disable status action" }
                    label { input { id: "account-top-layer", r#type: "checkbox", checked: top_layer(), onchange: move |event: FormEvent| top_layer.set(event.checked()) } "Native top layer" }
                }
            }
            for generation in [epoch()] { AccountPreparation { key: "account-preparation-{generation}" } }
            if signed_in() {
                div { id: "account-menu-container", class: "account-menu-container",
                    for generation in [epoch()] {
                        MenuAvatar { key: "local-account-menu-{generation}", id: "account-menu", top: current.name, bottom: current.role, initials: current.initials,
                            tooltip_text: format!("{} · {}", current.name, current.role), aria_label_tooltip: if german() { "Kontoinformationen" } else { "Account information" },
                            i18n_logout: if german() { "Abmelden" } else { "Logout" }, hide_logout_button: hide_logout(), enable_top_layer: top_layer(), children: custom_actions.clone(),
                            onbeforechange: move |_| { let count = change_requests.peek().saturating_add(1); change_requests.set(count); },
                            onchange: move |open| menu_open.set(open),
                            onlogout: move |_| {
                                if *signed_in.peek() {
                                    signed_in.set(false); menu_open.set(false); focus_recovery.set(true);
                                    let count = logouts.peek().saturating_add(1); logouts.set(count);
                                }
                            },
                        }
                    }
                }
            } else {
                div { id: "account-signed-out", class: "account-signed-out",
                    p { "Signed out of the local example. Your sample accounts and saved settings are retained." }
                    button { id: "account-sign-in", class: "ix-button ix-button-primary", r#type: "button",
                        onmounted: move |event: MountedEvent| sign_in_button.set(Some(event.data())),
                        onclick: move |_| { signed_in.set(true); sign_in_button.set(None); focus_recovery.set(true); }, "Sign in sample"
                    }
                }
            }
            table { id: "account-records", class: "account-records",
                caption { "Saved local account settings" }
                thead { tr { th { scope: "col", "Account" } th { scope: "col", "Saved settings" } } }
                tbody {
                    for account in records {
                        tr { key: "{account.id}", id: format!("account-record-{}", account.id), "data-user": account.id,
                            th { scope: "row", "{account.name}" }
                            td {
                                span { "Notifications: " span { "data-field": "notifications", if account.notifications { "On" } else { "Off" } } }
                                span { "data-field": "status", "{status(account.available)}" }
                                span { "data-field": "note", "{account.preparation}" }
                            }
                        }
                    }
                }
            }
            output { id: "account-result", class: "account-result", role: "status",
                "data-user": selected(), "data-signed-in": signed_in().to_string(), "data-menu-open": menu_open().to_string(), "data-applies": applies().to_string(),
                "data-status-actions": status_actions().to_string(), "data-notification-changes": notification_changes().to_string(), "data-logouts": logouts().to_string(), "data-change-requests": change_requests().to_string(),
                "data-german": german().to_string(), "data-hide-logout": hide_logout().to_string(), "data-hide-actions": hide_actions().to_string(), "data-disable-action": disable_action().to_string(), "data-top-layer": top_layer().to_string(),
                if signed_in() { "Signed in locally as {current.name}." } else { "Signed out locally." }
            }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.account-workspace-example{min-width:0;max-width:100%;margin-bottom:48px}.account-intro{max-width:72ch;line-height:1.55}.account-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;margin:20px 0 16px;min-width:0}.account-toolbar .ix-button{max-width:100%;white-space:normal}.account-select{display:flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;font-size:14px;line-height:20px}.account-select select{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.account-options{margin:0 0 24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.account-options>summary{padding:10px 12px;font-size:14px;font-weight:600;cursor:pointer}.account-option-fields{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;padding:12px;min-width:0}.account-option-fields>label{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;font-size:14px;line-height:20px}.account-option-fields input,.account-notifications-field input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.account-preparation{display:flex;flex-direction:column;gap:8px;min-width:0;width:100%;max-width:480px;margin-bottom:24px;font-size:14px;line-height:20px;color:var(--ix-color-text)}.account-preparation input,.account-menu-fields input[type=text]{box-sizing:border-box;width:100%;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.account-menu-container{width:280px;max-width:100%;min-width:0;margin-bottom:24px}.account-menu-fields{display:flex;flex-direction:column;gap:12px;padding:12px 16px;color:var(--ix-color-text);min-width:0}.account-menu-fields>label{display:flex;flex-direction:column;gap:8px;min-width:0;font-size:14px;line-height:20px}.account-menu-fields>.account-notifications-field{flex-direction:row;align-items:flex-start;gap:8px}.account-notifications-field span{min-width:0;overflow-wrap:anywhere}.account-signed-out{max-width:72ch;margin-bottom:24px;line-height:1.55}.account-records{width:100%;max-width:720px;table-layout:fixed;border-collapse:collapse;font-size:14px;line-height:20px;color:var(--ix-color-text)}.account-records caption{text-align:start;font-weight:600;padding-bottom:8px}.account-records th,.account-records td{padding:8px;border-bottom:1px solid var(--ix-color-border);text-align:start;vertical-align:top;overflow-wrap:anywhere}.account-records thead th{background:var(--ix-color-surface-1)}.account-records th:first-child{width:30%}.account-records td>span{display:block;margin-bottom:4px}.account-result{display:block;margin-top:24px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}.account-preparation input:focus-visible,.account-menu-fields input:focus-visible,.account-select select:focus-visible,.account-options>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.account-option-fields input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_workspace_renders_real_preferences_and_retained_native_controls() {
        let mut dom = VirtualDom::new(AccountWorkspaceExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "account-workspace",
            "account-menu",
            "account-user",
            "account-note",
            "account-notifications",
            "account-apply",
            "account-status",
            "account-preparation",
            "account-record-alex",
            "account-record-jo",
            "account-result",
            "account-reset",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("Alex Taylor")
                && html.contains("Jo Rivera")
                && html.contains("Line 1 operator")
        );
        assert!(
            html.contains("Prepare Motor 001 review")
                && html.contains("Prepare account settings review")
        );
        assert!(
            html.matches("No applied preparation").count() == 2
                && html.contains("Available")
                && html.contains("Away")
        );
        assert!(
            html.contains("aria-haspopup=\"menu\"")
                && html.contains("Logout")
                && html.contains("Apply preparation")
        );
        assert!(html.contains("data-signed-in=\"true\"") && html.contains("data-logouts=\"0\""));
    }
}
