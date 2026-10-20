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

pub(crate) const STYLES: &str = include_str!("styles/layout.css");

#[cfg(test)]
mod tests;
