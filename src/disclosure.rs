//! Retained disclosure composition with native Dioxus controls and mounted reveal.
use crate::{CardVariant, Icon};
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_DISCLOSURE: AtomicUsize = AtomicUsize::new(1);

fn use_disclosure_ids(kind: &str) -> (String, String) {
    let prefix = use_hook(|| {
        format!(
            "ix-{kind}-{}",
            NEXT_DISCLOSURE.fetch_add(1, Ordering::Relaxed)
        )
    });
    (format!("{prefix}-toggle"), format!("{prefix}-panel"))
}

// The panel remains mounted while hidden. A real opening can wait for that
// exact Dioxus mount without inspecting DOM or introducing an animation timer.
fn use_opening_reveal(open: bool, enabled: bool) -> EventHandler<MountedEvent> {
    let mut panel = use_signal(|| None::<Rc<MountedData>>);
    let previous = use_hook(|| Rc::new(Cell::new(open)));
    let pending = use_hook(|| Rc::new(Cell::new(false)));
    let current = use_hook(|| Rc::new(Cell::new((open, enabled))));
    current.set((open, enabled));
    use_effect(use_reactive((&open, &enabled), move |(open, enabled)| {
        let was_open = previous.replace(open);
        if !open || !enabled {
            pending.set(false);
        } else if !was_open {
            pending.set(true);
        }
        let mounted = panel();
        if pending.get()
            && let Some(node) = mounted
        {
            pending.set(false);
            let current = current.clone();
            // Dioxus cancels this task when its owning scope is unmounted.
            spawn(async move {
                if current.get() == (true, true) {
                    let _ = node
                        .scroll_to_with_options(dioxus::html::ScrollToOptions {
                            behavior: dioxus::html::ScrollBehavior::Smooth,
                            vertical: dioxus::html::ScrollLogicalPosition::Nearest,
                            horizontal: dioxus::html::ScrollLogicalPosition::Nearest,
                        })
                        .await;
                }
            });
        }
    }));
    use_callback(move |event: MountedEvent| panel.set(Some(event.data())))
}

/// Accessible card disclosure with retained children. `expanded` is controlled;
/// otherwise `default_expanded` initializes local state (closed by default).
/// `onexpand` reports one request per native activation. A held controlled
/// request does not reveal content. `reveal_content` scrolls an actual opening
/// into view through Dioxus, using the nearest edge on both axes.
#[component]
pub fn CardAccordion(
    children: Element,
    #[props(default = "Details".to_string())] label: String,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] variant: CardVariant,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default = true)] reveal_content: bool,
) -> Element {
    let mut local = use_signal(|| default_expanded);
    let open = expanded.unwrap_or(local());
    let (toggle_id, panel_id) = use_disclosure_ids("card-accordion");
    let onmounted = use_opening_reveal(open, reveal_content);
    rsx! { div { id, class: "ix-card-accordion ix-card-{variant.class()} {class}",
        button { id: toggle_id.clone(), class: "ix-disclosure-button", r#type: "button",
            "aria-expanded": open, "aria-controls": panel_id.clone(), "aria-label": aria_label,
            onclick: move |event| {
                event.stop_propagation();
                if expanded.is_none() { local.set(!open); }
                onexpand.call(!open);
            },
            span { class: "ix-chevron", "data-open": open, Icon { name: "chevron-down" } } "{label}"
        }
        div { id: panel_id, class: "ix-card-accordion-content", role: "region",
            "aria-labelledby": toggle_id, hidden: !open, onmounted, {children} }
    } }
}

/// Retained information section with independent header actions. Custom header
/// content replaces the default label/icon presentation inside the native
/// toggle; keep it noninteractive and supply visible text or `aria_label`.
/// `header_actions` remain separate controls. `expanded` is controlled when set;
/// otherwise local state starts at `default_expanded` (closed by default).
#[component]
pub fn Blind(
    #[props(default)] label: String,
    children: Element,
    #[props(default)] sublabel: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default = CardVariant::Filled)] variant: CardVariant,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] custom_header: Option<Element>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let mut local = use_signal(|| default_expanded);
    let open = expanded.unwrap_or(local());
    let (toggle_id, panel_id) = use_disclosure_ids("blind");
    rsx! { section { id, class: "ix-blind ix-card-{variant.class()} {class}",
        div { class: "ix-blind-heading",
            button { id: toggle_id.clone(), class: "ix-blind-header", r#type: "button",
                "aria-expanded": open, "aria-controls": panel_id.clone(), "aria-label": aria_label,
                onclick: move |_| {
                    if expanded.is_none() { local.set(!open); }
                    onexpand.call(!open);
                },
                span { class: "ix-chevron", "data-open": open, Icon { name: "chevron-down" } }
                if let Some(custom) = custom_header { span { class: "ix-blind-custom-header", {custom} } }
                else {
                    if let Some(icon) = icon { Icon { name: icon } }
                    span { class: "ix-blind-label", strong { "{label}" } if let Some(text) = sublabel { span { "{text}" } } }
                }
            }
            if let Some(actions) = header_actions { div { class: "ix-blind-actions", {actions} } }
        }
        div { id: panel_id, class: "ix-blind-content", role: "region", "aria-labelledby": toggle_id,
            hidden: !open, {children} }
    } }
}

#[cfg(test)]
mod tests;
