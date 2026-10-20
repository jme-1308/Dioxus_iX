//! Dioxus-owned retained flip pages and native CSS animation transitions.
use crate::Icon;
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// Semantic treatment of a flip tile. Colored variants tint the footer and frame;
/// the main content surface remains neutral, matching the upstream component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlipTileVariant {
    Outline,
    #[default]
    Filled,
    Primary,
    Info,
    Warning,
    Alarm,
}
impl FlipTileVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Filled => "filled",
            Self::Primary => "primary",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Alarm => "alarm",
        }
    }
}
/// Tile dimensions use rem units, or auto sizing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlipTileDimension {
    Rem(f64),
    Auto,
}
impl FlipTileDimension {
    fn css(self, fallback: f64) -> String {
        match self {
            Self::Rem(value) if value.is_finite() && value > 0.0 => format!("{value}rem"),
            Self::Auto => "auto".into(),
            _ => format!("{fallback}rem"),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum FlipPhase {
    #[default]
    Idle,
    Outgoing,
    Incoming,
}
impl FlipPhase {
    fn class(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Outgoing => "outgoing",
            Self::Incoming => "incoming",
        }
    }
}
fn flip_index(index: usize, count: usize) -> usize {
    if count == 0 { 0 } else { index.min(count - 1) }
}
fn next_flip_index(index: usize, count: usize) -> usize {
    if count == 0 {
        0
    } else {
        (flip_index(index, count) + 1) % count
    }
}

/// An explicitly keyed, retained Dioxus page. IDs must be unique within the tile.
/// Hidden pages retain local component state; removing an item drops its state.
#[derive(Clone, PartialEq)]
pub struct FlipTileItem {
    pub id: String,
    pub label: String,
    pub content: Element,
}
impl FlipTileItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, content: Element) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            content,
        }
    }
}

/// Synchronous toggle request. Copies share cancellation for this request only.
#[derive(Clone, Debug)]
pub struct FlipTileChangeEvent {
    pub previous: usize,
    pub next: usize,
    prevented: Rc<Cell<bool>>,
}
impl FlipTileChangeEvent {
    fn new(previous: usize, next: usize) -> Self {
        Self {
            previous,
            next,
            prevented: Rc::new(Cell::new(false)),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

/// A tile cycling through any number of sides using native CSS animation events.
/// Prefer stable `items` to retain composed Dioxus state while pages are hidden
/// or reordered. Nonempty items take precedence over legacy `pages`, which keep
/// their visible-only mounting behavior. Each page can contain [`FlipTileContent`].
///
/// `index` makes the view controlled; holding it preserves the current view.
/// `onbeforetoggle` can veto a request before `ontoggle` or local state changes.
/// Changing the actual visible page while focus is in its content returns focus
/// to the eye button when multiple pages remain, otherwise to the labelled tile article.
#[component]
pub fn FlipTile(
    #[props(default)] pages: Vec<Element>,
    #[props(default)] items: Vec<FlipTileItem>,
    #[props(default=VNode::empty())] header: Element,
    #[props(default)] footer: Option<Element>,
    #[props(default)] variant: FlipTileVariant,
    #[props(default)] index: Option<usize>,
    #[props(default)] default_index: usize,
    #[props(default)] ontoggle: EventHandler<usize>,
    #[props(default)] onbeforetoggle: EventHandler<FlipTileChangeEvent>,
    #[props(default=FlipTileDimension::Rem(16.0))] width: FlipTileDimension,
    #[props(default=FlipTileDimension::Rem(15.125))] height: FlipTileDimension,
    #[props(default="Show next content".to_string())] label: String,
    #[props(default="Flippable tile".to_string())] aria_label: String,
) -> Element {
    let item_mode = !items.is_empty();
    let count = if item_mode { items.len() } else { pages.len() };
    let mut local = use_signal(|| default_index);
    let target = flip_index(index.unwrap_or(local()), count);
    let mut displayed = use_signal(|| target);
    let mut phase = use_signal(FlipPhase::default);
    use_effect(use_reactive!(|target, count| {
        if count < 2 {
            if displayed() != target {
                displayed.set(target);
            }
            if phase() != FlipPhase::Idle {
                phase.set(FlipPhase::Idle);
            }
        } else if phase() == FlipPhase::Idle && displayed() != target {
            phase.set(FlipPhase::Outgoing);
        }
    }));
    let visible = flip_index(displayed(), count);
    let current_phase = phase();
    let enabled = count > 1 && current_phase == FlipPhase::Idle;
    let visible_key = if count == 0 {
        None
    } else if item_mode {
        Some(format!("item:{}", items[visible].id))
    } else {
        Some(format!("legacy:{visible}"))
    };
    let previous_key = use_hook(|| Rc::new(RefCell::new(visible_key.clone())));
    let content_focused = use_hook(|| Rc::new(Cell::new(false)));
    // Capture the need before rendering hidden attributes causes native blur.
    let return_focus = *previous_key.borrow() != visible_key && content_focused.get();
    *previous_key.borrow_mut() = visible_key.clone();
    let latest_key = use_hook(|| Rc::new(RefCell::new(visible_key.clone())));
    *latest_key.borrow_mut() = visible_key.clone();
    let eye_enabled = use_hook(|| Rc::new(Cell::new(count > 1)));
    eye_enabled.set(count > 1);
    let mut article = use_signal(|| None::<Rc<MountedData>>);
    let mut eye = use_signal(|| None::<Rc<MountedData>>);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let drop_alive = alive.clone();
    use_drop(move || drop_alive.set(false));
    use_effect(use_reactive!(|return_focus, visible_key| {
        if return_focus {
            let alive = alive.clone();
            let latest_key = latest_key.clone();
            let eye_enabled = eye_enabled.clone();
            spawn(async move {
                if alive.get() && *latest_key.borrow() == visible_key {
                    let node = if eye_enabled.get() {
                        eye.peek().clone()
                    } else {
                        article.peek().clone()
                    };
                    if let Some(node) = node {
                        let _ = node.set_focus(true).await;
                    }
                }
            });
        }
    }));
    let focus_in_state = content_focused.clone();
    let focus_in = use_callback(move |_: FocusEvent| focus_in_state.set(true));
    let focus_out = use_callback(move |_: FocusEvent| content_focused.set(false));
    let display_number = if count == 0 { 0 } else { visible + 1 };
    rsx! {
        article {
            class: "ix-flip-tile ix-flip-tile-{variant.class()}",
            style: format!("width:{};height:{}", width.css(16.0), height.css(15.125)),
            "aria-label": aria_label, tabindex: "-1", "data-index": visible, "data-count": count,
            onmounted: move |event| article.set(Some(event.data())),
            div {
                class: "ix-flip-tile-container", "data-phase": current_phase.class(),
                onanimationend: move |event: AnimationEvent| {
                    if event.animation_name() == "ix-pattern-flip-out" && phase() == FlipPhase::Outgoing {
                        // Resolve the current target, not an obsolete queued index.
                        displayed.set(target);
                        phase.set(if count > 1 { FlipPhase::Incoming } else { FlipPhase::Idle });
                    } else if event.animation_name() == "ix-pattern-flip-in" && phase() == FlipPhase::Incoming {
                        phase.set(FlipPhase::Idle);
                    }
                },
                header { class: "ix-flip-tile-header", div { {header} }
                    button {
                        class: "ix-flip-tile-toggle", r#type: "button", disabled: count < 2,
                        "aria-disabled": !enabled,
                        "aria-label": label,
                        onmounted: move |event| eye.set(Some(event.data())),
                        onclick: move |_| {
                            if !enabled || phase() != FlipPhase::Idle { return; }
                            let next = next_flip_index(target, count);
                            let request = FlipTileChangeEvent::new(visible, next);
                            onbeforetoggle.call(request.clone());
                            if !request.default_prevented() {
                                if index.is_none() { local.set(next); }
                                ontoggle.call(next);
                            }
                        },
                        Icon { name: "eye", size: 24 }
                    }
                }
                if item_mode {
                    div { class: "ix-flip-tile-content", onfocusin: focus_in, onfocusout: focus_out,
                        for (position, item) in items.into_iter().enumerate() {
                            div {
                                key: "{item.id}", class: "ix-flip-tile-page",
                                "data-flip-tile-page": item.id, role: "group", "aria-label": item.label,
                                hidden: position != visible, {item.content}
                            }
                        }
                    }
                } else {
                    div {
                        key: "{visible}", class: "ix-flip-tile-content", onfocusin: focus_in, onfocusout: focus_out,
                        if let Some(page) = pages.get(visible) { {page.clone()} }
                    }
                }
                if let Some(footer) = footer { footer { class: "ix-flip-tile-footer", {footer} } }
            }
            span { class: "ix-sr-only", role: "status", "Content {display_number} of {count}" }
        }
    }
}

/// One flip tile side. Explicit `items` retain it while hidden; legacy `pages`
/// mount only the currently visible side.
#[component]
pub fn FlipTileContent(children: Element) -> Element {
    rsx! { div { class: "ix-flip-tile-side", {children} } }
}

#[cfg(test)]
mod tests;
