//! Cards, navigation, disclosure, and industrial information layouts.
use crate::Icon;
use dioxus::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

/// Semantic card surface treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    #[default]
    Outline,
    Filled,
    Alarm,
    Critical,
    Warning,
    Success,
    Info,
    Neutral,
    Primary,
}
impl CardVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Filled => "filled",
            Self::Alarm => "alarm",
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Success => "success",
            Self::Info => "info",
            Self::Neutral => "neutral",
            Self::Primary => "primary",
        }
    }
}

/// Selectable card with a separate primary action, so composed actions and
/// disclosures never create nested buttons or trigger the primary selection.
/// `aria_label` names the primary action. Set `passive` to omit it entirely.
#[component]
pub fn Card(
    children: Element,
    #[props(default)] variant: CardVariant,
    #[props(default)] selected: bool,
    #[props(default)] passive: bool,
    #[props(default)] class: String,
    #[props(default = "Select card".to_string())] aria_label: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] onselect: EventHandler<()>,
) -> Element {
    if passive {
        rsx! { article { class: "ix-card ix-card-{variant.class()} {class}", "data-selected": selected, "data-passive": true, {children} } }
    } else {
        rsx! { article { class: "ix-card ix-card-{variant.class()} {class}", role: "group", "data-selected": selected, "data-passive": false,
            button { class: "ix-card-select", r#type: "button", "aria-label": aria_label, "aria-pressed": selected,
                onclick: move |event| { onclick.call(event); onselect.call(()); }
            }
            div { class: "ix-card-body", {children} }
        } }
    }
}

/// Standard card content inset.
#[component]
pub fn CardContent(children: Element, #[props(default)] class: String) -> Element {
    rsx! { div { class: "ix-card-content {class}", {children} } }
}

/// Card title with a separately composed action area.
#[component]
pub fn CardTitle(
    children: Element,
    #[props(default = VNode::empty())] actions: Element,
) -> Element {
    rsx! { div { class: "ix-card-title", div { class: "ix-card-title-text", {children} } div { class: "ix-card-title-actions", {actions} } } }
}

/// Accessible disclosure in a card footer. Supply `expanded` for controlled state.
#[component]
pub fn CardAccordion(
    children: Element,
    #[props(default = "Details".to_string())] label: String,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
) -> Element {
    let mut local = use_signal(|| default_expanded);
    let open = expanded.unwrap_or(local());
    rsx! { div { class: "ix-card-accordion",
        button { class: "ix-disclosure-button", r#type: "button", "aria-expanded": open,
            onclick: move |event| { event.stop_propagation(); local.set(!open); onexpand.call(!open); },
            span { class: "ix-chevron", "data-open": open, Icon { name: "chevron-down" } } "{label}"
        }
        div { class: "ix-card-accordion-content", hidden: !open, {children} }
    } }
}

/// Layout of a card collection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardListStyle {
    /// Cards wrap into multiple rows.
    #[default]
    Stack,
    /// Cards stay in a horizontally scrollable row.
    Scroll,
}

/// Responsive, collapsible card collection with stack and scroll layouts.
/// `show_all_count` adds a show-all action; when activated all cards are revealed.
#[component]
pub fn CardList(
    children: Element,
    #[props(default)] label: Option<String>,
    #[props(default)] list_style: CardListStyle,
    #[props(default)] collapsed: Option<bool>,
    #[props(default)] default_collapsed: bool,
    #[props(default)] oncollapse: EventHandler<bool>,
    #[props(default)] show_all_count: Option<usize>,
    #[props(default)] onshowall: EventHandler<bool>,
) -> Element {
    let mut local = use_signal(|| default_collapsed);
    let mut show_all = use_signal(|| false);
    let is_collapsed = collapsed.unwrap_or(local());
    rsx! { section { class: "ix-card-list", "data-scroll": list_style == CardListStyle::Scroll && !show_all(),
        if let Some(label) = label {
            div { class: "ix-card-list-header",
                button { class: "ix-disclosure-button", r#type: "button", "aria-expanded": !is_collapsed,
                    onclick: move |_| { local.set(!is_collapsed); oncollapse.call(!is_collapsed); },
                    span { class: "ix-chevron", "data-open": !is_collapsed, Icon { name: "chevron-down" } }
                    "{label}"
                }
                if let Some(count) = show_all_count {
                    button { class: "ix-card-list-show-all", r#type: "button", "aria-pressed": show_all(),
                        onclick: move |_| { let next = !show_all(); show_all.set(next); onshowall.call(next); },
                        if show_all() { "Show less" } else { "Show all ({count})" }
                    }
                }
            }
        }
        div { class: "ix-card-list-items", hidden: is_collapsed, {children} }
    } }
}

/// Default-density tile height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TileSize {
    Small,
    #[default]
    Medium,
    Big,
}
impl TileSize {
    fn class(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Big => "big",
        }
    }
}

/// Fixed-width information tile with header, subheader and footer areas.
#[component]
pub fn Tile(
    children: Element,
    #[props(default)] header: Option<String>,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] subheader: Option<String>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] size: TileSize,
) -> Element {
    rsx! { article { class: "ix-tile ix-tile-{size.class()}",
        if let Some(header) = header { header { class: "ix-tile-header", span { "{header}" } {header_actions} } }
        if let Some(subheader) = subheader { div { class: "ix-tile-subheader", "{subheader}" } }
        div { class: "ix-tile-content", {children} }
        if let Some(footer) = footer { footer { class: "ix-tile-footer", {footer} } }
    } }
}

/// Compact keyboard-operable action card.
#[component]
pub fn ActionCard(
    heading: String,
    #[props(default)] subheading: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] variant: CardVariant,
    #[props(default)] selected: bool,
    #[props(default)] passive: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { button { class: "ix-card ix-card-{variant.class()} ix-action-card", r#type: "button", disabled: passive, "aria-pressed": selected, "data-selected": selected, onclick: move |event| onclick.call(event),
        if let Some(icon) = icon { Icon { name: icon } } strong { "{heading}" }
        if let Some(subheading) = subheading { span { class: "ix-card-subheading", "{subheading}" } }
    } }
}

/// Notification card with expandable details.
#[component]
pub fn PushCard(
    heading: String,
    children: Element,
    #[props(default)] subheading: Option<String>,
    #[props(default)] notification: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] variant: CardVariant,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
) -> Element {
    rsx! { Card { variant, passive: true,
        CardTitle { if let Some(icon) = icon { Icon { name: icon } } "{heading}" }
        CardContent { if let Some(text) = notification { strong { class: "ix-push-notification", "{text}" } } if let Some(text) = subheading { p { class: "ix-card-subheading", "{text}" } } }
        CardAccordion { expanded, default_expanded, onexpand, {children} }
    } }
}

/// Application main content inset.
#[component]
pub fn Content(children: Element, #[props(default)] class: String) -> Element {
    rsx! { main { class: "ix-content {class}", {children} } }
}

/// Content title, subtitle, optional back action and composed header actions.
#[component]
pub fn ContentHeader(
    title: String,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] show_back: bool,
    #[props(default)] onback: EventHandler<()>,
    #[props(default)] header_content: Option<Element>,
    #[props(default)] ellipsis: bool,
    #[props(default = VNode::empty())] children: Element,
) -> Element {
    rsx! { header { class: "ix-content-header", "data-ellipsis": ellipsis,
        if show_back { button { class: "ix-layout-icon-button", r#type: "button", "aria-label": "Go back", onclick: move |_| onback.call(()), Icon { name: "arrow-left" } } }
        div { class: "ix-content-header-titles", div { class: "ix-content-header-title-line", h1 { title: title.clone(), "{title}" } {header_content} } if let Some(subtitle) = subtitle { p { title: subtitle.clone(), "{subtitle}" } } }
        div { class: "ix-content-header-actions", {children} }
    } }
}

/// Stable-keyed tab definition for [`Tabs`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabItem {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
    pub counter: Option<usize>,
    pub closable: bool,
}
impl TabItem {
    /// Create an enabled text tab.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            counter: None,
            closable: false,
        }
    }
}
fn next_tab(items: &[TabItem], current: usize, forward: bool) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    let len = items.len();
    let current = current.min(len - 1);
    (1..=len)
        .map(|offset| {
            if forward {
                (current + offset) % len
            } else {
                (current + len - (offset % len)) % len
            }
        })
        .find(|index| !items[*index].disabled)
}
fn mounted_tab<'a, T>(
    items: &[TabItem],
    target: usize,
    mounted: &'a BTreeMap<String, T>,
) -> Option<&'a T> {
    mounted.get(&items.get(target)?.key)
}

/// Whether arrow-key focus also activates a tab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabKeyboardNavigation {
    /// Display the focused tab's panel immediately.
    #[default]
    Automatic,
    /// Activate with Enter or Space after moving focus.
    Manual,
}

/// Edge carrying the strip and selected-tab indicator, as in upstream iX.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabPlacement {
    #[default]
    Bottom,
    Top,
}
impl TabPlacement {
    fn class(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Top => "top",
        }
    }
}

/// Explicit tab selection, including a deliberately empty strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabSelection {
    Empty,
    Index(usize),
}
impl Default for TabSelection {
    fn default() -> Self {
        Self::Index(0)
    }
}
impl TabSelection {
    fn resolve(self, items: &[TabItem]) -> Option<usize> {
        match self {
            Self::Empty => None,
            Self::Index(index) => selected_tab(items, index),
        }
    }
}

/// Selection request shared by the indexed strip and keyed panel set.
/// Call `prevent_default` synchronously to retain selection and suppress all
/// accepted-change callbacks. Focus may still move during arrow navigation.
#[derive(Clone, Debug)]
pub struct TabChangeEvent<T> {
    pub previous: Option<T>,
    pub next: Option<T>,
    prevented: std::rc::Rc<std::cell::Cell<bool>>,
}
impl<T> TabChangeEvent<T> {
    pub(crate) fn new(previous: Option<T>, next: Option<T>) -> Self {
        Self {
            previous,
            next,
            prevented: Default::default(),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}
pub type TabSelectionChange = TabChangeEvent<usize>;
pub type TabKeySelectionChange = TabChangeEvent<String>;

fn accept_tab_request(
    items: &[TabItem],
    previous: Option<usize>,
    next: Option<usize>,
    before: impl FnOnce(TabSelectionChange),
) -> bool {
    if next.is_some_and(|index| items.get(index).is_none_or(|item| item.disabled)) {
        return false;
    }
    let event = TabChangeEvent::new(previous, next);
    before(event.clone());
    !event.default_prevented()
}

#[derive(serde::Deserialize)]
struct TabMeasurements {
    content: f64,
    available: f64,
}

fn tabs_overflow(content: f64, available: f64) -> bool {
    content.is_finite() && available.is_finite() && available > 0.0 && content > available
}

fn selected_tab(items: &[TabItem], requested: usize) -> Option<usize> {
    items
        .get(requested)
        .filter(|item| !item.disabled)
        .map(|_| requested)
        .or_else(|| items.iter().position(|item| !item.disabled))
}

// Measurement/scroll/focus plumbing only. Rust owns overflow, selection and keys.
// Observe intrinsic widths too, so font, counter and label changes are measured.
const TABS_MEASURE: &str = r#"
const id = await dioxus.recv();
const root = document.getElementById(id);
if (!root) return;
let frame = 0, previous = '', done = () => {};
const cleanup = () => { resize.disconnect(); changes.disconnect(); removal.disconnect(); cancelAnimationFrame(frame); document.fonts?.removeEventListener('loadingdone', schedule); done(); };
const measure = () => {
  frame = 0;
  if (!root.isConnected) { cleanup(); return; }
  const strip = root.querySelector('.ix-tabs');
  if (!strip) return;
  const payload = { content: strip.scrollWidth, available: strip.clientWidth };
  const json = JSON.stringify(payload);
  if (json !== previous) { previous = json; dioxus.send(payload); }
};
const schedule = () => { if (!frame) frame = requestAnimationFrame(measure); };
const resize = new ResizeObserver(schedule);
const observe = () => { resize.disconnect(); resize.observe(root); root.querySelectorAll('.ix-tabs,.ix-tab-wrapper').forEach(n => resize.observe(n)); schedule(); };
const changes = new MutationObserver(observe);
changes.observe(root, { childList: true, subtree: true, characterData: true, attributes: true });
const removal = new MutationObserver(() => { if (!root.isConnected) cleanup(); });
removal.observe(document.documentElement, { childList: true, subtree: true });
document.fonts?.addEventListener('loadingdone', schedule);
await new Promise(resolve => { done = resolve; observe(); });
return null;
"#;

fn reveal_tab(root: String, target: String, focus: bool) {
    spawn(async move {
        let task = crate::dom_query::eval(
            r#"
const [rootId, targetId, focus] = await dioxus.recv();
await new Promise(requestAnimationFrame);
const root = document.getElementById(rootId), tab = document.getElementById(targetId);
if (root?.contains(tab)) {
  if (focus) tab.focus({preventScroll:true});
  tab.scrollIntoView({block:'nearest',inline:'nearest',behavior:'smooth'});
}
"#,
        );
        let _ = task.send((root, target, focus));
        let _ = task.await;
    });
}

/// Tabs with measured overflow, Arrow/Home/End navigation and close actions.
/// Omit `selected` and `selection` for internal selection. `selection` takes
/// precedence and supports [`TabSelection::Empty`]. Explicit prop updates are
/// authoritative; interaction requests can be vetoed with `onbeforechange`.
/// All tabs retain their source order; the all-tabs menu reveals the selected tab
/// in the scrolling strip. Callbacks use item indices. Render selected content
/// alongside this list, or use [`crate::TabSet`] for persistent keyed panels.
#[component]
pub fn Tabs(
    items: Vec<TabItem>,
    selected: Option<usize>,
    selection: Option<TabSelection>,
    #[props(default)] default_selection: TabSelection,
    #[props(default)] onbeforechange: EventHandler<TabSelectionChange>,
    #[props(default)] onselectionchange: EventHandler<Option<usize>>,
    #[props(default)] onchange: EventHandler<usize>,
    #[props(default)] onclose: EventHandler<usize>,
    #[props(default)] small: bool,
    #[props(default)] rounded: bool,
    #[props(default)] stretched: bool,
    /// Force the all-tabs menu even when the strip fits. Otherwise it appears
    /// automatically on overflow, except with the stretched layout (as upstream).
    #[props(default)]
    overflow_menu: bool,
    #[props(default)] keyboard_navigation: TabKeyboardNavigation,
    #[props(default)] placement: TabPlacement,
    #[props(default = "Tabs".to_string())] label: String,
    #[props(default = "Show all tabs".to_string())] aria_label_more_tabs: String,
    id_prefix: Option<String>,
) -> Element {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static TABS_ID: AtomicUsize = AtomicUsize::new(1);
    let root_id = use_hook(|| format!("ix-tabs-{}", TABS_ID.fetch_add(1, Ordering::Relaxed)));
    let prefix = id_prefix.clone().unwrap_or_else(|| root_id.clone());
    let mount_id = root_id.clone();
    let mut measured_overflow = use_signal(|| false);
    let mut mounted = use_signal(BTreeMap::<String, MountedEvent>::new);
    let live_keys: BTreeSet<_> = items.iter().map(|item| item.key.clone()).collect();
    use_effect(use_reactive!(|live_keys| {
        mounted.write().retain(|key, _| live_keys.contains(key));
    }));
    let mut internal = use_signal(|| default_selection);
    let controlled = selection.is_some() || selected.is_some();
    let active = selection
        .or(selected.map(TabSelection::Index))
        .unwrap_or(internal())
        .resolve(&items);
    let request_items = items.clone();
    let request = use_callback(move |next: Option<usize>| {
        if accept_tab_request(&request_items, active, next, |event| {
            onbeforechange.call(event)
        }) {
            if !controlled {
                internal.set(next.map_or(TabSelection::Empty, TabSelection::Index));
            }
            onselectionchange.call(next);
            if let Some(index) = next {
                onchange.call(index);
            }
        }
    });
    let active_key = active.map(|index| items[index].key.clone());
    let reveal_root = root_id.clone();
    let reveal_prefix = prefix.clone();
    use_effect(use_reactive!(|active_key| {
        if let Some(key) = active_key {
            reveal_tab(
                reveal_root.clone(),
                format!(
                    "{reveal_prefix}-tab-{}",
                    crate::composition::tab_key_id(&key)
                ),
                false,
            );
        }
    }));
    let overflowing = measured_overflow();
    let show_menu = overflow_menu || (overflowing && !stretched);
    rsx! { div { id: root_id.clone(), class: "ix-tabs-shell", "data-overflow": overflowing,
        onmounted: move |_| {
            let id = mount_id.clone();
            spawn(async move {
                let mut task = dioxus::document::eval(TABS_MEASURE);
                if task.send(id).is_err() { return; }
                while let Ok(measured) = task.recv::<TabMeasurements>().await {
                    let overflow = tabs_overflow(measured.content, measured.available);
                    if *measured_overflow.peek() != overflow { measured_overflow.set(overflow); }
                }
            });
        },
        div { class: "ix-tabs", "data-placement": placement.class(), role: "tablist", "aria-label": label, tabindex: if overflowing { 0 } else { -1 }, "data-small": small, "data-rounded": rounded, "data-stretched": stretched,
        for (index, item) in items.iter().enumerate() {
            div { key: "{item.key}", class: "ix-tab-wrapper", "data-tab-key": item.key.clone(), "data-selected": active == Some(index),
                button { class: "ix-tab", r#type: "button", role: "tab", disabled: item.disabled, "aria-label": if rounded { Some(item.label.clone()) } else { None }, title: if rounded { Some(item.label.clone()) } else { None }, "aria-selected": active == Some(index), tabindex: if active.or_else(|| items.iter().position(|item| !item.disabled)) == Some(index) { 0 } else { -1 },
                    id: format!("{prefix}-tab-{}", crate::composition::tab_key_id(&item.key)),
                    "aria-controls": id_prefix.as_ref().map(|prefix| format!("{prefix}-panel-{}", crate::composition::tab_key_id(&item.key))),
                    onmounted: { let key = item.key.clone(); move |event| { mounted.write().insert(key.clone(), event); } },
                    onclick: move |_| request.call(Some(index)),
                    onkeydown: { let items = items.clone(); let root = root_id.clone(); let prefix = prefix.clone(); move |event: KeyboardEvent| {
                        let target = match event.key() { Key::ArrowRight => next_tab(&items, index, true), Key::ArrowLeft => next_tab(&items, index, false), Key::Home => items.iter().position(|item| !item.disabled), Key::End => items.iter().rposition(|item| !item.disabled), _ => return };
                        event.prevent_default();
                        if let Some(target) = target {
                            if keyboard_navigation == TabKeyboardNavigation::Automatic { request.call(Some(target)); }
                            let node = mounted_tab(&items, target, &mounted.peek()).cloned();
                            if let Some(node) = node { spawn(async move { let _ = node.set_focus(true).await; }); }
                            reveal_tab(root.clone(), format!("{prefix}-tab-{}", crate::composition::tab_key_id(&items[target].key)), false);
                        }
                    } },
                    if rounded { span { class: "ix-tab-rounded-circle", if let Some(icon) = &item.icon { Icon { name: icon.clone(), size: 32 } } else { "{item.label}" } } }
                    else { if let Some(icon) = &item.icon { Icon { name: icon.clone() } } span { "{item.label}" } }
                    if let Some(counter) = item.counter { span { class: "ix-tab-counter", "{counter}" } }
                }
                if item.closable && !rounded { button { class: "ix-tab-close", r#type: "button", disabled: item.disabled, "aria-label": "Close {item.label}", onclick: move |_| onclose.call(index), Icon { name: "close", size: 16 } } }
            }
        }
    }
        if show_menu { crate::Dropdown { label: "All tabs", aria_label: aria_label_more_tabs, icon: "more-menu", variant: crate::ButtonVariant::SubtleTertiary, class: "ix-tabs-overflow",
            for (index, item) in items.iter().enumerate() { crate::DropdownItem { key: "{item.key}", label: item.label.clone(), icon: item.icon.clone(), disabled: item.disabled, checked: active == Some(index), checkable: true, onclick: { let key = item.key.clone(); let root = root_id.clone(); let prefix = prefix.clone(); move |_| { request.call(Some(index)); reveal_tab(root.clone(), format!("{prefix}-tab-{}", crate::composition::tab_key_id(&key)), true); } } } }
        } }
    } }
}

/// Stable breadcrumb definition for managed truncation and next-item menus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreadcrumbEntry {
    pub key: String,
    pub label: String,
    pub icon: Option<String>,
    pub href: Option<String>,
}
impl BreadcrumbEntry {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
            href: None,
        }
    }
}
fn hidden_breadcrumb_count(count: usize, visible: usize) -> usize {
    count.saturating_sub(visible.max(1))
}
/// Ordered breadcrumb navigation. Pass `items` for managed truncation/next menus,
/// or compose children and mark the final [`BreadcrumbItem`] as `current`.
#[component]
pub fn Breadcrumb(
    #[props(default = VNode::empty())] children: Element,
    #[props(default = "Breadcrumb".to_string())] label: String,
    #[props(default)] items: Vec<BreadcrumbEntry>,
    #[props(default)] next_items: Vec<BreadcrumbEntry>,
    #[props(default = 9)] visible_item_count: usize,
    #[props(default)] onitem: EventHandler<BreadcrumbEntry>,
    #[props(default)] onnext: EventHandler<BreadcrumbEntry>,
) -> Element {
    let hidden = hidden_breadcrumb_count(items.len(), visible_item_count);
    rsx! { nav { class: "ix-breadcrumb", "aria-label": label, ol {
        if hidden > 0 { li { crate::Dropdown { label: "...", aria_label: "Show previous breadcrumb items", variant: crate::ButtonVariant::Tertiary,
            for item in items.iter().take(hidden) { crate::DropdownItem { label: item.label.clone(), onclick: { let item = item.clone(); move |_| onitem.call(item.clone()) } } }
        } } }
        for (index, item) in items.iter().enumerate().skip(hidden) {
            if index + 1 == items.len() && !next_items.is_empty() { li { crate::Dropdown { label: item.label.clone(), aria_label: "Show next breadcrumb items", variant: crate::ButtonVariant::Tertiary,
                for next in &next_items { crate::DropdownItem { label: next.label.clone(), onclick: { let next = next.clone(); move |_| onnext.call(next.clone()) } } }
            } } }
            else { BreadcrumbItem { label: item.label.clone(), icon: item.icon.clone(), href: item.href.clone(), current: index + 1 == items.len(), onclick: { let item = item.clone(); move |_| onitem.call(item.clone()) } } }
        }
        {children}
    } } }
}

/// Breadcrumb link or application navigation action.
#[component]
pub fn BreadcrumbItem(
    label: String,
    #[props(default)] href: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] current: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! { li { class: "ix-breadcrumb-item",
        if current { span { "aria-current": "page", if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        else if let Some(href) = href { a { href, onclick: move |event| onclick.call(event), if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        else { button { r#type: "button", onclick: move |event| onclick.call(event), if let Some(icon) = icon { Icon { name: icon, size: 16 } } "{label}" } }
        span { class: "ix-breadcrumb-separator", "aria-hidden": "true", Icon { name: "chevron-right", size: 16 } }
    } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageToken {
    Page(usize),
    Jump(usize),
}
fn page_tokens(count: usize, selected: usize) -> Vec<PageToken> {
    if count <= 7 {
        return (0..count).map(PageToken::Page).collect();
    }
    let selected = selected.min(count - 1);
    let (start, end) = if selected <= 3 {
        (1, 4)
    } else if selected >= count - 4 {
        (count - 5, count - 2)
    } else {
        (selected - 1, selected + 1)
    };
    let mut result = vec![PageToken::Page(0)];
    if start > 1 {
        result.push(PageToken::Jump(start / 2));
    }
    result.extend((start..=end).map(PageToken::Page));
    if end < count - 2 {
        result.push(PageToken::Jump(end + (count - 1 - end) / 2));
    }
    result.push(PageToken::Page(count - 1));
    result
}

/// Zero-based pagination; `count` is the total number of pages, as in upstream iX.
/// Advanced mode adds direct page entry and items-per-page selection.
#[component]
pub fn Pagination(
    count: usize,
    #[props(default)] selected_page: usize,
    #[props(default)] onpage: EventHandler<usize>,
    #[props(default)] advanced: bool,
    #[props(default = 15)] item_count: usize,
    #[props(default = vec![10,15,20,40,100])] item_count_options: Vec<usize>,
    #[props(default)] onitemcount: EventHandler<usize>,
    #[props(default)] hide_item_count: bool,
    #[props(default = "Pagination".to_string())] label: String,
) -> Element {
    let selected = selected_page.min(count.saturating_sub(1));
    rsx! { nav { class: "ix-pagination", "aria-label": label,
        button { class: "ix-layout-icon-button", r#type: "button", disabled: count == 0 || selected == 0, "aria-label": "Previous page", onclick: move |_| onpage.call(selected.saturating_sub(1)), Icon { name: "chevron-left" } }
        if advanced { span { "Page" } input { class: "ix-pagination-input", r#type: "number", min: "1", max: count.max(1), value: if count == 0 { 0 } else { selected + 1 }, disabled: count == 0, "aria-label": "Page selection", onchange: move |event| if let Ok(value) = event.value().parse::<usize>() { if count > 0 { onpage.call(value.saturating_sub(1).min(count - 1)); } } } span { "of {count}" } }
        else { for token in page_tokens(count, selected) { match token {
            PageToken::Page(page) => rsx! { button { class: "ix-page-button", r#type: "button", "aria-label": "Page {page + 1}", "aria-current": if selected == page { Some("page") } else { None }, onclick: move |_| onpage.call(page), "{page + 1}" } },
            PageToken::Jump(page) => rsx! { button { class: "ix-page-button", r#type: "button", "aria-label": "Jump to page {page + 1}", onclick: move |_| onpage.call(page), "…" } },
        } } }
        button { class: "ix-layout-icon-button", r#type: "button", disabled: count == 0 || selected + 1 >= count, "aria-label": "Next page", onclick: move |_| onpage.call((selected + 1).min(count.saturating_sub(1))), Icon { name: "chevron-right" } }
        if advanced && !hide_item_count { label { class: "ix-pagination-count", "Items" select { value: "{item_count}", "aria-label": "Items per page", onchange: move |event| if let Ok(value) = event.value().parse::<usize>() { if value > 0 { onitemcount.call(value); } }, for value in item_count_options.iter().copied().filter(|value| *value > 0) { option { value: "{value}", "{value}" } } } } }
    } }
}

/// Expandable information section with optional status treatment.
#[component]
pub fn Blind(
    label: String,
    children: Element,
    #[props(default)] sublabel: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default = CardVariant::Filled)] variant: CardVariant,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
) -> Element {
    let mut local = use_signal(|| default_expanded);
    let open = expanded.unwrap_or(local());
    rsx! { section { class: "ix-blind ix-card-{variant.class()}",
        div { class: "ix-blind-heading", button { class: "ix-blind-header", r#type: "button", "aria-expanded": open, onclick: move |_| { local.set(!open); onexpand.call(!open); },
            span { class: "ix-chevron", "data-open": open, Icon { name: "chevron-down" } } if let Some(icon) = icon { Icon { name: icon } }
            span { class: "ix-blind-label", strong { "{label}" } if let Some(text) = sublabel { span { "{text}" } } }
        } if let Some(actions) = header_actions { div { class: "ix-blind-actions", {actions} } } } div { class: "ix-blind-content", hidden: !open, {children} }
    } }
}

/// Collapsible collection of selectable [`GroupItem`] rows.
#[component]
pub fn Group(
    header: String,
    children: Element,
    #[props(default)] subheader: Option<String>,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] selected: Option<bool>,
    #[props(default)] onselect: EventHandler<bool>,
    #[props(default)] suppress_header_selection: bool,
    #[props(default)] expand_on_header_click: bool,
    #[props(default)] header_actions: Option<Element>,
) -> Element {
    let mut local = use_signal(|| default_expanded);
    let mut local_selected = use_signal(|| false);
    let is_selected = selected.unwrap_or(local_selected());
    let mut disclosure = use_signal(|| None::<MountedEvent>);
    let open = expanded.unwrap_or(local());
    rsx! { section { class: "ix-group", onkeydown: move |event: KeyboardEvent| if event.key() == Key::Escape && open { event.prevent_default(); event.stop_propagation(); local.set(false); onexpand.call(false); let node = disclosure.peek().clone(); if let Some(node) = node { spawn(async move { let _ = node.set_focus(true).await; }); } },
        div { class: "ix-group-header-row", "data-selected": is_selected,
            button { class: "ix-group-header", r#type: "button", "aria-pressed": if suppress_header_selection { None } else { Some(is_selected.to_string()) }, "aria-expanded": if suppress_header_selection || expand_on_header_click { Some(open.to_string()) } else { None },
                onclick: move |_| {
                    if !suppress_header_selection { local_selected.set(!is_selected); onselect.call(!is_selected); }
                    if suppress_header_selection || expand_on_header_click { local.set(!open); onexpand.call(!open); }
                },
                span { strong { "{header}" } if let Some(text) = subheader { small { "{text}" } } }
            }
            {header_actions}
            button { class: "ix-group-expand", r#type: "button", "aria-label": format!("{} {header}", if open { "Collapse" } else { "Expand" }), "aria-expanded": open, onmounted: move |event| disclosure.set(Some(event)), onclick: move |_| { local.set(!open); onexpand.call(!open); }, span { class: "ix-chevron", "data-open": open, Icon { name: "chevron-down" } } }
        }
        div { class: "ix-group-content", hidden: !open, {children} }
    } }
}

/// Parent-controlled selectable group row.
#[component]
pub fn GroupItem(
    #[props(default)] text: String,
    #[props(default)] children: Option<Element>,
    #[props(default)] secondary_text: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onselect: EventHandler<()>,
) -> Element {
    if let Some(children) = children {
        return rsx! { div { class: "ix-group-item ix-group-custom-item", {children} } };
    }
    rsx! { button { class: "ix-group-item", r#type: "button", disabled, "aria-pressed": selected, "data-selected": selected, onclick: move |_| onselect.call(()),
        if let Some(icon) = icon { Icon { name: icon } } span { "{text}" } if let Some(text) = secondary_text { span { class: "ix-group-secondary", "{text}" } }
    } }
}

/// Empty state alignment and density.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyStateLayout {
    #[default]
    Large,
    Compact,
    CompactBreak,
}
impl EmptyStateLayout {
    fn class(self) -> &'static str {
        match self {
            Self::Large => "large",
            Self::Compact => "compact",
            Self::CompactBreak => "compact-break",
        }
    }
}

/// Explanation of an empty collection with an optional recovery action.
#[component]
pub fn EmptyState(
    header: String,
    #[props(default)] subheader: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] action: Option<String>,
    #[props(default)] layout: EmptyStateLayout,
    #[props(default)] onaction: EventHandler<()>,
) -> Element {
    rsx! { div { class: "ix-empty-state ix-empty-state-{layout.class()}",
        if let Some(icon) = icon { Icon { name: icon, size: if layout == EmptyStateLayout::Large { 48 } else { 32 } } }
        div { class: "ix-empty-state-labels", strong { "{header}" } if let Some(text) = subheader { p { "{text}" } } }
        if let Some(text) = action { button { class: "ix-empty-state-action", r#type: "button", onclick: move |_| onaction.call(()), "{text}" } }
    } }
}

/// Label placement in a key/value entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyValueLabelPosition {
    #[default]
    Top,
    Left,
}

/// Semantic label/value pair with optional custom value children.
#[component]
pub fn KeyValue(
    label: String,
    #[props(default)] value: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] label_position: KeyValueLabelPosition,
    #[props(default = VNode::empty())] children: Element,
) -> Element {
    rsx! { div { class: "ix-key-value", "data-horizontal": label_position == KeyValueLabelPosition::Left,
        if let Some(icon) = icon { Icon { name: icon } }
        dl { dt { "{label}" } dd { if let Some(value) = value { "{value}" } {children} } }
    } }
}

/// Collection of key/value entries, optionally horizontal.
#[component]
pub fn KeyValueList(
    children: Element,
    #[props(default)] horizontal: bool,
    #[props(default)] striped: bool,
) -> Element {
    rsx! { div { class: "ix-key-value-list", "data-horizontal": horizontal, "data-striped": striped, {children} } }
}

/// KPI severity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KpiState {
    #[default]
    Neutral,
    Warning,
    Alarm,
}
impl KpiState {
    fn class(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Warning => "warning",
            Self::Alarm => "alarm",
        }
    }
}

/// Compact industrial measurement, with label, unit and alarm status.
#[component]
pub fn Kpi(
    label: String,
    value: String,
    #[props(default)] unit: Option<String>,
    #[props(default)] state: KpiState,
    #[props(default)] vertical: bool,
) -> Element {
    rsx! { div { class: "ix-kpi ix-kpi-{state.class()}", "data-vertical": vertical,
        div { class: "ix-kpi-label", if state != KpiState::Neutral { Icon { name: (if state == KpiState::Alarm { "error" } else { "warning" }).to_string(), size: 16, label: state.class().to_string() } } "{label}" }
        div { class: "ix-kpi-measurement", strong { "{value}" } if let Some(unit) = unit { span { "{unit}" } } }
    } }
}

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
/// Event collection with optional compact row spacing and shared row height.
#[component]
pub fn EventList(
    children: Element,
    #[props(default)] compact: bool,
    #[props(default)] item_height: EventItemHeight,
) -> Element {
    rsx! { div { class: "ix-event-list", role: "list", "data-compact": compact, style: format!("--ix-event-height:{}px", item_height.pixels()), {children} } }
}

/// Selectable event row with a semantic status indicator.
#[component]
pub fn EventListItem(
    children: Element,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] chevron: bool,
    #[props(default)] filled: bool,
    #[props(default)] status: CardVariant,
    #[props(default)] onselect: EventHandler<()>,
) -> Element {
    rsx! { div { role: "listitem", button { class: "ix-event-list-item ix-event-status-{status.class()}", r#type: "button", disabled, "aria-pressed": selected, "data-selected": selected, "data-filled": filled, onclick: move |_| onselect.call(()),
        span { class: "ix-event-indicator", "aria-hidden": "true" } span { class: "ix-event-content", {children} } if chevron { Icon { name: "chevron-right" } }
    } } }
}

/// Workflow step status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkflowStatus {
    #[default]
    Open,
    Success,
    Done,
    Warning,
    Error,
}
impl WorkflowStatus {
    fn class(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Success => "success",
            Self::Done => "done",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::Open => "circle",
            Self::Success | Self::Done => "check",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Ordered workflow, horizontal by default and vertical for sidebar layouts.
#[component]
pub fn WorkflowSteps(
    children: Element,
    #[props(default)] vertical: bool,
    #[props(default = "Workflow".to_string())] label: String,
) -> Element {
    rsx! { ol { class: "ix-workflow-steps", "data-vertical": vertical, "aria-label": label, {children} } }
}

/// Parent-controlled workflow action with explicit status and selection.
#[component]
pub fn WorkflowStep(
    label: String,
    #[props(default)] status: WorkflowStatus,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] clickable: bool,
    #[props(default)] onselect: EventHandler<()>,
) -> Element {
    let content = rsx! { span { class: "ix-workflow-marker", Icon { name: status.icon().to_string() } } span { class: "ix-workflow-label", "{label}" } };
    rsx! { li { class: "ix-workflow-step ix-workflow-{status.class()}", "data-selected": selected, "data-disabled": disabled, "aria-current": if selected { Some("step") } else { None },
        if clickable { button { class: "ix-workflow-step-content", r#type: "button", disabled, "aria-label": "{label}: {status.class()}", onclick: move |_| onselect.call(()), {content} } }
        else { div { class: "ix-workflow-step-content", "aria-disabled": disabled, "aria-label": "{label}: {status.class()}", {content} } }
    } }
}

pub(crate) const STYLES: &str = r#"
.ix-card{--ix-card-bg:transparent;--ix-card-fg:var(--ix-color-text);display:flex;flex-direction:column;align-items:stretch;width:320px;max-width:100%;overflow:hidden;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-card-bg);color:var(--ix-card-fg);position:relative;text-align:start;font:inherit;padding:0}
.ix-card[data-passive=false]{cursor:pointer}.ix-card-select{position:absolute;inset:0;width:100%;height:100%;padding:0;border:0;border-radius:inherit;background:transparent;color:inherit;z-index:0;cursor:pointer}.ix-card-body{display:contents;pointer-events:none}.ix-card-body :is(button,a,input,select,textarea,summary,[role=button],[tabindex]){position:relative;z-index:1;pointer-events:auto}.ix-card-select:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-card[data-passive=false]:hover{background:color-mix(in srgb,var(--ix-card-bg) 85%,var(--ix-color-text) 8%)}.ix-card[data-selected=true]{outline:2px solid var(--ix-color-primary);outline-offset:-2px}
.ix-card-filled{--ix-card-bg:var(--ix-color-surface-1);border-color:transparent}.ix-card-alarm{--ix-card-bg:var(--ix-color-danger);--ix-card-fg:var(--ix-color-on-danger,#fff);border-color:transparent}.ix-card-critical{--ix-card-bg:var(--ix-color-critical,#eb780a);--ix-card-fg:var(--ix-color-on-critical,#000028);border-color:transparent}.ix-card-warning{--ix-card-bg:var(--ix-color-warning);--ix-card-fg:var(--ix-color-on-warning,#000028);border-color:transparent}.ix-card-success{--ix-card-bg:var(--ix-color-success);--ix-card-fg:var(--ix-color-on-success,#000028);border-color:transparent}.ix-card-info{--ix-card-bg:var(--ix-color-info);--ix-card-fg:var(--ix-color-on-info,#000028);border-color:transparent}.ix-card-neutral{--ix-card-bg:var(--ix-color-secondary);--ix-card-fg:var(--ix-color-on-secondary,#000028);border-color:transparent}.ix-card-primary{--ix-card-bg:var(--ix-color-primary);--ix-card-fg:var(--ix-color-on-primary);border-color:transparent}
.ix-card-content{padding:16px;min-width:0}.ix-card-content p{margin:8px 0 0}.ix-card-title{min-height:48px;display:flex;align-items:center;gap:8px;padding:8px 16px}.ix-card-title-text{display:flex;align-items:center;gap:8px;font-weight:600;min-width:0;flex:1}.ix-card-title-actions{display:flex;gap:4px;align-items:center}.ix-card-subheading{font-size:14px;line-height:20px;opacity:.85}.ix-push-notification{font-size:24px;line-height:32px;font-weight:600}
.ix-card-accordion{border-top:1px solid color-mix(in srgb,currentColor 20%,transparent)}.ix-disclosure-button{appearance:none;background:transparent;color:inherit;border:0;display:flex;align-items:center;gap:8px;width:100%;min-height:32px;padding:4px 16px;text-align:start;cursor:pointer;font:inherit}.ix-disclosure-button:hover{background:color-mix(in srgb,currentColor 8%,transparent)}.ix-card-accordion-content{padding:16px}.ix-chevron{display:inline-flex;flex-shrink:0;transition:transform .18s ease-out}.ix-chevron[data-open=true]{transform:rotate(180deg)}
.ix-card-list-header{display:flex;align-items:center;justify-content:space-between;gap:16px;margin-bottom:8px}.ix-card-list-header>.ix-disclosure-button{width:auto;padding-inline:0;font-size:16px}.ix-card-list-show-all{white-space:nowrap;border:0;border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-primary);font:inherit;font-size:14px;padding:4px 8px;cursor:pointer}.ix-card-list-show-all:hover{background:var(--ix-color-surface-2)}.ix-card-list[data-scroll=true]>.ix-card-list-items{flex-wrap:nowrap;overflow-x:auto;padding:2px 2px 12px}.ix-card-list-items{display:flex;flex-wrap:wrap;gap:16px;align-items:flex-start}.ix-card-list-items>.ix-card{flex-shrink:0}
.ix-tile{display:flex;flex-direction:column;width:191px;max-width:100%;min-height:80px;background:var(--ix-color-surface-1);border-radius:var(--ix-radius);overflow:hidden}.ix-tile-small{min-height:40px}.ix-tile-big{min-height:160px}.ix-tile-header{min-height:40px;display:flex;align-items:center;padding:8px 16px;font-size:16px;font-weight:600}.ix-tile-subheader{padding:0 16px 8px;color:var(--ix-color-text-secondary);font-size:14px}.ix-tile-content{padding:8px 16px;flex:1}.ix-tile-footer{padding:8px 16px;border-top:1px solid var(--ix-color-surface)}
.ix-action-card{width:212px;min-height:120px;align-items:flex-start;justify-content:center;gap:8px;padding:16px;font:inherit;cursor:pointer}.ix-action-card:hover:not(:disabled){filter:brightness(1.08)}.ix-action-card:disabled{cursor:default}.ix-action-card strong{font-size:16px;line-height:24px}.ix-action-card .ix-card-subheading{font-weight:400}
.ix-content{padding:24px 32px;min-width:0;width:100%;box-sizing:border-box}.ix-content-header{display:flex;align-items:center;gap:8px;margin-bottom:24px;min-width:0}.ix-content-header-titles{flex:1;min-width:0}.ix-content-header h1{font-size:28px;line-height:36px;font-weight:600;margin:0}.ix-content-header p{font-size:14px;color:var(--ix-color-text-secondary);margin:4px 0 0}.ix-content-header-actions{display:flex;align-items:center;flex-wrap:wrap;gap:8px}
.ix-layout-icon-button,.ix-page-button{display:inline-flex;align-items:center;justify-content:center;flex-shrink:0;min-width:32px;height:32px;background:transparent;color:var(--ix-color-primary);border:0;border-radius:var(--ix-radius);padding:4px;cursor:pointer;font:inherit}.ix-layout-icon-button:hover:not(:disabled),.ix-page-button:hover{background:var(--ix-color-surface-2)}.ix-layout-icon-button:disabled{color:var(--ix-color-text-secondary);opacity:.4;cursor:default}
.ix-tabs{display:flex;position:relative;overflow-x:auto;overflow-y:hidden;max-width:100%;border-bottom:2px solid var(--ix-color-border);gap:0;scrollbar-width:none;scroll-behavior:smooth}.ix-tabs::-webkit-scrollbar{display:none}.ix-tabs-shell[data-overflow=true]>.ix-tabs{mask-image:linear-gradient(90deg,black calc(100% - 24px),transparent 100%)}.ix-tabs:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-tab-wrapper{display:flex;align-items:stretch;position:relative;flex-shrink:0;min-width:0;border-bottom:2px solid transparent;margin-bottom:-2px}.ix-tab-wrapper[data-selected=true]{border-bottom-color:var(--ix-color-primary)}.ix-tab{display:flex;align-items:center;justify-content:center;gap:8px;min-height:48px;padding:8px 16px;background:transparent;border:0;color:var(--ix-color-text-secondary);font:inherit;cursor:pointer;white-space:nowrap}.ix-tab-wrapper[data-selected=true] .ix-tab{color:var(--ix-color-primary)}.ix-tab:hover:not(:disabled),.ix-tab-close:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-tab:disabled,.ix-tab-close:disabled{opacity:.4;cursor:default}.ix-tab-counter{font-size:12px;line-height:16px;background:var(--ix-color-surface-2);color:var(--ix-color-text);padding:0 5px;border-radius:8px}.ix-tab-close{border:0;background:transparent;color:var(--ix-color-text-secondary);display:flex;align-items:center;padding:4px 8px;cursor:pointer}.ix-tabs[data-small=true] .ix-tab{min-height:32px;font-size:14px;padding:4px 12px}.ix-tabs[data-stretched=true] .ix-tab-wrapper{flex:1}.ix-tabs[data-stretched=true] .ix-tab{flex:1}.ix-tabs[data-placement=top]:not([data-rounded=true]){border-bottom:0;border-top:2px solid var(--ix-color-border)}.ix-tabs[data-placement=top]:not([data-rounded=true]) .ix-tab-wrapper{border-bottom:0;border-top:2px solid transparent;margin-bottom:0;margin-top:-2px}.ix-tabs[data-placement=top]:not([data-rounded=true]) .ix-tab-wrapper[data-selected=true]{border-top-color:var(--ix-color-primary)}.ix-tabs[data-rounded=true]{border:0;gap:8px}.ix-tabs[data-rounded=true] .ix-tab-wrapper{border:0;border-radius:24px;overflow:hidden;margin:0}.ix-tabs[data-rounded=true] .ix-tab-wrapper[data-selected=true]{background:var(--ix-color-surface-2)}
.ix-breadcrumb ol{list-style:none;display:flex;align-items:center;flex-wrap:wrap;padding:0;margin:0;gap:4px}.ix-breadcrumb-item{display:flex;align-items:center;gap:4px;font-size:14px;line-height:20px}.ix-breadcrumb-item>a,.ix-breadcrumb-item>button,.ix-breadcrumb-item>span:first-child{display:inline-flex;align-items:center;gap:4px;padding:4px}.ix-breadcrumb-item>a,.ix-breadcrumb-item>button{color:var(--ix-color-primary);text-decoration:none;background:transparent;border:0;font:inherit;cursor:pointer;border-radius:var(--ix-radius)}.ix-breadcrumb-item>a:hover,.ix-breadcrumb-item>button:hover{background:var(--ix-color-surface-2)}.ix-breadcrumb-item [aria-current]{color:var(--ix-color-text)}.ix-breadcrumb-separator{display:flex;color:var(--ix-color-text-secondary)}.ix-breadcrumb-item:last-child>.ix-breadcrumb-separator{display:none}
.ix-pagination{display:flex;align-items:center;flex-wrap:wrap;gap:4px;font-size:14px;line-height:20px;min-height:32px}.ix-page-button[aria-current=page]{background:var(--ix-color-primary);color:var(--ix-color-on-primary)}.ix-pagination-input{width:64px;min-height:32px;margin:0 4px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface);color:var(--ix-color-text);font:inherit;text-align:center}.ix-pagination-count{display:flex;align-items:center;gap:8px;margin-inline-start:24px}.ix-pagination-count select{height:32px;padding:4px 8px;background:var(--ix-color-surface);color:var(--ix-color-text);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);font:inherit}
.ix-blind{--ix-card-bg:var(--ix-color-surface-1);--ix-card-fg:var(--ix-color-text);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);overflow:hidden;color:var(--ix-card-fg);background:var(--ix-card-bg)}.ix-blind-header{display:flex;align-items:center;gap:12px;width:100%;min-height:48px;padding:12px 16px;background:transparent;color:inherit;border:0;cursor:pointer;text-align:start;font:inherit}.ix-blind-header:hover{background:color-mix(in srgb,currentColor 8%,transparent)}.ix-blind-label{display:flex;flex-direction:column;gap:4px}.ix-blind-label>span{font-size:14px;opacity:.8}.ix-blind-content{padding:16px;border-top:1px solid color-mix(in srgb,currentColor 16%,transparent)}
.ix-group{width:318px;max-width:100%}.ix-group-header{width:100%;min-height:64px;border:0;background:var(--ix-color-surface-1);color:var(--ix-color-text);border-radius:var(--ix-radius) var(--ix-radius) 0 0;display:flex;align-items:center;justify-content:space-between;gap:16px;padding:8px 16px;font:inherit;cursor:pointer;text-align:start}.ix-group-header>span:first-child{display:flex;flex-direction:column;gap:4px}.ix-group-header strong{font-size:18px;font-weight:600}.ix-group-header small{font-size:14px;color:var(--ix-color-text-secondary)}.ix-group-content{display:flex;flex-direction:column;gap:1px;margin-top:1px}.ix-group-item{position:relative;display:flex;align-items:center;gap:8px;min-height:40px;padding:8px 16px;width:100%;background:var(--ix-color-surface-1);border:0;color:var(--ix-color-text);font:inherit;text-align:start;cursor:pointer}.ix-group-item:hover:not(:disabled),.ix-group-header:hover{background:var(--ix-color-surface-2)}.ix-group-item[data-selected=true]{background:var(--ix-color-surface-2);box-shadow:inset 2px 0 0 var(--ix-color-primary)}.ix-group-item:disabled{opacity:.4;cursor:default}.ix-group-secondary{margin-inline-start:auto;color:var(--ix-color-text-secondary);font-size:14px}
.ix-empty-state{display:flex;align-items:center;justify-content:center;gap:16px;padding:24px}.ix-empty-state-large{flex-direction:column;text-align:center;gap:24px}.ix-empty-state-labels{display:flex;flex-direction:column;gap:4px;min-width:0}.ix-empty-state-labels strong{font-size:18px;line-height:24px;font-weight:600}.ix-empty-state-labels p{font-size:14px;color:var(--ix-color-text-secondary);margin:0;max-width:65ch}.ix-empty-state-action{display:inline-flex;align-items:center;justify-content:center;min-height:32px;padding:4px 16px;border:1px solid var(--ix-color-primary);border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-primary);font:inherit;cursor:pointer}.ix-empty-state-action:hover{background:var(--ix-color-surface-2)}.ix-empty-state-compact-break{justify-content:flex-start;flex-wrap:wrap}.ix-empty-state-compact-break .ix-empty-state-action{margin-inline-start:48px}
.ix-key-value{display:flex;align-items:center;gap:8px;padding:8px 0;border-bottom:1px solid var(--ix-color-border);min-width:0}.ix-key-value dl{display:flex;flex-direction:column;gap:4px;min-width:0;margin:0;width:100%}.ix-key-value dt{font-size:14px;line-height:20px;color:var(--ix-color-text-secondary)}.ix-key-value dd{margin:0;line-height:24px;overflow-wrap:anywhere}.ix-key-value[data-horizontal=true]{border:0}.ix-key-value[data-horizontal=true] dl{flex-direction:row;gap:16px}.ix-key-value[data-horizontal=true] dt{min-width:100px}.ix-key-value-list{display:flex;flex-direction:column}.ix-key-value-list[data-horizontal=true]{flex-direction:row;flex-wrap:wrap;gap:24px}.ix-key-value-list[data-horizontal=true]>.ix-key-value{flex:1;border:0}
.ix-kpi{display:flex;align-items:baseline;gap:16px;min-height:40px;min-width:0;padding:4px;border-bottom:2px solid var(--ix-color-border);font-variant-numeric:tabular-nums}.ix-kpi-label{display:flex;align-items:center;gap:4px;color:var(--ix-color-text-secondary);font-size:14px;flex:1}.ix-kpi-measurement{display:flex;align-items:baseline;gap:8px}.ix-kpi-measurement strong{font-size:24px;line-height:32px;font-weight:600}.ix-kpi-measurement>span{font-size:14px}.ix-kpi-warning{border-color:var(--ix-color-warning-text)}.ix-kpi-alarm{border-color:var(--ix-color-danger-text)}.ix-kpi[data-vertical=true]{flex-direction:column;gap:4px;align-items:flex-start;min-height:72px}
.ix-event-list{display:flex;flex-direction:column;gap:8px}.ix-event-list[data-compact=true]{gap:1px}.ix-event-list-item{--ix-event-status:transparent;display:flex;align-items:center;gap:12px;position:relative;width:100%;min-height:48px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);padding:8px 12px 8px 20px;overflow:hidden;background:transparent;color:var(--ix-color-text);font:inherit;text-align:start;cursor:pointer}.ix-event-list-item[data-filled=true]{background:var(--ix-color-surface-1);border-color:transparent}.ix-event-list-item:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-event-list-item[data-selected=true]{border-color:var(--ix-color-primary);background:var(--ix-color-surface-2)}.ix-event-list-item:disabled{opacity:.4;cursor:default}.ix-event-indicator{position:absolute;inset-block:0;left:0;width:4px;background:var(--ix-event-status)}.ix-event-content{display:flex;align-items:center;gap:16px;flex:1;min-width:0}.ix-event-status-alarm{--ix-event-status:var(--ix-color-danger)}.ix-event-status-warning{--ix-event-status:var(--ix-color-warning)}.ix-event-status-success{--ix-event-status:var(--ix-color-success)}.ix-event-status-info{--ix-event-status:var(--ix-color-info)}.ix-event-status-critical{--ix-event-status:var(--ix-color-critical,#eb780a)}.ix-event-status-primary{--ix-event-status:var(--ix-color-primary)}
.ix-workflow-steps{display:flex;list-style:none;padding:0;margin:0;max-width:100%;overflow-x:auto}.ix-workflow-step{position:relative;flex:1;min-width:80px;max-width:192px;--ix-step-color:var(--ix-color-text-secondary)}.ix-workflow-step>.ix-workflow-step-content{display:flex;flex-direction:column;gap:12px;align-items:center;width:100%;min-height:96px;padding:12px 8px;border:0;border-radius:2px;background:transparent;color:var(--ix-color-text);font:inherit;cursor:pointer}.ix-workflow-step[data-disabled=true]{opacity:.4}.ix-workflow-step>div.ix-workflow-step-content{cursor:default}.ix-workflow-step>button:disabled{cursor:default}.ix-workflow-step>button:hover:not(:disabled),.ix-workflow-step[data-selected=true]>.ix-workflow-step-content{background:var(--ix-color-surface-2)}.ix-workflow-marker{position:relative;z-index:1;display:flex;align-items:center;justify-content:center;width:32px;height:32px;background:var(--ix-color-surface);color:var(--ix-step-color);border:1px solid currentColor;border-radius:50%}.ix-workflow-step::before,.ix-workflow-step::after{content:"";position:absolute;top:28px;width:calc(50% - 16px);height:1px;background:var(--ix-step-color)}.ix-workflow-step::before{left:0}.ix-workflow-step::after{right:0}.ix-workflow-step:first-child::before,.ix-workflow-step:last-child::after{display:none}.ix-workflow-label{font-size:14px;line-height:20px;text-align:center}.ix-workflow-done,.ix-workflow-step[data-selected=true]{--ix-step-color:var(--ix-color-primary)}.ix-workflow-success{--ix-step-color:var(--ix-color-success)}.ix-workflow-warning{--ix-step-color:var(--ix-color-warning-text)}.ix-workflow-error{--ix-step-color:var(--ix-color-danger-text)}.ix-workflow-steps[data-vertical=true]{flex-direction:column}.ix-workflow-steps[data-vertical=true] .ix-workflow-step{max-width:none;min-width:0}.ix-workflow-steps[data-vertical=true] .ix-workflow-step>.ix-workflow-step-content{flex-direction:row;min-height:64px;padding:16px}.ix-workflow-steps[data-vertical=true] .ix-workflow-step::before,.ix-workflow-steps[data-vertical=true] .ix-workflow-step::after{left:32px;width:1px;height:calc(50% - 16px)}.ix-workflow-steps[data-vertical=true] .ix-workflow-step::before{top:0}.ix-workflow-steps[data-vertical=true] .ix-workflow-step::after{top:auto;bottom:0}
.ix-tabs[data-rounded=true] .ix-tab-wrapper{border-radius:0;overflow:visible;background:transparent}.ix-tabs[data-rounded=true] .ix-tab{position:relative;width:72px;height:72px;padding:12px;background:transparent}.ix-tab-rounded-circle{display:flex;align-items:center;justify-content:center;overflow:hidden;width:48px;height:48px;border:2px solid transparent;border-radius:50%;background:var(--ix-color-surface-1);color:var(--ix-color-text)}.ix-tabs[data-rounded=true] .ix-tab-wrapper[data-selected=true]{background:transparent}.ix-tab-wrapper[data-selected=true] .ix-tab-rounded-circle{background:transparent;border-color:var(--ix-color-primary);color:var(--ix-color-primary)}.ix-tabs[data-rounded=true] .ix-tab-counter{position:absolute;bottom:4px;left:50%;transform:translateX(-50%);min-width:24px;text-align:center;border:1px solid var(--ix-color-border)}.ix-tile-header>span{flex:1;min-width:0}.ix-content-header-title-line{display:flex;align-items:center;gap:8px;min-width:0}.ix-content-header-title-line>h1{min-width:0}.ix-content-header[data-ellipsis=true] .ix-content-header-titles h1,.ix-content-header[data-ellipsis=true] .ix-content-header-titles p{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.ix-content-header[data-ellipsis=true] .ix-content-header-title-line>h1{flex:1}.ix-tabs-shell{display:flex;align-items:stretch;min-width:0;width:100%}.ix-tabs-shell>.ix-tabs{flex:1;min-width:0}.ix-tabs-overflow{flex-shrink:0;align-self:center}.ix-tabs-overflow>.ix-dropdown-trigger{font-size:0;width:32px;min-width:32px;padding:4px;gap:0}.ix-tabs-overflow>.ix-dropdown-trigger>.ix-icon:last-child{display:none}.ix-blind-heading{display:flex;align-items:center;min-width:0}.ix-blind-header{flex:1;min-width:0}.ix-blind-actions{display:flex;align-items:center;gap:4px;padding-inline-end:8px}.ix-group-header-row{display:flex;align-items:center;min-height:64px;background:var(--ix-color-surface-1);border-radius:var(--ix-radius) var(--ix-radius) 0 0}.ix-group-header-row[data-selected=true]{background:var(--ix-color-surface-2);box-shadow:inset 2px 0 0 var(--ix-color-primary)}.ix-group-header-row>.ix-group-header{width:auto;flex:1;min-width:0;background:transparent}.ix-group-expand{display:flex;align-items:center;justify-content:center;width:32px;height:32px;flex-shrink:0;margin-right:8px;border:0;border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-text);cursor:pointer}.ix-group-expand:hover{background:var(--ix-color-surface-2)}.ix-group-expand:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-group-custom-item{cursor:default}.ix-key-value-list[data-striped=true]>.ix-key-value:nth-child(odd){background:var(--ix-color-surface-1)}.ix-key-value-list>.ix-key-value{border-bottom:1px solid var(--ix-color-border);padding-inline:8px}.ix-event-list-item{height:var(--ix-event-height,40px);min-height:var(--ix-event-height,40px);max-height:var(--ix-event-height,40px);padding-left:24px}.ix-event-indicator{width:8px}.ix-event-list[data-compact=true]{gap:0}.ix-event-list[data-compact=true] .ix-event-list-item{border-radius:0}.ix-table{border-collapse:collapse;border-spacing:0;width:100%;margin-bottom:16px;color:var(--ix-color-text);font-size:14px;line-height:20px;text-align:left}.ix-table th,.ix-table td{padding:8px;border-bottom:1px solid var(--ix-color-border);text-align:left;vertical-align:top}.ix-table thead{vertical-align:bottom}.ix-table-striped>tbody>tr:nth-child(odd)>*{background:var(--ix-color-surface-1)}.ix-table-striped>tbody>tr:nth-child(even)>*{background:var(--ix-color-surface)}
.ix-card-list-items[hidden],.ix-card-accordion-content[hidden],.ix-blind-content[hidden],.ix-group-content[hidden]{display:none}
.ix-card:focus-visible,.ix-disclosure-button:focus-visible,.ix-layout-icon-button:focus-visible,.ix-page-button:focus-visible,.ix-tab:focus-visible,.ix-tab-close:focus-visible,.ix-breadcrumb-item>a:focus-visible,.ix-breadcrumb-item>button:focus-visible,.ix-pagination-input:focus-visible,.ix-pagination-count select:focus-visible,.ix-blind-header:focus-visible,.ix-group-header:focus-visible,.ix-group-item:focus-visible,.ix-empty-state-action:focus-visible,.ix-workflow-step>button:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}
@media(max-width:600px){.ix-content{padding:16px}.ix-content-header{flex-wrap:wrap}.ix-content-header-actions{width:100%}.ix-content-header h1{font-size:24px;line-height:32px}.ix-empty-state-compact{flex-wrap:wrap;justify-content:flex-start}.ix-event-content{flex-wrap:wrap;gap:4px 12px}.ix-pagination-count{margin-inline-start:8px}}
@media(prefers-reduced-motion:reduce){.ix-chevron{transition:none}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn breadcrumb_truncation_keeps_the_current_location_visible() {
        assert_eq!(hidden_breadcrumb_count(5, 3), 2);
        assert_eq!(hidden_breadcrumb_count(3, 9), 0);
        assert_eq!(hidden_breadcrumb_count(3, 0), 2);
        assert_eq!(hidden_breadcrumb_count(0, 3), 0);
    }
    fn disclosure_slots_fixture() -> Element {
        rsx! {
            Blind { label: "Example", header_actions: rsx! { button { "Header action" } }, "Content" }
            Group { header: "Header text", header_actions: rsx! { button { "Group action" } }, GroupItem { button { "Test" } } }
            EventList { item_height: EventItemHeight::Pixels(60), EventListItem { "Test 1" } }
        }
    }
    #[test]
    fn disclosure_and_custom_group_slots_do_not_nest_controls_or_lose_row_height() {
        let mut dom = VirtualDom::new(disclosure_slots_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("--ix-event-height:60px"));
        assert!(html.contains("Header action"));
        assert!(html.contains("Group action"));
        let mut depth = 0;
        for tag in html.split('<').skip(1) {
            if tag.starts_with("button ") || tag.starts_with("button>") {
                depth += 1;
                assert_eq!(depth, 1, "nested interactive control: {html}");
            } else if tag.starts_with("/button>") {
                depth -= 1;
            }
        }
        assert_eq!(depth, 0);
    }
    fn card_with_actions_fixture() -> Element {
        rsx! {
            Card {
                aria_label: "Select device",
                CardTitle { actions: rsx! { button { "Inspect device" } }, "Device" }
                CardAccordion { "Device details" }
            }
        }
    }
    #[test]
    fn selectable_card_keeps_primary_and_nested_actions_as_separate_buttons() {
        let mut dom = VirtualDom::new(card_with_actions_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("<article"));
        assert!(html.contains("aria-label=\"Select device\""));
        let mut depth = 0;
        let mut buttons = 0;
        for tag in html.split('<').skip(1) {
            if tag.starts_with("button ") || tag.starts_with("button>") {
                depth += 1;
                buttons += 1;
                assert_eq!(depth, 1, "card actions must never nest buttons: {html}");
            } else if tag.starts_with("/button>") {
                depth -= 1;
            }
        }
        assert_eq!(depth, 0);
        assert_eq!(
            buttons, 3,
            "primary selection, title action and accordion action"
        );
    }
    #[test]
    fn mounted_tab_targets_follow_stable_keys_after_close_and_reorder() {
        let mut items = vec![
            TabItem::new("a", "A"),
            TabItem::new("b", "B"),
            TabItem::new("c", "C"),
        ];
        // Keyed DOM nodes retain these identities without firing onmounted again.
        let mounted = BTreeMap::from([
            ("a".into(), "DOM A"),
            ("b".into(), "DOM B"),
            ("c".into(), "DOM C"),
        ]);
        assert_eq!(
            mounted_tab(&items, next_tab(&items, 0, true).unwrap(), &mounted),
            Some(&"DOM B")
        );
        items.remove(0);
        assert_eq!(
            mounted_tab(&items, next_tab(&items, 0, true).unwrap(), &mounted),
            Some(&"DOM C")
        );
        items.swap(0, 1);
        assert_eq!(mounted_tab(&items, 0, &mounted), Some(&"DOM C"));
        assert_eq!(
            mounted_tab(&items, next_tab(&items, 0, true).unwrap(), &mounted),
            Some(&"DOM B")
        );
        items.push(TabItem::new("new", "New"));
        assert_eq!(mounted_tab(&items, 2, &mounted), None);
        assert_eq!(mounted_tab(&items, 99, &mounted), None);
    }
    #[test]
    fn tab_overflow_uses_measured_content_and_rejects_invalid_geometry() {
        assert!(!tabs_overflow(340.0, 340.0));
        assert!(tabs_overflow(340.5, 340.0));
        assert!(!tabs_overflow(340.0, 400.0));
        assert!(!tabs_overflow(340.0, 0.0));
        assert!(!tabs_overflow(f64::NAN, 340.0));
        assert!(!tabs_overflow(340.0, f64::INFINITY));
    }
    #[test]
    fn tab_selection_resolves_disabled_removed_and_empty_data() {
        let mut items = vec![
            TabItem::new("a", "A"),
            TabItem::new("b", "B"),
            TabItem::new("c", "C"),
        ];
        items[0].disabled = true;
        assert_eq!(selected_tab(&items, 0), Some(1));
        assert_eq!(selected_tab(&items, 2), Some(2));
        items.remove(2);
        assert_eq!(selected_tab(&items, 2), Some(1));
        items[1].disabled = true;
        assert_eq!(selected_tab(&items, 0), None);
        assert_eq!(selected_tab(&[], 0), None);
    }
    #[test]
    fn explicit_empty_selection_never_falls_back_to_an_enabled_tab() {
        let items = vec![TabItem::new("a", "A"), TabItem::new("b", "B")];
        assert_eq!(TabSelection::Empty.resolve(&items), None);
        assert_eq!(TabSelection::Index(0).resolve(&items), Some(0));
        assert_eq!(TabSelection::default().resolve(&items), Some(0));
    }

    #[test]
    fn tab_change_veto_is_shared_and_blocks_the_accepted_path() {
        let items = vec![TabItem::new("a", "A"), TabItem::new("b", "B")];
        let mut state = Some(0);
        let mut posts = 0;
        if accept_tab_request(&items, state, Some(1), |event| {
            assert_eq!(event.previous, Some(0));
            assert_eq!(event.next, Some(1));
            let retained = event.clone();
            event.prevent_default();
            assert!(retained.default_prevented());
        }) {
            state = Some(1);
            posts += 1;
        }
        assert_eq!(state, Some(0));
        assert_eq!(posts, 0);
        assert!(accept_tab_request(&items, state, None, |event| {
            assert_eq!(event.next, None);
        }));
        assert!(!accept_tab_request(&items, state, None, |event| event.prevent_default()));
    }

    #[test]
    fn invalid_and_disabled_tab_requests_emit_no_before_event() {
        let mut disabled = TabItem::new("disabled", "Disabled");
        disabled.disabled = true;
        let items = vec![TabItem::new("a", "A"), disabled];
        assert!(!accept_tab_request(&items, Some(0), Some(1), |_| panic!(
            "disabled request emitted"
        )));
        assert!(!accept_tab_request(&items, Some(0), Some(99), |_| panic!(
            "unknown request emitted"
        )));
        // Upstream tabClick delivers a change event even for the selected tab.
        assert!(accept_tab_request(&items, Some(0), Some(0), |_| {}));
    }

    #[test]
    fn tab_placement_defaults_to_the_upstream_bottom_edge() {
        assert_eq!(TabPlacement::default(), TabPlacement::Bottom);
        assert_eq!(TabPlacement::Top.class(), "top");
    }

    #[test]
    fn tab_navigation_skips_disabled_and_wraps() {
        let mut items = vec![
            TabItem::new("a", "A"),
            TabItem::new("b", "B"),
            TabItem::new("c", "C"),
        ];
        items[1].disabled = true;
        assert_eq!(next_tab(&items, 0, true), Some(2));
        assert_eq!(next_tab(&items, 2, true), Some(0));
        assert_eq!(next_tab(&items, 0, false), Some(2));
        items.iter_mut().for_each(|item| item.disabled = true);
        assert_eq!(next_tab(&items, 0, true), None);
        assert_eq!(next_tab(&[], 0, false), None);
    }
    #[test]
    fn pagination_empty_and_small_ranges_are_exact() {
        assert!(page_tokens(0, 5).is_empty());
        assert_eq!(page_tokens(1, 20), vec![PageToken::Page(0)]);
        assert_eq!(
            page_tokens(7, 3),
            (0..7).map(PageToken::Page).collect::<Vec<_>>()
        );
    }
    #[test]
    fn pagination_windows_preserve_edges_current_and_valid_jump_targets() {
        for count in 8..80 {
            for selected in 0..count + 2 {
                let tokens = page_tokens(count, selected);
                assert_eq!(tokens.first(), Some(&PageToken::Page(0)));
                assert_eq!(tokens.last(), Some(&PageToken::Page(count - 1)));
                assert!(tokens.contains(&PageToken::Page(selected.min(count - 1))));
                assert!(tokens.len() <= 7);
                let indices: Vec<_> = tokens
                    .iter()
                    .map(|token| match *token {
                        PageToken::Page(index) | PageToken::Jump(index) => index,
                    })
                    .collect();
                assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
                assert!(indices.iter().all(|index| *index < count));
            }
        }
    }
}
