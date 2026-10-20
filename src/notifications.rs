//! Scoped toast service with native CSS expiry, stacking, and Rust result callbacks.
//!
//! Adapted from the Siemens iX toast container and toast (MIT). Expiry follows
//! the upstream animation clock; pointer hover, keyboard focus, and explicit
//! pauses suspend it. No JavaScript service or global singleton is required.
use crate::{MessageType, Toast};
use dioxus::prelude::*;
use std::time::Duration;

/// Upstream toast-container placements.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPosition {
    #[default]
    BottomRight,
    TopRight,
}
impl ToastPosition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BottomRight => "bottom-right",
            Self::TopRight => "top-right",
        }
    }
}

/// The reason a toast left the queue. Programmatic closes may carry a result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastCloseReason {
    Expired,
    Dismissed,
    Programmatic(Option<String>),
}

/// Toast content and lifetime. `new` matches the upstream five-second default.
#[derive(Clone, PartialEq)]
pub struct ToastOptions {
    pub title: String,
    pub message: Element,
    pub action: Option<Element>,
    pub message_type: MessageType,
    pub auto_close: bool,
    pub auto_close_delay: Duration,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub hide_icon: bool,
    pub close_label: String,
    pub onclose: Option<EventHandler<ToastCloseReason>>,
}
impl ToastOptions {
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            title: String::new(),
            message: rsx! { "{message}" },
            action: None,
            message_type: MessageType::Info,
            auto_close: true,
            auto_close_delay: Duration::from_millis(5000),
            icon: None,
            icon_color: None,
            hide_icon: false,
            close_label: "Close toast".into(),
            onclose: None,
        }
    }
}

#[derive(Clone, PartialEq)]
struct ToastRecord {
    id: u64,
    options: ToastOptions,
    paused: bool,
    hovered: bool,
    focused: bool,
}
impl ToastRecord {
    fn is_paused(&self) -> bool {
        self.paused || self.hovered || self.focused
    }
}
fn take_toast(entries: &mut Vec<ToastRecord>, id: u64) -> Option<ToastRecord> {
    entries
        .iter()
        .position(|entry| entry.id == id)
        .map(|index| entries.remove(index))
}

/// A scoped queue. Place one [`ToastContainer`] for each controller.
#[derive(Clone, Copy, PartialEq)]
pub struct ToastController {
    entries: Signal<Vec<ToastRecord>>,
    sequence: Signal<u64>,
    position: Signal<ToastPosition>,
}
/// Create a controller in a component. Share it through props or Dioxus context.
pub fn use_toast() -> ToastController {
    ToastController {
        entries: use_signal(Vec::new),
        sequence: use_signal(|| 0),
        position: use_signal(ToastPosition::default),
    }
}
impl ToastController {
    /// Append a notification and return its independent close/pause handle.
    pub fn show(mut self, options: ToastOptions) -> ToastHandle {
        let next = self.sequence.peek().wrapping_add(1);
        self.sequence.set(next);
        self.entries.write().push(ToastRecord {
            id: next,
            options,
            paused: false,
            hovered: false,
            focused: false,
        });
        ToastHandle {
            controller: self,
            id: next,
        }
    }
    pub fn set_position(mut self, position: ToastPosition) {
        self.position.set(position);
    }
    pub fn position(self) -> ToastPosition {
        (self.position)()
    }
    pub fn len(self) -> usize {
        self.entries.read().len()
    }
    pub fn is_empty(self) -> bool {
        self.entries.read().is_empty()
    }
    fn remove(mut self, id: u64, reason: ToastCloseReason) {
        // Drop the writable borrow before callbacks are allowed to enqueue again.
        let removed = { take_toast(&mut self.entries.write(), id) };
        if let Some(callback) = removed.and_then(|record| record.options.onclose) {
            callback.call(reason);
        }
    }
}

/// A stable identity for one queued toast. Repeated closes and expiry callbacks
/// for an already removed entry do nothing while its controller scope is alive.
#[derive(Clone, Copy, PartialEq)]
pub struct ToastHandle {
    controller: ToastController,
    id: u64,
}
impl ToastHandle {
    pub fn close(self, result: Option<String>) {
        self.controller
            .remove(self.id, ToastCloseReason::Programmatic(result));
    }
    pub fn pause(self) {
        self.set_flag(0, true);
    }
    pub fn resume(self) {
        self.set_flag(0, false);
    }
    pub fn is_paused(self) -> bool {
        self.controller
            .entries
            .read()
            .iter()
            .find(|entry| entry.id == self.id)
            .is_some_and(ToastRecord::is_paused)
    }
    pub fn is_open(self) -> bool {
        self.controller
            .entries
            .read()
            .iter()
            .any(|entry| entry.id == self.id)
    }
    fn set_flag(mut self, flag: u8, value: bool) {
        if let Some(entry) = self
            .controller
            .entries
            .write()
            .iter_mut()
            .find(|entry| entry.id == self.id)
        {
            match flag {
                1 => entry.hovered = value,
                2 => entry.focused = value,
                _ => entry.paused = value,
            }
        }
    }
}

/// Fixed viewport notification stack. Its empty container does not intercept input.
#[component]
pub fn ToastContainer(controller: ToastController, #[props(default)] class: String) -> Element {
    let entries = controller.entries.read().clone();
    let position = controller.position();
    rsx! { div { class: "ix-toast-container {class}", "data-position": position.as_str(),
        for entry in entries { ManagedToast { key: "{entry.id}", entry, controller } }
    } }
}

#[component]
fn ManagedToast(entry: ToastRecord, controller: ToastController) -> Element {
    let handle = ToastHandle {
        controller,
        id: entry.id,
    };
    let mut closing = use_signal(|| false);
    let paused = entry.is_paused();
    let delay = entry.options.auto_close_delay.as_millis().max(1);
    rsx! {
        div { class: "ix-managed-toast", "data-closing": closing().to_string(),
            onpointerenter: move |_| handle.set_flag(1, true), onpointerleave: move |_| handle.set_flag(1, false),
            onfocusin: move |_| handle.set_flag(2, true),
            onfocusout: move |_| { handle.set_flag(2, false); },
            Toast { title: entry.options.title, message_type: entry.options.message_type,
                icon: entry.options.icon, icon_color: entry.options.icon_color,
                hide_icon: entry.options.hide_icon, action: entry.options.action, close_label: entry.options.close_label,
                onclose: move |_| controller.remove(handle.id, ToastCloseReason::Dismissed),
                {entry.options.message}
            }
            if entry.options.auto_close {
                div { class: "ix-toast-expiry", "aria-hidden": "true",
                    style: format!("animation-duration:{delay}ms;animation-play-state:{}", if paused { "paused" } else { "running" }),
                    onanimationend: move |event| {
                        if event.animation_name() != "ix-toast-expire" || closing() { return; }
                        closing.set(true);
                        spawn(async move {
                            futures_timer::Delay::new(Duration::from_millis(200)).await;
                            controller.remove(handle.id, ToastCloseReason::Expired);
                        });
                    }
                }
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/notifications.css");

#[cfg(test)]
mod tests {
    use super::*;
    fn record(id: u64) -> ToastRecord {
        ToastRecord {
            id,
            options: ToastOptions::new("Message"),
            paused: false,
            hovered: false,
            focused: false,
        }
    }
    #[test]
    fn stale_expiry_cannot_remove_another_toast_or_emit_twice() {
        let mut records = vec![record(1), record(2), record(3)];
        assert_eq!(take_toast(&mut records, 2).unwrap().id, 2);
        assert!(take_toast(&mut records, 2).is_none());
        assert_eq!(
            records.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            [1, 3]
        );
    }
    #[test]
    fn resume_does_not_override_hover_or_keyboard_pause() {
        let mut entry = record(1);
        entry.paused = true;
        entry.hovered = true;
        entry.paused = false;
        assert!(entry.is_paused());
        entry.hovered = false;
        entry.focused = true;
        assert!(entry.is_paused());
        entry.focused = false;
        assert!(!entry.is_paused());
    }

    #[test]
    fn controller_closes_once_and_allows_callback_to_enqueue_a_followup() {
        fn probe() -> Element {
            let queue = use_toast();
            use_hook(move || {
                let calls = std::rc::Rc::new(std::cell::Cell::new(0));
                let observed = calls.clone();
                let mut options = ToastOptions::new("Original");
                options.onclose = Some(EventHandler::new(move |reason| {
                    assert_eq!(reason, ToastCloseReason::Programmatic(Some("saved".into())));
                    observed.set(observed.get() + 1);
                    queue.show(ToastOptions::new("Follow-up"));
                }));
                let handle = queue.show(options);
                assert!(handle.is_open());
                handle.pause();
                assert!(handle.is_paused());
                handle.resume();
                assert!(!handle.is_paused());
                handle.close(Some("saved".into()));
                handle.close(Some("duplicate".into()));
                queue.remove(handle.id, ToastCloseReason::Expired);
                assert_eq!(calls.get(), 1);
                assert_eq!(queue.len(), 1);
                assert!(!handle.is_open());
            });
            VNode::empty()
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
    }
}
