//! Keyed tab content and composed dropdown/settings/about examples.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn CompositionExamples() -> Element {
    let mut active = use_signal(|| "tab-1".to_string());
    let mut settings_open = use_signal(|| false);
    let mut about_open = use_signal(|| false);
    let mut news_open = use_signal(|| true);
    let mut settings_tab = use_signal(|| "general".to_string());
    let mut about_tab = use_signal(|| "about".to_string());
    let mut notifications = use_signal(|| true);
    let mut last_action = use_signal(|| "No action selected".to_string());
    let tabs = vec![
        TabItem::new("tab-1", "Tab 1"),
        TabItem::new("tab-2", "Tab 2"),
        TabItem::new("tab-3", "Tab 3"),
    ];
    rsx! {
        section { class: "demo-section", id: "tab-set",
            h2 { "Tabs and linked panels" }
            p { "The upstream Tab 1, Tab 2, and Tab 3 example. Panels are associated by stable keys and retain their state while hidden." }
            TabSet { items: tabs, active_key: active(), onselectedchange: move |key| active.set(key),
                TabPanel { tab_key: "tab-1", "Content Tab 1" }
                TabPanel { tab_key: "tab-2", PersistentTabDraft {} }
                TabPanel { tab_key: "tab-3", "Content Tab 3" }
            }
            p { role: "status", "Selected key: {active}" }
        }
        section { class: "demo-section", id: "dropdown-buttons",
            h2 { "Dropdown button" }
            div { class: "demo-row",
                DropdownButton { label: "Dropdown button",
                    DropdownItem { label: "Item 1", onclick: move |_| last_action.set("Item 1".into()) }
                    DropdownItem { label: "Item 2", onclick: move |_| last_action.set("Item 2".into()) }
                }
                DropdownButton { label: "With icon", icon: "cut", variant: ButtonVariant::Secondary,
                    DropdownItem { label: "Cut", icon: "cut", onclick: move |_| last_action.set("Cut".into()) }
                    DropdownItem { label: "Delete", icon: "trashcan", onclick: move |_| last_action.set("Delete".into()) }
                }
                DropdownButton { label: "Disabled", disabled: true, DropdownItem { label: "Item 1" } }
                DropdownButton { label: "Loading", loading: true, DropdownItem { label: "Item 1" } }
            }
        }
        section { class: "demo-section", id: "dropdown-quick-actions",
            h2 { "Dropdown quick actions" }
            p { "The upstream cut, bulb, trashcan, star, and document example, with visible action feedback." }
            Dropdown { label: "Open",
                DropdownQuickActions {
                    DropdownItem { icon: "cut", label: "Cut", compact: true, onclick: move |_| last_action.set("Cut".into()) }
                    DropdownItem { icon: "bulb", label: "Light", compact: true, onclick: move |_| last_action.set("Light".into()) }
                    DropdownItem { icon: "trashcan", label: "Delete", compact: true, onclick: move |_| last_action.set("Delete".into()) }
                }
                Divider {}
                DropdownItem { icon: "star", label: "Item 1", onclick: move |_| last_action.set("Item 1".into()) }
                DropdownItem { icon: "document", label: "Item 2", onclick: move |_| last_action.set("Item 2".into()) }
                DropdownItem { icon: "bulb", label: "Item 3", onclick: move |_| last_action.set("Item 3".into()) }
            }
            p { role: "status", "{last_action}" }
        }
        section { class: "demo-section", id: "settings-about",
            h2 { "Settings and about tabs" }
            p { "Settings and about sections use keyed tabs and panels. This preview contains illustrative application information." }
            div { class: "demo-row",
                Button { onclick: move |_| settings_open.set(!settings_open()), "Toggle settings" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| about_open.set(!about_open()), "Toggle about" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| news_open.set(true), "Restore news" }
            }
            MenuSettings { show: settings_open(), onclose: move |_| settings_open.set(false),
                TabSet { items: vec![TabItem::new("general", "General"), TabItem::new("notifications", "Notifications")],
                    active_key: settings_tab(), onselectedchange: move |key| settings_tab.set(key), label: "Settings sections",
                    TabPanel { tab_key: "general", h3 { "General settings" } p { "Example application settings." } }
                    TabPanel { tab_key: "notifications", Toggle { label: "Enable notifications", checked: notifications(), oncheckedchange: move |value| notifications.set(value) } }
                }
            }
            MenuAbout { show: about_open(), onclose: move |_| about_open.set(false),
                TabSet { items: vec![TabItem::new("about", "About"), TabItem::new("news", "What's new"), TabItem::new("legal", "Legal")],
                    active_key: about_tab(), onselectedchange: move |key| about_tab.set(key), label: "About sections",
                    TabPanel { tab_key: "about", h3 { "Dioxus iX examples" } p { "An independent Rust port review application." } }
                    TabPanel { tab_key: "news", h3 { "Example release note" } p { "This is illustrative news content for reviewing the about panel." } }
                    TabPanel { tab_key: "legal", p { "See the repository README for license and upstream attribution." } }
                }
            }
            MenuAboutNews { label: "What's new", show: news_open(), about_item_label: "Example release note", active_about_tab_key: "news",
                onclose: move |_| news_open.set(false),
                onshowmore: move |key: Option<String>| { about_tab.set(key.unwrap_or_else(|| "news".into())); about_open.set(true); },
                "Illustrative content. Open the associated about tab to review the full example."
            }
        }
    }
}

#[component]
fn PersistentTabDraft() -> Element {
    let mut value = use_signal(String::new);
    rsx! { div { p { "Content Tab 2" }
        Input { label: "Tab 2 draft", value: value(), onvaluechange: move |next| value.set(next) }
    } }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TaskActionRecord {
    id: String,
    title: String,
    note: String,
}

#[component]
fn TaskActionFields(blocked: bool, onadd: EventHandler<(String, String)>) -> Element {
    let mut title = use_signal(|| "Inspect Motor 001".to_string());
    let mut note = use_signal(|| "Check drive coupling".to_string());
    let mut error = use_signal(|| false);
    let title_binding = use_dropdown_item_with_options(
        "task-action-title",
        DropdownItemOptions {
            order: Some(0),
            tabindex: Some(0),
            ..Default::default()
        },
    );
    let note_binding = use_dropdown_item_with_options(
        "task-action-note",
        DropdownItemOptions {
            order: Some(1),
            tabindex: Some(0),
            ..Default::default()
        },
    );
    rsx! {
        div { class: "task-action-fields", onclick: move |event: MouseEvent| event.stop_propagation(),
            label { r#for: "task-action-title", span { "Request title" }
                input { id: "task-action-title", r#type: "text", maxlength: "80", value: title(),
                    "aria-invalid": error().to_string(), "aria-describedby": error().then_some("task-action-error"),
                    onmounted: title_binding.onmounted, tabindex: title_binding.tabindex.or(Some(0)),
                    oninput: move |event: FormEvent| { title.set(event.value()); error.set(false); },
                    onkeydown: move |event: KeyboardEvent| { if !matches!(event.key(), Key::Escape | Key::Tab) { event.stop_propagation(); } },
                    ..title_binding.attributes(),
                }
            }
            label { r#for: "task-action-note", span { "Request note" }
                input { id: "task-action-note", r#type: "text", maxlength: "160", value: note(),
                    onmounted: note_binding.onmounted, tabindex: note_binding.tabindex.or(Some(0)),
                    oninput: move |event: FormEvent| note.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| { if !matches!(event.key(), Key::Escape | Key::Tab) { event.stop_propagation(); } },
                    ..note_binding.attributes(),
                }
            }
            if error() { p { id: "task-action-error", role: "alert", "Enter a request title, then choose Add request." } }
        }
        DropdownItem { id: "task-action-add", label: "Add request", icon: "add", order: 2, disabled: blocked, tabindex: if blocked { "-1" } else { "0" },
            onclick: move |event: MouseEvent| {
                if blocked || title.peek().trim().is_empty() { event.prevent_default(); }
            },
            onitemclick: move |event: DropdownItemClickEvent| {
                if blocked || title.peek().trim().is_empty() {
                    event.prevent_default();
                    if !blocked { error.set(true); }
                } else { onadd.call((title.peek().clone(), note.peek().clone())); }
            },
        }
    }
}

#[component]
fn TaskActionsPreparation() -> Element {
    let mut draft = use_signal(|| "Prepare the local task queue".to_string());
    rsx! { label { class: "task-actions-preparation", r#for: "task-actions-preparation",
        span { "Page preparation" }
        input { id: "task-actions-preparation", r#type: "text", maxlength: "160", value: draft(), oninput: move |event: FormEvent| draft.set(event.value()) }
    } }
}

/// Local request creation through one persistent dropdown, with reactive trigger content.
#[component]
pub fn DropdownTaskActionsExample() -> Element {
    let mut records = use_signal(Vec::<TaskActionRecord>::new);
    let mut next_id = use_signal(|| 1_usize);
    let mut mode = use_signal(|| "normal".to_string());
    let mut placement = use_signal(|| "bottom-start".to_string());
    let mut disabled = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut veto = use_signal(|| false);
    let mut top_layer = use_signal(|| false);
    let mut open = use_signal(|| false);
    let mut requests = use_signal(|| 0_usize);
    let mut changes = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let count = records.read().len();
    let menu_placement = match placement().as_str() {
        "bottom-end" => DropdownPlacement::BottomEnd,
        "top-start" => DropdownPlacement::TopStart,
        "top-end" => DropdownPlacement::TopEnd,
        _ => DropdownPlacement::BottomStart,
    };
    let custom_label = (mode() == "custom").then(|| rsx! {
        span { class: "task-actions-custom-label", "Requests" span { id: "task-actions-badge", "{count}" } }
    });
    rsx! {
        section { id: "dropdown-task-actions", class: "demo-section task-actions-example",
            h2 { "Task queue actions" }
            p { class: "task-actions-intro", "Create illustrative local maintenance requests. The request draft stays in the menu when it closes or its button presentation changes; page preparation is independent." }
            div { class: "task-actions-toolbar",
                Button { id: "task-actions-reset", variant: ButtonVariant::Secondary, onclick: move |_| {
                    records.set(Vec::new()); next_id.set(1); mode.set("normal".into()); placement.set("bottom-start".into());
                    disabled.set(false); loading.set(false); veto.set(false); top_layer.set(false); open.set(false); requests.set(0); changes.set(0);
                    let next = epoch.peek().wrapping_add(1); epoch.set(next);
                }, "Restore task actions" }
                Button { id: "task-actions-clear", variant: ButtonVariant::Secondary, onclick: move |_| records.set(Vec::new()), "Clear requests" }
            }
            details { id: "task-actions-options", class: "task-actions-options",
                summary { "Task action options" }
                div { class: "task-actions-option-fields",
                    label { r#for: "task-actions-mode", span { "Button presentation" }
                        select { id: "task-actions-mode", value: mode(), onchange: move |event: FormEvent| {
                            let value = event.value(); if matches!(value.as_str(), "normal" | "custom" | "icon") { mode.set(value); }
                        },
                            option { value: "normal", selected: mode() == "normal", "Text and icon" }
                            option { value: "custom", selected: mode() == "custom", "Request count label" }
                            option { value: "icon", selected: mode() == "icon", "Icon only" }
                        }
                    }
                    label { r#for: "task-actions-placement", span { "Menu placement" }
                        select { id: "task-actions-placement", value: placement(), onchange: move |event: FormEvent| {
                            let value = event.value(); if matches!(value.as_str(), "bottom-start" | "bottom-end" | "top-start" | "top-end") { placement.set(value); }
                        },
                            option { value: "bottom-start", selected: placement() == "bottom-start", "Below, start" }
                            option { value: "bottom-end", selected: placement() == "bottom-end", "Below, end" }
                            option { value: "top-start", selected: placement() == "top-start", "Above, start" }
                            option { value: "top-end", selected: placement() == "top-end", "Above, end" }
                        }
                    }
                    label { input { id: "task-actions-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disabled" }
                    label { input { id: "task-actions-loading", r#type: "checkbox", checked: loading(), onchange: move |event: FormEvent| loading.set(event.checked()) } "Loading" }
                    label { input { id: "task-actions-veto", r#type: "checkbox", checked: veto(), onchange: move |event: FormEvent| veto.set(event.checked()) } "Prevent opening" }
                    label { input { id: "task-actions-top-layer", r#type: "checkbox", checked: top_layer(), onchange: move |event: FormEvent| top_layer.set(event.checked()) } "Native top layer" }
                }
            }
            for generation in [epoch()] { TaskActionsPreparation { key: "task-preparation-{generation}" } }
            div { id: "task-actions-button", class: "task-actions-button",
                for generation in [epoch()] {
                    DropdownButton { key: "task-dropdown-{generation}", label: if mode() == "icon" { "" } else { "Task actions" }, button_label: custom_label.clone(),
                        icon: "add", aria_label: "Task queue actions", placement: menu_placement, disabled: disabled(), loading: loading(), enable_top_layer: top_layer(),
                        onbeforechange: move |event: DropdownChangeEvent| {
                            let count = requests.peek().saturating_add(1); requests.set(count);
                            if event.open && *veto.peek() { event.prevent_default(); }
                        },
                        onchange: move |visible| { open.set(visible); let count = changes.peek().saturating_add(1); changes.set(count); },
                        TaskActionFields { blocked: disabled() || loading(), onadd: move |(title, note): (String, String)| {
                            if *disabled.peek() || *loading.peek() || title.trim().is_empty() { return; }
                            let number = *next_id.peek(); let mut next = records.peek().clone();
                            next.push(TaskActionRecord { id: format!("task-{number}"), title, note });
                            records.set(next); next_id.set(number.saturating_add(1));
                        } }
                    }
                }
            }
            table { id: "task-action-records", class: "task-action-records",
                caption { "Local maintenance requests" }
                thead { tr { th { scope: "col", "Request" } th { scope: "col", "Preparation note" } } }
                tbody {
                    for record in records() {
                        tr { key: "{record.id}", id: format!("task-record-{}", record.id), "data-task-id": record.id.clone(),
                            th { scope: "row", span { class: "task-action-id", "{record.id}" } span { "data-field": "title", "{record.title}" } }
                            td { "data-field": "note", "{record.note}" }
                        }
                    }
                    if count == 0 { tr { td { colspan: "2", "No requests yet. Open Task queue actions to add one." } } }
                }
            }
            output { id: "task-actions-result", class: "task-actions-result", role: "status",
                "data-count": count.to_string(), "data-next-id": next_id().to_string(), "data-mode": mode(), "data-placement": placement(),
                "data-disabled": disabled().to_string(), "data-loading": loading().to_string(), "data-veto": veto().to_string(), "data-top-layer": top_layer().to_string(),
                "data-open": open().to_string(), "data-change-requests": requests().to_string(), "data-changes": changes().to_string(),
                "{count} local requests saved."
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/composition.css");

#[cfg(test)]
mod task_actions_tests {
    use super::*;

    #[test]
    fn task_actions_renders_empty_real_queue_and_retained_native_draft() {
        let mut dom = VirtualDom::new(DropdownTaskActionsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "dropdown-task-actions",
            "task-actions-button",
            "task-action-title",
            "task-action-note",
            "task-action-add",
            "task-actions-preparation",
            "task-action-records",
            "task-actions-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(html.contains("Inspect Motor 001") && html.contains("Check drive coupling"));
        assert!(html.contains("Prepare the local task queue") && html.contains("No requests yet."));
        assert!(html.contains("data-count=\"0\"") && html.contains("data-next-id=\"1\""));
        assert!(
            html.contains("Task queue actions")
                && html.contains("Task actions")
                && html.contains("Add request")
        );
    }
}
