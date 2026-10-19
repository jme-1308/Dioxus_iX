//! Feedback primitives ported from iX behavior and examples.
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.
use crate::Icon;
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// Inline contextual message. A persistent message cannot be dismissed.
#[component]
pub fn MessageBar(
    #[props(default)] message_type: MessageType,
    #[props(default)] persistent: bool,
    #[props(default)] onclose: Option<EventHandler<()>>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let mut closed = use_signal(|| false);
    if closed() {
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
                    "aria-label": "Close message",
                    onclick: move |_| { closed.set(true); if let Some(handler) = onclose { handler.call(()); } },
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

pub(crate) const STYLES: &str = r#"
.ix-message-bar,.ix-toast { --ix-message-color:var(--ix-color-info); display:flex;align-items:flex-start;gap:12px;padding:12px 16px;background:var(--ix-color-surface-1);border-radius:var(--ix-radius);color:var(--ix-color-text); }
.ix-message-bar { border:1px solid color-mix(in srgb,var(--ix-message-color) 55%,var(--ix-color-border)); }
.ix-message-bar>.ix-icon,.ix-toast>.ix-icon,.ix-toast>.ix-toast-icon { color:var(--ix-message-color);margin-top:1px; }
.ix-toast-action { margin-top:12px; }
.ix-message-bar[data-type=alarm],.ix-toast[data-type=alarm],.ix-message-bar[data-type=critical],.ix-toast[data-type=critical] { --ix-message-color:var(--ix-color-danger); }
.ix-message-bar[data-type=warning],.ix-toast[data-type=warning] { --ix-message-color:var(--ix-color-warning); }
.ix-message-bar[data-type=success],.ix-toast[data-type=success] { --ix-message-color:var(--ix-color-success); }
.ix-message-bar[data-type=neutral],.ix-toast[data-type=neutral] { --ix-message-color:var(--ix-color-text-secondary); }
.ix-message-bar[data-type=primary],.ix-toast[data-type=primary] { --ix-message-color:var(--ix-color-primary); }
.ix-message-bar-content,.ix-toast-body { flex:1;min-width:0; }
.ix-message-close { display:inline-flex;align-items:center;justify-content:center;color:inherit;background:transparent;border:0;padding:2px;border-radius:var(--ix-radius); }
.ix-message-close:hover { background:var(--ix-color-surface-2); }
.ix-toast { width:min(440px,100%);border:1px solid var(--ix-color-border);box-shadow:0 6px 20px #0004; }
.ix-toast-body strong { display:block; }
.ix-toast[data-type=info] { --ix-message-color:var(--ix-color-text); }
.ix-tooltip-anchor { display:inline-flex;position:relative; }
.ix-tooltip { position:fixed;inset:auto;margin:0;box-sizing:border-box;border:0;padding:8px 12px;max-width:min(280px,calc(100vw - 20px));width:max-content;white-space:normal;overflow:visible;overflow-wrap:anywhere;background:var(--ix-color-surface-2);color:var(--ix-color-text);border-radius:var(--ix-radius);font-size:14px;line-height:1.5;box-shadow:0 3px 12px #0004;visibility:hidden; }
.ix-tooltip:popover-open[data-positioned=true] { visibility:visible; }
.ix-tooltip[data-interactive=false] { pointer-events:none; }
.ix-tooltip-content { max-height:calc(100dvh - 52px);overflow:auto; }
.ix-tooltip-title { display:flex;align-items:center;gap:8px;margin-bottom:8px; }
.ix-tooltip-arrow { position:absolute;width:12px;height:12px;background:inherit;transform:rotate(45deg);z-index:-1; }
.ix-tooltip[data-placement=top]>.ix-tooltip-arrow { left:var(--ix-tooltip-arrow);bottom:-6px; }
.ix-tooltip[data-placement=bottom]>.ix-tooltip-arrow { left:var(--ix-tooltip-arrow);top:-6px; }
.ix-tooltip[data-placement=left]>.ix-tooltip-arrow { top:var(--ix-tooltip-arrow);right:-6px; }
.ix-tooltip[data-placement=right]>.ix-tooltip-arrow { top:var(--ix-tooltip-arrow);left:-6px; }
"#;
