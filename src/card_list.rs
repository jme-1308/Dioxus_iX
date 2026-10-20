//! Explicit, Rust-owned card collections and overflow visibility.
use crate::{CardListItem, CardListRevealEvent, CardListRevealSource, CardListStyle, Icon};
use dioxus::prelude::*;
use std::{cell::Cell, rc::Rc};

/// Responsive, collapsible card collection with stack and scroll layouts.
///
/// Prefer uniquely keyed `items` for automatic counting, limiting and reveal.
/// Hidden entries stay mounted, retaining their composed Dioxus state. Revealing
/// focuses the labelled first new item wrapper; native Tab enters its controls.
/// Legacy `children` remain unbounded and use `show_all_count` only for a manual
/// action. Nonempty `items` take precedence when both are provided.
///
/// Controlled `collapsed` and `show_all` remain authoritative. Requests can be
/// synchronously vetoed with `onrevealrequest`; accepted requests emit `onshowall`
/// even when their controlled value is held. Provide a label or nonempty
/// `aria_label_expand_button` for focus return when externally collapsing.
#[component]
pub fn CardList(
    #[props(default = VNode::empty())] children: Element,
    #[props(default)] items: Vec<CardListItem>,
    #[props(default)] label: Option<String>,
    #[props(default)] aria_label_expand_button: Option<String>,
    #[props(default)] list_style: CardListStyle,
    #[props(default)] collapsed: Option<bool>,
    #[props(default)] default_collapsed: bool,
    #[props(default)] oncollapse: EventHandler<bool>,
    #[props(default)] show_all: Option<bool>,
    #[props(default)] default_show_all: bool,
    #[props(default)] show_all_count: Option<usize>,
    #[props(default = 12)] max_visible_cards: usize,
    #[props(default)] suppress_overflow_handling: bool,
    #[props(default)] hide_show_all: bool,
    #[props(default = "Show all".to_string())] show_all_label: String,
    #[props(default = "Show less".to_string())] show_less_label: String,
    #[props(default = "There are more cards available".to_string())] more_cards_label: String,
    #[props(default)] onshowall: EventHandler<bool>,
    #[props(default)] onrevealrequest: EventHandler<CardListRevealEvent>,
    #[props(default)] class: String,
) -> Element {
    let mut local_collapsed = use_signal(|| default_collapsed);
    let mut local_show_all = use_signal(|| default_show_all);
    let is_collapsed = collapsed.unwrap_or(local_collapsed());
    let showing_all = show_all.unwrap_or(local_show_all());
    let item_mode = !items.is_empty();
    let total = items.len();
    let limited = item_mode && !suppress_overflow_handling;
    let visible = if limited && !showing_all {
        total.min(max_visible_cards)
    } else {
        total
    };
    let overflowing = limited && total > max_visible_cards;
    let remaining = total - visible;
    let count = show_all_count.unwrap_or(total);
    let custom_aria = aria_label_expand_button.filter(|value| !value.trim().is_empty());
    let has_header = label.is_some() || custom_aria.is_some();
    let show_header_action =
        !hide_show_all && (overflowing || (!item_mode && show_all_count.is_some()));

    // These are ownership/focus facts delivered by Dioxus events, not DOM scans.
    // Capture the collapse need during render, before hidden content emits blur.
    let content_focused = use_hook(|| Rc::new(Cell::new(false)));
    let previous = use_hook(|| Rc::new(Cell::new((is_collapsed, showing_all))));
    let (was_collapsed, was_showing_all) = previous.replace((is_collapsed, showing_all));
    let collapse_focus = !was_collapsed && is_collapsed && content_focused.get();
    let reveal_id = (!was_showing_all && showing_all && limited && !is_collapsed)
        .then(|| items.get(max_visible_cards).map(|item| item.id.clone()))
        .flatten();
    let mut collapse_node = use_signal(|| None::<Rc<MountedData>>);
    let current_collapsed = use_hook(|| Rc::new(Cell::new(is_collapsed)));
    current_collapsed.set(is_collapsed && has_header);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let drop_alive = alive.clone();
    use_drop(move || drop_alive.set(false));
    use_effect(use_reactive!(|collapse_focus| {
        if collapse_focus {
            if let Some(node) = collapse_node.peek().clone() {
                let alive = alive.clone();
                let current_collapsed = current_collapsed.clone();
                spawn(async move {
                    if alive.get()
                        && current_collapsed.get()
                        && collapse_node
                            .peek()
                            .as_ref()
                            .is_some_and(|current| Rc::ptr_eq(current, &node))
                    {
                        let _ = node.set_focus(true).await;
                    }
                });
            }
        }
    }));
    let focus_in = content_focused.clone();
    let focus_out = content_focused;
    let mut request_reveal = move |source| {
        let next = !showing_all;
        let request = CardListRevealEvent::new(next, source);
        onrevealrequest.call(request.clone());
        if !request.default_prevented() {
            local_show_all.set(next);
            onshowall.call(next);
        }
    };

    rsx! {
        section {
            class: "ix-card-list {class}",
            "data-scroll": list_style == CardListStyle::Scroll && !showing_all,
            "data-count": total,
            "data-visible-count": if is_collapsed { 0 } else { visible },
            "data-show-all": showing_all,
            if has_header {
                div { class: "ix-card-list-header",
                    button {
                        class: "ix-disclosure-button", r#type: "button",
                        "aria-expanded": !is_collapsed,
                        "aria-label": custom_aria,
                        onmounted: move |event| collapse_node.set(Some(event.data())),
                        onclick: move |_| {
                            local_collapsed.set(!is_collapsed);
                            oncollapse.call(!is_collapsed);
                        },
                        span { class: "ix-chevron", "data-open": !is_collapsed,
                            Icon { name: "chevron-down" }
                        }
                        {label}
                    }
                    if show_header_action {
                        button {
                            class: "ix-card-list-show-all", r#type: "button", "aria-pressed": showing_all,
                            onclick: move |_| request_reveal(CardListRevealSource::Header),
                            if showing_all { "{show_less_label}" } else { "{show_all_label} ({count})" }
                        }
                    }
                }
            }
            div {
                class: "ix-card-list-items", role: if item_mode { "list" }, hidden: is_collapsed,
                onfocusin: move |_| focus_in.set(true),
                onfocusout: move |_| focus_out.set(false),
                if item_mode {
                    for (index, item) in items.into_iter().enumerate() {
                        CardListEntry {
                            key: "{item.id}",
                            visible: !is_collapsed && index < visible,
                            focus_requested: reveal_id.as_ref() == Some(&item.id),
                            item,
                        }
                    }
                } else { {children} }
                if remaining > 0 {
                    button {
                        class: "ix-card-list-more", r#type: "button",
                        onclick: move |_| request_reveal(CardListRevealSource::MoreCard),
                        Icon { name: "context-menu" }
                        span { "{more_cards_label} ({remaining})" }
                    }
                }
            }
        }
    }
}

#[component]
fn CardListEntry(item: CardListItem, visible: bool, focus_requested: bool) -> Element {
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let active = use_hook(|| Rc::new(Cell::new(visible && focus_requested)));
    active.set(visible && focus_requested);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let drop_alive = alive.clone();
    use_drop(move || drop_alive.set(false));
    use_effect(use_reactive!(|visible, focus_requested| {
        let node = mounted();
        if visible && focus_requested {
            if let Some(node) = node {
                let alive = alive.clone();
                let active = active.clone();
                spawn(async move {
                    if alive.get() && active.get() {
                        let _ = node.set_focus(true).await;
                    }
                });
            }
        }
    }));
    rsx! {
        div {
            class: "ix-card-list-item", "data-card-list-entry": item.id,
            role: "listitem", "aria-label": item.label, tabindex: "-1", hidden: !visible,
            onmounted: move |event| mounted.set(Some(event.data())),
            {item.content}
        }
    }
}

#[cfg(test)]
mod tests;
