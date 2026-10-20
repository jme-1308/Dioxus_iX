//! Cards, navigation, disclosure, and industrial information layouts.
use crate::Icon;
use dioxus::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

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
    pub(crate) fn class(self) -> &'static str {
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
    #[props(default)] id: Option<String>,
    #[props(default = "Select card".to_string())] aria_label: String,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] onselect: EventHandler<()>,
) -> Element {
    rsx! { article { id, class: "ix-card ix-card-{variant.class()} {class}", role: (!passive).then_some("group"), "data-selected": selected, "data-passive": passive,
        if !passive {
            button { class: "ix-card-select", r#type: "button", "aria-label": aria_label, "aria-pressed": selected,
                onclick: move |event| { onclick.call(event); onselect.call(()); }
            }
        }
        div { class: "ix-card-body", {children} }
    } }
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

pub use crate::disclosure::{Blind, BlindProps, CardAccordion, CardAccordionProps};

// Preserve the original layout module import paths.
pub use crate::card_list::{CardList, CardListProps};
pub use crate::card_list_model::*;

pub use crate::page_layout::{
    ContentHeader, ContentHeaderProps, ContentHeaderVariant, Tile, TileProps, TileSize,
};

pub use crate::task_cards::{ActionCard, ActionCardProps, PushCard, PushCardProps};

/// Application main content inset.
#[component]
pub fn Content(children: Element, #[props(default)] class: String) -> Element {
    rsx! { main { class: "ix-content {class}", {children} } }
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

// The framework owns element identity and platform-specific focus/scroll plumbing.
fn reveal_tab(node: Rc<MountedData>, focus: bool) {
    spawn(async move {
        if focus {
            let _ = node.set_focus(true).await;
        }
        let _ = node
            .scroll_to_with_options(dioxus::html::ScrollToOptions {
                behavior: dioxus::html::ScrollBehavior::Smooth,
                vertical: dioxus::html::ScrollLogicalPosition::Nearest,
                horizontal: dioxus::html::ScrollLogicalPosition::Nearest,
            })
            .await;
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
    /// Format each close button's accessible label from the current tab item.
    #[props(default)]
    close_label: Option<Callback<TabItem, String>>,
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
    let mut measured_overflow = use_signal(|| false);
    let mut mounted = use_signal(BTreeMap::<String, Rc<MountedData>>::new);
    let mut strip = use_signal(|| None::<Rc<MountedData>>);
    let measure = use_callback(move |()| {
        let node = strip.peek().clone();
        spawn(async move {
            if let Some(node) = node {
                if let (Ok(content), Ok(bounds)) =
                    (node.get_scroll_size().await, node.get_client_rect().await)
                {
                    // The strip has no inline border or padding; round the
                    // viewport width like the browser's integer scroll size.
                    let overflow = tabs_overflow(content.width, bounds.size.width.round());
                    if *measured_overflow.peek() != overflow {
                        measured_overflow.set(overflow);
                    }
                }
            }
        });
    });
    // Dioxus onresize observes the strip and intrinsic tab widths, including
    // changes caused by loaded fonts. Prop-driven removals also need a measure.
    use_effect(use_reactive!(|items, small, rounded, stretched| {
        let _ = (items, small, rounded, stretched);
        measure.call(());
    }));
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
    use_effect(use_reactive!(|active_key| {
        if let Some(key) = active_key {
            let node = mounted.read().get(&key).cloned();
            if let Some(node) = node {
                reveal_tab(node, false);
            }
        }
    }));
    let overflowing = measured_overflow();
    let show_menu = overflow_menu || (overflowing && !stretched);
    rsx! { div { id: root_id.clone(), class: "ix-tabs-shell", "data-overflow": overflowing,
        div { class: "ix-tabs", "data-placement": placement.class(), role: "tablist", "aria-label": label, tabindex: if overflowing { 0 } else { -1 }, "data-small": small, "data-rounded": rounded, "data-stretched": stretched,
        onmounted: move |event| { strip.set(Some(event.data())); measure.call(()); },
        onresize: move |_| measure.call(()),
        for (index, item) in items.iter().enumerate() {
            div { key: "{item.key}", class: "ix-tab-wrapper", "data-tab-key": item.key.clone(), "data-selected": active == Some(index),
                onresize: move |_| measure.call(()),
                button { class: "ix-tab", r#type: "button", role: "tab", disabled: item.disabled, "aria-label": if rounded { Some(item.label.clone()) } else { None }, title: if rounded { Some(item.label.clone()) } else { None }, "aria-selected": active == Some(index), tabindex: if active.or_else(|| items.iter().position(|item| !item.disabled)) == Some(index) { 0 } else { -1 },
                    id: format!("{prefix}-tab-{}", crate::composition::tab_key_id(&item.key)),
                    "aria-controls": id_prefix.as_ref().map(|prefix| format!("{prefix}-panel-{}", crate::composition::tab_key_id(&item.key))),
                    onmounted: { let key = item.key.clone(); move |event| { mounted.write().insert(key.clone(), event.data()); } },
                    onclick: move |_| request.call(Some(index)),
                    onkeydown: { let items = items.clone(); move |event: KeyboardEvent| {
                        if event.key() == Key::Delete {
                            if items.get(index).is_some_and(|item|item.closable && !item.disabled) {
                                event.prevent_default();
                                onclose.call(index);
                            }
                            return;
                        }
                        let target = match event.key() { Key::ArrowRight => next_tab(&items, index, true), Key::ArrowLeft => next_tab(&items, index, false), Key::Home => items.iter().position(|item| !item.disabled), Key::End => items.iter().rposition(|item| !item.disabled), _ => return };
                        event.prevent_default();
                        if let Some(target) = target {
                            if keyboard_navigation == TabKeyboardNavigation::Automatic { request.call(Some(target)); }
                            let node = mounted_tab(&items, target, &mounted.peek()).cloned();
                            if let Some(node) = node { reveal_tab(node, true); }
                        }
                    } },
                    if rounded { span { class: "ix-tab-rounded-circle", if let Some(icon) = &item.icon { Icon { name: icon.clone(), size: 32 } } else { "{item.label}" } } }
                    else { if let Some(icon) = &item.icon { Icon { name: icon.clone() } } span { "{item.label}" } }
                    if let Some(counter) = item.counter { span { class: "ix-tab-counter", "{counter}" } }
                }
                if item.closable && !rounded { button { class: "ix-tab-close", r#type: "button", disabled: item.disabled,
                    "aria-label": close_label.map(|format|format.call(item.clone())).unwrap_or_else(||format!("Close {}",item.label)),
                    onclick: {let disabled=item.disabled;move |_| {if !disabled {onclose.call(index);}}}, Icon { name: "close", size: 16 } } }
            }
        }
    }
        if show_menu { crate::Dropdown { label: "All tabs", aria_label: aria_label_more_tabs, icon: "more-menu", variant: crate::ButtonVariant::SubtleTertiary, class: "ix-tabs-overflow",
            for (index, item) in items.iter().enumerate() { crate::DropdownItem { key: "{item.key}", label: item.label.clone(), icon: item.icon.clone(), disabled: item.disabled, checked: active == Some(index), checkable: true, onclick: { let key = item.key.clone(); move |_| { request.call(Some(index)); let node = mounted.peek().get(&key).cloned(); if let Some(node) = node { reveal_tab(node, true); } } } } }
        } }
    } }
}

pub use crate::breadcrumb::{
    Breadcrumb, BreadcrumbEntry, BreadcrumbItem, BreadcrumbItemProps, BreadcrumbLabels,
    BreadcrumbNavigateEvent, BreadcrumbNavigationSource, BreadcrumbProps,
};

pub use crate::pagination::{Pagination, PaginationChangeEvent, PaginationLabels, PaginationProps};

pub use crate::groups::{
    Group, GroupChange, GroupChangeRequest, GroupEntry, GroupItem, GroupItemProps, GroupProps,
};

pub use crate::information_states::{
    EmptyState, EmptyStateLayout, EmptyStateProps, Kpi, KpiProps, KpiState,
};

pub use crate::key_value::{
    KeyValue, KeyValueLabelPosition, KeyValueList, KeyValueListProps, KeyValueProps,
};

pub use crate::event_list::{
    EventItemHeight, EventList, EventListItem, EventListItemProps, EventListProps,
};

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
    fn icon(self, selected: bool) -> &'static str {
        match self {
            Self::Open if selected => "circle-dot",
            Self::Open => "circle",
            Self::Success => "success",
            Self::Done => "circle-filled",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
    fn icon_label(self, selected: bool) -> &'static str {
        match self {
            Self::Open if selected => "Circle dot",
            Self::Open => "Circle",
            Self::Success => "Success",
            Self::Done => "Done",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
}

/// Cancelable request emitted before a workflow selection changes.
/// Repeated activation of the selected step still emits this request.
#[derive(Clone, Debug)]
pub struct WorkflowStepSelectedEvent {
    pub index: usize,
    pub previous: Option<usize>,
    prevented: Rc<Cell<bool>>,
}
impl WorkflowStepSelectedEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

static WORKFLOW_ID: AtomicUsize = AtomicUsize::new(1);
fn workflow_mount_id() -> String {
    format!(
        "ix-workflow-{}",
        WORKFLOW_ID.fetch_add(1, Ordering::Relaxed)
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct WorkflowSettings {
    selected_index: isize,
    clickable: bool,
    vertical: bool,
}

struct WorkflowMember {
    update: Arc<dyn Fn() + Send + Sync>,
    order: Option<usize>,
    sequence: usize,
}
struct WorkflowModel {
    settings: WorkflowSettings,
    members: BTreeMap<String, WorkflowMember>,
    next_sequence: usize,
    selected: Option<String>,
    onstepselected: Option<EventHandler<WorkflowStepSelectedEvent>>,
    alive: bool,
}
impl Default for WorkflowModel {
    fn default() -> Self {
        Self {
            settings: WorkflowSettings::default(),
            members: BTreeMap::new(),
            next_sequence: 0,
            selected: None,
            onstepselected: None,
            alive: true,
        }
    }
}
impl WorkflowModel {
    fn order(&self) -> Vec<String> {
        let mut members: Vec<_> = self.members.iter().collect();
        members.sort_by_key(|(_, member)| {
            (
                member.order.is_none(),
                member.order.unwrap_or(member.sequence),
                member.sequence,
            )
        });
        members.into_iter().map(|(id, _)| id.clone()).collect()
    }
    fn reset_selection(&mut self) {
        self.selected = usize::try_from(self.settings.selected_index)
            .ok()
            .and_then(|index| self.order().get(index).cloned());
    }
    fn position(&self, id: &str) -> &'static str {
        let order = self.order();
        match (order.iter().position(|entry| entry == id), order.len()) {
            (Some(_), 1) => "single",
            (Some(0), _) => "first",
            (Some(index), length) if index + 1 == length => "last",
            _ => "undefined",
        }
    }
}

#[derive(Clone, Default)]
struct WorkflowContext(Rc<RefCell<WorkflowModel>>);
impl WorkflowContext {
    fn notify(&self) {
        let callbacks: Vec<_> = {
            let model = self.0.borrow();
            if !model.alive {
                return;
            }
            model
                .members
                .values()
                .map(|member| member.update.clone())
                .collect()
        };
        for callback in callbacks {
            callback();
        }
    }
    fn register(&self, id: String, order: Option<usize>, update: Arc<dyn Fn() + Send + Sync>) {
        {
            let mut model = self.0.borrow_mut();
            if !model.alive {
                return;
            }
            let sequence = model.next_sequence;
            model.next_sequence = model.next_sequence.wrapping_add(1);
            model.members.insert(
                id,
                WorkflowMember {
                    update,
                    order,
                    sequence,
                },
            );
            model.reset_selection();
        }
        self.notify();
    }
    fn unregister(&self, id: &str) {
        {
            let mut model = self.0.borrow_mut();
            if !model.alive || model.members.remove(id).is_none() {
                return;
            }
            model.reset_selection();
        }
        self.notify();
    }
    fn update_order(&self, id: &str, order: Option<usize>) {
        {
            let mut model = self.0.borrow_mut();
            if !model.alive {
                return;
            }
            let Some(member) = model.members.get_mut(id) else {
                return;
            };
            if member.order == order {
                return;
            }
            member.order = order;
            model.reset_selection();
        }
        self.notify();
    }
    fn apply(
        &self,
        settings: WorkflowSettings,
        onstepselected: EventHandler<WorkflowStepSelectedEvent>,
    ) {
        let changed = {
            let mut model = self.0.borrow_mut();
            let previous = (model.settings, model.selected.clone());
            model.settings = settings;
            model.onstepselected = Some(onstepselected);
            model.reset_selection();
            previous != (settings, model.selected.clone())
        };
        if changed {
            self.notify();
        }
    }
}

fn request_workflow_selection(
    context: &WorkflowContext,
    id: &str,
    disabled: bool,
    before: impl FnOnce(WorkflowStepSelectedEvent),
) -> bool {
    let event = {
        let model = context.0.borrow();
        if !model.alive || disabled || !model.settings.clickable {
            return false;
        }
        let order = model.order();
        let Some(index) = order.iter().position(|entry| entry == id) else {
            return false;
        };
        WorkflowStepSelectedEvent {
            index,
            previous: model
                .selected
                .as_ref()
                .and_then(|selected| order.iter().position(|entry| entry == selected)),
            prevented: Default::default(),
        }
    };
    before(event.clone());
    if event.default_prevented() {
        return false;
    }
    {
        let mut model = context.0.borrow_mut();
        if !model.alive || !model.settings.clickable || !model.members.contains_key(id) {
            return false;
        }
        model.selected = Some(id.to_string());
    }
    context.notify();
    true
}

/// Ordered workflow with parent-authoritative settings and cancelable navigation.
/// Accepted activation selects a step locally without changing `selected_index`;
/// a later render of this component or topology change reapplies that prop.
/// Negative and out-of-range indices select nothing; Rust indices exclude the
/// JavaScript fractional/NaN domain. Descendants may be wrapped in components.
/// Dynamic/reordered lists supply the `WorkflowStep` `order` prop;
/// omitted orders preserve stable registration order for static compositions.
/// Nested workflows form independent
/// ownership boundaries, so an outer workflow does not own inner steps.
#[component]
pub fn WorkflowSteps(
    children: Element,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] vertical: bool,
    #[props(default)] clickable: bool,
    #[props(default)] selected_index: isize,
    #[props(default = "Workflow".to_string())] label: String,
    #[props(default)] onstepselected: EventHandler<WorkflowStepSelectedEvent>,
) -> Element {
    let generated = use_hook(workflow_mount_id);
    let id = id.unwrap_or(generated);
    let context = use_context_provider(WorkflowContext::default);
    context.apply(
        WorkflowSettings {
            selected_index,
            clickable,
            vertical,
        },
        onstepselected,
    );
    let drop_context = context.clone();
    use_drop(move || {
        let mut model = drop_context.0.borrow_mut();
        model.alive = false;
        model.members.clear();
        model.selected = None;
    });
    rsx! {
        div { id, class: "ix-workflow-steps {class}", role: "group", "aria-label": label,
            "data-vertical": vertical,
            onclick: move |event| event.stop_propagation(),
            onkeydown: move |event| if event.key() == Key::Enter || event.key() == Key::Character(" ".into()) { event.stop_propagation(); },
            {children}
        }
    }
}

/// Workflow content with its own status and disabled state. Within a workflow,
/// parent settings override `selected`, `clickable`, and `vertical`. Each enabled
/// clickable step is independently tabbable; Enter and Space activate it.
/// `custom_icon` replaces both built-in icon layers. `children` supplies arbitrary
/// default content, replacing `label`; `label` is the plain-text fallback.
/// Standalone steps retain the legacy accepted `onselect` callback.
/// Enumerate `order` for dynamic/keyed lists, including wrapped steps. Explicit
/// orders sort first; equal orders and omitted orders retain registration order.
#[component]
pub fn WorkflowStep(
    #[props(default)] label: String,
    #[props(default)] children: Option<Element>,
    #[props(default)] id: Option<String>,
    #[props(default)] custom_icon: Option<Element>,
    #[props(default)] status: WorkflowStatus,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] clickable: bool,
    #[props(default)] vertical: bool,
    /// Zero-based visual order within the nearest workflow. Supply this for
    /// dynamically inserted or reordered lists; static compositions may omit it.
    #[props(default)]
    order: Option<usize>,
    #[props(default)] onselect: EventHandler<()>,
) -> Element {
    let context = try_use_context::<WorkflowContext>();
    let registration_context = context.clone();
    let update = dioxus::core::schedule_update();
    let mount_id = use_hook(move || {
        let id = workflow_mount_id();
        if let Some(context) = registration_context {
            context.register(id.clone(), order, update);
        }
        id
    });
    let drop_context = context.clone();
    let drop_id = mount_id.clone();
    use_drop(move || {
        if let Some(context) = drop_context {
            context.unregister(&drop_id);
        }
    });
    if let Some(context) = &context {
        context.update_order(&mount_id, order);
    }
    let (selected, clickable, vertical, position, index) = if let Some(context) = &context {
        let model = context.0.borrow();
        (
            model.selected.as_ref() == Some(&mount_id),
            model.settings.clickable,
            model.settings.vertical,
            model.position(&mount_id),
            model.order().iter().position(|id| id == &mount_id),
        )
    } else {
        (selected, clickable, vertical, "single", None)
    };
    let activate = use_callback(move |_| {
        if disabled || !clickable {
            return;
        }
        if let Some(context) = &context {
            let before = context.0.borrow().onstepselected;
            if request_workflow_selection(context, &mount_id, disabled, |event| {
                if let Some(before) = before {
                    before.call(event);
                }
            }) {
                onselect.call(());
            }
        } else {
            onselect.call(());
        }
    });
    let icon_name = status.icon(selected);
    let icon_label = status.icon_label(selected);
    rsx! {
        div { id, class: "ix-workflow-step ix-workflow-{status.class()}",
            "data-selected": selected, "data-disabled": disabled, "data-vertical": vertical, "data-clickable": clickable,
            "data-position": position, "data-index": index.map(|index| index.to_string()),
            div { class: "ix-workflow-step-content", role: clickable.then_some("button"),
                tabindex: if disabled || !clickable { -1 } else { 0 },
                "aria-disabled": disabled.then_some("true"), "aria-current": selected.then_some("step"),
                onclick: move |_| activate.call(()),
                onkeydown: move |event| if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
                    event.prevent_default(); activate.call(());
                },
                div { class: "ix-workflow-marker-wrap",
                    span { class: "ix-workflow-line", "aria-hidden": "true" }
                    span { class: "ix-workflow-marker",
                        if let Some(custom_icon) = custom_icon { {custom_icon} }
                        else {
                            Icon { name: if status == WorkflowStatus::Warning { "triangle-filled" } else { "circle-filled" }, class: "ix-workflow-icon-underlay" }
                            Icon { name: icon_name, label: icon_label.to_string(), class: "ix-workflow-status-icon" }
                        }
                    }
                }
                div { class: "ix-workflow-label", if let Some(children) = children { {children} } else { "{label}" } }
            }
        }
    }
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
.ix-workflow-steps{display:flex;position:relative;width:fit-content;height:fit-content;min-width:var(--ix-workflow-steps--min-width,var(--si-sys-sizing-size-80,80px));max-width:100%;overflow-x:auto}.ix-workflow-steps[data-vertical=true]{flex-direction:column}
.ix-workflow-step{display:inline-block;position:relative;flex:none;width:var(--ix-workflow-step-horizontal--width,calc(var(--si-sys-sizing-size-150,160px) + var(--si-sys-sizing-spacing-x-90,32px)));height:var(--ix-workflow-step--height,var(--si-sys-sizing-size-110,96px));min-width:var(--ix-workflow-step-horizontal--min-width,var(--si-sys-sizing-size-80,80px));min-height:var(--ix-workflow-step-horizontal--min-height,var(--si-sys-sizing-size-110,96px));max-width:var(--ix-workflow-step-horizontal--max-width,calc(var(--si-sys-sizing-size-150,160px) + var(--si-sys-sizing-spacing-x-90,32px)));--ix-step-icon:var(--ix-workflow-step-icon-default--color,var(--ix-color-text-secondary));--ix-step-line:var(--ix-step-icon)}
.ix-workflow-step-content{display:flex;flex-direction:column;align-items:center;border-radius:var(--ix-workflow-step--border-radius,calc(var(--si-sys-sizing-border-radius-xs,4px)/2));padding:var(--ix-workflow-step-step--padding,calc(var(--si-sys-sizing-spacing-y-60,12px) + var(--si-sys-sizing-spacing-y-10,4px)) 0 var(--si-sys-sizing-spacing-y-40,8px));background:var(--ix-workflow-step--background,transparent);color:var(--ix-color-text);font:inherit}
.ix-workflow-step-content[aria-current=step]{background:var(--ix-workflow-step--background--selected,var(--si-sys-color-background-accent-secondary-active,var(--ix-color-surface-2)))}.ix-workflow-step-content[role=button]:not([aria-disabled=true]){cursor:pointer}.ix-workflow-step-content[role=button]:not([aria-disabled=true]):hover{background:var(--ix-workflow-step--background--hover,var(--si-sys-color-background-hover,var(--ix-color-surface-2)))}.ix-workflow-step-content[role=button]:not([aria-disabled=true]):active{background:var(--ix-workflow-step--background--active,var(--si-sys-color-background-accent-secondary-active,var(--ix-color-surface-2)))}
.ix-workflow-marker-wrap{display:flex;width:100%;align-items:center;justify-content:center;position:relative}.ix-workflow-line{width:100%;height:var(--ix-workflow-step-line--height,var(--si-sys-sizing-size-10,1px));background:var(--ix-step-line)}.ix-workflow-step[data-position=first]>.ix-workflow-step-content>.ix-workflow-marker-wrap>.ix-workflow-line{width:50%;margin-inline-start:auto}.ix-workflow-step[data-position=last]>.ix-workflow-step-content>.ix-workflow-marker-wrap>.ix-workflow-line{width:50%;margin-inline-end:auto}.ix-workflow-step[data-position=single]>.ix-workflow-step-content>.ix-workflow-marker-wrap>.ix-workflow-line{width:0}
.ix-workflow-marker{display:flex;align-items:center;justify-content:center;position:absolute;width:24px;height:24px;color:var(--ix-step-icon)}.ix-workflow-marker>.ix-icon{position:absolute}.ix-workflow-icon-underlay{color:var(--si-sys-color-background-0,var(--ix-color-surface))}.ix-workflow-label{margin-top:var(--ix-workflow-step-text--margin-top,var(--si-sys-sizing-spacing-y-60,12px));width:100%;padding:var(--ix-workflow-step-text--padding,0 var(--si-sys-sizing-spacing-x-40,8px));white-space:nowrap;text-overflow:ellipsis;overflow:hidden;text-align:center;font-size:.875rem;line-height:1.25rem}
.ix-workflow-step[data-selected=true]{--ix-step-icon:var(--ix-workflow-step-icon-default--color--selected,var(--si-sys-color-text-accent-active,var(--ix-color-primary)))}.ix-workflow-done{--ix-step-icon:var(--ix-workflow-step-icon-done--color,var(--si-sys-color-text-accent,var(--ix-color-primary)))}.ix-workflow-done[data-selected=true]{--ix-step-icon:var(--ix-workflow-step-icon-done--color--selected,var(--si-sys-color-text-accent-active,var(--ix-color-primary)))}.ix-workflow-success{--ix-step-icon:var(--ix-workflow-step-icon-success--color,var(--ix-color-success));--ix-step-line:var(--ix-workflow-step-line--background--success,var(--ix-color-success))}.ix-workflow-warning{--ix-step-icon:var(--ix-workflow-step-icon-warning--color,var(--ix-color-warning-text));--ix-step-line:var(--ix-workflow-step-line--background--warning,var(--ix-color-warning-text))}.ix-workflow-error{--ix-step-icon:var(--ix-workflow-step-icon-error--color,var(--ix-color-danger-text));--ix-step-line:var(--ix-workflow-step-line--background--error,var(--ix-color-danger-text))}
.ix-workflow-success[data-selected=true]{--ix-step-icon:var(--ix-workflow-step-icon-success--color,var(--ix-color-success))}.ix-workflow-warning[data-selected=true]{--ix-step-icon:var(--ix-workflow-step-icon-warning--color,var(--ix-color-warning-text))}.ix-workflow-error[data-selected=true]{--ix-step-icon:var(--ix-workflow-step-icon-error--color,var(--ix-color-danger-text))}
.ix-workflow-done[data-disabled=false][data-clickable=true]:hover{--ix-step-icon:var(--ix-workflow-step-icon-done--color--hover,var(--si-sys-color-text-accent-hover,var(--ix-color-primary)))}.ix-workflow-done[data-disabled=false][data-clickable=true]:active{--ix-step-icon:var(--ix-workflow-step-icon-done--color--active,var(--si-sys-color-text-accent-active,var(--ix-color-primary)))}.ix-workflow-open[data-selected=true][data-disabled=false][data-clickable=true]:hover{--ix-step-icon:var(--ix-workflow-step-icon-default--color--selected--hover,var(--si-sys-color-text-accent-active,var(--ix-color-primary)))}.ix-workflow-open[data-selected=true][data-disabled=false][data-clickable=true]:active{--ix-step-icon:var(--ix-workflow-step-icon-default--color--selected--active,var(--si-sys-color-text-accent-active,var(--ix-color-primary)))}.ix-workflow-done[data-selected=true][data-disabled=false][data-clickable=true]:hover{--ix-step-icon:var(--ix-workflow-step-icon-done--color--selected--hover,var(--si-sys-color-text-accent-active,var(--ix-color-primary)))}.ix-workflow-done[data-selected=true][data-disabled=false][data-clickable=true]:active{--ix-step-icon:var(--ix-workflow-step-icon-done--color--selected--active,var(--si-sys-color-text-accent-active,var(--ix-color-primary)))}
.ix-workflow-step[data-disabled=true]{--ix-step-icon:var(--ix-workflow-step-icon-status--color--disabled,var(--si-sys-color-text-disabled,var(--ix-color-text-secondary)));--ix-step-line:var(--ix-workflow-step-icon-default--color--disabled,var(--si-sys-color-text-disabled,var(--ix-color-text-secondary)))}.ix-workflow-step[data-disabled=true]>.ix-workflow-step-content{background:var(--ix-workflow-step--background--disabled,transparent);color:var(--ix-workflow-step--color--disabled,var(--si-sys-color-text-disabled,var(--ix-color-text-secondary)))}
.ix-workflow-step[data-vertical=true]{width:100%;min-width:var(--ix-workflow-step-vertical--min-width,var(--si-sys-sizing-size-110,64px));min-height:var(--ix-workflow-step-vertical--min-height,var(--si-sys-sizing-size-80,64px));max-height:var(--ix-workflow-step-vertical--max-height,calc(var(--si-sys-sizing-size-150,160px) + var(--si-sys-sizing-spacing-y-90,32px)));max-width:none}.ix-workflow-step[data-vertical=true]>.ix-workflow-step-content{flex-direction:row;padding:0;height:100%}.ix-workflow-step[data-vertical=true]>.ix-workflow-step-content>.ix-workflow-marker-wrap{width:auto;padding-inline-start:var(--ix-workflow-step-wrapper-vertical--padding-left,calc(var(--si-sys-sizing-spacing-x-60,20px) + var(--si-sys-sizing-spacing-x-10,4px)));height:100%}.ix-workflow-step[data-vertical=true]>.ix-workflow-step-content>.ix-workflow-marker-wrap>.ix-workflow-line{width:var(--ix-workflow-step-line--width,var(--si-sys-sizing-size-10,1px));height:100%;margin:0}.ix-workflow-step[data-vertical=true][data-position=first]>.ix-workflow-step-content>.ix-workflow-marker-wrap>.ix-workflow-line{height:50%;margin-block-start:auto}.ix-workflow-step[data-vertical=true][data-position=last]>.ix-workflow-step-content>.ix-workflow-marker-wrap>.ix-workflow-line{height:50%;margin-block-end:auto}.ix-workflow-step[data-vertical=true][data-position=single]>.ix-workflow-step-content>.ix-workflow-marker-wrap>.ix-workflow-line{width:0}.ix-workflow-step[data-vertical=true]>.ix-workflow-step-content>.ix-workflow-label{margin:var(--ix-workflow-step-text-vertical--margin,0 var(--si-sys-sizing-spacing-x-60,20px));padding:0;width:auto;text-align:start}
.ix-workflow-step-content:focus-visible{outline:var(--ix-workflow-step--outline-width--focus,var(--si-sys-sizing-border-width-default,1px)) solid var(--ix-workflow-step-step--outline-color--focus,var(--ix-color-focus));outline-offset:var(--ix-workflow-step-step--outline-offset,var(--si-sys-sizing-focus-ring-offset,-2px))}
.ix-tabs[data-rounded=true] .ix-tab-wrapper{border-radius:0;overflow:visible;background:transparent}.ix-tabs[data-rounded=true] .ix-tab{position:relative;width:72px;height:72px;padding:12px;background:transparent}.ix-tab-rounded-circle{display:flex;align-items:center;justify-content:center;overflow:hidden;width:48px;height:48px;border:2px solid transparent;border-radius:50%;background:var(--ix-color-surface-1);color:var(--ix-color-text)}.ix-tabs[data-rounded=true] .ix-tab-wrapper[data-selected=true]{background:transparent}.ix-tab-wrapper[data-selected=true] .ix-tab-rounded-circle{background:transparent;border-color:var(--ix-color-primary);color:var(--ix-color-primary)}.ix-tabs[data-rounded=true] .ix-tab-counter{position:absolute;bottom:4px;left:50%;transform:translateX(-50%);min-width:24px;text-align:center;border:1px solid var(--ix-color-border)}.ix-tile-header>span{flex:1;min-width:0}.ix-content-header-title-line{display:flex;align-items:center;gap:8px;min-width:0}.ix-content-header-title-line>h1{min-width:0}.ix-content-header[data-ellipsis=true] .ix-content-header-titles h1,.ix-content-header[data-ellipsis=true] .ix-content-header-titles p{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.ix-content-header[data-ellipsis=true] .ix-content-header-title-line>h1{flex:1}.ix-tabs-shell{display:flex;align-items:stretch;min-width:0;width:100%}.ix-tabs-shell>.ix-tabs{flex:1;min-width:0}.ix-tabs-overflow{flex-shrink:0;align-self:center}.ix-tabs-overflow>.ix-dropdown-trigger{font-size:0;width:32px;min-width:32px;padding:4px;gap:0}.ix-tabs-overflow>.ix-dropdown-trigger>.ix-icon:last-child{display:none}.ix-blind-heading{display:flex;align-items:center;min-width:0}.ix-blind-header{flex:1;min-width:0}.ix-blind-actions{display:flex;align-items:center;gap:4px;padding-inline-end:8px}.ix-group-header-row{display:flex;align-items:center;min-height:64px;background:var(--ix-color-surface-1);border-radius:var(--ix-radius) var(--ix-radius) 0 0}.ix-group-header-row[data-selected=true]{background:var(--ix-color-surface-2);box-shadow:inset 2px 0 0 var(--ix-color-primary)}.ix-group-header-row>.ix-group-header{width:auto;flex:1;min-width:0;background:transparent}.ix-group-expand{display:flex;align-items:center;justify-content:center;width:32px;height:32px;flex-shrink:0;margin-right:8px;border:0;border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-text);cursor:pointer}.ix-group-expand:hover{background:var(--ix-color-surface-2)}.ix-group-expand:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-group-custom-item{cursor:default}.ix-key-value-list[data-striped=true]>.ix-key-value:nth-child(odd){background:var(--ix-color-surface-1)}.ix-key-value-list>.ix-key-value{border-bottom:1px solid var(--ix-color-border);padding-inline:8px}.ix-event-list-item{height:var(--ix-event-height,40px);min-height:var(--ix-event-height,40px);max-height:var(--ix-event-height,40px);padding-left:24px}.ix-event-indicator{width:8px}.ix-event-list[data-compact=true]{gap:0}.ix-event-list[data-compact=true] .ix-event-list-item{border-radius:0}.ix-table{border-collapse:collapse;border-spacing:0;width:100%;margin-bottom:16px;color:var(--ix-color-text);font-size:14px;line-height:20px;text-align:left}.ix-table th,.ix-table td{padding:8px;border-bottom:1px solid var(--ix-color-border);text-align:left;vertical-align:top}.ix-table thead{vertical-align:bottom}.ix-table-striped>tbody>tr:nth-child(odd)>*{background:var(--ix-color-surface-1)}.ix-table-striped>tbody>tr:nth-child(even)>*{background:var(--ix-color-surface)}
.ix-card-list-items[hidden],.ix-card-accordion-content[hidden],.ix-blind-content[hidden],.ix-group-content[hidden]{display:none}
.ix-card:focus-visible,.ix-disclosure-button:focus-visible,.ix-layout-icon-button:focus-visible,.ix-page-button:focus-visible,.ix-tab:focus-visible,.ix-tab-close:focus-visible,.ix-breadcrumb-item>a:focus-visible,.ix-breadcrumb-item>button:focus-visible,.ix-pagination-input:focus-visible,.ix-pagination-count select:focus-visible,.ix-blind-header:focus-visible,.ix-group-header:focus-visible,.ix-group-item:focus-visible,.ix-empty-state-action:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}
@media(max-width:600px){.ix-content{padding:16px}.ix-content-header{flex-wrap:wrap}.ix-content-header-actions{width:100%}.ix-content-header h1{font-size:24px;line-height:32px}.ix-empty-state-compact{flex-wrap:wrap;justify-content:flex-start}.ix-event-content{flex-wrap:wrap;gap:4px 12px}.ix-pagination-count{margin-inline-start:8px}}
@media(prefers-reduced-motion:reduce){.ix-chevron{transition:none}}
.ix-card-list-item{width:320px;max-width:100%;min-width:0;flex-shrink:0}.ix-card-list-item[hidden]{display:none}.ix-card-list-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-card-list-more{width:320px;max-width:100%;min-width:0;flex-shrink:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px;padding:24px 16px;min-height:160px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-text);font:inherit;text-align:center;cursor:pointer}.ix-card-list-more:hover{background:var(--ix-color-surface-2)}.ix-card-list-more:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-card-list-more>.ix-icon{width:32px;height:32px}

.ix-breadcrumb-menu{display:flex;align-items:center;min-width:0}.ix-breadcrumb-menu>.ix-dropdown-trigger{max-width:100%;gap:4px}.ix-breadcrumb[data-subtle=true] .ix-breadcrumb-item>a,.ix-breadcrumb[data-subtle=true] .ix-breadcrumb-item>button,.ix-breadcrumb-item[data-subtle=true]>a,.ix-breadcrumb-item[data-subtle=true]>button,.ix-breadcrumb-menu[data-subtle=true]>.ix-dropdown-trigger{background:transparent}.ix-breadcrumb-menu[data-subtle=true]>.ix-dropdown-trigger:hover{background:var(--ix-color-surface-2)}.ix-breadcrumb-menu>.ix-dropdown-trigger[aria-current=page]{color:var(--ix-color-text)}

.ix-card-accordion{background:var(--ix-card-bg,transparent);color:var(--ix-card-fg,inherit)}
.ix-blind.ix-card-outline{--ix-card-bg:transparent;--ix-card-fg:var(--ix-color-text)}
.ix-card-accordion.ix-card-outline{--ix-card-bg:transparent;--ix-card-fg:inherit}
.ix-blind.ix-card-filled,.ix-card-accordion.ix-card-filled{--ix-card-bg:var(--ix-color-surface-1);--ix-card-fg:var(--ix-color-text)}
.ix-blind.ix-card-alarm,.ix-card-accordion.ix-card-alarm{--ix-card-bg:var(--ix-color-danger);--ix-card-fg:var(--ix-color-on-danger,#fff)}
.ix-blind.ix-card-critical,.ix-card-accordion.ix-card-critical{--ix-card-bg:var(--ix-color-critical,#eb780a);--ix-card-fg:var(--ix-color-on-critical,#000028)}
.ix-blind.ix-card-warning,.ix-card-accordion.ix-card-warning{--ix-card-bg:var(--ix-color-warning);--ix-card-fg:var(--ix-color-on-warning,#000028)}
.ix-blind.ix-card-success,.ix-card-accordion.ix-card-success{--ix-card-bg:var(--ix-color-success);--ix-card-fg:var(--ix-color-on-success,#000028)}
.ix-blind.ix-card-info,.ix-card-accordion.ix-card-info{--ix-card-bg:var(--ix-color-info);--ix-card-fg:var(--ix-color-on-info,#000028)}
.ix-blind.ix-card-neutral,.ix-card-accordion.ix-card-neutral{--ix-card-bg:var(--ix-color-secondary);--ix-card-fg:var(--ix-color-on-secondary,#000028)}
.ix-blind.ix-card-primary,.ix-card-accordion.ix-card-primary{--ix-card-bg:var(--ix-color-primary);--ix-card-fg:var(--ix-color-on-primary)}
.ix-blind-custom-header{display:block;min-width:0;flex:1;overflow-wrap:anywhere}.ix-blind-label{min-width:0;overflow-wrap:anywhere}

.ix-event-list{--ix-event-row-radius:var(--ix-radius)}.ix-event-list[data-compact=true]{--ix-event-row-radius:0}
.ix-event-row{position:relative;height:var(--ix-event-height,40px);min-height:var(--ix-event-height,40px);max-height:var(--ix-event-height,40px);min-width:0;border-radius:var(--ix-event-row-radius,var(--ix-radius));opacity:1}
.ix-event-row>.ix-event-list-item{position:absolute;inset:0;width:100%;height:100%;min-height:100%;max-height:100%;box-sizing:border-box}
.ix-event-row>.ix-event-row-body{position:relative;display:flex;align-items:center;gap:12px;box-sizing:border-box;width:100%;height:100%;min-width:0;padding:8px 12px 8px 24px;border-radius:inherit;pointer-events:none;overflow:hidden;color:var(--ix-color-text);font:inherit}
.ix-event-row-body>.ix-event-indicator{inset-block:1px;left:1px;width:8px;border-radius:var(--ix-event-row-radius,var(--ix-radius)) 0 0 var(--ix-event-row-radius,var(--ix-radius))}
.ix-event-row-body>.ix-event-content{height:100%;min-width:0}
.ix-event-row-body :is(button,a[href],input,select,textarea,label,summary,[role=button],[tabindex]){position:relative;pointer-events:auto}
.ix-event-row[data-disabled=true]>.ix-event-row-body{opacity:.4}
.ix-event-row>.ix-event-list-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}
.ix-event-list .ix-event-row>.ix-event-list-item{border-radius:inherit}
.ix-event-row[data-animated=true]{animation:ix-event-row-enter .15s ease-out}
@keyframes ix-event-row-enter{from{opacity:.6;transform:translateY(4px)}to{opacity:1;transform:translateY(0)}}
@media(prefers-reduced-motion:reduce){.ix-event-row[data-animated=true]{animation:none}}

.ix-card[data-passive=true]>.ix-card-body{pointer-events:auto}

.ix-content-header[data-variant=secondary] h1{font:var(--si-sys-typography-h4,20px/28px var(--ix-font));font-weight:600}
.ix-content-header-title-line>h1,.ix-content-header-titles>p{overflow-wrap:anywhere;max-width:100%}.ix-content-header[data-ellipsis=false] .ix-content-header-title-line{flex-wrap:wrap}
.ix-tile-header{gap:8px;flex-wrap:wrap;min-width:0}.ix-tile-header>span,.ix-tile-subheader,.ix-tile-content,.ix-tile-footer{min-width:0;overflow-wrap:anywhere}

.ix-group-entry{display:contents}.ix-group-header-row[data-item-selected]{box-shadow:inset 2px 0 0 var(--ix-color-primary)}
.ix-group-header-content{display:flex;align-items:center;gap:8px;min-width:0;overflow-wrap:anywhere}.ix-group-footer{padding:12px 16px;min-width:0;background:var(--ix-color-surface-1);overflow-wrap:anywhere}
.ix-group-header-row{flex-wrap:wrap}.ix-group-header>span,.ix-group-item>span{min-width:0;overflow-wrap:anywhere}.ix-group-secondary{flex-shrink:0}

.ix-key-value dl{flex:1}.ix-key-value dt,.ix-key-value dd{min-width:0;max-width:100%;overflow-wrap:anywhere}.ix-key-value[data-horizontal=true] dt{flex:0 1 100px;min-width:min(100px,100%)}.ix-key-value[data-horizontal=true] dd{flex:1}
.ix-key-value-list[data-horizontal=true]>.ix-key-value{flex-basis:min(240px,100%)}
.ix-key-value-custom{min-width:0;max-width:100%}.ix-key-value-custom[hidden]{display:none!important}.ix-key-value-custom :is(input,select,textarea,button){min-width:0;max-width:100%;box-sizing:border-box}

.ix-kpi:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-kpi{max-width:100%}.ix-kpi-label,.ix-kpi-measurement,.ix-kpi-measurement>strong,.ix-kpi-measurement>span{min-width:0;overflow-wrap:anywhere}.ix-kpi-measurement{max-width:100%;flex-wrap:wrap}
.ix-empty-state{min-width:0;box-sizing:border-box}.ix-empty-state-labels{overflow-wrap:anywhere}.ix-empty-state-action{max-width:100%;overflow-wrap:anywhere}.ix-empty-state-action:disabled{cursor:default;opacity:.55}.ix-empty-state-action:disabled:hover{background:transparent}.ix-empty-state-action[aria-busy=true]{gap:8px}

"#;

#[cfg(test)]
mod tests;
