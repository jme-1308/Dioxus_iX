//! Composite filtering, flippable tiles, information pages, and responsive layouts.
use crate::Icon;
use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_PATTERN_ID: AtomicUsize = AtomicUsize::new(1);

/// Cancelable reset request shared by all copies of the event.
/// Call [`Self::prevent_default`] synchronously to preserve the filter and draft.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilterClearEvent(Rc<Cell<bool>>);
impl FilterClearEvent {
    pub fn prevent_default(&self) {
        self.0.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.0.get()
    }
}
fn pattern_id(prefix: &str) -> String {
    format!(
        "ix-{prefix}-{}",
        NEXT_PATTERN_ID.fetch_add(1, Ordering::Relaxed)
    )
}

/// Comparison used in a category filter token.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogicalFilterOperator {
    #[default]
    Equal,
    NotEqual,
}
impl LogicalFilterOperator {
    /// Display notation used by the upstream category filter.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::NotEqual => "!=",
        }
    }
    fn toggled(self) -> Self {
        match self {
            Self::Equal => Self::NotEqual,
            Self::NotEqual => Self::Equal,
        }
    }
}
/// A categorized criterion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategoryToken {
    pub id: String,
    pub value: String,
    pub operator: LogicalFilterOperator,
}
/// Category filter value, preserving the upstream free-text/categorized split.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilterState {
    pub tokens: Vec<String>,
    pub categories: Vec<CategoryToken>,
}
impl FilterState {
    /// Add trimmed free text. Empty and duplicate tokens are rejected.
    pub fn add_text(&mut self, value: impl AsRef<str>) -> bool {
        let value = value.as_ref().trim();
        if value.is_empty() || self.tokens.iter().any(|token| token == value) {
            return false;
        }
        self.tokens.push(value.into());
        true
    }
    /// Add a categorized value. Duplicate id/value pairs are rejected regardless
    /// of operator; `unique` also limits each category to one value.
    pub fn add_category(
        &mut self,
        id: impl AsRef<str>,
        value: impl AsRef<str>,
        operator: LogicalFilterOperator,
        unique: bool,
    ) -> bool {
        let id = id.as_ref();
        let value = value.as_ref().trim();
        if id.is_empty()
            || value.is_empty()
            || self
                .categories
                .iter()
                .any(|token| token.id == id && (unique || token.value == value))
        {
            return false;
        }
        self.categories.push(CategoryToken {
            id: id.into(),
            value: value.into(),
            operator,
        });
        true
    }
    /// Whether the filter contains no committed criteria.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.categories.is_empty()
    }
    /// Remove the last chip (categorized chips follow free-text chips).
    pub fn remove_last(&mut self) -> bool {
        self.categories.pop().is_some() || self.tokens.pop().is_some()
    }
}
/// A programmatic CategoryFilter command needs exactly one native mount.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CategoryFilterError {
    NotMounted,
    MultipleMounted,
}
impl std::fmt::Display for CategoryFilterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::NotMounted => "CategoryFilter controller has no mounted filter",
            Self::MultipleMounted => "CategoryFilter controller has multiple mounted filters",
        })
    }
}
impl std::error::Error for CategoryFilterError {}

#[derive(Clone)]
struct CategoryFilterBinding {
    alive: Rc<Cell<bool>>,
    apply: Rc<dyn Fn(FilterState)>,
}

/// Mount-scoped source-like programmatic state commands. Clone a controller to
/// share its handle; attaching it to multiple native filters makes commands
/// ambiguous and returns [`CategoryFilterError::MultipleMounted`].
#[derive(Clone, Default)]
pub struct CategoryFilterController {
    bindings: Rc<RefCell<BTreeMap<usize, CategoryFilterBinding>>>,
}
impl PartialEq for CategoryFilterController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bindings, &other.bindings)
    }
}
impl CategoryFilterController {
    /// Clear committed criteria, process plain tokens before category tokens,
    /// then emit one `onchange`, including repeated equal or empty commands.
    /// This does not focus or open/close the filter. Controlled `state` remains
    /// the rendering authority. Nested commands normalize synchronously; their
    /// notifications wait until the current callback returns, in FIFO order.
    pub fn set_filter_state(&self, state: FilterState) -> Result<(), CategoryFilterError> {
        let binding = {
            let bindings = self.bindings.borrow();
            let mut mounted = bindings.values().filter(|binding| binding.alive.get());
            let Some(binding) = mounted.next() else {
                return Err(CategoryFilterError::NotMounted);
            };
            if mounted.next().is_some() {
                return Err(CategoryFilterError::MultipleMounted);
            }
            binding.clone()
        };
        (binding.apply)(state);
        Ok(())
    }
}

/// Create a stable CategoryFilter controller for this component.
pub fn use_category_filter() -> CategoryFilterController {
    use_hook(CategoryFilterController::default)
}

fn bind_category_filter(
    current: &Rc<RefCell<Option<CategoryFilterController>>>,
    requested: Option<&CategoryFilterController>,
    id: usize,
    binding: CategoryFilterBinding,
) {
    let previous = current.borrow().clone();
    if previous.as_ref() != requested {
        if let Some(previous) = previous {
            previous.bindings.borrow_mut().remove(&id);
        }
        *current.borrow_mut() = requested.cloned();
    }
    if let Some(requested) = requested {
        requested.bindings.borrow_mut().insert(id, binding);
    }
}

/// Category identity, visible label and suggested values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterCategory {
    pub id: String,
    pub label: String,
    pub options: Vec<String>,
}
impl FilterCategory {
    pub fn new(id: impl Into<String>, label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options,
        }
    }
}
/// The current uncommitted input and optional category.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilterInputState {
    pub token: String,
    pub category: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FilterDraft {
    input: FilterInputState,
    operator: LogicalFilterOperator,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum FilterChoice {
    Category(String, String),
    Value(String),
    Suggestion(String),
}
impl FilterChoice {
    fn label(&self) -> &str {
        match self {
            Self::Category(_, label) | Self::Value(label) | Self::Suggestion(label) => label,
        }
    }
}
const CUSTOM_FILTER_CATEGORY: &str = "CW_CUSTOM_FILTER_VALUE";

#[derive(Clone, Debug, PartialEq, Eq)]
struct FilterEntry {
    occurrence: usize,
    token: CategoryToken,
    // Direct split-state adapters retain their declared kind. Source-like
    // insertions derive it from the reserved custom-category ID instead.
    plain: bool,
}
fn filter_entry(token: CategoryToken, plain: bool) -> FilterEntry {
    FilterEntry {
        occurrence: NEXT_PATTERN_ID.fetch_add(1, Ordering::Relaxed),
        token,
        plain,
    }
}
fn project_filter_entries(entries: &[FilterEntry]) -> FilterState {
    let mut state = FilterState::default();
    for entry in entries {
        if entry.plain {
            state.tokens.push(entry.token.value.clone());
        } else {
            state.categories.push(entry.token.clone());
        }
    }
    state
}
fn canonical_filter_entries(state: &FilterState) -> Vec<FilterEntry> {
    state
        .tokens
        .iter()
        .map(|value| {
            filter_entry(
                CategoryToken {
                    id: CUSTOM_FILTER_CATEGORY.into(),
                    value: value.clone(),
                    operator: LogicalFilterOperator::Equal,
                },
                true,
            )
        })
        .chain(
            state
                .categories
                .iter()
                .cloned()
                .map(|token| filter_entry(token, false)),
        )
        .collect()
}
struct FilterOrderModel {
    authoritative: Vec<FilterEntry>,
    authoritative_state: FilterState,
    pending: Option<Vec<FilterEntry>>,
}
impl FilterOrderModel {
    fn new(state: &FilterState) -> Self {
        Self {
            authoritative: canonical_filter_entries(state),
            authoritative_state: state.clone(),
            pending: None,
        }
    }
    fn resolve(&mut self, state: &FilterState) -> Vec<FilterEntry> {
        if self
            .pending
            .as_ref()
            .is_some_and(|entries| project_filter_entries(entries) == *state)
        {
            self.authoritative = self.pending.take().unwrap();
            self.authoritative_state = state.clone();
        } else if self.authoritative_state != *state {
            self.authoritative = canonical_filter_entries(state);
            self.authoritative_state = state.clone();
        }
        self.authoritative.clone()
    }
    fn propose(&mut self, entries: Vec<FilterEntry>, controlled: bool) {
        if controlled {
            self.pending = Some(entries);
        } else {
            self.authoritative_state = project_filter_entries(&entries);
            self.authoritative = entries;
            self.pending = None;
        }
    }
}
fn has_filter_entry(entries: &[FilterEntry], draft: &FilterDraft, value: &str) -> bool {
    entries.iter().any(|entry| {
        entry.token.value == value
            && if let Some(category) = draft.input.category.as_deref().filter(|id| !id.is_empty()) {
                entry.token.id == category
            } else {
                entry.token.id.is_empty() || entry.token.id == CUSTOM_FILTER_CATEGORY
            }
    })
}
fn append_filter_entry(
    entries: &mut Vec<FilterEntry>,
    draft: &mut FilterDraft,
    value: &str,
    id: String,
    operator: LogicalFilterOperator,
) -> bool {
    let value = trim_programmatic_token(value);
    if value.is_empty() || has_filter_entry(entries, draft, value) {
        return false;
    }
    let plain = id == CUSTOM_FILTER_CATEGORY;
    entries.push(filter_entry(
        CategoryToken {
            id,
            value: value.into(),
            operator,
        },
        plain,
    ));
    *draft = FilterDraft::default();
    true
}

enum FilterNotification {
    Change(FilterState),
    Category(Option<String>),
    Clear,
}
struct FilterUpdate {
    entries: Vec<FilterEntry>,
    draft: Option<FilterDraft>,
    scroll: bool,
    preserve_scroll: bool,
    focus: bool,
    cleared_category: bool,
    cleared: bool,
}

fn trim_programmatic_token(value: &str) -> &str {
    // ECMAScript String.trim: notably includes BOM and excludes NEL.
    value.trim_matches(|character| {
        matches!(character,
            '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' |
            '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
            '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    })
}

/// The source setter checks duplicates against its changing preview context,
/// not the incoming token category, and does not enforce unique/static settings.
fn normalize_filter_state(
    state: FilterState,
    mut draft: FilterDraft,
) -> (Vec<FilterEntry>, FilterDraft, bool) {
    let mut accepted = Vec::new();
    for value in state.tokens {
        let operator = draft.operator;
        append_filter_entry(
            &mut accepted,
            &mut draft,
            &value,
            CUSTOM_FILTER_CATEGORY.into(),
            operator,
        );
    }
    for token in state.categories {
        append_filter_entry(
            &mut accepted,
            &mut draft,
            &token.value,
            token.id,
            token.operator,
        );
    }
    let dirty = !accepted.is_empty();
    (accepted, draft, dirty)
}

#[derive(Clone)]
struct FilterScrollRequest {
    revision: u64,
    expected: FilterState,
}

fn suggestion_duplicate(state: &FilterState, draft: &FilterDraft, value: &str) -> bool {
    if let Some(category) = draft.input.category.as_deref().filter(|id| !id.is_empty()) {
        (category == CUSTOM_FILTER_CATEGORY && state.tokens.iter().any(|token| token == value))
            || state
                .categories
                .iter()
                .any(|token| token.id == category && token.value == value)
    } else {
        state.tokens.iter().any(|token| token == value)
            || state.categories.iter().any(|token| {
                (token.id.is_empty() || token.id == CUSTOM_FILTER_CATEGORY) && token.value == value
            })
    }
}

fn filter_choices(
    categories: Option<&[FilterCategory]>,
    suggestions: &[String],
    state: &FilterState,
    draft: &FilterDraft,
    unique: bool,
) -> Vec<FilterChoice> {
    let query = draft.input.token.to_lowercase();
    let Some(categories) = categories else {
        return suggestions
            .iter()
            .filter(|value| {
                value.to_lowercase().contains(&query) && !suggestion_duplicate(state, draft, value)
            })
            .cloned()
            .map(FilterChoice::Suggestion)
            .collect();
    };
    if let Some(id) = &draft.input.category {
        return categories
            .iter()
            .find(|category| &category.id == id)
            .map(|category| {
                category
                    .options
                    .iter()
                    .filter(|value| {
                        value.to_lowercase().contains(&query)
                            && !suggestion_duplicate(state, draft, value)
                    })
                    .cloned()
                    .map(FilterChoice::Value)
                    .collect()
            })
            .unwrap_or_default();
    }
    categories
        .iter()
        .filter(|category| {
            category.label.to_lowercase().contains(&query)
                && (!unique
                    || !(state.categories.iter().any(|token| token.id == category.id)
                        || (category.id == CUSTOM_FILTER_CATEGORY && !state.tokens.is_empty())))
        })
        .map(|category| FilterChoice::Category(category.id.clone(), category.label.clone()))
        .collect()
}
fn focus_filter(input: Signal<Option<MountedEvent>>, generation: Signal<u64>) {
    let request = *generation.peek();
    let node = input.peek().clone();
    if let Some(node) = node {
        spawn(async move {
            if *generation.peek() != request {
                return;
            }
            // Native focus requests can otherwise overtake a reset event that
            // is already queued in the WebView transport. A native read gives
            // that event a delivery boundary before issuing the focus command.
            // Unsupported geometry retains the non-DOM/mock behavior.
            let _ = node.get_client_rect().await;
            if *generation.peek() == request {
                let _ = node.set_focus(true).await;
            }
        });
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum FilterFocusTarget {
    Element(String),
    PreviousControl,
    NextControl,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct FilterKeyboardFocus {
    generation: u64,
    target: FilterFocusTarget,
}
fn request_filter_keyboard_focus(
    target: FilterFocusTarget,
    mut pending: Signal<Option<FilterKeyboardFocus>>,
    mut generation: Signal<u64>,
) {
    let next = generation.peek().wrapping_add(1);
    generation.set(next);
    pending.set(Some(FilterKeyboardFocus {
        generation: next,
        target,
    }));
}
fn cancel_filter_keyboard_focus(
    mut pending: Signal<Option<FilterKeyboardFocus>>,
    mut generation: Signal<u64>,
) {
    let next = generation.peek().wrapping_add(1);
    generation.set(next);
    pending.set(None);
}

type FilterFocusMounts = Signal<BTreeMap<String, Rc<MountedData>>>;

/// Each rendered button owns its mounted registration. ID/epoch changes rebind
/// that same Dioxus element explicitly; an old request cannot follow the new ID.
#[component]
fn FilterFocusButton(
    id: String,
    mut mounts: FilterFocusMounts,
    #[props(default)] class: Option<String>,
    #[props(default)] title: Option<String>,
    #[props(default)] data_id: Option<String>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] aria_pressed: Option<String>,
    onkeydown: EventHandler<KeyboardEvent>,
    onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let registered = use_hook(|| Rc::new(RefCell::new(None::<(String, Rc<MountedData>)>)));
    let binding = registered.clone();
    use_effect(use_reactive!(|id| {
        let node = mounted();
        if node.is_none() && binding.borrow().is_none() {
            return;
        }
        let mut targets = mounts.write();
        if let Some((old_id, old_node)) = binding.borrow_mut().take() {
            if targets
                .get(&old_id)
                .is_some_and(|current| Rc::ptr_eq(current, &old_node))
            {
                targets.remove(&old_id);
            }
        }
        if let Some(node) = node {
            targets.insert(id.clone(), node.clone());
            *binding.borrow_mut() = Some((id, node));
        }
    }));
    use_drop(move || {
        if let Some((id, node)) = registered.borrow_mut().take() {
            if let Ok(mut targets) = mounts.try_write() {
                if targets
                    .get(&id)
                    .is_some_and(|current| Rc::ptr_eq(current, &node))
                {
                    targets.remove(&id);
                }
            }
        }
    });
    rsx! {button {id:id.clone(),r#type:"button",class,title,"data-id":data_id,"aria-label":aria_label,"aria-pressed":aria_pressed,
        onmounted:move |event|mounted.set(Some(event.data())),onkeydown,onclick,{children}
    }}
}

fn filter_popup_key(
    event: KeyboardEvent,
    position: usize,
    targets: &[String],
    mut open: Signal<bool>,
    pending: Signal<Option<FilterKeyboardFocus>>,
    generation: Signal<u64>,
) {
    match event.key() {
        Key::Escape => {
            event.stop_propagation();
            cancel_filter_keyboard_focus(pending, generation);
            open.set(false);
        }
        Key::Tab => {
            let target = if event.modifiers().shift() && position == 0 {
                targets.last()
            } else if !event.modifiers().shift() && position + 1 == targets.len() {
                targets.first()
            } else {
                None
            };
            if let Some(target) = target {
                event.prevent_default();
                event.stop_propagation();
                request_filter_keyboard_focus(
                    FilterFocusTarget::Element(target.clone()),
                    pending,
                    generation,
                );
            }
        }
        Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End => event.prevent_default(),
        // Native button Enter/Space activation reaches the existing onclick once.
        _ => {}
    }
}
#[derive(Clone)]
struct FilterInteraction {
    draft: Signal<FilterDraft>,
    active: Signal<Option<usize>>,
    input: Signal<Option<MountedEvent>>,
    focus_generation: Signal<u64>,
    order: Rc<RefCell<FilterOrderModel>>,
    apply: Rc<dyn Fn(FilterUpdate)>,
    notify: Rc<dyn Fn(Vec<FilterNotification>)>,
}
fn commit_filter_choice(choice: FilterChoice, interaction: &FilterInteraction, pointer: bool) {
    let mut draft = interaction.draft;
    let mut active = interaction.active;
    active.set(None);
    let mut next_draft = draft.peek().clone();
    let suggestion = matches!(choice, FilterChoice::Suggestion(_));
    match choice {
        FilterChoice::Category(id, _) => {
            next_draft.input.category = Some(id.clone());
            next_draft.input.token.clear();
            draft.set(next_draft);
            if pointer {
                focus_filter(interaction.input, interaction.focus_generation);
            }
            (interaction.notify)(vec![FilterNotification::Category(Some(id))]);
        }
        FilterChoice::Suggestion(value) | FilterChoice::Value(value) => {
            let id = if suggestion {
                CUSTOM_FILTER_CATEGORY.into()
            } else {
                next_draft
                    .input
                    .category
                    .clone()
                    .filter(|id| !id.is_empty())
                    .unwrap_or_else(|| CUSTOM_FILTER_CATEGORY.into())
            };
            let mut entries = interaction.order.borrow().authoritative.clone();
            let operator = next_draft.operator;
            if append_filter_entry(&mut entries, &mut next_draft, &value, id, operator) {
                (interaction.apply)(FilterUpdate {
                    entries,
                    draft: Some(next_draft),
                    scroll: true,
                    preserve_scroll: false,
                    focus: pointer,
                    cleared_category: false,
                    cleared: false,
                });
            } else if pointer {
                focus_filter(interaction.input, interaction.focus_generation);
            }
        }
    }
}
fn remove_filter_occurrence(occurrence: usize, interaction: &FilterInteraction) {
    let mut entries = interaction.order.borrow().authoritative.clone();
    let before = entries.len();
    entries.retain(|entry| entry.occurrence != occurrence);
    if entries.len() != before {
        (interaction.apply)(FilterUpdate {
            entries,
            draft: None,
            scroll: false,
            preserve_scroll: true,
            focus: false,
            cleared_category: false,
            cleared: false,
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FilterViewport {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct FilterPopupMetrics {
    anchor_x: f64,
    anchor_y: f64,
    anchor_width: f64,
    anchor_height: f64,
    content_height: f64,
    viewport: FilterViewport,
}
struct FilterPopupLayout {
    style: String,
    placement: &'static str,
}
fn place_filter_popup(metrics: Option<FilterPopupMetrics>, native: bool) -> FilterPopupLayout {
    if !native {
        return FilterPopupLayout {
            // Alpha.2 merges style declarations instead of deleting omitted
            // fields. Restore the original inline geometry on native failure.
            style:
                "visibility:visible;position:absolute;left:0;top:100%;width:auto;max-height:320px"
                    .into(),
            placement: "bottom",
        };
    }
    let Some(metrics) = metrics else {
        return FilterPopupLayout {
            style: "visibility:hidden".into(),
            placement: "bottom",
        };
    };
    let viewport = metrics.viewport;
    let width = metrics.anchor_width.min((viewport.width - 16.0).max(0.0));
    let desired = (metrics.content_height + 2.0).clamp(0.0, 320.0);
    let bottom = metrics.anchor_y + metrics.anchor_height;
    let below = (viewport.top + viewport.height - 8.0 - bottom - 2.0).max(0.0);
    let above = (metrics.anchor_y - viewport.top - 8.0 - 2.0).max(0.0);
    let flipped = below < desired && above > below;
    let room = (if flipped { above } else { below }).min(320.0);
    let actual = desired.min(room);
    let left = metrics
        .anchor_x
        .min(viewport.left + viewport.width - 8.0 - width)
        .max(viewport.left + 8.0);
    let top = if flipped {
        metrics.anchor_y - 2.0 - actual
    } else {
        bottom + 2.0
    };
    FilterPopupLayout {
        style: format!(
            "visibility:visible;position:fixed;left:{left}px;top:{top}px;width:{width}px;max-height:{room}px"
        ),
        placement: if flipped { "top" } else { "bottom" },
    }
}

#[component]
fn FilterOptionsSurface(
    anchor_id: String,
    anchor: Signal<Option<Rc<MountedData>>>,
    mut ready: Signal<bool>,
    #[props(default)] anchor_revision: u64,
    #[props(default)] content_revision: usize,
    enable_top_layer: bool,
    onnativeerror: EventHandler<crate::NativeSurfaceError>,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    let surface_id = use_signal(|| pattern_id("category-surface"))();
    let mut native = use_signal(|| enable_top_layer);
    let mut promoted = use_signal(|| !enable_top_layer);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut metrics = use_signal(|| None::<FilterPopupMetrics>);
    // The root document stream supplies viewport facts and ancestor-scroll
    // invalidations; each surface only measures its own Dioxus-mounted nodes.
    let viewport = crate::document_viewport::use_document_viewport(native());
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let generation = use_hook(|| Rc::new(Cell::new(0_u64)));
    let measure_alive = alive.clone();
    let measure_generation = generation.clone();
    let measure_viewport = viewport.clone();
    let measure = use_callback(move |()| {
        // A mode change also invalidates measurements already awaiting native
        // element results, including a failed top-layer promotion.
        measure_generation.set(measure_generation.get().wrapping_add(1));
        let revision = measure_generation.get();
        if !*native.peek() {
            return;
        }
        let anchor_signal = anchor;
        let (Some(panel), Some(anchor), Some(viewport)) = (
            mounted.peek().clone(),
            anchor_signal.peek().clone(),
            measure_viewport.facts(),
        ) else {
            return;
        };
        let alive = measure_alive.clone();
        let generation = measure_generation.clone();
        spawn(async move {
            let Ok(rect) = anchor.get_client_rect().await else {
                return;
            };
            if !alive.get() || generation.get() != revision {
                return;
            }
            let Ok(content) = panel.get_scroll_size().await else {
                return;
            };
            if !alive.get()
                || generation.get() != revision
                || rect.size.width <= 0.0
                || !anchor_signal
                    .peek()
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(current, &anchor))
            {
                return;
            }
            let next = FilterPopupMetrics {
                anchor_x: rect.origin.x,
                anchor_y: rect.origin.y,
                anchor_width: rect.size.width,
                anchor_height: rect.size.height,
                content_height: content.height,
                viewport: FilterViewport {
                    left: viewport.visual_left,
                    top: viewport.visual_top,
                    width: viewport.visual_width,
                    height: viewport.visual_height,
                },
            };
            if *metrics.peek() != Some(next) {
                metrics.set(Some(next));
            }
        });
    });
    // Effects run after the positioning style has been rendered. Promotion
    // alone is insufficient: visibility:hidden layout is not focusable yet.
    use_effect(move || {
        let positioned = !native() || (promoted() && metrics().is_some());
        ready.set(mounted().is_some() && positioned);
    });
    use_effect(use_reactive!(|anchor_revision, content_revision| {
        let _ = (anchor_revision, content_revision, anchor.read().clone());
        measure.call(());
    }));
    use_effect(move || {
        if native() {
            // Scroll can move the anchor without changing the viewport box.
            let _ = viewport.layout_revision();
        }
        measure.call(());
    });
    let outside_mount =
        crate::outside_click::use_outside_click(format!("{anchor_id}-root"), true, onclose);
    let lifetime = alive.clone();
    use_drop(move || {
        lifetime.set(false);
        if let Ok(mut ready) = ready.try_write() {
            *ready = false;
        }
    });
    let mounted_id = surface_id.clone();
    let mounted_alive = alive.clone();
    let toggle_alive = alive;
    let layout = place_filter_popup(metrics(), native());
    rsx! {
        div {id:surface_id,class:"ix-category-filter-options",style:layout.style,"data-placement":layout.placement,
            "popover":native().then_some("manual"),"data-top-layer":native().to_string(),
            onmounted:move |event|{
                ready.set(false);
                mounted.set(Some(event.data()));
                outside_mount.call(event);
                if !enable_top_layer{return;}
                let alive=mounted_alive.clone();let id=mounted_id.clone();
                spawn(async move {
                    let result=crate::native_surface::present(&id,true,None,None).await;
                    if alive.get(){if let Err(error)=result{native.set(false);onnativeerror.call(error);}else{promoted.set(true);}measure.call(());}
                });
            },
            onresize:move |_|measure.call(()),
            ontoggle:move |_|{
                let node=mounted.peek().clone();let alive=toggle_alive.clone();
                spawn(async move{if let Some(node)=node{if let Ok(bounds)=node.get_client_rect().await{if alive.get()&&*native.peek()&&bounds.size.width==0.0&&bounds.size.height==0.0{onclose.call(());}}}});
            },
            div {onresize:move |_|measure.call(()),{children}}
        }
    }
}

/// Native input and option buttons for categorized criteria and free text. `state`
/// enables controlled usage. Enter commits draft text; ArrowDown focuses an option;
/// Backspace on
/// an empty input exits a category or removes the last chip. Escape closes options.
/// `enable_top_layer` uses the native Popover API to escape clipping ancestors,
/// with input-anchored viewport placement. Engines without that API retain the
/// inline dropdown and report the limitation through `onnativeerror`.
/// Reset first calls `onbeforeclear`; veto preserves all filter state. Accepted
/// reset closes options, proposes empty committed criteria, and clears only an
/// active category preview. Draft text and comparison operator survive. It emits
/// `oncategory(None)` only for a cleared preview, then `onchange(empty)`, and calls
/// the accepted-reset adapter `onclear` last. Reset never requests input focus.
/// In controlled usage `state` remains authoritative when a parent rejects the
/// empty-state proposal.
/// Omitted `categories` selects plain suggestions; `Some(vec![])` selects an
/// empty category menu. Existing RSX `categories: vec![...]` calls remain valid;
/// direct `CategoryFilterProps` field construction now uses `Some(vec![...])`.
/// Category-mode transitions preserve stored preview identity and draft text.
/// Preview labels come only from current selectable categories; removed labels
/// stay blank until restored. The existing Rust heading coalesces the source's
/// selected-category inner header and operator into one container. Plain suggestions ignore a retained preview when
/// rendering and committing a suggestion, while typed Enter retains that category.
/// `filter_state` applies source-like programmatic normalization after native
/// mount and later Rust value changes, emitting one `onchange` even for empty or
/// equal state. None is unset. JavaScript object identity has no Rust equivalent;
/// use the controller for repeated equal commands. `default_state` stays silent,
/// while controlled `state` remains the rendered authority. The setter preserves
/// focus/dropdown visibility, and only accepted tokens clear draft text/category/
/// operator. No category/input/reset events accompany programmatic commands.
/// SSR does not attach controllers or apply `filter_state`. Controllers detach
/// on swap/unmount; nested notifications use a FIFO instead of JS recursion.
/// Chips retain chronological occurrence identity. Controlled values matching the
/// latest proposal acknowledge its order; rejected proposals preserve the prior
/// displayed order. Unrelated external split values reconstruct plain tokens then
/// categories because FilterState cannot encode interleaving. Equal projections
/// cannot distinguish different acknowledgement intentions; the latest matching
/// proposal wins. Only authoritative and latest pending orders are retained.
/// Direct split-state adapters retain declared token kind, including reserved IDs;
/// source-like setters/interactive insertion classify the reserved custom ID as
/// free text. Public FilterState mutation helpers remain independent adapters.
/// Interactive commits emit only `onchange`; preview Backspace is silent and keeps
/// its operator. Removal never requests focus or starts token-list scrolling;
/// an unconsumed append scroll instead follows the remaining displayed entries.
/// Focus alone does not open options. Popup buttons retain the pinned source's
/// canceled Enter/Space behavior, with only first/last Tab boundaries wrapped.
/// Home/End preserve the input's native caret behavior; Escape never forces focus
/// back to the input. Unmatched input arrows call `onfocusprevious`/`onfocusnext`;
/// consumers focus their own Dioxus mounted controls rather than implicit DOM
/// siblings. Local keyboard targets use current mounted options/operator IDs.
/// Category pointer focus remains asynchronous; stale work is guarded by mount
/// identity, generation and option IDs. Rust propagation cancellation is
/// preserved, but Dioxus alpha.2 does not mirror those stops to external native
/// DOM listeners; native default cancellation is mirrored synchronously.
/// The optional `aria_label_*` props override native control labels live, including
/// explicit empty strings; omitted values use the compatible Rust label defaults.
#[component]
pub fn CategoryFilter(
    /// Omitted categories enables plain suggestions; an explicitly empty vector
    /// keeps category mode with no choices. Mode changes preserve filter state.
    #[props(default)]
    categories: Option<Vec<FilterCategory>>,
    #[props(default)] suggestions: Vec<String>,
    #[props(default)] non_selectable_categories: BTreeMap<String, String>,
    #[props(default)] state: Option<FilterState>,
    /// Source-like setter applied after native mount and later value changes.
    /// None is unset; repeated equal writes use the controller instead.
    #[props(default)]
    filter_state: Option<FilterState>,
    #[props(default)] controller: Option<CategoryFilterController>,
    #[props(default)] default_state: FilterState,
    #[props(default)] onchange: EventHandler<FilterState>,
    #[props(default)] oninput: EventHandler<FilterInputState>,
    #[props(default)] oncategory: EventHandler<Option<String>>,
    #[props(default)] onclear: EventHandler<()>,
    /// Unmatched backward navigation requests focus through consumer-owned
    /// Dioxus handles. No arbitrary DOM sibling lookup is performed.
    #[props(default)]
    onfocusprevious: EventHandler<()>,
    /// Unmatched forward navigation requests the next consumer-owned control.
    #[props(default)]
    onfocusnext: EventHandler<()>,
    /// Runs before clearing; cancellation preserves committed and draft values.
    #[props(default)]
    onbeforeclear: EventHandler<FilterClearEvent>,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] unique_categories: bool,
    #[props(default)] static_operator: Option<LogicalFilterOperator>,
    #[props(default="Filter by".to_string())] placeholder: String,
    /// Compatible fallback for the nested input's accessible label.
    #[props(default="Filter".to_string())]
    label: String,
    /// Overrides `label`, including when explicitly empty.
    #[props(default)]
    aria_label_filter_input: Option<String>,
    /// Nested reset-button label; defaults to "Clear filters" when omitted.
    #[props(default)]
    aria_label_reset_button: Option<String>,
    /// Nested comparison-button label; defaults to "Toggle comparison operator".
    #[props(default)]
    aria_label_operator_button: Option<String>,
    #[props(default="Categories".to_string())] label_categories: String,
    #[props(default="Filter by text".to_string())] label_plain_text: String,
    #[props(default="search".to_string())] icon: String,
    #[props(default)] hide_icon: bool,
    /// Match upstream's opt-in `enableTopLayer` setting.
    #[props(default)]
    enable_top_layer: bool,
    #[props(default)] onnativeerror: EventHandler<crate::NativeSurfaceError>,
) -> Element {
    let local = use_signal(|| default_state);
    let controlled = state.is_some();
    let snapshot = state.unwrap_or(local());
    let order = use_hook(|| Rc::new(RefCell::new(FilterOrderModel::new(&snapshot))));
    let entries = order.borrow_mut().resolve(&snapshot);
    let mut draft = use_signal(FilterDraft::default);
    let mut open = use_signal(|| false);
    let mut active = use_signal(|| None::<usize>);
    let mut input = use_signal(|| None::<MountedEvent>);
    let mut anchor_node = use_signal(|| None::<Rc<MountedData>>);
    let mut anchor_revision = use_signal(|| 0_u64);
    let mut token_list = use_signal(|| None::<Rc<MountedData>>);
    let pending_keyboard_focus = use_signal(|| None::<FilterKeyboardFocus>);
    let focus_generation = use_signal(|| 0u64);
    let surface_ready = use_signal(|| false);
    let focus_mounts = use_signal(BTreeMap::<String, Rc<MountedData>>::new);
    #[cfg(test)]
    if let Some(capture) = try_use_context::<tests::FilterKeyboardCapture>() {
        *capture.0.borrow_mut() = Some((pending_keyboard_focus, focus_generation, surface_ready));
    }
    let mut native_mounted = use_signal(|| false);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let binding_id = use_hook(|| NEXT_PATTERN_ID.fetch_add(1, Ordering::Relaxed));
    let bound_controller = use_hook(|| Rc::new(RefCell::new(None::<CategoryFilterController>)));
    let notifications = use_hook(|| Rc::new(RefCell::new(VecDeque::<FilterNotification>::new())));
    let notifying = use_hook(|| Rc::new(Cell::new(false)));
    let scroll_revision = use_signal(|| 0_u64);
    let pending_scroll = use_hook(|| Rc::new(RefCell::new(None::<FilterScrollRequest>)));
    #[cfg(test)]
    if let Some(capture) = try_use_context::<tests::FilterPendingCapture>() {
        *capture.0.borrow_mut() = Some(pending_scroll.clone());
    }
    let notify: Rc<dyn Fn(Vec<FilterNotification>)> = Rc::new(move |events| {
        notifications.borrow_mut().extend(events);
        if notifying.replace(true) {
            return;
        }
        loop {
            let next = notifications.borrow_mut().pop_front();
            let Some(next) = next else {
                break;
            };
            match next {
                FilterNotification::Change(state) => onchange.call(state),
                FilterNotification::Category(category) => oncategory.call(category),
                FilterNotification::Clear => onclear.call(()),
            }
        }
        notifying.set(false);
    });
    let apply_alive = alive.clone();
    let apply_scroll = pending_scroll.clone();
    let apply_order = order.clone();
    let apply_notify = notify.clone();
    let apply: Rc<dyn Fn(FilterUpdate)> = Rc::new(move |update| {
        if !apply_alive.get() {
            return;
        }
        let next = project_filter_entries(&update.entries);
        apply_order.borrow_mut().propose(update.entries, controlled);
        let mut local = local;
        let mut draft = draft;
        let mut revision = scroll_revision;
        let mut focus_generation = focus_generation;
        let next_focus = focus_generation.peek().wrapping_add(1);
        focus_generation.set(next_focus);
        let next_revision = revision.peek().wrapping_add(1);
        let dirty = update.scroll || (update.preserve_scroll && apply_scroll.borrow().is_some());
        *apply_scroll.borrow_mut() = dirty.then_some(FilterScrollRequest {
            revision: next_revision,
            expected: next.clone(),
        });
        revision.set(next_revision);
        local.set(next.clone());
        if let Some(next_draft) = update.draft {
            draft.set(next_draft);
        }
        if update.focus {
            focus_filter(input, focus_generation);
        }
        let mut events = Vec::new();
        if update.cleared_category {
            events.push(FilterNotification::Category(None));
        }
        events.push(FilterNotification::Change(next));
        if update.cleared {
            events.push(FilterNotification::Clear);
        }
        apply_notify(events);
    });
    let setter_apply = apply.clone();
    let setter: Rc<dyn Fn(FilterState)> = Rc::new(move |state| {
        let (entries, next_draft, dirty) = normalize_filter_state(state, draft.peek().clone());
        setter_apply(FilterUpdate {
            entries,
            draft: Some(next_draft),
            scroll: dirty,
            preserve_scroll: false,
            focus: false,
            cleared_category: false,
            cleared: false,
        });
    });
    let binding = CategoryFilterBinding {
        alive: alive.clone(),
        apply: setter.clone(),
    };
    if *native_mounted.peek() {
        bind_category_filter(
            &bound_controller,
            controller.as_ref(),
            binding_id,
            binding.clone(),
        );
    }
    let prop_setter = setter.clone();
    let observed_prop = use_hook(|| Rc::new(RefCell::new(None::<Option<FilterState>>)));
    use_effect(use_reactive((&filter_state,), move |(state,)| {
        if !native_mounted() {
            return;
        }
        // User callbacks can read Signals while Dioxus's effect context is
        // active. Such incidental dependencies must not reapply an equal prop.
        // Record None edges and store before callbacks, including reentrant ones.
        if observed_prop.borrow().as_ref() == Some(&state) {
            return;
        }
        *observed_prop.borrow_mut() = Some(state.clone());
        if let Some(state) = state {
            prop_setter(state);
        }
    }));
    let drop_alive = alive.clone();
    let drop_controller = bound_controller.clone();
    use_drop(move || {
        drop_alive.set(false);
        if let Some(controller) = drop_controller.borrow_mut().take() {
            controller.bindings.borrow_mut().remove(&binding_id);
        }
    });
    let interaction = FilterInteraction {
        draft,
        active,
        input,
        focus_generation,
        order: order.clone(),
        apply: apply.clone(),
        notify,
    };
    let keyboard_interaction = interaction.clone();
    let reset_apply = apply.clone();
    let list_id = use_signal(|| pattern_id("category-options"))();
    let anchor_id = use_signal(|| pattern_id("category-anchor"))();
    let root_id = format!("{anchor_id}-root");
    let token_list_id = format!("{anchor_id}-tokens");
    let scroll_pending = pending_scroll.clone();
    let scroll_alive = alive.clone();
    let displayed_state = use_hook(|| Rc::new(RefCell::new(snapshot.clone())));
    *displayed_state.borrow_mut() = snapshot.clone();
    use_effect(use_reactive((&snapshot,), move |(displayed,)| {
        let revision = scroll_revision();
        let Some(request) = scroll_pending.borrow().clone() else {
            return;
        };
        if request.revision != revision {
            scroll_pending.borrow_mut().take();
            return;
        }
        // Acknowledgment means equal values, not merely an equal chip count.
        if request.expected != displayed {
            return;
        }
        let Some(list) = token_list.read().clone() else {
            return;
        };
        scroll_pending.borrow_mut().take();
        let alive = scroll_alive.clone();
        let pending = scroll_pending.clone();
        let current = displayed_state.clone();
        spawn(async move {
            let expected = request.expected.clone();
            let valid = || {
                alive.get()
                    && *scroll_revision.peek() == revision
                    && *current.borrow() == expected
                    && token_list
                        .peek()
                        .as_ref()
                        .is_some_and(|current| Rc::ptr_eq(current, &list))
            };
            let result = async {
                if !valid() {
                    return None;
                }
                let size = list.get_scroll_size().await.ok()?;
                if !valid() {
                    return None;
                }
                let offset = list.get_scroll_offset().await.ok()?;
                if !valid() {
                    return None;
                }
                list.scroll(
                    dioxus::html::geometry::PixelsVector2D::new(offset.x, size.height),
                    dioxus::html::ScrollBehavior::Instant,
                )
                .await
                .ok()
            }
            .await;
            if result.is_none()
                && alive.get()
                && *scroll_revision.peek() == revision
                && pending.borrow().is_none()
            {
                *pending.borrow_mut() = Some(request);
            }
        });
    }));
    let choices = filter_choices(
        categories.as_deref(),
        &suggestions,
        &snapshot,
        &draft(),
        unique_categories,
    );
    let category_mode = categories.is_some();
    let preview_id = draft().input.category;
    let has_category = preview_id.is_some();
    let selected_category = preview_id.as_ref().and_then(|id| {
        categories
            .as_deref()
            .unwrap_or_default()
            .iter()
            .find(|category| &category.id == id)
            .map(|category| category.label.clone())
    });
    let input_label = aria_label_filter_input.unwrap_or(label);
    let reset_label = aria_label_reset_button.unwrap_or_else(|| "Clear filters".into());
    let operator_label =
        aria_label_operator_button.unwrap_or_else(|| "Toggle comparison operator".into());
    let current_operator = static_operator.unwrap_or(draft().operator);
    let blocked = disabled || readonly;
    let option_signature = (
        choices.clone(),
        category_mode,
        preview_id,
        static_operator,
        blocked,
    );
    let option_identity = use_hook(|| Rc::new(RefCell::new((option_signature.clone(), 0_usize))));
    let option_epoch = {
        let mut identity = option_identity.borrow_mut();
        if identity.0 != option_signature {
            identity.0 = option_signature;
            identity.1 = identity.1.wrapping_add(1);
        }
        identity.1
    };
    let option_ids: Vec<String> = (0..choices.len())
        .map(|index| format!("{list_id}-{option_epoch}-{index}"))
        .collect();
    let operator_id = format!("{list_id}-{option_epoch}-operator");
    let operator_visible = category_mode && has_category && static_operator.is_none();
    let mut popup_targets = Vec::new();
    if operator_visible {
        popup_targets.push(operator_id.clone());
    }
    popup_targets.extend(option_ids.clone());
    let input_first = option_ids.first().cloned();
    let operator_targets = popup_targets.clone();
    let keyboard_alive = alive.clone();
    let current_focus_options =
        use_hook(|| Rc::new(RefCell::new((blocked, popup_targets.clone()))));
    *current_focus_options.borrow_mut() = (blocked, popup_targets.clone());
    use_effect(use_reactive(
        (&blocked, &popup_targets),
        move |(blocked, targets)| {
            let request = pending_keyboard_focus();
            let generation = focus_generation();
            let is_open = open();
            let Some(request) = request else {
                return;
            };
            let mut pending = pending_keyboard_focus;
            if blocked
                || request.generation != generation
                || (matches!(request.target, FilterFocusTarget::Element(_)) && !is_open)
            {
                pending.set(None);
                return;
            }
            match request.target {
                FilterFocusTarget::PreviousControl => {
                    pending.set(None);
                    onfocusprevious.call(());
                }
                FilterFocusTarget::NextControl => {
                    pending.set(None);
                    onfocusnext.call(());
                }
                FilterFocusTarget::Element(id) => {
                    if !targets.contains(&id) {
                        pending.set(None);
                        return;
                    }
                    if enable_top_layer && !surface_ready() {
                        return;
                    }
                    let Some(node) = focus_mounts.read().get(&id).cloned() else {
                        return;
                    };
                    pending.set(None);
                    let lifetime = keyboard_alive.clone();
                    let current = current_focus_options.clone();
                    spawn(async move {
                        if !lifetime.get()
                            || *focus_generation.peek() != generation
                            || !*open.peek()
                            || current.borrow().0
                            || !current.borrow().1.contains(&id)
                            || !focus_mounts
                                .peek()
                                .get(&id)
                                .is_some_and(|mounted| Rc::ptr_eq(mounted, &node))
                        {
                            return;
                        }
                        // One Dioxus mounted attempt; the desktop void/bool response
                        // defect does not justify another focus command or fallback.
                        let _ = node.set_focus(true).await;
                    });
                }
            }
        },
    ));

    rsx! {
        div {id:root_id.clone(),class:"ix-category-filter","data-disabled":disabled.to_string(),"data-readonly":readonly.to_string(),"data-open":(open()&&!blocked).to_string(),
            div {id:anchor_id.clone(),class:"ix-category-filter-input",onmounted:move |event|anchor_node.set(Some(event.data())),onresize:move |_|anchor_revision.with_mut(|revision|*revision=revision.wrapping_add(1)),
                if !hide_icon {Icon {name:icon,size:16}}
                div {id:token_list_id,class:"ix-category-filter-tokens",onmounted:move |event|token_list.set(Some(event.data())),"data-scroll-revision":scroll_revision().to_string(),
                    for entry in &entries {
                        span {key:"{entry.occurrence}",class:"ix-category-filter-chip","data-filter-occurrence":entry.occurrence.to_string(),
                            onclick:move |event|event.stop_propagation(),
                            span {if entry.plain {"{entry.token.value}"} else {
                                "{categories.as_deref().unwrap_or_default().iter().find(|category|category.id==entry.token.id).map(|category|category.label.as_str()).or_else(||non_selectable_categories.get(&entry.token.id).map(String::as_str)).unwrap_or(&entry.token.id)} {entry.token.operator.symbol()} {entry.token.value}"
                            }}
                            if !blocked {button {r#type:"button","aria-label":"Remove {entry.token.value}",
                                onclick:{let interaction=interaction.clone();let occurrence=entry.occurrence;move |event|{
                                    event.prevent_default();event.stop_propagation();if !blocked{remove_filter_occurrence(occurrence,&interaction);}
                                }},
                                onkeydown:{let interaction=interaction.clone();let occurrence=entry.occurrence;move |event:KeyboardEvent|{
                                    match event.key() {
                                        Key::Tab=>{},
                                        Key::Enter=>{event.prevent_default();event.stop_propagation();if !blocked{remove_filter_occurrence(occurrence,&interaction);}},
                                        Key::Character(text) if text==" "=>{event.prevent_default();event.stop_propagation();if !blocked{remove_filter_occurrence(occurrence,&interaction);}},
                                        _=>{event.prevent_default();event.stop_propagation();}
                                    }
                                }},Icon {name:"close",size:12}
                            }}
                        }
                    }
                    if category_mode&&has_category {span {class:"ix-category-filter-preview","{selected_category.clone().unwrap_or_default()}"}}
                    input {r#type:"text",class:"ix-category-filter-text",disabled,readonly,value:draft().input.token,placeholder:if blocked||has_category{String::new()}else{placeholder},"aria-label":input_label,
                        onmounted:move |event|{
                            input.set(Some(event));
                            if !*native_mounted.peek() { native_mounted.set(true); }
                            bind_category_filter(&bound_controller, controller.as_ref(), binding_id, binding.clone());
                        },
                        onclick:move |_|if !blocked{open.set(true);},
                        oninput:move |event|{if blocked{return;}let mut next=draft();next.input.token=event.value();let input_state=next.input.clone();draft.set(next);oninput.call(input_state);active.set(None);open.set(true);},
                        onkeydown:move |event:KeyboardEvent|{
                            if blocked{return;}
                            match event.key(){
                                Key::Home|Key::End=>event.stop_propagation(),
                                Key::ArrowDown=>{
                                    if !open(){event.prevent_default();event.stop_propagation();open.set(true);if let Some(id)=input_first.clone(){request_filter_keyboard_focus(FilterFocusTarget::Element(id),pending_keyboard_focus,focus_generation);}}
                                    else if category_mode {if let Some(id)=input_first.clone(){event.stop_propagation();request_filter_keyboard_focus(FilterFocusTarget::Element(id),pending_keyboard_focus,focus_generation);}else{event.prevent_default();request_filter_keyboard_focus(FilterFocusTarget::NextControl,pending_keyboard_focus,focus_generation);}}
                                    else{event.prevent_default();request_filter_keyboard_focus(FilterFocusTarget::NextControl,pending_keyboard_focus,focus_generation);}
                                },
                                Key::ArrowUp=>{event.prevent_default();request_filter_keyboard_focus(FilterFocusTarget::PreviousControl,pending_keyboard_focus,focus_generation);},
                                Key::Escape=>{cancel_filter_keyboard_focus(pending_keyboard_focus,focus_generation);open.set(false);active.set(None);},
                                Key::Character(value) if value==" "=>{event.prevent_default();if !open(){open.set(true);if let Some(id)=input_first.clone(){request_filter_keyboard_focus(FilterFocusTarget::Element(id),pending_keyboard_focus,focus_generation);}}},
                                Key::Enter=>{event.prevent_default();commit_filter_choice(FilterChoice::Value(draft().input.token),&keyboard_interaction,false);},
                                Key::Backspace if draft().input.token.is_empty()=>{if draft().input.category.is_some(){let mut next=draft();next.input.category=None;draft.set(next);}else{let last=keyboard_interaction.order.borrow().authoritative.last().map(|entry|entry.occurrence);if let Some(last)=last{remove_filter_occurrence(last,&keyboard_interaction);}}},
                                _=>{}
                            }
                        }
                    }
                }
                if !blocked&&(!snapshot.is_empty()||has_category){button {class:"ix-category-filter-reset",r#type:"button",disabled:blocked,"aria-label":reset_label,
                    onclick:move |event|{
                        if blocked { return; }
                        let request=FilterClearEvent::default();
                        onbeforeclear.call(request.clone());
                        if request.default_prevented(){return;}
                        event.stop_propagation();
                        open.set(false);
                        active.set(None);
                        let mut next_draft=draft();
                        let cleared_category=next_draft.input.category.take().is_some();
                        reset_apply(FilterUpdate {entries:Vec::new(),draft:Some(next_draft),scroll:false,preserve_scroll:false,focus:false,cleared_category,cleared:true});
                    },Icon {name:"clear",size:24}}}
            }
            if open()&&!blocked {
                FilterOptionsSurface { anchor_id:anchor_id.clone(),anchor:anchor_node,ready:surface_ready,anchor_revision:anchor_revision(),content_revision:option_epoch,enable_top_layer, onnativeerror, onclose:move |_|{cancel_filter_keyboard_focus(pending_keyboard_focus,focus_generation);open.set(false);active.set(None);},
                    div {class:"ix-category-filter-options-heading",
                        span {if !category_mode {"{label_plain_text}"}else if has_category {"{selected_category.clone().unwrap_or_default()}"}else{"{label_categories}"}}
                        if operator_visible {FilterFocusButton {id:operator_id,mounts:focus_mounts,aria_label:operator_label,aria_pressed:(current_operator==LogicalFilterOperator::NotEqual).to_string(),
                            onkeydown:move |event|filter_popup_key(event,0,&operator_targets,open,pending_keyboard_focus,focus_generation),
                            onclick:move |event:MouseEvent|{event.stop_propagation();if blocked{return;}let mut next=draft();next.operator=next.operator.toggled();draft.set(next);},"{current_operator.toggled().symbol()}"}}
                    }
                    div {id:list_id.clone(),
                        for (index,choice) in choices.iter().enumerate(){
                            FilterFocusButton {id:option_ids[index].clone(),mounts:focus_mounts,class:"ix-category-filter-option",title:choice.label(),data_id:match choice{FilterChoice::Category(id,_)=>id.as_str(),FilterChoice::Value(value)|FilterChoice::Suggestion(value)=>value.as_str()},
                                onkeydown:{let targets=popup_targets.clone();move |event|filter_popup_key(event,index+usize::from(operator_visible),&targets,open,pending_keyboard_focus,focus_generation)},
                                onclick:{let choice=choice.clone();let interaction=interaction.clone();move |event:MouseEvent|if !blocked{if !matches!(choice,FilterChoice::Suggestion(_)){event.prevent_default();}commit_filter_choice(choice.clone(),&interaction,true)}},
                                if matches!(choice,FilterChoice::Value(_))&&draft().input.category.is_some(){span {"{current_operator.symbol()} "}}
                                "{choice.label()}"
                            }
                        }
                        if choices.is_empty(){div {class:"ix-category-filter-no-options","No suggestions. Press Enter to add your text."}}
                    }
                }
            }
        }
    }
}

pub use crate::flip_tile::{
    FlipTile, FlipTileChangeEvent, FlipTileContent, FlipTileContentProps, FlipTileDimension,
    FlipTileItem, FlipTileProps, FlipTileVariant,
};

/// Full-page information/error explanation with optional illustration and actions.
#[component]
pub fn InfoPage(
    title_text: String,
    #[props(default)] copy_text: Option<String>,
    #[props(default)] instructions: Option<String>,
    #[props(default="warning".to_string())] icon: String,
    #[props(default="var(--ix-color-warning)".to_string())] icon_color: String,
    #[props(default)] image: Option<Element>,
    #[props(default)] actions: Option<Element>,
) -> Element {
    rsx! {section{class:"ix-info-page",div{class:"ix-info-page-content",
        div{class:"ix-info-page-image",if let Some(image)=image{{image}}else{span{style:format!("color:{icon_color}"),Icon{name:icon,size:64}}}}
        div{class:"ix-info-page-message",h1{"{title_text}"}if let Some(copy)=copy_text{p{class:"ix-info-page-copy","{copy}"}}if let Some(instructions)=instructions{p{class:"ix-info-page-instructions","{instructions}"}}}
        if let Some(actions)=actions{div{class:"ix-info-page-actions",{actions}}}
    }}}
}

/// Validation failure for a CSS grid area matrix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GridTemplateError {
    Empty,
    RaggedRows,
    InvalidArea(String),
    NonRectangularArea(String),
}
impl std::fmt::Display for GridTemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "a grid template needs at least one nonempty row"),
            Self::RaggedRows => write!(f, "grid rows must have the same width"),
            Self::InvalidArea(name) => write!(f, "invalid CSS grid area name: {name}"),
            Self::NonRectangularArea(name) => write!(f, "grid area {name} must form a rectangle"),
        }
    }
}
impl std::error::Error for GridTemplateError {}
/// Validated, rectangular CSS grid area matrix. `.` denotes an unassigned cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridTemplate {
    rows: Vec<Vec<String>>,
}
impl GridTemplate {
    /// Validate dimensions, identifiers and rectangular repeated areas.
    pub fn new(rows: Vec<Vec<String>>) -> Result<Self, GridTemplateError> {
        let Some(first) = rows.first() else {
            return Err(GridTemplateError::Empty);
        };
        let width = first.len();
        if width == 0 {
            return Err(GridTemplateError::Empty);
        }
        if rows.iter().any(|row| row.len() != width) {
            return Err(GridTemplateError::RaggedRows);
        }
        let mut areas = BTreeMap::<String, (usize, usize, usize, usize, usize)>::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, name) in row.iter().enumerate() {
                if name == "." {
                    continue;
                }
                if !valid_grid_area(name) {
                    return Err(GridTemplateError::InvalidArea(name.clone()));
                }
                let area = areas.entry(name.clone()).or_insert((x, y, x, y, 0));
                area.0 = area.0.min(x);
                area.1 = area.1.min(y);
                area.2 = area.2.max(x);
                area.3 = area.3.max(y);
                area.4 += 1;
            }
        }
        for (name, (left, top, right, bottom, count)) in areas {
            if (right - left + 1) * (bottom - top + 1) != count {
                return Err(GridTemplateError::NonRectangularArea(name));
            }
        }
        Ok(Self { rows })
    }
    /// Convenience constructor from borrowed row slices.
    pub fn from_rows(rows: &[&[&str]]) -> Result<Self, GridTemplateError> {
        Self::new(
            rows.iter()
                .map(|row| row.iter().map(|name| name.to_string()).collect())
                .collect(),
        )
    }
    /// Matrix serialized for `grid-template-areas`.
    pub fn css(&self) -> String {
        self.rows
            .iter()
            .map(|row| format!("\"{}\"", row.join(" ")))
            .collect::<Vec<_>>()
            .join(" ")
    }
    pub fn columns(&self) -> usize {
        self.rows[0].len()
    }
}
fn valid_grid_area(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(),Some(first)if first.is_ascii_alphabetic()||first=='_')
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}
/// Upstream responsive area breakpoints: small <=576px, medium <=768px,
/// and large above 768px. Missing smaller templates fall back to larger ones.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridTemplates {
    pub sm: Option<GridTemplate>,
    pub md: Option<GridTemplate>,
    pub lg: Option<GridTemplate>,
}
impl GridTemplates {
    fn css(&self, scope: &str) -> String {
        let rule = |template: Option<&GridTemplate>| {
            template
                .map(|template| {
                    format!(
                        "grid-template-areas:{};grid-template-columns:repeat({},minmax(0,1fr));",
                        template.css(),
                        template.columns()
                    )
                })
                .unwrap_or_default()
        };
        let large = rule(self.lg.as_ref());
        let medium = rule(self.md.as_ref().or(self.lg.as_ref()));
        let small = rule(self.sm.as_ref().or(self.md.as_ref()).or(self.lg.as_ref()));
        format!(
            "[data-ix-grid-scope=\"{scope}\"]{{{large}}}@media(max-width:768px){{[data-ix-grid-scope=\"{scope}\"]{{{medium}}}}}@media(max-width:576px){{[data-ix-grid-scope=\"{scope}\"]{{{small}}}}}"
        )
    }
}
/// Responsive named-area grid. Use [`GridArea`] for area assignment.
///
/// `id` and `style` apply to the grid div; `onmounted` receives that div's
/// Dioxus handle. Responsive rules retain a private scope when the ID changes.
#[component]
pub fn CssGrid(
    children: Element,
    #[props(default)] templates: GridTemplates,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] style: String,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
) -> Element {
    let scope = use_hook(|| pattern_id("css-grid"));
    let id = id.unwrap_or_else(|| scope.clone());
    let css = templates.css(&scope);
    rsx! {style{{css}}div{id,class:"ix-css-grid {class}",style,"data-ix-grid-scope":scope,onmounted,{children}}}
}
/// Item assigned to an area in a [`CssGrid`]. Invalid names fall back to auto placement.
/// `id`, `class` and `onmounted` refer to the area div containing its children.
#[component]
pub fn GridArea(
    children: Element,
    name: String,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
) -> Element {
    rsx! {div{id,class:"ix-grid-area {class}",style:if valid_grid_area(&name){Some(format!("grid-area:{name}"))}else{None},onmounted,{children}}}
}

/// Viewport breakpoint and number of columns in [`LayoutAuto`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoLayoutBreakpoint {
    pub min_width: String,
    pub columns: usize,
}
impl AutoLayoutBreakpoint {
    pub fn new(min_width: impl Into<String>, columns: usize) -> Self {
        Self {
            min_width: min_width.into(),
            columns: columns.max(1),
        }
    }
}
fn valid_media_width(value: &str) -> bool {
    if value == "0" {
        return true;
    }
    ["px", "em", "rem"].iter().any(|unit| {
        value
            .strip_suffix(unit)
            .and_then(|number| number.parse::<f64>().ok())
            .is_some_and(|number| number.is_finite() && number >= 0.0)
    })
}
fn auto_layout_css(id: &str, layout: &[AutoLayoutBreakpoint]) -> String {
    let mut css = format!("#{id}{{--ix-layout-columns:1;}}");
    for breakpoint in layout
        .iter()
        .filter(|breakpoint| valid_media_width(&breakpoint.min_width))
    {
        let columns = breakpoint.columns.clamp(1, 64);
        css.push_str(&format!(
            "@media(min-width:{}){{#{id}{{--ix-layout-columns:{columns};}}}}",
            breakpoint.min_width
        ));
    }
    css
}
/// Form layout using viewport breakpoints and clamped child column spans.
/// Wrap children in [`LayoutAutoItem`] to declare a span; unwrapped children span one.
#[component]
pub fn LayoutAuto(
    children: Element,
    #[props(default=vec![AutoLayoutBreakpoint::new("0",1),AutoLayoutBreakpoint::new("48em",2)])]
    layout: Vec<AutoLayoutBreakpoint>,
    #[props(default)] class: String,
) -> Element {
    let id = use_signal(|| pattern_id("layout-auto"))();
    let css = auto_layout_css(&id, &layout);
    rsx! {style{{css}}div{id,class:"ix-layout-auto {class}",{children}}}
}
/// A form-layout child whose requested span is clamped to the current column count.
#[component]
pub fn LayoutAutoItem(children: Element, #[props(default = 1)] colspan: usize) -> Element {
    rsx! {div{class:"ix-layout-auto-item","data-colspan":colspan.max(1),style:format!("--ix-colspan:{}",colspan.max(1)),{children}}}
}

pub(crate) const STYLES: &str = include_str!("styles/patterns.css");

#[cfg(test)]
mod tests;
