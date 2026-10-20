//! Event collections with reactive Rust composition and independent native controls.
use crate::{CardVariant, Icon};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_EVENT_ROW: AtomicUsize = AtomicUsize::new(1);

/// Default-density event row size or an explicit pixel height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventItemHeight {
    #[default]
    Small,
    Large,
    Pixels(u32),
}
impl EventItemHeight {
    fn pixels(self) -> u32 {
        match self {
            Self::Small => 40,
            Self::Large => 104,
            Self::Pixels(value) => value.max(1),
        }
    }
}

#[derive(Clone, Copy)]
struct EventListContext {
    chevron: ReadSignal<Option<bool>>,
    animated: ReadSignal<bool>,
}

/// Event collection with shared height, compact spacing and reactive row options.
/// `chevron: None` preserves each item's own setting; `Some` overrides it.
/// Wrapped/keyed rows follow the nearest list. Nested lists own their settings.
/// Animation is opt-in CSS entrance motion and respects reduced-motion settings.
#[component]
pub fn EventList(
    children: Element,
    #[props(default)] compact: bool,
    #[props(default)] item_height: EventItemHeight,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] chevron: ReadSignal<Option<bool>>,
    #[props(default)] animated: ReadSignal<bool>,
) -> Element {
    use_context_provider(|| EventListContext { chevron, animated });
    rsx! { div { id, class: "ix-event-list {class}", role: "list", "aria-label": aria_label,
    "data-compact": compact, style: format!("--ix-event-height:{}px", item_height.pixels()), {children} } }
}

fn indicator_style(token: Option<&str>) -> Option<String> {
    token
        .filter(|token| {
            token.strip_prefix("--").is_some_and(|name| {
                !name.is_empty()
                    && name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            })
        })
        .map(|token| format!("--ix-event-status:var({token})"))
}

/// Selectable event row with a sibling content area for ordinary Dioxus controls.
/// Text selects the row; composed inputs, buttons, links and labels retain their
/// own native interactions. `disabled` blocks only primary selection: consumers
/// set disabled state on their composed controls separately.
///
/// `item_color` accepts a CSS custom-property name such as `--ix-color-primary`.
/// Invalid names are ignored so the semantic `status` indicator remains active.
/// Supply `aria_label` to name primary selection explicitly, or its name comes
/// from the stable visible content element referenced by `aria-labelledby`.
#[component]
pub fn EventListItem(
    children: Element,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] chevron: bool,
    #[props(default)] filled: bool,
    #[props(default)] status: CardVariant,
    #[props(default)] onselect: EventHandler<()>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] item_color: Option<String>,
) -> Element {
    let context = try_use_context::<EventListContext>();
    let show_chevron = context
        .and_then(|context| (context.chevron)())
        .unwrap_or(chevron);
    let animated = context.is_some_and(|context| (context.animated)());
    let content_id = use_hook(|| {
        format!(
            "ix-event-content-{}",
            NEXT_EVENT_ROW.fetch_add(1, Ordering::Relaxed)
        )
    });
    let labelledby = aria_label.is_none().then(|| content_id.clone());
    let style = indicator_style(item_color.as_deref());
    rsx! { div { id, class: "ix-event-row ix-event-status-{status.class()} {class}", role: "listitem",
        "data-disabled": disabled, "data-animated": animated, style,
        button { class: "ix-event-list-item ix-event-status-{status.class()}", r#type: "button", disabled,
            "aria-pressed": selected, "aria-label": aria_label, "aria-labelledby": labelledby,
            "data-selected": selected, "data-filled": filled,
            onclick: move |_| { if !disabled { onselect.call(()); } }
        }
        div { class: "ix-event-row-body",
            span { class: "ix-event-indicator", "aria-hidden": "true" }
            div { id: content_id, class: "ix-event-content", {children} }
            if show_chevron { span { class: "ix-event-chevron", "aria-hidden": "true", Icon { name: "chevron-right" } } }
        }
    } }
}

#[cfg(test)]
mod tests;
