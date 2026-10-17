//! Feedback and menu primitives ported from iX behavior and examples.
//!
//! Copyright (c) 2023-2026 Siemens AG. SPDX-License-Identifier: MIT.

use crate::Icon;
use dioxus::prelude::*;
use serde::Deserialize;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static MENU_ITEM_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct MenuEntry {
    label: String,
    disabled: bool,
    mounted: Rc<MountedData>,
}

#[derive(Deserialize)]
struct MenuOrder {
    ids: Vec<usize>,
    focused: bool,
}

#[derive(Clone)]
struct OpenSubmenu {
    id: usize,
    open: Signal<bool>,
    child: Rc<RefCell<Option<OpenSubmenu>>>,
}

impl OpenSubmenu {
    fn close(mut self) {
        if let Some(child) = self.child.borrow_mut().take() {
            child.close();
        }
        self.open.set(false);
    }
}

#[derive(Clone)]
struct DropdownContext {
    open: Signal<bool>,
    focus_last: Signal<bool>,
    focus_on_open: Signal<bool>,
    entries: Rc<RefCell<BTreeMap<usize, MenuEntry>>>,
    trigger: Rc<RefCell<Option<Rc<MountedData>>>>,
    parent: Option<Rc<DropdownContext>>,
    child: Rc<RefCell<Option<OpenSubmenu>>>,
    focus_epoch: Rc<Cell<u64>>,
    alive: Rc<Cell<bool>>,
}

impl DropdownContext {
    fn invalidate_focus(&self) -> u64 {
        let epoch = self.focus_epoch.get().wrapping_add(1);
        self.focus_epoch.set(epoch);
        epoch
    }

    fn focus_is_current(&self, epoch: u64) -> bool {
        self.alive.get()
            && self.focus_epoch.get() == epoch
            && *self.open.peek()
            && self
                .parent
                .as_ref()
                .is_none_or(|parent| parent.focus_is_current(epoch))
    }

    fn close(&self, restore_focus: bool) {
        self.close_child_except(None);
        let epoch = self.invalidate_focus();
        let mut open = self.open;
        open.set(false);
        if restore_focus {
            if let Some(target) = self.trigger.borrow().clone() {
                let context = self.clone();
                spawn(async move {
                    if context.alive.get()
                        && context.focus_epoch.get() == epoch
                        && !*context.open.peek()
                        && context
                            .parent
                            .as_ref()
                            .is_none_or(|parent| parent.focus_is_current(epoch))
                    {
                        let _ = target.set_focus(true).await;
                    }
                });
            }
        }
    }

    fn close_all(&self, restore_focus: bool) {
        if let Some(parent) = &self.parent {
            parent.close_all(restore_focus);
        } else {
            self.close(restore_focus);
        }
    }

    fn close_child_except(&self, keep: Option<usize>) {
        let mut child = self.child.borrow_mut();
        if child.as_ref().is_some_and(|child| Some(child.id) != keep) {
            if let Some(child) = child.take() {
                self.invalidate_focus();
                child.close();
            }
        }
    }

    fn open_child(&self, id: usize, context: &Self) {
        self.close_child_except(Some(id));
        self.invalidate_focus();
        *self.child.borrow_mut() = Some(OpenSubmenu {
            id,
            open: context.open,
            child: context.child.clone(),
        });
        let mut open = context.open;
        open.set(true);
    }

    fn focus_id(&self, id: usize, epoch: u64) {
        if let Some(entry) = self.entries.borrow().get(&id) {
            let mounted = entry.mounted.clone();
            let context = self.clone();
            spawn(async move {
                let enabled = context
                    .entries
                    .borrow()
                    .get(&id)
                    .is_some_and(|entry| !entry.disabled);
                if enabled && context.focus_is_current(epoch) {
                    let _ = mounted.set_focus(true).await;
                }
            });
        }
    }

    // Keyed children retain their mount IDs when moved. Read the actual sibling
    // order rather than treating those identity tokens as ordering information.
    // The bridge only measures ownership/order; Rust selects and focuses items.
    async fn item_order(&self, probe: usize) -> Option<MenuOrder> {
        let query = crate::dom_query::eval(
            r#"const id = await dioxus.recv();
const item = document.querySelector(`[data-ix-dropdown-item="${id}"]`);
const menu = item?.closest('[role="menu"]');
if (!menu?.isConnected) return null;
const ids = [...menu.querySelectorAll('[data-ix-dropdown-item]')]
  .filter(node => node.closest('[role="menu"]') === menu && !node.disabled)
  .map(node => Number(node.dataset.ixDropdownItem));
return {ids, focused: menu.contains(document.activeElement)};"#,
        );
        query.send(probe).ok()?;
        query.join::<Option<MenuOrder>>().await.ok().flatten()
    }

    fn focus_edge(&self, last: bool) {
        self.focus_edge_if_needed(last, false);
    }

    fn focus_edge_if_needed(&self, last: bool, only_unfocused: bool) {
        let epoch = self.invalidate_focus();
        let Some(probe) = self.entries.borrow().keys().next().copied() else {
            return;
        };
        let context = self.clone();
        spawn(async move {
            let Some(order) = context.item_order(probe).await else {
                return;
            };
            if !context.focus_is_current(epoch) || (only_unfocused && order.focused) {
                return;
            }
            let next = menu_target(
                &context.entries.borrow(),
                &order.ids,
                None,
                &if last { Key::End } else { Key::Home },
            );
            if let Some(next) = next {
                context.focus_id(next, epoch);
            }
        });
    }

    fn navigate(&self, id: usize, key: &Key) {
        let epoch = self.invalidate_focus();
        let context = self.clone();
        let key = key.clone();
        spawn(async move {
            let Some(order) = context.item_order(id).await else {
                return;
            };
            if !context.focus_is_current(epoch) {
                return;
            }
            let next = menu_target(&context.entries.borrow(), &order.ids, Some(id), &key);
            if let Some(next) = next {
                context.focus_id(next, epoch);
            }
        });
    }
}

fn menu_target(
    entries: &BTreeMap<usize, MenuEntry>,
    order: &[usize],
    current: Option<usize>,
    key: &Key,
) -> Option<usize> {
    let enabled: Vec<_> = order
        .iter()
        .copied()
        .filter(|id| entries.get(id).is_some_and(|entry| !entry.disabled))
        .collect();
    if *key == Key::Home {
        return enabled.first().copied();
    }
    if *key == Key::End {
        return enabled.last().copied();
    }
    let current = enabled.iter().position(|item| Some(*item) == current)?;
    match key {
        Key::ArrowDown => enabled.get((current + 1) % enabled.len()).copied(),
        Key::ArrowUp => enabled
            .get((current + enabled.len() - 1) % enabled.len())
            .copied(),
        Key::Character(character) if character.chars().count() == 1 => {
            let prefix = character.to_lowercase();
            (1..=enabled.len())
                .map(|offset| enabled[(current + offset) % enabled.len()])
                .find(|candidate| entries[candidate].label.to_lowercase().starts_with(&prefix))
        }
        _ => None,
    }
}

// Dioxus moves keyed DOM nodes with insertBefore, which can reset browser focus
// even though the Rust component and its element both survive. This bridge
// preserves only existing native focus; selection and navigation stay in Rust.
#[component]
fn DropdownFocusRetention(menu_id: String) -> Element {
    let transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let dropped = transport.clone();
    use_drop(move || {
        if let Some(task) = dropped.borrow_mut().take() {
            let _ = task.send("finish");
        }
    });
    use_effect(move || {
        let task = dioxus::document::eval(DROPDOWN_FOCUS_RETENTION);
        if task.send(menu_id.clone()).is_ok() {
            if let Some(previous) = transport.borrow_mut().replace(task) {
                let _ = previous.send("finish");
            }
        }
    });
    VNode::empty()
}

const DROPDOWN_FOCUS_RETENTION: &str = r#"
const id = await dioxus.recv(), menu = document.getElementById(id);
if (!menu) return;
const owned = target => {
  const item = target instanceof Element ? target.closest('[data-ix-dropdown-item]') : null;
  return item?.closest('[role="menu"]') === menu ? item : null;
};
let focused = owned(document.activeElement), disposed = false, removed;
const removal = new Promise(resolve => { removed = resolve; });
const fallback = () => document.activeElement === document.body || document.activeElement === menu;
const contains = (nodes, target) => [...nodes].some(node => node === target || node.contains(target));
const changed = records => {
  if (disposed) return;
  if (!menu.isConnected) { removed(); return; }
  const target = focused;
  if (!target || !target.isConnected || owned(target) !== target || target.disabled || !fallback()) return;
  if (records.some(record => contains(record.removedNodes, target)) &&
      records.some(record => contains(record.addedNodes, target))) {
    target.focus({preventScroll: true});
  }
};
const observer = new MutationObserver(changed);
const onFocus = event => { focused = owned(event.target); };
const clear = () => { focused = null; };
const onPointer = event => { if (!menu.contains(event.target)) clear(); };
const onFocusOut = event => {
  if (event.target !== focused) return;
  if (event.relatedTarget) { focused = owned(event.relatedTarget); return; }
  const target = focused;
  queueMicrotask(() => {
    if (focused !== target || disposed) return;
    // A move and its focusout belong to the same DOM mutation batch. An
    // explicit blur without that move must invalidate the remembered node.
    changed(observer.takeRecords());
    if (focused === target && fallback()) clear();
  });
};
document.addEventListener('focusin', onFocus, true);
document.addEventListener('focusout', onFocusOut, true);
document.addEventListener('pointerdown', onPointer, true);
window.addEventListener('blur', clear);
observer.observe(document.body, {childList: true, subtree: true});
try { await Promise.race([dioxus.recv(), removal]); }
finally {
  disposed = true; observer.disconnect();
  document.removeEventListener('focusin', onFocus, true);
  document.removeEventListener('focusout', onFocusOut, true);
  document.removeEventListener('pointerdown', onPointer, true);
  window.removeEventListener('blur', clear);
}
"#;

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
    rsx! {
        span { id: anchor_id.clone(), class: "ix-tooltip-anchor {class}",
            {children}
            crate::TooltipFor {
                id, anchor_id: anchor_id.clone(), title, title_icon, placement, show_delay, hide_delay,
                interactive, animation_frame, onnativeerror,
                "{label}"
            }
        }
    }
}

/// Scoped controller shared by an external [`DropdownTrigger`] and [`Dropdown`].
/// Mount one dropdown surface per controller. Opening and selection remain Rust.
#[derive(Clone, Copy, PartialEq)]
pub struct DropdownHandle {
    context: Signal<DropdownContext>,
    blocked: Signal<bool>,
    id: usize,
}

impl DropdownHandle {
    fn context(self) -> DropdownContext {
        self.context.peek().clone()
    }

    pub fn is_open(self) -> bool {
        (self.context().open)()
    }

    /// Open and focus the first enabled item, unless the surface is disabled.
    pub fn open(self) {
        self.open_at(false);
    }

    fn open_at(self, last: bool) {
        if *self.blocked.peek() {
            return;
        }
        let mut context = self.context();
        context.invalidate_focus();
        context.focus_last.set(last);
        context.focus_on_open.set(true);
        if *context.open.peek() {
            context.focus_edge(last);
        } else {
            context.open.set(true);
        }
    }

    /// Close this menu and all descendants, optionally returning trigger focus.
    pub fn close(self, return_focus: bool) {
        self.context().close(return_focus);
    }

    pub fn toggle(self) {
        if *self.context().open.peek() {
            self.close(false);
        } else {
            self.open();
        }
    }
}

/// Create a controller for a menu whose trigger is elsewhere in the Rust tree.
pub fn use_dropdown() -> DropdownHandle {
    let context = DropdownContext {
        open: use_signal(|| false),
        focus_last: use_signal(|| false),
        focus_on_open: use_signal(|| true),
        entries: use_hook(|| Rc::new(RefCell::new(BTreeMap::new()))),
        trigger: use_hook(|| Rc::new(RefCell::new(None))),
        parent: None,
        child: use_hook(|| Rc::new(RefCell::new(None))),
        focus_epoch: use_hook(|| Rc::new(Cell::new(0))),
        alive: use_hook(|| Rc::new(Cell::new(true))),
    };
    let dropped = context.clone();
    use_drop(move || {
        dropped.alive.set(false);
    });
    DropdownHandle {
        context: use_signal(|| context),
        blocked: use_signal(|| false),
        id: use_hook(|| MENU_ITEM_ID.fetch_add(1, Ordering::Relaxed)),
    }
}

/// An external menu button, with the controller's native anchor and ARIA links.
#[component]
pub fn DropdownTrigger(
    dropdown: DropdownHandle,
    children: Element,
    #[props(default = crate::ButtonVariant::Primary)] variant: crate::ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    icon: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let context = dropdown.context();
    let trigger = context.trigger.clone();
    let dropped = trigger.clone();
    use_drop(move || {
        dropped.borrow_mut().take();
    });
    let blocked = disabled || loading || (dropdown.blocked)();
    rsx! { button {
        class: "ix-dropdown-external-trigger ix-button ix-button-{variant.as_str()} {class}",
        r#type: "button", disabled: blocked, "aria-label": aria_label,
        "aria-busy": loading.to_string(), "aria-haspopup": "menu",
        "aria-expanded": dropdown.is_open().to_string(), "aria-controls": "ix-dropdown-{dropdown.id}",
        style: "anchor-name:--ix-dropdown-anchor-{dropdown.id}",
        onmounted: move |event| *trigger.borrow_mut() = Some(event.data()),
        onclick: move |_| dropdown.toggle(),
        onkeydown: move |event: KeyboardEvent| match event.key() {
            Key::ArrowDown | Key::ArrowUp => { event.prevent_default(); event.stop_propagation(); dropdown.open_at(event.key() == Key::ArrowUp); }
            Key::Escape if dropdown.is_open() => { event.prevent_default(); event.stop_propagation(); dropdown.close(true); }
            _ => {}
        },
        if loading { crate::Spinner { size: crate::SpinnerSize::Sm, hide_track: true } }
        else if let Some(icon) = icon { Icon { name: icon, size: 20 } }
        {children}
    } }
}

/// Upstream's eight side/alignment combinations for a root menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropdownPlacement {
    #[default]
    BottomStart,
    BottomEnd,
    TopStart,
    TopEnd,
    LeftStart,
    LeftEnd,
    RightStart,
    RightEnd,
}
impl DropdownPlacement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BottomStart => "bottom-start",
            Self::BottomEnd => "bottom-end",
            Self::TopStart => "top-start",
            Self::TopEnd => "top-end",
            Self::LeftStart => "left-start",
            Self::LeftEnd => "left-end",
            Self::RightStart => "right-start",
            Self::RightEnd => "right-end",
        }
    }
    const fn area(self) -> &'static str {
        match self {
            Self::BottomStart => "bottom span-right",
            Self::BottomEnd => "bottom span-left",
            Self::TopStart => "top span-right",
            Self::TopEnd => "top span-left",
            Self::LeftStart => "left span-bottom",
            Self::LeftEnd => "left span-top",
            Self::RightStart => "right span-bottom",
            Self::RightEnd => "right span-top",
        }
    }
}

/// Dropdown with outside dismissal, return focus, and roving keyboard navigation.
/// Use [`DropdownSubmenu`] for a nested menu with independent keyboard navigation.
/// Passing `controller` omits the built-in trigger; render [`DropdownTrigger`]
/// elsewhere with the same handle. CSS anchors follow its actual position.
#[component]
pub fn Dropdown(
    label: String,
    controller: Option<DropdownHandle>,
    #[props(default)] placement: DropdownPlacement,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    #[props(default = crate::ButtonVariant::Secondary)] variant: crate::ButtonVariant,
    icon: Option<String>,
    aria_label: Option<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let internal = use_dropdown();
    let external = controller.is_some();
    let mut handle = controller.unwrap_or(internal);
    let context = handle.context();
    let open = context.open;
    let blocked = disabled || loading;
    let id = handle.id;
    use_effect(use_reactive!(|blocked| {
        handle.blocked.set(blocked);
        if blocked {
            handle.close(false);
        }
    }));
    use_drop(move || handle.close(false));
    use_context_provider(|| context.clone());
    let escape_context = context.clone();
    let mounted_trigger = context.trigger.clone();
    let backdrop_context = context.clone();
    rsx! {
        div { class: "ix-dropdown {class}",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape { event.stop_propagation(); escape_context.close(true); }
                else if event.key() == Key::Tab { escape_context.close(false); }
            },
            if !external { button { r#type: "button", class: "ix-dropdown-trigger ix-button ix-button-{variant.as_str()}", disabled: blocked,
                style: "anchor-name:--ix-dropdown-anchor-{id}", "aria-controls": "ix-dropdown-{id}",
                "aria-label": aria_label, "aria-busy": loading.to_string(),
                "aria-expanded": (open() && !blocked).to_string(), "aria-haspopup": "menu",
                onmounted: move |event| *mounted_trigger.borrow_mut() = Some(event.data()),
                onclick: move |_| handle.toggle(),
                onkeydown: move |event: KeyboardEvent| {
                    if matches!(event.key(), Key::ArrowDown | Key::ArrowUp) {
                        event.prevent_default(); event.stop_propagation();
                        handle.open_at(event.key() == Key::ArrowUp);
                    }
                },
                if loading { crate::Spinner { size: crate::SpinnerSize::Sm, hide_track: true } }
                else if let Some(icon) = icon { Icon { name: icon, size: 20 } }
                "{label}" Icon { name: "chevron-down-small", size: 16 }
            } }
            if open() && !blocked {
                button { class: "ix-dropdown-backdrop", r#type: "button", tabindex: "-1", "aria-label": "Close dropdown",
                    onclick: move |_| backdrop_context.close(false) }
                div { id: "ix-dropdown-{id}", class: "ix-dropdown-menu ix-dropdown-root-menu", role: "menu", "aria-label": label.clone(),
                    "data-placement": placement.as_str(),
                    style: "position-anchor:--ix-dropdown-anchor-{id};--ix-dropdown-area:{placement.area()}",
                    DropdownFocusRetention { menu_id: "ix-dropdown-{id}" }
                    {children} }
            }
        }
    }
}

/// A category label within a dropdown menu.
#[component]
pub fn DropdownHeader(label: String) -> Element {
    rsx! { div { class: "ix-dropdown-header", role: "presentation", "{label}" } }
}

/// A nested menu entry. The parent stays open while its submenu is active.
///
/// Right Arrow opens the submenu and focuses its first enabled entry. Left Arrow
/// or Escape closes only that level and returns focus to its trigger. Selecting
/// a leaf entry closes the entire menu hierarchy. CSS anchor positioning flips
/// the submenu at viewport edges; older browsers use a contained fixed fallback.
#[component]
pub fn DropdownSubmenu(
    label: String,
    icon: Option<String>,
    #[props(default)] disabled: bool,
    children: Element,
) -> Element {
    let id = use_hook(|| MENU_ITEM_ID.fetch_add(1, Ordering::Relaxed));
    // This hook caches the ancestor before providing this level. A raw context
    // lookup on later renders would instead find our own provider.
    let parent = try_use_context::<DropdownContext>();
    let open = use_signal(|| false);
    let focus_last = use_signal(|| false);
    let mut focus_on_open = use_signal(|| false);
    let entries = use_hook(|| Rc::new(RefCell::new(BTreeMap::new())));
    let trigger = use_hook(|| Rc::new(RefCell::new(None)));
    let child = use_hook(|| Rc::new(RefCell::new(None)));
    let focus_epoch = use_hook(|| {
        parent
            .as_ref()
            .map(|parent| parent.focus_epoch.clone())
            .unwrap_or_else(|| Rc::new(Cell::new(0)))
    });
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let context = DropdownContext {
        open,
        focus_last,
        focus_on_open,
        entries,
        trigger,
        parent: parent.clone().map(Rc::new),
        child,
        focus_epoch,
        alive,
    };
    use_context_provider(|| context.clone());
    let dropped_parent = parent.clone();
    let dropped_context = context.clone();
    use_drop(move || {
        dropped_context.alive.set(false);
        if let Some(parent) = dropped_parent {
            parent.entries.borrow_mut().remove(&id);
            if parent
                .child
                .borrow()
                .as_ref()
                .is_some_and(|child| child.id == id)
            {
                parent.child.borrow_mut().take();
            }
        }
    });
    if let Some(parent) = &parent {
        if let Some(entry) = parent.entries.borrow_mut().get_mut(&id) {
            entry.label = label.clone();
            entry.disabled = disabled;
        }
    }
    let mount_parent = parent.clone();
    let mount_trigger = context.trigger.clone();
    let mount_label = label.clone();
    let hover_parent = parent.clone();
    let hover_context = context.clone();
    let click_parent = parent.clone();
    let click_context = context.clone();
    let key_parent = parent.clone();
    let key_context = context.clone();
    let level_context = context.clone();
    let focus_parent = parent.clone();
    rsx! {
        div { class: "ix-dropdown-submenu",
            onkeydown: move |event: KeyboardEvent| match event.key() {
                Key::ArrowLeft | Key::Escape if open() => {
                    event.prevent_default(); event.stop_propagation(); level_context.close(true);
                }
                Key::Tab => { event.stop_propagation(); level_context.close_all(false); }
                _ => {}
            },
            button { class: "ix-dropdown-item ix-dropdown-submenu-trigger", r#type: "button", role: "menuitem", "data-ix-dropdown-item": id,
                disabled, tabindex: "-1", "aria-haspopup": "menu", "aria-expanded": open().to_string(),
                "aria-controls": "ix-submenu-{id}", style: "anchor-name:--ix-submenu-anchor-{id}",
                onmounted: move |event| {
                    *mount_trigger.borrow_mut() = Some(event.data());
                    if let Some(parent) = &mount_parent {
                        parent.entries.borrow_mut().insert(id, MenuEntry { label: mount_label.clone(), disabled, mounted: event.data() });
                        if (parent.focus_on_open)() { parent.focus_edge_if_needed((parent.focus_last)(), true); }
                    }
                },
                onfocusin: move |_| if let Some(parent) = &focus_parent { parent.close_child_except(Some(id)); },
                onmouseenter: move |_| if !disabled {
                    focus_on_open.set(false);
                    if let Some(parent) = &hover_parent { parent.open_child(id, &hover_context); }
                },
                onclick: move |_| if !disabled {
                    focus_on_open.set(true);
                    if let Some(parent) = &click_parent { parent.open_child(id, &click_context); }
                    click_context.focus_edge(false);
                },
                onkeydown: move |event: KeyboardEvent| {
                    if disabled { return; }
                    if event.key() == Key::ArrowRight {
                        event.prevent_default(); event.stop_propagation(); focus_on_open.set(true);
                        if let Some(parent) = &key_parent { parent.open_child(id, &key_context); }
                        key_context.focus_edge(false);
                    } else if matches!(event.key(), Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End | Key::Character(_))
                        && !matches!(event.key(), Key::Character(ref key) if key == " ") {
                        event.prevent_default(); event.stop_propagation(); key_context.close(false);
                        if let Some(parent) = &key_parent { parent.navigate(id, &event.key()); }
                    }
                },
                if let Some(icon) = icon { Icon { name: icon, size: 20 } }
                span { "{label}" }
                Icon { name: "chevron-right-small", size: 16 }
            }
            if open() && !disabled {
                div { id: "ix-submenu-{id}", class: "ix-dropdown-menu ix-dropdown-submenu-menu", role: "menu", "aria-label": label,
                    style: "position-anchor:--ix-submenu-anchor-{id}",
                    DropdownFocusRetention { menu_id: "ix-submenu-{id}" }
                    {children} }
            }
        }
    }
}

/// Menu entry with optional icon, check state, and a trailing slot.
#[component]
pub fn DropdownItem(
    label: String,
    /// Render an icon-only action while retaining its menu-item label and navigation.
    #[props(default)]
    compact: bool,
    #[props(default)] icon: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] checked: bool,
    #[props(default)] checkable: bool,
    #[props(default)] keep_open: bool,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] children: Option<Element>,
) -> Element {
    let id = use_hook(|| MENU_ITEM_ID.fetch_add(1, Ordering::Relaxed));
    let context = try_use_context::<DropdownContext>();
    let mount_context = context.clone();
    let click_context = context.clone();
    let key_context = context.clone();
    let focus_context = context.clone();
    let hover_context = context.clone();
    let drop_context = context.clone();
    let entry_label = label.clone();
    use_drop(move || {
        if let Some(context) = drop_context {
            context.entries.borrow_mut().remove(&id);
        }
    });
    if let Some(context) = &context {
        if let Some(entry) = context.entries.borrow_mut().get_mut(&id) {
            entry.disabled = disabled;
            entry.label = label.clone();
        }
    }
    rsx! {
        button { class: if compact { "ix-dropdown-item ix-dropdown-quick-action" } else { "ix-dropdown-item" }, r#type: "button", role: if checkable || checked { "menuitemcheckbox" } else { "menuitem" }, "data-ix-dropdown-item": id,
            "aria-label": if compact { Some(label.clone()) } else { None }, title: if compact { Some(label.clone()) } else { None },
            disabled, tabindex: "-1", "aria-checked": if checkable || checked { Some(checked.to_string()) } else { None },
            onmounted: move |event| {
                if let Some(context) = &mount_context {
                    context.entries.borrow_mut().insert(id, MenuEntry { label: entry_label.clone(), disabled, mounted: event.data() });
                    if (context.focus_on_open)() { context.focus_edge_if_needed((context.focus_last)(), true); }
                }
            },
            onclick: move |event| {
                if !disabled {
                    if let Some(handler) = onclick { handler.call(event); }
                    if !keep_open { if let Some(context) = &click_context { context.close_all(true); } }
                }
            },
            onfocusin: move |_| if let Some(context) = &focus_context { context.close_child_except(None); },
            onmouseenter: move |_| if let Some(context) = &hover_context { context.close_child_except(None); },
            onkeydown: move |event: KeyboardEvent| {
                if let Some(context) = &key_context {
                    if matches!(event.key(), Key::ArrowDown | Key::ArrowUp | Key::Home | Key::End | Key::Character(_))
                        && !matches!(event.key(), Key::Character(ref key) if key == " ") {
                        event.prevent_default(); event.stop_propagation(); context.navigate(id, &event.key());
                    }
                }
            },
            if let Some(name) = icon { Icon { name, size: 20 } }
            if !compact { span { "{label}" } }
            {children}
            if checked { Icon { name: "single-check", size: 16 } }
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
.ix-dropdown { display:inline-block;position:relative; }
.ix-dropdown-backdrop { position:fixed;inset:0;z-index:18;border:0;padding:0;background:transparent;cursor:default!important; }
.ix-dropdown-trigger { position:relative;z-index:19; }
.ix-dropdown-trigger { display:flex;align-items:center;gap:12px;padding:6px 12px;min-height:var(--ix-control-height);border:1px solid var(--ix-color-primary);border-radius:var(--ix-radius);color:var(--ix-color-primary);background:transparent; }
.ix-dropdown-menu { position:absolute;top:calc(100% + 4px);left:0;z-index:20;min-width:200px;max-height:320px;overflow:auto;padding:4px 0;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);box-shadow:0 6px 20px #0004; }
.ix-dropdown-external-trigger{position:relative;z-index:19}
.ix-dropdown-root-menu{position:fixed;inset:50% auto auto 50%;transform:translate(-50%,-50%);box-sizing:border-box;width:max-content;min-width:min(200px,calc(100vw - 16px));max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px))}
@supports(position-anchor:--example) and (position-area:bottom){
 .ix-dropdown-root-menu{inset:auto;transform:none;position-area:var(--ix-dropdown-area);justify-self:start;align-self:start;margin:4px;position-try-fallbacks:flip-block,flip-inline,flip-block flip-inline,--ix-dropdown-contained}
 .ix-dropdown-root-menu[data-placement^=top]{align-self:end}
 .ix-dropdown-root-menu[data-placement=bottom-end],.ix-dropdown-root-menu[data-placement=top-end],.ix-dropdown-root-menu[data-placement^=left]{justify-self:end}
 .ix-dropdown-root-menu[data-placement=left-end],.ix-dropdown-root-menu[data-placement=right-end]{align-self:end}
 @position-try --ix-dropdown-contained{position-area:none;inset:8px;margin:auto;height:max-content;max-width:calc(100% - 16px);max-height:min(320px,calc(100dvh - 16px))}
}
.ix-dropdown-item { display:flex;align-items:center;gap:8px;width:100%;padding:8px 16px;border:0;text-align:left;color:var(--ix-color-text);background:transparent; }
.ix-dropdown-item>span { flex:1; }
.ix-dropdown-item.ix-dropdown-quick-action { width:40px;height:40px;justify-content:center;padding:8px; }
.ix-dropdown-item:hover:not(:disabled) { background:var(--ix-color-surface-2); }
.ix-dropdown-item:disabled { opacity:.45; }
.ix-dropdown-header { padding:8px 16px;font-size:14px;color:var(--ix-color-text-secondary); }
.ix-dropdown-submenu-menu{position:fixed;inset:50% auto auto 50%;transform:translate(-50%,-50%);box-sizing:border-box;width:min(200px,calc(100vw - 16px));min-width:0;max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px));z-index:21}
.ix-dropdown-submenu-trigger[aria-expanded=true]{background:var(--ix-color-surface-2)}
@supports (position-anchor:--example) and (position-area:right){
 .ix-dropdown-submenu-menu{inset:auto;transform:none;position-area:right span-bottom;align-self:start;margin:0 4px;position-try-fallbacks:flip-inline,flip-block,flip-inline flip-block,--ix-submenu-contained,--ix-submenu-contained-above}
 @position-try --ix-submenu-contained{position-area:bottom span-right;inset:auto;justify-self:start;align-self:start;margin:4px 0;width:min(200px,calc(100vw - 16px));height:max-content;max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px))}
 @position-try --ix-submenu-contained-above{position-area:top span-right;inset:auto;justify-self:start;align-self:end;margin:4px 0;width:min(200px,calc(100vw - 16px));height:max-content;max-width:calc(100vw - 16px);max-height:min(320px,calc(100dvh - 16px))}
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entries() -> BTreeMap<usize, MenuEntry> {
        [
            (1, "A", false),
            (2, "Unavailable", true),
            (3, "B", false),
            (4, "C", false),
        ]
        .into_iter()
        .map(|(id, label, disabled)| {
            (
                id,
                MenuEntry {
                    label: label.into(),
                    disabled,
                    mounted: Rc::new(MountedData::new(())),
                },
            )
        })
        .collect()
    }

    #[test]
    fn keyed_menu_navigation_uses_live_order_and_skips_disabled_entries() {
        let entries = test_entries();
        let original = [1, 2, 3, 4];
        let moved = [4, 1, 2, 3];
        assert_eq!(
            menu_target(&entries, &original, Some(1), &Key::ArrowDown),
            Some(3)
        );
        assert_eq!(
            menu_target(&entries, &moved, Some(4), &Key::ArrowDown),
            Some(1)
        );
        assert_eq!(
            menu_target(&entries, &moved, Some(1), &Key::ArrowDown),
            Some(3)
        );
        assert_eq!(
            menu_target(&entries, &moved, Some(4), &Key::ArrowUp),
            Some(3)
        );
        assert_eq!(menu_target(&entries, &moved, Some(1), &Key::Home), Some(4));
        assert_eq!(menu_target(&entries, &moved, Some(1), &Key::End), Some(3));
        assert_eq!(
            menu_target(&entries, &moved, Some(4), &Key::Character("b".into())),
            Some(3)
        );
        assert_eq!(
            menu_target(&entries, &moved, Some(3), &Key::Character("c".into())),
            Some(4)
        );
    }

    #[test]
    fn menu_navigation_uses_only_registered_owned_items_and_handles_empty_menus() {
        let entries = test_entries();
        // A nested level's IDs cannot participate in its ancestor's registry.
        assert_eq!(
            menu_target(&entries, &[99, 4, 100, 3], None, &Key::Home),
            Some(4)
        );
        assert_eq!(
            menu_target(&entries, &[99, 4, 100, 3], Some(4), &Key::ArrowDown),
            Some(3)
        );
        assert_eq!(menu_target(&entries, &[2, 99], None, &Key::End), None);
        assert_eq!(menu_target(&entries, &[], Some(4), &Key::ArrowUp), None);
        assert_eq!(
            menu_target(&entries, &[1, 3], Some(4), &Key::ArrowDown),
            None
        );
    }

    fn use_menu(parent: Option<DropdownContext>) -> DropdownContext {
        let focus_epoch = use_hook(|| {
            parent
                .as_ref()
                .map(|parent| parent.focus_epoch.clone())
                .unwrap_or_else(|| Rc::new(Cell::new(0)))
        });
        DropdownContext {
            open: use_signal(|| true),
            focus_last: use_signal(|| false),
            focus_on_open: use_signal(|| true),
            entries: use_hook(|| Rc::new(RefCell::new(BTreeMap::new()))),
            trigger: use_hook(|| Rc::new(RefCell::new(None))),
            parent: parent.map(Rc::new),
            child: use_hook(|| Rc::new(RefCell::new(None))),
            focus_epoch,
            alive: use_hook(|| Rc::new(Cell::new(true))),
        }
    }

    #[test]
    fn opening_submenu_cancels_pending_parent_focus_and_close_cancels_descendants() {
        fn probe() -> Element {
            let root = use_menu(None);
            let child = use_menu(Some(root.clone()));
            let independent = use_menu(None);
            use_hook(move || {
                let first_navigation = root.invalidate_focus();
                assert!(root.focus_is_current(first_navigation));
                let next_navigation = root.invalidate_focus();
                assert!(!root.focus_is_current(first_navigation));
                assert!(root.focus_is_current(next_navigation));

                root.open_child(7, &child);
                assert!(!root.focus_is_current(next_navigation));
                let child_navigation = child.invalidate_focus();
                assert!(child.focus_is_current(child_navigation));
                independent.invalidate_focus();
                assert!(child.focus_is_current(child_navigation));

                root.close(false);
                assert!(!root.focus_is_current(child_navigation));
                assert!(!child.focus_is_current(child_navigation));
                let mut open = root.open;
                open.set(true);
                let reopened = root.invalidate_focus();
                assert!(!root.focus_is_current(child_navigation));
                assert!(root.focus_is_current(reopened));
                root.alive.set(false);
                assert!(!root.focus_is_current(reopened));
            });
            VNode::empty()
        }
        VirtualDom::new(probe).rebuild_in_place();
    }

    #[test]
    fn switching_submenus_closes_descendants_but_keeps_the_root_open() {
        fn probe() -> Element {
            let root = use_menu(None);
            let first = use_menu(Some(root.clone()));
            let grandchild = use_menu(Some(first.clone()));
            let sibling = use_menu(Some(root.clone()));
            use_hook(move || {
                root.open_child(1, &first);
                first.open_child(2, &grandchild);
                root.open_child(3, &sibling);
                assert!(*root.open.peek());
                assert!(!*first.open.peek());
                assert!(!*grandchild.open.peek());
                assert!(*sibling.open.peek());
                sibling.close(false);
                assert!(*root.open.peek());
                assert!(!*sibling.open.peek());
                root.open_child(3, &sibling);
                sibling.close_all(false);
                assert!(!*root.open.peek());
                assert!(!*sibling.open.peek());
            });
            VNode::empty()
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
    }

    #[test]
    fn nested_trigger_exposes_its_menu_relationship_without_eager_child_content() {
        fn probe() -> Element {
            let context = use_menu(None);
            use_context_provider(|| context);
            rsx! { DropdownSubmenu { label: "Tools", DropdownItem { label: "Calibrate" } } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("aria-haspopup=\"menu\""));
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("aria-controls=\"ix-submenu-"));
        assert!(html.contains("Tools"));
        assert!(!html.contains("Calibrate"));
    }

    #[test]
    fn external_controller_is_scoped_and_closes_nested_levels() {
        fn probe() -> Element {
            let first = use_dropdown();
            let second = use_dropdown();
            let child = use_menu(Some(first.context()));
            use_hook(move || {
                first.open();
                assert!(*first.context().open.peek());
                assert!(!*second.context().open.peek());
                first.context().open_child(42, &child);
                first.close(false);
                assert!(!*first.context().open.peek());
                assert!(!*child.open.peek());
                let mut blocked = first.blocked;
                blocked.set(true);
                first.open();
                assert!(!*first.context().open.peek());
                second.open_at(true);
                assert!(*second.context().focus_last.peek());
                assert!(*second.context().open.peek());
            });
            VNode::empty()
        }
        VirtualDom::new(probe).rebuild_in_place();
    }

    #[test]
    fn external_trigger_and_separate_surface_share_a_unique_aria_anchor() {
        fn probe() -> Element {
            let handle = use_dropdown();
            use_hook(move || handle.open());
            rsx! {
                div { DropdownTrigger { dropdown: handle, "External trigger" } }
                div { Dropdown { controller: handle, label: "External menu", placement: DropdownPlacement::TopEnd,
                    DropdownItem { label: "Calibrate" }
                } }
            }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let control = html
            .split("aria-controls=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(html.contains(&format!("id=\"{control}\"")));
        assert_eq!(html.matches("ix-dropdown-external-trigger").count(), 1);
        assert!(!html.contains("class=\"ix-dropdown-trigger "));
        assert!(html.contains("data-placement=\"top-end\""));
        assert!(html.contains("Calibrate"));
    }
}
