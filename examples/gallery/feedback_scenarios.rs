//! Rust counterparts of individual upstream feedback preview sources.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{cell::Cell, rc::Rc};

pub const SCENARIOS: &[&str] = &[
    "dropdown",
    "dropdown-button",
    "dropdown-button-icon",
    "dropdown-icon",
    "dropdown-quick-actions",
    "dropdown-roving-tabindex",
    "dropdown-submenu",
    "message",
    "message-bar",
    "message-bar-removal",
    "toast",
    "toast-custom",
    "toast-position",
    "tooltip",
    "tooltip-with-icon",
];

#[component]
pub fn FeedbackScenario(name: String) -> Element {
    let mut selected = use_signal(|| "No event yet".to_string());
    let mut visible = use_signal(|| true);
    let mut generation = use_signal(|| 0);
    let notifications = use_toast();
    let modal = use_modal();
    let dropdown = use_dropdown();
    let closed = use_callback(move |reason: ToastCloseReason| selected.set(format!("{reason:?}")));
    let is_custom = name == "toast-custom";
    let top_right = name == "toast-position";
    use_effect(use_reactive!(|top_right| notifications.set_position(
        if top_right {
            ToastPosition::TopRight
        } else {
            ToastPosition::BottomRight
        }
    )));
    let toast = move |with_action: bool| {
        let action_handle = Rc::new(Cell::new(None::<ToastHandle>));
        let mut options = ToastOptions::new("My toast message!");
        options.onclose = Some(closed);
        if is_custom {
            options.title = "Toast headline".into();
            options.message = rsx! { div { "This message is from template" } };
            if with_action {
                let target = action_handle.clone();
                options.action = Some(
                    rsx! { Button { icon: "undo", variant: ButtonVariant::Tertiary,
                    onclick: move |_| { if let Some(handle) = target.get() { handle.close(Some("undo".into())); } }, "Undo" } },
                );
            } else {
                options.message_type = MessageType::Success;
            }
        }
        let handle = notifications.show(options);
        action_handle.set(Some(handle));
    };
    let content = match name.as_str() {
        "toast" | "toast-position" => {
            rsx! { Button { onclick: move |_| toast(false), "Trigger toast" } }
        }
        "toast-custom" => rsx! { div { class: "source-feedback-buttons",
            Button { onclick: move |_| toast(false), "Trigger toast with custom message" }
            Button { onclick: move |_| toast(true), "Trigger toast with action button" }
        } },
        "message" => rsx! {
            ModalTrigger { modal, "Show 'success' message" }
            Modal { modal, label: "Example title", ondialogclose: move |value| selected.set(value),
                ModalHeader { icon: "success", "Example title" }
                ModalContent { "message" }
                ModalFooter {
                    ModalClose { value: "payload:cancel", variant: ButtonVariant::SubtlePrimary, "Cancel" }
                    ModalClose { value: "payload:save", autofocus: true, "Save" }
                }
            }
        },
        "message-bar-removal" => rsx! {
            if visible() { MessageBar { key: "{generation}", onclose: move |_| visible.set(false), "Message text" } }
            else { Button { onclick: move |_| { generation += 1; visible.set(true); }, "Show message bar" } }
        },
        "message-bar" => rsx! { div { class: "demo-stack",
            MessageBar { persistent: true, "Message text" }
            MessageBar { persistent: true, message_type: MessageType::Warning, "Message text" }
            MessageBar { persistent: true, message_type: MessageType::Alarm, div { class: "source-feedback-buttons", "Message text" Button { onclick: move |_| selected.set("Action".into()), "Action" } } }
        } },
        "tooltip" => rsx! { div { class: "source-feedback-buttons",
            Tooltip { id: "source-tooltip", label: "Simple selector", Button { aria_describedby: "source-tooltip", "Hover me" } }
            Tooltip { id: "source-tooltip-custom", label: "Custom selector", Button { aria_describedby: "source-tooltip-custom", "Also hover me" } }
        } },
        "tooltip-with-icon" => {
            rsx! { Tooltip { id: "source-tooltip", title: "Tooltip Title", title_icon: "info", label: "Simple selector with icon", Button { aria_describedby: "source-tooltip", "Hover me" } } }
        }
        "dropdown-button" | "dropdown-button-icon" => {
            rsx! { div { class: "source-feedback-buttons",
                for (label, variant, disabled) in [("Primary dropdown",ButtonVariant::Primary,false),("Subtle primary dropdown",ButtonVariant::SubtlePrimary,false),("Tertiary dropdown",ButtonVariant::Tertiary,false),("Disabled dropdown",ButtonVariant::Primary,true)] {
                    DropdownButton { label: if name.ends_with("-icon") { String::new() } else { label.into() }, aria_label: label,
                        icon: "checkboxes", variant, disabled,
                        if !disabled {
                            DropdownItem { label: "Item 1", onclick: move |_| selected.set("Item 1".into()) }
                            DropdownItem { label: "Item 2", onclick: move |_| selected.set("Item 2".into()) }
                        }
                    }
                }
            } }
        }
        "dropdown-quick-actions" => {
            rsx! { DropdownTrigger { dropdown, "Open" } Dropdown { controller: dropdown, label: "Open", variant: ButtonVariant::Primary,
                DropdownQuickActions {
                    IconButton { icon: "cut", label: "Cut", variant: ButtonVariant::Tertiary, onclick: move |_| selected.set("Cut".into()) }
                    IconButton { icon: "bulb", label: "Light", variant: ButtonVariant::Tertiary, onclick: move |_| selected.set("Light".into()) }
                    IconButton { icon: "trashcan", label: "Delete", variant: ButtonVariant::Tertiary, onclick: move |_| selected.set("Delete".into()) }
                }
                Divider {}
                DropdownItem { icon: "star", label: "Item 1", onitemclick: move |_| selected.set("Item 1".into()) }
                DropdownItem { icon: "document", label: "Item 2", onitemclick: move |_| selected.set("Item 2".into()) }
                DropdownItem { icon: "bulb", label: "Item 3", onitemclick: move |_| selected.set("Item 3".into()) }
            } }
        }
        "dropdown-icon" => {
            rsx! { DropdownTrigger { dropdown, "Open" } Dropdown { controller: dropdown, label: "Open", variant: ButtonVariant::Primary,
                DropdownItem { icon: "star", label: "Item 1", onclick: move |_| selected.set("Item 1".into()) }
                DropdownItem { icon: "document", label: "Item 2", onclick: move |_| selected.set("Item 2".into()) }
                DropdownItem { icon: "bulb", label: "Item 3", onclick: move |_| selected.set("Item 3".into()) }
            } }
        }
        "dropdown-submenu" => {
            rsx! { DropdownTrigger { dropdown, "Open" } Dropdown { controller: dropdown, label: "Open", variant: ButtonVariant::Primary,
                DropdownSubmenu { label: "Submenu",
                    DropdownItem { icon: "star", label: "Item 1", onclick: move |_| selected.set("Submenu: Item 1".into()) }
                    DropdownItem { icon: "document", label: "Item 2", onclick: move |_| selected.set("Submenu: Item 2".into()) }
                }
                DropdownItem { icon: "star", label: "Item 1", onclick: move |_| selected.set("Item 1".into()) }
                DropdownItem { icon: "document", label: "Item 2", onclick: move |_| selected.set("Item 2".into()) }
                DropdownItem { icon: "bulb", label: "Item 3", onclick: move |_| selected.set("Item 3".into()) }
            } }
        }
        "dropdown" | "dropdown-roving-tabindex" => {
            rsx! { DropdownTrigger { dropdown, "Open" } Dropdown { controller: dropdown, label: "Open", variant: ButtonVariant::Primary,
                navigation_mode: if name == "dropdown-roving-tabindex" { DropdownNavigationMode::RovingTabindex } else { DropdownNavigationMode::ActiveDescendant },
                DropdownHeader { label: "Category" }
                DropdownItem { label: "Item 2", onitemclick: move |_| selected.set("Item 2".into()) }
                DropdownItem { label: "Item 3", onitemclick: move |_| selected.set("Item 3".into()) }
                DropdownItem { label: "Item 4", onitemclick: move |_| selected.set("Item 4".into()) }
                Divider {}
                DropdownItem { label: "Item 5", onitemclick: move |_| selected.set("Item 5".into()) }
            } }
        }
        _ => rsx! { p { "This feedback source has not been reproduced yet." } },
    };
    rsx! { {content} ToastContainer { controller: notifications }
        p { class: "source-event-feedback", role: "status", "{selected}" }
    }
}

pub const STYLES: &str = include_str!("styles/feedback_scenarios.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_feedback_source_renders_without_unknown_dispatch_or_eager_notifications() {
        for name in SCENARIOS {
            let props = FeedbackScenarioProps {
                name: (*name).to_string(),
            };
            let mut dom = VirtualDom::new_with_props(FeedbackScenario, props);
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(!html.contains("has not been reproduced"), "{name}");
            assert!(html.contains("No event yet"), "{name}");
            assert!(
                !html.contains("ix-managed-toast"),
                "Notifications must wait for activation: {name}"
            );
        }
    }
}
