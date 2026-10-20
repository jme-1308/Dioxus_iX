//! Explicit breadcrumb definitions and retained Dioxus dropdown composition.
use crate::{
    ButtonVariant, Dropdown, DropdownItem, DropdownNavigationMode, Icon, use_dropdown,
    use_dropdown_trigger_binding,
};
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Stable breadcrumb definition for managed truncation and next-item menus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbEntry {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
    pub href: Option<String>,
    pub target: Option<String>,
    pub rel: Option<String>,
}
impl BreadcrumbEntry {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
            href: None,
            target: None,
            rel: None,
        }
    }
}

/// Accessible labels for previous-level and current-location child menus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbLabels {
    pub previous_button: String,
    pub next_button: String,
}
impl Default for BreadcrumbLabels {
    fn default() -> Self {
        Self {
            previous_button: "Show previous breadcrumb items".into(),
            next_button: "Show next breadcrumb items".into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreadcrumbNavigationSource {
    Item,
    Previous,
    Next,
}

/// A synchronous navigation request. Clones share cancellation for this request.
/// Cancelling a visible link also prevents its native browser navigation.
#[derive(Clone, Debug)]
pub struct BreadcrumbNavigateEvent {
    pub item: BreadcrumbEntry,
    pub source: BreadcrumbNavigationSource,
    prevented: Rc<Cell<bool>>,
}
impl BreadcrumbNavigateEvent {
    fn new(item: BreadcrumbEntry, source: BreadcrumbNavigationSource) -> Self {
        Self {
            item,
            source,
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
fn hidden_breadcrumb_count(count: usize, visible: usize) -> usize {
    count.saturating_sub(visible.max(1))
}

/// Ordered breadcrumb navigation. Pass `items` for explicit managed truncation
/// and child menus, or compose children and mark a final [`BreadcrumbItem`]
/// `current`. Stable unique keys retain managed menu/item ownership and order.
///
/// Visible anchors preserve native href/target/rel and modifier behavior unless
/// `onbeforenavigate` vetoes them. Menu entries are Rust navigation actions: their
/// href is included in the callback metadata but is not executed implicitly.
/// Accepted requests call `onitem` or `onnext` once. A consumer replacing the
/// current location owns focus into its new content.
#[component]
pub fn Breadcrumb(
    #[props(default = VNode::empty())] children: Element,
    #[props(default = "Breadcrumb".to_string())] label: String,
    #[props(default)] items: Vec<BreadcrumbEntry>,
    #[props(default)] next_items: Vec<BreadcrumbEntry>,
    #[props(default = 9)] visible_item_count: usize,
    #[props(default)] onitem: EventHandler<BreadcrumbEntry>,
    #[props(default)] onnext: EventHandler<BreadcrumbEntry>,
    #[props(default)] labels: BreadcrumbLabels,
    #[props(default)] subtle: bool,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] onbeforenavigate: EventHandler<BreadcrumbNavigateEvent>,
) -> Element {
    let hidden = hidden_breadcrumb_count(items.len(), visible_item_count);
    let request = use_callback(
        move |(item, source): (BreadcrumbEntry, BreadcrumbNavigationSource)| {
            let event = BreadcrumbNavigateEvent::new(item, source);
            onbeforenavigate.call(event.clone());
            if event.default_prevented() {
                return false;
            }
            if source == BreadcrumbNavigationSource::Next {
                onnext.call(event.item);
            } else {
                onitem.call(event.item);
            }
            true
        },
    );
    rsx! {
        nav { class: "ix-breadcrumb", "aria-label": label, "data-subtle": subtle,
            ol {
                if hidden > 0 {
                    BreadcrumbMenu {
                        label: "...", aria_label: labels.previous_button,
                        entries: items.iter().take(hidden).cloned().collect(),
                        source: BreadcrumbNavigationSource::Previous, current: false,
                        subtle, enable_top_layer, onrequest: request,
                    }
                }
                for (position, item) in items.iter().enumerate().skip(hidden) {
                    if position + 1 == items.len() && !next_items.is_empty() {
                        BreadcrumbMenu {
                            key: "{item.key}", label: item.label.clone(), icon: item.icon.clone(),
                            aria_label: labels.next_button.clone(), entries: next_items.clone(),
                            source: BreadcrumbNavigationSource::Next, current: true,
                            subtle, enable_top_layer, onrequest: request,
                        }
                    } else {
                        BreadcrumbItem {
                            key: "{item.key}", label: item.label.clone(), icon: item.icon.clone(),
                            href: item.href.clone(), target: item.target.clone(), rel: item.rel.clone(),
                            current: position + 1 == items.len(), subtle,
                            onclick: { let item = item.clone(); move |event: MouseEvent| {
                                if !request.call((item.clone(), BreadcrumbNavigationSource::Item)) { event.prevent_default(); }
                            } },
                        }
                    }
                }
                {children}
            }
        }
    }
}

#[component]
fn BreadcrumbMenu(
    label: String,
    aria_label: String,
    #[props(default)] icon: Option<String>,
    entries: Vec<BreadcrumbEntry>,
    source: BreadcrumbNavigationSource,
    current: bool,
    subtle: bool,
    enable_top_layer: bool,
    onrequest: Callback<(BreadcrumbEntry, BreadcrumbNavigationSource), bool>,
) -> Element {
    static NEXT_MENU: AtomicUsize = AtomicUsize::new(1);
    let trigger_id = use_hook(|| {
        format!(
            "ix-breadcrumb-trigger-{}",
            NEXT_MENU.fetch_add(1, Ordering::Relaxed)
        )
    });
    let dropdown = use_dropdown();
    let binding = use_dropdown_trigger_binding(dropdown);
    let variant = if subtle {
        ButtonVariant::SubtlePrimary
    } else {
        ButtonVariant::Tertiary
    };
    let menu_kind = if current { "next" } else { "previous" };
    rsx! {
        li { class: "ix-breadcrumb-item", "data-subtle": subtle,
            div { class: "ix-breadcrumb-menu", "data-subtle": subtle,
                button {
                    id: trigger_id.clone(), class: "ix-dropdown-trigger ix-button ix-button-{variant.as_str()}", r#type: "button",
                    "data-breadcrumb-menu": menu_kind, "aria-label": aria_label,
                    "aria-current": current.then_some("page"), "aria-haspopup": "menu",
                    "aria-expanded": dropdown.is_open(), "aria-controls": dropdown.menu_id(),
                    "aria-activedescendant": dropdown.active_descendant(),
                    onmounted: binding.onmounted, onvisible: binding.onvisible,
                    onclick: binding.onclick, onkeydown: binding.onkeydown,
                    ..binding.attributes(),
                    if let Some(icon) = icon { Icon { name: icon, size: 16 } }
                    "{label}"
                    Icon { name: "chevron-down-small", size: 16 }
                }
                Dropdown {
                    controller: dropdown, trigger: trigger_id.clone(), label, enable_top_layer,
                    navigation_mode: DropdownNavigationMode::RovingTabindex,
                    class: "ix-breadcrumb-menu-surface",
                    for (position, item) in entries.into_iter().enumerate() {
                        BreadcrumbMenuEntry { key: "{item.key}", item, source, order: position, onrequest }
                    }
                }
            }
            span { class: "ix-breadcrumb-separator", "aria-hidden": "true", Icon { name: "chevron-right", size: 16 } }
        }
    }
}

#[component]
fn BreadcrumbMenuEntry(
    item: BreadcrumbEntry,
    source: BreadcrumbNavigationSource,
    order: usize,
    onrequest: Callback<(BreadcrumbEntry, BreadcrumbNavigationSource), bool>,
) -> Element {
    // Dropdown's generic item event owns both click and keyboard activation.
    // Its compatibility mouse callback cancels the original native click only
    // when this same request was vetoed, preserving normal menu dismissal.
    let vetoed = use_hook(|| Rc::new(Cell::new(false)));
    let native_veto = vetoed.clone();
    rsx! {
        DropdownItem {
            label: item.label.clone(), icon: item.icon.clone(), order,
            "data-breadcrumb-entry": item.key.clone(),
            onitemclick: move |event: crate::DropdownItemClickEvent| {
                let accepted = onrequest.call((item.clone(), source));
                vetoed.set(!accepted);
                if !accepted { event.prevent_default(); }
            },
            onclick: move |event: MouseEvent| if native_veto.replace(false) { event.prevent_default(); },
        }
    }
}

/// Breadcrumb link or application navigation action. A current location is
/// noninteractive. Anchors retain native metadata and defaults; `onclick` may
/// prevent navigation normally.
#[component]
pub fn BreadcrumbItem(
    label: String,
    #[props(default)] href: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] current: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] target: Option<String>,
    #[props(default)] rel: Option<String>,
    #[props(default)] subtle: bool,
) -> Element {
    rsx! { li { class: "ix-breadcrumb-item", "data-subtle": subtle,
        if current { span { "aria-current": "page", if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        else if let Some(href) = href { a { href, target, rel, onclick: move |event| onclick.call(event), if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        else { button { r#type: "button", onclick: move |event| onclick.call(event), if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        span { class: "ix-breadcrumb-separator", "aria-hidden": "true", Icon { name: "chevron-right", size: 16 } }
    } }
}

#[cfg(test)]
mod tests;
