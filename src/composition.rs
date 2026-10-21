//! Keyed tab panels and composed dropdown controls.
//!
//! Adapted from Siemens iX (MIT). Child definitions use Rust data and slots;
//! the tab strip uses Dioxus mounted measurements and resize events for overflow.
//! Selection and panel state remain in Rust.

use crate::layout::{
    TabKeySelectionChange, TabKeyboardNavigation, TabPlacement, TabSelection, TabSelectionChange,
};
use crate::{ButtonVariant, Dropdown, Icon, TabItem, Tabs};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static TAB_SET_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
pub(crate) struct TabSetContext {
    prefix: String,
    selected: Memo<Option<String>>,
}

/// Create a DOM-safe, injective suffix from a stable user-provided tab key.
pub(crate) fn tab_key_id(key: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(key.len() * 2);
    for byte in key.bytes() {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

/// Explicit keyed selection. `Empty` keeps every persistent panel hidden.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabKeySelection {
    Empty,
    Key(String),
}

/// Preserve established fallback for invalid keys; intentional empty is separate.
fn active_tab_key(items: &[TabItem], requested: Option<&str>) -> Option<String> {
    let requested = requested?;
    items
        .iter()
        .find(|item| item.key == requested && !item.disabled)
        .or_else(|| items.iter().find(|item| !item.disabled))
        .map(|item| item.key.clone())
}

/// Manage a tab list and persistent keyed [`TabPanel`] children.
///
/// The data-driven `items` list replaces upstream child inspection. Panel state
/// survives switching because inactive panels remain mounted with `hidden`.
/// Keys must be unique and must match the corresponding panel's `tab_key`.
#[component]
pub fn TabSet(
    items: Vec<TabItem>,
    children: Element,
    active_key: Option<String>,
    selection: Option<TabKeySelection>,
    default_selection: Option<TabKeySelection>,
    #[props(default)] onbeforechange: EventHandler<TabKeySelectionChange>,
    #[props(default)] onselectionchange: EventHandler<Option<String>>,
    #[props(default)] default_active_key: String,
    #[props(default)] onselectedchange: EventHandler<String>,
    #[props(default)] onclose: EventHandler<String>,
    /// Format each native close button's accessible label from its current item.
    #[props(default)]
    close_label: Option<Callback<TabItem, String>>,
    /// Compose noninteractive primary tab content, forwarded to [`Tabs`].
    /// Keep each item's label meaningful for native/overflow/close names; return
    /// a reactive child component for live values rather than calling hooks here.
    #[props(default)]
    render_label: Option<Callback<TabItem, Element>>,
    /// Force the all-tabs menu, including when every tab fits.
    #[props(default)]
    overflow_menu: bool,
    #[props(default = "Show all tabs".to_string())] aria_label_more_tabs: String,
    #[props(default)] small: bool,
    #[props(default)] rounded: bool,
    #[props(default)] stretched: bool,
    #[props(default)] keyboard_navigation: TabKeyboardNavigation,
    #[props(default)] placement: TabPlacement,
    #[props(default = "Tabs".to_string())] label: String,
    #[props(default)] class: String,
) -> Element {
    let mut internal_key = use_signal(|| match default_selection {
        Some(TabKeySelection::Empty) => None,
        Some(TabKeySelection::Key(key)) => Some(key),
        None => Some(default_active_key),
    });
    let controlled = selection.is_some() || active_key.is_some();
    let selected = use_memo(use_reactive!(|items, active_key, selection| {
        let requested = match &selection {
            Some(TabKeySelection::Empty) => None,
            Some(TabKeySelection::Key(key)) => Some(key.clone()),
            None => active_key.clone().or_else(|| internal_key.read().clone()),
        };
        active_tab_key(&items, requested.as_deref())
    }));
    let prefix = use_hook(|| format!("ix-tabset-{}", TAB_SET_ID.fetch_add(1, Ordering::Relaxed)));
    use_context_provider(|| TabSetContext {
        prefix: prefix.clone(),
        selected,
    });
    let selected_index = items
        .iter()
        .position(|item| Some(&item.key) == selected.read().as_ref());
    let before_items = items.clone();
    let select_items = items.clone();
    let close_items = items.clone();
    rsx! {
        div { class: "ix-tab-set {class}",
            Tabs { items, selection: selected_index.map_or(TabSelection::Empty, TabSelection::Index), small, rounded, stretched, keyboard_navigation, placement, label, id_prefix: prefix,
                close_label, render_label, overflow_menu, aria_label_more_tabs,
                onbeforechange: move |event: TabSelectionChange| {
                    let keyed = TabKeySelectionChange::new(
                        event.previous.and_then(|index| before_items.get(index).map(|item| item.key.clone())),
                        event.next.and_then(|index| before_items.get(index).map(|item| item.key.clone())),
                    );
                    onbeforechange.call(keyed.clone());
                    if keyed.default_prevented() { event.prevent_default(); }
                },
                onselectionchange: move |index: Option<usize>| {
                    let key = index.and_then(|index| select_items.get(index).map(|item| item.key.clone()));
                    if !controlled { internal_key.set(key.clone()); }
                    onselectionchange.call(key.clone());
                    if let Some(key) = key { onselectedchange.call(key); }
                },
                onclose: move |index: usize| { if let Some(item) = close_items.get(index) { onclose.call(item.key.clone()); } },
            }
            {children}
        }
    }
}

/// Content associated with a stable tab key. Hidden until selected by [`TabSet`].
/// Use `active` and `labelled_by` when placing a panel outside a managed set.
#[component]
pub fn TabPanel(
    tab_key: String,
    children: Element,
    #[props(default)] active: bool,
    labelled_by: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let context = try_use_context::<TabSetContext>();
    let selected = context.as_ref().map_or(active, |context| {
        (context.selected)().as_deref() == Some(tab_key.as_str())
    });
    let id = context
        .as_ref()
        .map(|context| format!("{}-panel-{}", context.prefix, tab_key_id(&tab_key)));
    let labelled_by = labelled_by.or_else(|| {
        context
            .as_ref()
            .map(|context| format!("{}-tab-{}", context.prefix, tab_key_id(&tab_key)))
    });
    rsx! {
        section { class: "ix-tab-panel {class}", role: "tabpanel", id, hidden: !selected,
            "aria-labelledby": labelled_by, tabindex: "0", {children} }
    }
}

/// Dismissible news banner associated with an about-menu tab.
/// `show` is controlled; close and show-more callbacks are distinct events.
/// Hiding retains composed child state. Unmount or key the owner to reset it.
#[component]
pub fn MenuAboutNews(
    label: String,
    #[props(default)] show: bool,
    #[props(default)] expanded: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default = "Close news".to_string())] close_label: String,
    about_item_label: Option<String>,
    active_about_tab_key: Option<String>,
    #[props(default = "Show more".to_string())] show_more_label: String,
    #[props(default)] onshowmore: EventHandler<Option<String>>,
    #[props(default)] onclose: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        aside { id, class: if class.is_empty() { "ix-about-news".to_string() } else { format!("ix-about-news {class}") }, hidden: !show,
            "data-expanded": expanded.to_string(), "aria-label": label.clone(),
            div { class: "ix-about-news-banner", "aria-hidden": true,
                svg { view_box: "0 0 48 56", polygon { points: "0 0 48 0 48 56 24 48 0 56" } }
                Icon { name: "shout", size: 32 }
            }
            div { class: "ix-about-news-heading",
                h3 { "{label}" }
                button { r#type: "button", class: "ix-about-news-close", "aria-label": close_label,
                    onclick: move |_| onclose.call(()), Icon { name: "close", size: 20 } }
            }
            if let Some(subtitle) = about_item_label { p { class: "ix-about-news-subtitle", "{subtitle}" } }
            div { class: "ix-about-news-content", {children} }
            if active_about_tab_key.is_some() { div { class: "ix-about-news-footer",
                crate::Button { onclick: move |_| onshowmore.call(active_about_tab_key.clone()), "{show_more_label}" }
            } }
        }
    }
}

/// Combined button and dropdown with iX button variants and optional icon.
/// Empty or omitted `label` uses a compact icon trigger. `button_label` supplies
/// presentational Dioxus content instead of the visible text; provide `label` or
/// `aria_label` for its accessible name. Menu children remain mounted as these
/// presentation props change.
#[component]
pub fn DropdownButton(
    #[props(default)] label: String,
    button_label: Option<Element>,
    #[props(default)] placement: crate::DropdownPlacement,
    children: Element,
    icon: Option<String>,
    #[props(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default)] disabled: bool,
    #[props(default)] loading: bool,
    aria_label: Option<String>,
    #[props(default)] class: String,
    #[props(default)] navigation_mode: crate::DropdownNavigationMode,
    #[props(default)] close_behavior: crate::DropdownCloseBehavior,
    #[props(default)] enable_top_layer: bool,
    #[props(default)] disable_focus_handling: bool,
    #[props(default = true)] disable_focus_trap: bool,
    #[props(default)] focus_checked_item: bool,
    #[props(default)] onbeforechange: EventHandler<crate::DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
) -> Element {
    let icon_only = button_label.is_none() && label.trim().is_empty();
    let unnamed = label.trim().is_empty();
    let label = if unnamed {
        aria_label
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "Menu".into())
    } else {
        label
    };
    let aria_label = if unnamed {
        aria_label
            .filter(|name| !name.trim().is_empty())
            .or_else(|| Some("Open dropdown".into()))
    } else {
        aria_label
    };
    rsx! { Dropdown { label, button_label, icon_only, placement, disabled, loading, variant, icon, aria_label, class,
        navigation_mode, close_behavior, enable_top_layer, disable_focus_handling,
        disable_focus_trap, focus_checked_item, onbeforechange, onchange, {children}
    } }
}

/// Horizontal group of quick action controls inside an open dropdown.
#[component]
pub fn DropdownQuickActions(
    children: Element,
    #[props(default = "Quick actions".to_string())] label: String,
) -> Element {
    rsx! { div { class: "ix-dropdown-quick-actions", role: "group", "aria-label": label, {children} } }
}

pub(crate) const STYLES: &str = include_str!("styles/composition.css");

#[cfg(test)]
#[path = "menu_about_news_tests.rs"]
mod menu_about_news_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn news_footer_requires_a_linked_about_tab() {
        fn app() -> Element {
            rsx! {
                MenuAboutNews { label: "Unlinked news", show: true, "Summary" }
                MenuAboutNews { label: "Linked news", show: true, active_about_tab_key: "details", "Summary" }
                MenuAboutNews { label: "Hidden news", active_about_tab_key: "details", "Hidden" }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("Show more").count(), 2);
        assert_eq!(html.matches("ix-about-news-banner").count(), 3);
        let hidden_news = html
            .split("<aside")
            .skip(1)
            .find(|aside| {
                aside
                    .split('>')
                    .next()
                    .unwrap()
                    .contains("aria-label=\"Hidden news\"")
            })
            .unwrap();
        assert!(hidden_news.split('>').next().unwrap().contains("hidden"));
        assert!(hidden_news.contains("Hidden"));
    }

    #[test]
    fn tab_ids_are_safe_unique_and_preserve_unicode_keys() {
        assert_ne!(tab_key_id("a b"), tab_key_id("a-b"));
        assert_ne!(tab_key_id("æ"), tab_key_id("ae"));
        assert!(
            tab_key_id("A / æ")
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        );
    }

    #[test]
    fn selection_reconciles_removed_and_disabled_keys() {
        let mut items = vec![TabItem::new("a", "A"), TabItem::new("b", "B")];
        items[0].disabled = true;
        assert_eq!(active_tab_key(&items, Some("a")), Some("b".into()));
        assert_eq!(active_tab_key(&items, Some("missing")), Some("b".into()));
        assert!(active_tab_key(&[], Some("a")).is_none());
        assert!(active_tab_key(&items, None).is_none());
    }

    #[test]
    fn explicitly_empty_keyed_set_preserves_hidden_panels_and_roving_entry() {
        fn app() -> Element {
            rsx! { TabSet { items: vec![TabItem::new("a", "A"), TabItem::new("b", "B")], selection: TabKeySelection::Empty, placement: TabPlacement::Top,
                TabPanel { tab_key: "a", "Persistent A" }
                TabPanel { tab_key: "b", "Persistent B" }
            } }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Persistent A") && html.contains("Persistent B"));
        assert_eq!(
            html.matches("hidden=true").count() + html.matches("hidden=\"true\"").count(),
            2
        );
        assert!(!html.contains("aria-selected=\"true\""));
        assert!(html.contains("tabindex=\"0\""));
        assert!(html.contains("data-placement=\"top\""));
        assert_eq!(html.matches("aria-controls=").count(), 2);
    }

    #[test]
    fn initially_empty_uncontrolled_set_does_not_select_the_first_panel() {
        fn app() -> Element {
            rsx! { TabSet { items: vec![TabItem::new("a", "A")], default_selection: TabKeySelection::Empty,
                TabPanel { tab_key: "a", "Still mounted" }
            } }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Still mounted"));
        assert!(html.contains("hidden=true") || html.contains("hidden=\"true\""));
        assert!(!html.contains("aria-selected=\"true\""));
    }

    #[test]
    fn managed_panels_are_persistent_hidden_and_linked() {
        fn app() -> Element {
            rsx! { TabSet { items: vec![TabItem::new("a", "A"), TabItem::new("b", "B")], active_key: "b",
                TabPanel { tab_key: "a", "Content A" }
                TabPanel { tab_key: "b", "Content B" }
            } }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Content A") && html.contains("Content B"));
        assert!(html.contains("role=\"tabpanel\""));
        assert!(html.contains("aria-controls="));
        assert!(html.contains("aria-labelledby="));
        assert_eq!(
            html.matches("hidden=true").count() + html.matches("hidden=\"true\"").count(),
            1
        );
    }
}
