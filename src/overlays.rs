//! Feedback primitives ported from iX behavior and examples.
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.
use crate::Icon;
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

pub use crate::dropdown::*;
static MENU_ITEM_ID: AtomicUsize = AtomicUsize::new(1);

/// The seven contextual notification types from the upstream message bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageType {
    Alarm,
    Critical,
    Warning,
    Success,
    #[default]
    Info,
    Neutral,
    Primary,
}

impl MessageType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Alarm => "alarm",
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Success => "success",
            Self::Info => "info",
            Self::Neutral => "neutral",
            Self::Primary => "primary",
        }
    }
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Alarm => "error",
            Self::Critical => "warning-rhomb",
            Self::Warning => "warning",
            Self::Success => "success",
            Self::Info => "info",
            Self::Neutral | Self::Primary => "notification",
        }
    }
}

/// A synchronous dismissal request. Clones share the same cancellation state.
#[derive(Clone, Debug, Default)]
pub struct MessageBarCloseRequest {
    prevented: Rc<Cell<bool>>,
}
impl MessageBarCloseRequest {
    /// Keep the message visible and suppress accepted/completed notifications.
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

/// Inline contextual message. A persistent message cannot be dismissed.
///
/// `oncloserequest` can veto dismissal. `onclose` reports accepted dismissal;
/// `onclosecomplete` runs once after the closed Dioxus render. Completion is not
/// an animation event, and is skipped if the parent unmounts this component.
#[component]
pub fn MessageBar(
    #[props(default)] message_type: MessageType,
    #[props(default)] persistent: bool,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default)] oncloserequest: Option<EventHandler<MessageBarCloseRequest>>,
    #[props(default)] onclosecomplete: Option<EventHandler<()>>,
    #[props(default = "Close message".to_string())] close_label: String,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let mut closed = use_signal(|| false);
    let completed = use_hook(|| Rc::new(Cell::new(false)));
    let is_closed = closed();
    // Track the rendered snapshot, rather than observing the signal before its
    // closed render. Pinned use_effect queues work after that render.
    use_effect(use_reactive((&is_closed,), move |(is_closed,)| {
        if is_closed && !completed.replace(true) {
            if let Some(handler) = onclosecomplete {
                handler.call(());
            }
        }
    }));
    if is_closed {
        return VNode::empty();
    }
    rsx! {
        div {
            class: "ix-message-bar {class}",
            "data-type": message_type.as_str(),
            role: "alert",
            Icon { name: message_type.icon() }
            div { class: "ix-message-bar-content", {children} }
            if !persistent {
                button {
                    class: "ix-message-close", r#type: "button",
                    "aria-label": close_label,
                    onclick: move |_| {
                        if persistent || *closed.peek() { return; }
                        let request = MessageBarCloseRequest::default();
                        if let Some(handler) = oncloserequest { handler.call(request.clone()); }
                        if request.default_prevented() { return; }
                        closed.set(true);
                        if let Some(handler) = onclose { handler.call(()); }
                    },
                    Icon { name: "close", size: 20 }
                }
            }
        }
    }
}

/// A controlled toast surface. Use [`crate::ToastContainer`] with
/// [`crate::use_toast`] for stacking, automatic dismissal, and pause/resume.
#[component]
pub fn Toast(
    #[props(default)] title: String,
    #[props(default)] message_type: MessageType,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default)] children: Option<Element>,
    icon: Option<String>,
    icon_color: Option<String>,
    #[props(default)] hide_icon: bool,
    action: Option<Element>,
    #[props(default = "Close notification".to_string())] close_label: String,
) -> Element {
    rsx! {
        section { class: "ix-toast", "data-type": message_type.as_str(), role: "status",
            if !hide_icon { span { class: "ix-toast-icon", style: icon_color.map(|color| format!("color:var({color})")), Icon { name: icon.unwrap_or_else(|| message_type.icon().into()) } } }
            div { class: "ix-toast-body", if !title.is_empty() { strong { "{title}" } } {children} if let Some(action) = action { div { class: "ix-toast-action", {action} } } }
            if let Some(handler) = onclose {
                button { r#type: "button", class: "ix-message-close", "aria-label": close_label,
                    onclick: move |_| handler.call(()), Icon { name: "close", size: 20 }
                }
            }
        }
    }
}

/// Tooltip shown on hover/focus, rendered in the native popover top layer.
/// Use [`crate::TooltipFor`] for external or multiple anchors.
#[component]
pub fn Tooltip(
    id: String,
    label: String,
    title: Option<String>,
    title_icon: Option<String>,
    #[props(default)] class: String,
    #[props(default)] placement: crate::TooltipPlacement,
    #[props(default)] show_delay: u64,
    #[props(default = 50)] hide_delay: u64,
    #[props(default)] interactive: bool,
    #[props(default)] animation_frame: bool,
    #[props(default)] onnativeerror: EventHandler<String>,
    children: Element,
) -> Element {
    let anchor_id = use_hook(|| {
        format!(
            "ix-tooltip-anchor-{}",
            MENU_ITEM_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let controller = crate::use_tooltip(id.clone());
    let binding = crate::use_tooltip_anchor(controller.clone(), anchor_id.clone());
    let describedby = binding.describedby("");
    rsx! {
        span { id: anchor_id.clone(), class: "ix-tooltip-anchor {class}",
            "aria-describedby":describedby,
            onmounted:binding.onmounted,onresize:binding.onresize,onvisible:binding.onvisible,
            onmouseenter:binding.onmouseenter,onmouseleave:binding.onmouseleave,
            onfocusin:binding.onfocusin,onfocusout:binding.onfocusout,onkeydown:binding.onkeydown,
            {children}
            crate::TooltipFor {
                id, controller:controller.clone(), title, title_icon, placement, show_delay, hide_delay,
                interactive, animation_frame, onnativeerror,
                "{label}"
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/overlays.css");

#[cfg(test)]
#[path = "overlays/message_bar_tests.rs"]
mod message_bar_tests;
