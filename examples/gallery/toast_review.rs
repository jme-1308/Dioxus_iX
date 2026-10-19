//! Managed notification lifetimes, pause controls and result callbacks.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{cell::Cell, rc::Rc, time::Duration};

/// This example inherits the gallery layout and production notification styles.
pub const STYLES: &str = "";

/// A scoped notification queue with timed and persistent messages.
#[component]
pub fn ToastLifetimeExample(
    #[props(default = Duration::from_secs(5))] lifetime: Duration,
) -> Element {
    let notifications = use_toast();
    let mut latest = use_signal(|| None::<ToastHandle>);
    let mut closed = use_signal(|| 0usize);
    let mut last = use_signal(|| None::<ToastCloseReason>);
    let onclose = use_callback(move |reason: ToastCloseReason| {
        closed += 1;
        last.set(Some(reason));
    });

    let latest_open = latest().is_some_and(ToastHandle::is_open);
    let latest_state = match latest() {
        None => "none",
        Some(handle) if !handle.is_open() => "closed",
        Some(handle) if handle.is_paused() => "paused",
        Some(_) => "running",
    };
    let (last_value, last_message) = match last() {
        None => ("none", "No notifications have closed."),
        Some(ToastCloseReason::Expired) => ("expired", "The last notification expired."),
        Some(ToastCloseReason::Dismissed) => ("dismissed", "The last notification was dismissed."),
        Some(ToastCloseReason::Programmatic(Some(result))) if result == "acknowledged" => (
            "programmatic:acknowledged",
            "The connection review was acknowledged.",
        ),
        Some(ToastCloseReason::Programmatic(_)) => (
            "programmatic:closed",
            "The last notification was closed with its handle.",
        ),
    };
    let position = match notifications.position() {
        ToastPosition::TopRight => "Top right",
        ToastPosition::BottomRight => "Bottom right",
    };

    rsx! {
        section { id: "toast-lifetime", class: "demo-section",
            h2 { "Managed toast notifications" }
            p { "Show a timed confirmation or a persistent warning. Hover a toast or focus its controls to pause the countdown. Pause, resume and close target the latest notification; each notification reports its own close result." }
            div { class: "demo-row",
                Button { id: "toast-show-timed", onclick: move |_| {
                    let mut options = ToastOptions::new("Your configuration is ready. Hover this notification or focus its dismiss button to pause the countdown.");
                    options.title = "Configuration saved".into();
                    options.message_type = MessageType::Success;
                    options.auto_close_delay = lifetime;
                    options.close_label = "Dismiss notification".into();
                    options.onclose = Some(onclose);
                    latest.set(Some(notifications.show(options)));
                }, "Show timed toast" }
                Button { id: "toast-show-persistent", variant: ButtonVariant::Secondary, onclick: move |_| {
                    let action_handle = Rc::new(Cell::new(None::<ToastHandle>));
                    let target = action_handle.clone();
                    let mut options = ToastOptions::new("Review the device connection before continuing. This notification stays open until acknowledged or dismissed.");
                    options.title = "Connection review".into();
                    options.message_type = MessageType::Warning;
                    options.auto_close = false;
                    options.close_label = "Dismiss notification".into();
                    options.onclose = Some(onclose);
                    options.action = Some(rsx! {
                        Button { variant: ButtonVariant::Tertiary, onclick: move |_| {
                            if let Some(handle) = target.get() {
                                handle.close(Some("acknowledged".into()));
                            }
                        }, "Acknowledge" }
                    });
                    let handle = notifications.show(options);
                    action_handle.set(Some(handle));
                    latest.set(Some(handle));
                }, "Show persistent toast" }
            }
            div { class: "demo-row",
                Button { id: "toast-pause-latest", variant: ButtonVariant::Tertiary, disabled: !latest_open, onclick: move |_| {
                    if let Some(handle) = latest() { handle.pause(); }
                }, "Pause latest" }
                Button { id: "toast-resume-latest", variant: ButtonVariant::Tertiary, disabled: !latest_open, onclick: move |_| {
                    if let Some(handle) = latest() { handle.resume(); }
                }, "Resume latest" }
                Button { id: "toast-close-latest", variant: ButtonVariant::Tertiary, disabled: !latest_open, onclick: move |_| {
                    if let Some(handle) = latest() { handle.close(Some("closed".into())); }
                }, "Close latest" }
            }
            div { class: "demo-row",
                span { class: "demo-label", "Position: {position}" }
                Button { id: "toast-position-top", variant: ButtonVariant::Tertiary,
                    onclick: move |_| notifications.set_position(ToastPosition::TopRight), "Top right" }
                Button { id: "toast-position-bottom", variant: ButtonVariant::Tertiary,
                    onclick: move |_| notifications.set_position(ToastPosition::BottomRight), "Bottom right" }
            }
            p { id: "toast-lifetime-status", role: "status",
                "data-open": notifications.len(), "data-closed": closed(),
                "data-last": last_value, "data-latest": latest_state,
                "Open: {notifications.len()}. Closed: {closed}. Latest: {latest_state}. {last_message}"
            }
            ToastContainer { controller: notifications }
        }
    }
}
