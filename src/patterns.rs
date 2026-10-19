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
        Key::Enter | Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End => {
            event.prevent_default()
        }
        Key::Character(value) if value == " " => event.prevent_default(),
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

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
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

// Pinned MountedData has no showPopover/hidePopover method; DesktopElement's
// query handle is private. This primary adapter only promotes the exact unique
// Dioxus-mounted panel, inert during promotion to preserve the input's focus.
#[cfg(any(feature = "web", feature = "desktop"))]
const FILTER_TOP_LAYER_OPEN: &str = r#"
const id=await dioxus.recv(),panel=document.getElementById(id);
if(!panel?.isConnected)return {error:'Surface is not mounted'};
if(typeof panel.showPopover!=='function')return {error:'Popover API is unavailable'};
try {const inert=panel.inert;panel.inert=true;try{panel.showPopover();}finally{panel.inert=inert;}return {error:null};}
catch(error){return {error:String(error)};}
"#;

// Alpha.2 exposes element measurements and typed resize, but neither viewport
// bounds nor document/visualViewport/global ancestor-scroll subscriptions.
// This stream reports only native viewport facts/invalidations; Rust lays out.
#[cfg(any(feature = "web", feature = "desktop"))]
const FILTER_LAYOUT_EVENTS: &str = r#"
const events=new AbortController(),opts={signal:events.signal,passive:true};
const changed=()=>{const v=window.visualViewport;dioxus.send({left:v?.offsetLeft??0,top:v?.offsetTop??0,width:v?.width??document.documentElement.clientWidth,height:v?.height??window.innerHeight});};
window.addEventListener('resize',changed,opts);
document.addEventListener('scroll',changed,{...opts,capture:true});
window.visualViewport?.addEventListener('resize',changed,opts);
window.visualViewport?.addEventListener('scroll',changed,opts);
try{changed();await dioxus.recv();}finally{events.abort();}
"#;

// Pinned Dioxus has no document click subscription. Only native outside-click
// containment is reported; Rust owns closing and the stream's component lifetime.
#[cfg(any(feature = "web", feature = "desktop"))]
const FILTER_OUTSIDE_CLICK: &str = r#"
const rootId = await dioxus.recv(), root = document.getElementById(rootId);
if (!root?.isConnected) return;
const outside = event => { if (!event.composedPath().includes(root)) dioxus.send(null); };
document.addEventListener('click', outside, true);
try { await dioxus.recv(); } finally { document.removeEventListener('click', outside, true); }
"#;

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
    let viewport = use_signal(|| None::<FilterViewport>);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let generation = use_hook(|| Rc::new(Cell::new(0_u64)));
    let measure_alive = alive.clone();
    let measure_generation = generation.clone();
    let measure = use_callback(move |()| {
        if !*native.peek() {
            return;
        }
        let anchor_signal = anchor;
        let (Some(panel), Some(anchor), Some(viewport)) = (
            mounted.peek().clone(),
            anchor_signal.peek().clone(),
            *viewport.peek(),
        ) else {
            return;
        };
        measure_generation.set(measure_generation.get().wrapping_add(1));
        let revision = measure_generation.get();
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
                viewport,
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
    let _ = &anchor_id;
    let outside_transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let layout_transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let drop_outside = outside_transport.clone();
    let drop_layout = layout_transport.clone();
    let lifetime = alive.clone();
    use_drop(move || {
        lifetime.set(false);
        if let Ok(mut ready) = ready.try_write() {
            *ready = false;
        }
        for stream in [drop_outside, drop_layout] {
            if let Some(stream) = stream.borrow_mut().take() {
                let _ = stream.send(());
            }
        }
    });
    #[cfg(any(feature = "web", feature = "desktop"))]
    {
        let transport = layout_transport.clone();
        let layout_alive = alive.clone();
        let mut viewport = viewport;
        use_effect(move || {
            if !native() {
                if let Some(stream) = transport.borrow_mut().take() {
                    let _ = stream.send(());
                }
                return;
            }
            let mut stream = dioxus::document::eval(FILTER_LAYOUT_EVENTS);
            *transport.borrow_mut() = Some(stream);
            let alive = layout_alive.clone();
            spawn(async move {
                while let Ok(next) = stream.recv::<FilterViewport>().await {
                    if !alive.get() {
                        break;
                    }
                    if next.width > 0.0
                        && next.height > 0.0
                        && [next.left, next.top, next.width, next.height]
                            .iter()
                            .all(|v| v.is_finite())
                    {
                        viewport.set(Some(next));
                        measure.call(());
                    }
                }
            });
        });
    }
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
                #[cfg(any(feature="web",feature="desktop"))]
                {
                    let lifetime=mounted_alive.clone();let root_id=format!("{anchor_id}-root");let transport=outside_transport.clone();
                    spawn(async move{if !lifetime.get(){return;}let mut stream=dioxus::document::eval(FILTER_OUTSIDE_CLICK);if stream.send(root_id).is_err(){return;}*transport.borrow_mut()=Some(stream);while stream.recv::<()>().await.is_ok(){if !lifetime.get(){break;}onclose.call(());}});
                }
                if !enable_top_layer{return;}
                let alive=mounted_alive.clone();let id=mounted_id.clone();
                spawn(async move {
                    #[cfg(any(feature="web",feature="desktop"))]
                    let failure={
                        #[derive(serde::Deserialize)]struct Reply{error:Option<String>}
                        let bridge=crate::dom_query::eval(FILTER_TOP_LAYER_OPEN);
                        if let Err(error)=bridge.send(id){Some(crate::NativeSurfaceError::Rejected(error.to_string()))}
                        else{match bridge.join::<Reply>().await{Ok(reply)=>reply.error.map(crate::NativeSurfaceError::Rejected),Err(error)=>Some(crate::NativeSurfaceError::Rejected(error.to_string()))}}
                    };
                    #[cfg(not(any(feature="web",feature="desktop")))]
                    let failure={let _=id;Some(crate::NativeSurfaceError::UnsupportedRenderer)};
                    if alive.get(){if let Some(error)=failure{native.set(false);onnativeerror.call(error);}else{promoted.set(true);}measure.call(());}
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

/// A tile cycling through any number of sides. Each item in `pages` normally
/// contains [`FlipTileContent`]. Only the visible side is mounted. `index` makes
/// cycling controlled; the parent can reject a request by leaving it unchanged.
/// Outgoing and incoming half-rotations use native CSS animation events.
#[component]
pub fn FlipTile(
    pages: Vec<Element>,
    #[props(default=VNode::empty())] header: Element,
    #[props(default)] footer: Option<Element>,
    #[props(default)] variant: FlipTileVariant,
    #[props(default)] index: Option<usize>,
    #[props(default)] default_index: usize,
    #[props(default)] ontoggle: EventHandler<usize>,
    #[props(default=FlipTileDimension::Rem(16.0))] width: FlipTileDimension,
    #[props(default=FlipTileDimension::Rem(15.125))] height: FlipTileDimension,
    #[props(default="Show next content".to_string())] label: String,
) -> Element {
    let mut local = use_signal(|| default_index);
    let target = flip_index(index.unwrap_or(local()), pages.len());
    let mut displayed = use_signal(|| target);
    let mut pending = use_signal(|| None::<usize>);
    let mut phase = use_signal(FlipPhase::default);
    use_effect(use_reactive!(|target| {
        if phase() == FlipPhase::Idle && displayed() != target {
            pending.set(Some(target));
            phase.set(FlipPhase::Outgoing);
        }
    }));
    let visible = flip_index(displayed(), pages.len());
    let count = pages.len();
    let display_number = if count == 0 { 0 } else { visible + 1 };
    rsx! {
        article {class:"ix-flip-tile ix-flip-tile-{variant.class()}",style:format!("width:{};height:{}",width.css(16.0),height.css(15.125)),"aria-label":"Flippable tile",
            div {class:"ix-flip-tile-container","data-phase":phase().class(),
                onanimationend:move |event:AnimationEvent|{
                    if event.animation_name()=="ix-pattern-flip-out"&&phase()==FlipPhase::Outgoing{let next=*pending.peek();if let Some(next)=next{displayed.set(next);}phase.set(FlipPhase::Incoming);}
                    else if event.animation_name()=="ix-pattern-flip-in"&&phase()==FlipPhase::Incoming{phase.set(FlipPhase::Idle);pending.set(None);}
                },
                header {class:"ix-flip-tile-header",div {{header}}button {class:"ix-flip-tile-toggle",r#type:"button",disabled:count<2||phase()!=FlipPhase::Idle,"aria-label":label,onclick:move |_|{let next=next_flip_index(target,count);if index.is_none(){local.set(next);}ontoggle.call(next);},Icon {name:"eye",size:24}}}
                div {class:"ix-flip-tile-content",key:"{visible}",if let Some(page)=pages.get(visible){{page.clone()}}}
                if let Some(footer)=footer{footer {class:"ix-flip-tile-footer",{footer}}}
            }
            span {class:"ix-sr-only",role:"status","Content {display_number} of {count}"}
        }
    }
}
/// One flip tile side. Hidden sides are unmounted by the parent [`FlipTile`].
#[component]
pub fn FlipTileContent(children: Element) -> Element {
    rsx! {div{class:"ix-flip-tile-side",{children}}}
}

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
    fn css(&self, id: &str) -> String {
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
            "#{id}{{{large}}}@media(max-width:768px){{#{id}{{{medium}}}}}@media(max-width:576px){{#{id}{{{small}}}}}"
        )
    }
}
/// Responsive named-area grid. Use [`GridArea`] for area assignment.
#[component]
pub fn CssGrid(
    children: Element,
    #[props(default)] templates: GridTemplates,
    #[props(default)] class: String,
) -> Element {
    let id = use_signal(|| pattern_id("css-grid"))();
    let css = templates.css(&id);
    rsx! {style{{css}}div{id,class:"ix-css-grid {class}",{children}}}
}
/// Item assigned to an area in a [`CssGrid`]. Invalid names fall back to auto placement.
#[component]
pub fn GridArea(children: Element, name: String) -> Element {
    rsx! {div{class:"ix-grid-area",style:if valid_grid_area(&name){Some(format!("grid-area:{name}"))}else{None},{children}}}
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

pub(crate) const STYLES: &str = r#"
.ix-category-filter{position:relative;min-width:0;width:100%;font:inherit}.ix-category-filter-input{display:flex;align-items:flex-start;gap:8px;min-height:40px;padding:4px 8px;border:1px solid var(--ix-color-border);border-bottom-color:var(--ix-color-text-secondary);border-radius:var(--ix-radius) var(--ix-radius) 0 0;background:var(--ix-color-surface);color:var(--ix-color-text)}.ix-category-filter-input>.ix-icon{margin-top:7px;color:var(--ix-color-primary)}.ix-category-filter-input:focus-within{border-color:var(--ix-color-primary);box-shadow:inset 0 -1px var(--ix-color-primary)}.ix-category-filter-tokens{display:flex;flex:1;min-width:0;flex-wrap:wrap;align-items:center;gap:4px;max-height:60px;overflow-y:auto}.ix-category-filter-text{flex:1;min-width:120px;height:30px;padding:2px 0;border:0;background:transparent;color:inherit;outline:none!important;font:inherit}.ix-category-filter-text::placeholder{color:var(--ix-color-text-secondary);opacity:1}.ix-category-filter-chip{display:inline-flex;align-items:center;gap:4px;padding:2px 8px;max-width:100%;min-height:24px;background:var(--ix-color-surface-2);color:var(--ix-color-text);border-radius:12px;font-size:12px;line-height:20px}.ix-category-filter-chip>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-category-filter-chip>button{display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;min-width:16px;background:transparent;color:inherit;border:0;padding:0;border-radius:50%;cursor:pointer}.ix-category-filter-chip>button:hover{background:color-mix(in srgb,currentColor 15%,transparent)}.ix-category-filter-preview{display:inline-flex;align-items:center;min-height:28px;padding:0 8px;background:var(--ix-color-surface-2);white-space:nowrap}.ix-category-filter-reset{display:inline-flex;align-items:center;justify-content:center;width:30px;height:30px;min-width:30px;padding:0;border:0;background:transparent;color:var(--ix-color-primary);border-radius:var(--ix-radius);cursor:pointer}.ix-category-filter-reset:hover{background:var(--ix-color-surface-2)}.ix-category-filter-options{position:absolute;inset-inline:0;top:100%;z-index:30;max-height:320px;overflow:auto;padding:4px 0;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);box-shadow:0 8px 20px #0003}.ix-category-filter-options[data-top-layer=true]{position:fixed;inset:auto;margin:0;box-sizing:border-box;max-width:calc(100vw - 16px)}.ix-category-filter-options-heading{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:4px 12px;color:var(--ix-color-text-secondary);font-size:12px;line-height:20px}.ix-category-filter-options-heading>button{font:inherit;background:transparent;border:0;border-radius:var(--ix-radius);color:var(--ix-color-primary);cursor:pointer;min-width:32px;min-height:24px}.ix-category-filter-option{display:block;box-sizing:border-box;width:100%;padding:6px 12px;min-height:32px;border:0;border-radius:0;background:transparent;text-align:start;font:inherit;cursor:pointer;color:var(--ix-color-text)}.ix-category-filter-option:hover{background:var(--ix-color-surface-2)}.ix-category-filter-no-options{padding:8px 12px;font-size:12px;color:var(--ix-color-text-secondary)}.ix-category-filter[data-disabled=true]{opacity:.45}.ix-category-filter[data-disabled=true] *{cursor:not-allowed}.ix-category-filter[data-readonly=true] .ix-category-filter-input{background:transparent;border-color:var(--ix-color-border)}.ix-category-filter button:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}
.ix-flip-tile{--ix-flip-accent:var(--ix-color-border);--ix-flip-footer-bg:var(--ix-color-surface-1);--ix-flip-footer-color:var(--ix-color-text);perspective:1000px;max-width:100%;min-height:100px;display:flex;flex-direction:column;border:1px solid var(--ix-flip-accent);border-radius:var(--ix-radius) var(--ix-radius) 0 0;overflow:visible}.ix-flip-tile-container{height:100%;display:flex;flex-direction:column;background:var(--ix-color-surface-1);color:var(--ix-color-text);transform-style:preserve-3d;backface-visibility:hidden;border-radius:inherit}.ix-flip-tile-header{display:flex;align-items:center;min-height:40px;padding:0 8px 0 16px;gap:8px}.ix-flip-tile-header>div{flex:1;min-width:0;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-flip-tile-toggle{display:inline-flex;align-items:center;justify-content:center;width:32px;height:32px;flex-shrink:0;border:0;border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-primary);padding:4px;cursor:pointer}.ix-flip-tile-toggle:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-flip-tile-toggle:disabled{opacity:.4;cursor:default}.ix-flip-tile-toggle:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:0}.ix-flip-tile-content{flex:1;min-height:0;overflow:auto;margin:16px}.ix-flip-tile-side{display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100%;min-width:0}.ix-flip-tile-footer{display:flex;align-items:center;justify-content:center;min-height:48px;padding:4px 8px;border-top:1px solid var(--ix-flip-accent);background:var(--ix-flip-footer-bg);color:var(--ix-flip-footer-color);text-align:center;font-size:14px;line-height:20px}.ix-flip-tile-filled{--ix-flip-accent:transparent}.ix-flip-tile-outline .ix-flip-tile-container{background:transparent}.ix-flip-tile-primary{--ix-flip-accent:var(--ix-color-primary);--ix-flip-footer-bg:var(--ix-color-primary);--ix-flip-footer-color:var(--ix-color-on-primary)}.ix-flip-tile-info{--ix-flip-accent:var(--ix-color-info);--ix-flip-footer-bg:var(--ix-color-info);--ix-flip-footer-color:var(--ix-color-on-info,#000028)}.ix-flip-tile-warning{--ix-flip-accent:var(--ix-color-warning);--ix-flip-footer-bg:var(--ix-color-warning);--ix-flip-footer-color:var(--ix-color-on-warning,#000028)}.ix-flip-tile-alarm{--ix-flip-accent:var(--ix-color-danger);--ix-flip-footer-bg:var(--ix-color-danger);--ix-flip-footer-color:var(--ix-color-on-danger,#fff)}.ix-flip-tile-container[data-phase=outgoing]{animation:ix-pattern-flip-out .15s ease-in forwards}.ix-flip-tile-container[data-phase=incoming]{animation:ix-pattern-flip-in .15s ease-out both}@keyframes ix-pattern-flip-out{from{transform:rotateY(0)}to{transform:rotateY(90deg)}}@keyframes ix-pattern-flip-in{from{transform:rotateY(270deg)}to{transform:rotateY(360deg)}}
.ix-info-page{display:flex;width:100%;min-height:100%;padding:32px 16px}.ix-info-page-content{display:flex;flex-direction:column;align-items:center;max-width:800px;margin:auto;text-align:center}.ix-info-page-image{display:flex;align-items:flex-end;justify-content:center;width:260px;height:196px;max-width:100%;margin-bottom:40px}.ix-info-page-image>*{max-width:100%;max-height:100%;object-fit:contain}.ix-info-page-message{display:flex;flex-direction:column;align-items:center;max-width:640px}.ix-info-page-message h1{margin:0;font-size:32px;line-height:40px;font-weight:600;text-wrap:balance}.ix-info-page-copy{margin:16px 0 0;font-size:16px;line-height:24px}.ix-info-page-instructions{margin:16px 0 0;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary)}.ix-info-page-actions{display:flex;flex-wrap:wrap;justify-content:center;gap:8px;width:100%;margin-top:32px}
.ix-css-grid{display:grid;position:relative;gap:16px;min-width:0}.ix-grid-area{min-width:0}.ix-layout-auto{--ix-layout-columns:1;--ix-layout-gap:24px;display:flex;flex-wrap:wrap;align-items:stretch;gap:var(--ix-layout-gap);margin:0 12px}.ix-layout-auto>*{flex:0 0 auto;min-width:0;width:calc((100% + var(--ix-layout-gap)) / var(--ix-layout-columns) * min(var(--ix-colspan,1),var(--ix-layout-columns)) - var(--ix-layout-gap))}
@media(max-width:576px){.ix-info-page{padding:24px 16px}.ix-info-page-message h1{font-size:28px;line-height:36px}}
@media(prefers-reduced-motion:reduce){.ix-flip-tile-container[data-phase=outgoing],.ix-flip-tile-container[data-phase=incoming]{animation-duration:.01ms}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FilterTestMouse;
    impl dioxus::html::InteractionLocation for FilterTestMouse {
        fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
            Default::default()
        }
        fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
            Default::default()
        }
        fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
            Default::default()
        }
    }
    impl dioxus::html::InteractionElementOffset for FilterTestMouse {
        fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl dioxus::html::ModifiersInteraction for FilterTestMouse {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::PointerInteraction for FilterTestMouse {
        fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
            None
        }
        fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
            Default::default()
        }
    }
    impl dioxus::html::HasMouseData for FilterTestMouse {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct FilterTestForm(String);
    impl dioxus::html::HasFileData for FilterTestForm {
        fn files(&self) -> Vec<dioxus::html::FileData> {
            Vec::new()
        }
    }
    impl dioxus::html::HasFormData for FilterTestForm {
        fn value(&self) -> String {
            self.0.clone()
        }
        fn valid(&self) -> bool {
            true
        }
        fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
            Vec::new()
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct FilterTestKeyboard(Key, Modifiers);
    impl dioxus::html::ModifiersInteraction for FilterTestKeyboard {
        fn modifiers(&self) -> Modifiers {
            self.1
        }
    }
    impl dioxus::html::HasKeyboardData for FilterTestKeyboard {
        fn key(&self) -> Key {
            self.0.clone()
        }
        fn code(&self) -> dioxus::html::Code {
            dioxus::html::Code::Unidentified
        }
        fn location(&self) -> dioxus::html::Location {
            dioxus::html::Location::Standard
        }
        fn is_auto_repeating(&self) -> bool {
            false
        }
        fn is_composing(&self) -> bool {
            false
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct FilterTestFocus;
    impl dioxus::html::HasFocusData for FilterTestFocus {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct FilterTestMount(Rc<Cell<usize>>);
    impl dioxus::html::RenderedElementBacking for FilterTestMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            let count = self.0.clone();
            Box::pin(async move {
                assert!(focus);
                count.set(count.get() + 1);
                Ok(())
            })
        }
    }
    #[derive(Clone, Default)]
    struct FilterTokenMount {
        scrolls: Rc<RefCell<Vec<f64>>>,
        blocked: Rc<Cell<bool>>,
        started: Rc<Cell<bool>>,
        waker: Rc<RefCell<Option<std::task::Waker>>>,
    }
    impl dioxus::html::RenderedElementBacking for FilterTokenMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn get_scroll_size(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsSize>,
                    >,
            >,
        > {
            let state = self.clone();
            Box::pin(async move {
                std::future::poll_fn(|cx| {
                    state.started.set(true);
                    if state.blocked.get() {
                        *state.waker.borrow_mut() = Some(cx.waker().clone());
                        std::task::Poll::Pending
                    } else {
                        std::task::Poll::Ready(Ok(dioxus::html::geometry::PixelsSize::new(
                            200.0, 200.0,
                        )))
                    }
                })
                .await
            })
        }
        fn get_scroll_offset(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = dioxus::html::MountedResult<
                            dioxus::html::geometry::PixelsVector2D,
                        >,
                    >,
            >,
        > {
            Box::pin(async { Ok(dioxus::html::geometry::PixelsVector2D::new(0.0, 0.0)) })
        }
        fn scroll(
            &self,
            point: dioxus::html::geometry::PixelsVector2D,
            behavior: dioxus::html::ScrollBehavior,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert_eq!(behavior, dioxus::html::ScrollBehavior::Instant);
            let scrolls = self.scrolls.clone();
            Box::pin(async move {
                scrolls.borrow_mut().push(point.y);
                Ok(())
            })
        }
    }
    #[derive(Clone, Default)]
    struct FilterDelayedMount {
        focus_count: Rc<Cell<usize>>,
        started: Rc<Cell<bool>>,
        ready: Rc<Cell<bool>>,
        waiter: Rc<RefCell<Option<std::task::Waker>>>,
        read_dropped: Rc<Cell<bool>>,
    }
    struct FilterReadLifetime(Rc<Cell<bool>>);
    impl Drop for FilterReadLifetime {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    impl dioxus::html::RenderedElementBacking for FilterDelayedMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn get_client_rect(
            &self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                    >,
            >,
        > {
            let state = self.clone();
            let lifetime = FilterReadLifetime(state.read_dropped.clone());
            Box::pin(async move {
                let _lifetime = lifetime;
                std::future::poll_fn(move |context| {
                    state.started.set(true);
                    if state.ready.get() {
                        std::task::Poll::Ready(Err(dioxus::html::MountedError::NotSupported))
                    } else {
                        *state.waiter.borrow_mut() = Some(context.waker().clone());
                        std::task::Poll::Pending
                    }
                })
                .await
            })
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            let count = self.focus_count.clone();
            Box::pin(async move {
                assert!(focus);
                count.set(count.get() + 1);
                Ok(())
            })
        }
    }
    #[derive(Clone, Default)]
    struct FilterDelayedFocus(FilterDelayedMount);
    impl dioxus::html::RenderedElementBacking for FilterDelayedFocus {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            assert!(focus);
            let state = self.0.clone();
            let lifetime = FilterReadLifetime(state.read_dropped.clone());
            Box::pin(async move {
                let _lifetime = lifetime;
                std::future::poll_fn(|cx| {
                    state.started.set(true);
                    if state.ready.get() {
                        std::task::Poll::Ready(())
                    } else {
                        *state.waiter.borrow_mut() = Some(cx.waker().clone());
                        std::task::Poll::Pending
                    }
                })
                .await;
                state.focus_count.set(state.focus_count.get() + 1);
                Ok(())
            })
        }
    }
    struct FilterTestConverter;
    macro_rules! filter_test_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for FilterTestConverter {
                $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data { let _ = event; filter_test_converter!(@data $group, event) })*
            }
        };
        (@data Mouse, $event:ident) => { dioxus::html::MouseData::new(FilterTestMouse) };
        (@data Form, $event:ident) => { dioxus::html::FormData::new(FilterTestForm($event.downcast::<String>().unwrap().clone())) };
        (@data Keyboard, $event:ident) => {{
            let (key, modifiers) = match $event.downcast::<(Key, Modifiers)>() {
                Some((key, modifiers)) => (key.clone(), *modifiers),
                None => ($event.downcast::<Key>().unwrap().clone(), Modifiers::empty()),
            };
            dioxus::html::KeyboardData::new(FilterTestKeyboard(key, modifiers))
        }};
        (@data Focus, $event:ident) => { dioxus::html::FocusData::new(FilterTestFocus) };
        (@data Mounted, $event:ident) => {
            if let Some(state)=$event.downcast::<FilterDelayedFocus>() {
                MountedData::new(state.clone())
            } else if let Some(state)=$event.downcast::<FilterTokenMount>() {
                MountedData::new(state.clone())
            } else if let Some(state)=$event.downcast::<FilterDelayedMount>() {
                MountedData::new(state.clone())
            } else {
                MountedData::new(FilterTestMount($event.downcast::<Rc<Cell<usize>>>().unwrap().clone()))
            }
        };
        (@data $other:ident, $event:ident) => { panic!("Unexpected filter test event") };
    }
    dioxus::html::with_html_event_groups!(filter_test_converter);

    fn filter_listener_ids(
        mutations: dioxus::core::Mutations,
        name: &str,
    ) -> Vec<dioxus::core::ElementId> {
        filter_listener_ids_ref(&mutations, name)
    }
    fn filter_listener_ids_ref(
        mutations: &dioxus::core::Mutations,
        name: &str,
    ) -> Vec<dioxus::core::ElementId> {
        let mut current = dioxus::core::ElementId::ROOT;
        let mut ids = Vec::new();
        for edit in &mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = *id
                }
                dioxus::core::Mutation::NewEventListener { name: event } if *event == name => {
                    ids.push(current)
                }
                _ => {}
            }
        }
        ids
    }
    fn filter_focus_button_mounts(
        mutations: &dioxus::core::Mutations,
    ) -> Vec<(dioxus::core::ElementId, String)> {
        let listeners = filter_listener_ids_ref(mutations, "mounted");
        let mut current = dioxus::core::ElementId::ROOT;
        let mut targets = Vec::new();
        for edit in &mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = *id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if *name == "id"
                    && value.starts_with("ix-category-options-")
                    && listeners.contains(&current) =>
                {
                    targets.push((current, value.clone()))
                }
                _ => {}
            }
        }
        targets
    }
    fn filter_choice_target(
        mutations: &dioxus::core::Mutations,
        choice: &str,
    ) -> dioxus::core::ElementId {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in &mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = *id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "data-id" && value == choice => return current,
                _ => {}
            }
        }
        panic!("missing native choice target {choice}");
    }
    fn dispatch_filter_event(
        dom: &VirtualDom,
        name: &str,
        id: dioxus::core::ElementId,
        payload: impl std::any::Any,
    ) -> bool {
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
        // Match pinned renderer metadata: mounted/focus do not bubble. A fake
        // bubbling input mount would overwrite its ancestors' mounted handles.
        let event = Event::new(
            data,
            !matches!(
                name,
                "mounted" | "focus" | "blur" | "resize" | "scroll" | "toggle"
            ),
        );
        dom.runtime().handle_event(name, event.clone(), id);
        event.propagates()
    }
    #[derive(Default)]
    struct FilterResetConfig {
        controlled: bool,
        accept: bool,
        veto: bool,
        events: Vec<String>,
    }
    fn reset_filter_fixture(config: Rc<RefCell<FilterResetConfig>>) -> Element {
        let initial = FilterState {
            tokens: vec!["Committed text".into()],
            categories: vec![CategoryToken {
                id: "product".into(),
                value: "Existing product".into(),
                operator: LogicalFilterOperator::Equal,
            }],
        };
        let mut controlled = use_signal(|| initial.clone());
        let is_controlled = config.borrow().controlled;
        let before = config.clone();
        let change = config.clone();
        let category = config.clone();
        let clear = config.clone();
        let input_events = config.clone();
        rsx! { CategoryFilter {
            categories: categories(), state: if is_controlled { Some(controlled()) } else { None }, default_state:initial,
            onbeforeclear:move |event:FilterClearEvent| {let mut config=before.borrow_mut();config.events.push("before".into());if config.veto {event.prevent_default();}},
            oncategory:move |value:Option<String>|category.borrow_mut().events.push(format!("category:{value:?}")),
            oninput:move |_|input_events.borrow_mut().events.push("input".into()),
            onchange:move |value:FilterState|{change.borrow_mut().events.push(format!("change:{}",value.is_empty()));if change.borrow().accept {controlled.set(value);}},
            onclear:move |_|clear.borrow_mut().events.push("accepted".into()),
        } }
    }
    struct FilterResetHarness {
        dom: VirtualDom,
        input: dioxus::core::ElementId,
        reset: dioxus::core::ElementId,
        config: Rc<RefCell<FilterResetConfig>>,
        focus_count: Rc<Cell<usize>>,
    }
    impl FilterResetHarness {
        fn new(controlled: bool, accept: bool, veto: bool) -> Self {
            let config = Rc::new(RefCell::new(FilterResetConfig {
                controlled,
                accept,
                veto,
                ..Default::default()
            }));
            let mut dom = VirtualDom::new_with_props(reset_filter_fixture, config.clone());
            dioxus::html::set_event_converter(Box::new(FilterTestConverter));
            // In the actual rendered filter the final two click targets are
            // the native text input and reset button, after committed chips.
            let mutations = dom.rebuild_to_vec();
            let clicks = filter_listener_ids(mutations, "click");
            let reset = *clicks.last().unwrap();
            let input = clicks[clicks.len() - 2];
            let focus_count = Rc::new(Cell::new(0_usize));
            dispatch_filter_event(&dom, "mounted", input, focus_count.clone());
            dom.render_immediate_to_vec();
            Self {
                dom,
                input,
                reset,
                config,
                focus_count,
            }
        }
        fn text(&mut self, text: &str) {
            dispatch_filter_event(&self.dom, "input", self.input, text.to_string());
            self.dom.render_immediate_to_vec();
        }
        fn preview(&mut self) -> dioxus::core::ElementId {
            dispatch_filter_event(&self.dom, "click", self.input, ());
            let ids = filter_listener_ids(self.dom.render_immediate_to_vec(), "click");
            let category = *ids.first().expect("first category option");
            dispatch_filter_event(&self.dom, "click", category, ());
            let ids = filter_listener_ids(self.dom.render_immediate_to_vec(), "click");
            *ids.first().expect("comparison operator button")
        }
        fn html(&self) -> String {
            dioxus_ssr::render(&self.dom)
        }
    }

    #[test]
    fn category_reset_without_preview_retains_text_and_emits_only_empty_filter_then_adapter() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterResetHarness::new(false, false, false);
        harness.text("Uncommitted text");
        harness.config.borrow_mut().events.clear();
        let before_focus = harness.focus_count.get();
        assert!(
            !dispatch_filter_event(&harness.dom, "click", harness.reset, ()),
            "accepted reset stops propagation"
        );
        harness.dom.render_immediate_to_vec();
        assert_eq!(
            harness.config.borrow().events,
            ["before", "change:true", "accepted"]
        );
        let html = harness.html();
        assert!(html.contains("value=\"Uncommitted text\""));
        assert!(!html.contains("Committed text"));
        assert!(!html.contains("Existing product"));
        assert!(html.contains("data-open=\"false\""));
        assert_eq!(
            harness.focus_count.get(),
            before_focus,
            "reset requests no focus"
        );
    }

    #[test]
    fn category_reset_clears_only_preview_preserves_operator_and_cancels_pre_reset_focus() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterResetHarness::new(false, false, false);
        let operator = harness.preview();
        assert!(
            harness.focus_count.get() > 0,
            "category selection exercises the mounted input-focus path before reset"
        );
        dispatch_filter_event(&harness.dom, "click", operator, ());
        filter_key_at(&harness.dom, harness.input, Key::Backspace);
        let vendor = filter_choice_target(&harness.dom.render_immediate_to_vec(), "vendor");
        // Queue category-pointer input focus, then input/reset before polling it.
        dispatch_filter_event(&harness.dom, "click", vendor, ());
        dispatch_filter_event(
            &harness.dom,
            "input",
            harness.input,
            "Draft text".to_string(),
        );
        harness.config.borrow_mut().events.clear();
        let before_focus = harness.focus_count.get();
        dispatch_filter_event(&harness.dom, "click", harness.reset, ());
        harness.dom.render_immediate_to_vec();
        assert_eq!(
            harness.config.borrow().events,
            ["before", "category:None", "change:true", "accepted"]
        );
        assert_eq!(
            harness.focus_count.get(),
            before_focus,
            "stale queued focus is canceled"
        );
        let html = harness.html();
        assert!(html.contains("value=\"Draft text\""));
        assert!(!html.contains("ix-category-filter-preview"));
        assert!(html.contains("data-open=\"false\""));
        dispatch_filter_event(&harness.dom, "focus", harness.input, ());
        harness.dom.render_immediate_to_vec();
        assert!(
            harness.html().contains("data-open=\"false\""),
            "focus alone follows the source and remains closed"
        );
        dispatch_filter_event(&harness.dom, "click", harness.input, ());
        harness.dom.render_immediate_to_vec();
        // Restore an empty query and select Vendor again to observe retained !=.
        dispatch_filter_event(&harness.dom, "input", harness.input, String::new());
        let ids = filter_listener_ids(harness.dom.render_immediate_to_vec(), "click");
        dispatch_filter_event(&harness.dom, "click", *ids.first().unwrap(), ());
        harness.dom.render_immediate_to_vec();
        assert!(
            harness.html().contains("aria-pressed=\"true\""),
            "reset retains logical operator"
        );
    }

    #[test]
    fn category_reset_invalidates_focus_while_native_delivery_barrier_is_pending() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterResetHarness::new(false, false, false);
        harness.preview();
        assert!(
            harness.focus_count.get() > 0,
            "ordinary mounted focus works before the delayed case"
        );
        let delayed = FilterDelayedMount {
            focus_count: harness.focus_count.clone(),
            ..Default::default()
        };
        dispatch_filter_event(&harness.dom, "mounted", harness.input, delayed.clone());
        harness.dom.render_immediate_to_vec();
        let before_focus = harness.focus_count.get();
        filter_key_at(&harness.dom, harness.input, Key::Backspace);
        let vendor = filter_choice_target(&harness.dom.render_immediate_to_vec(), "vendor");
        dispatch_filter_event(&harness.dom, "click", vendor, ());
        harness.dom.render_immediate_to_vec();
        assert!(
            delayed.started.get(),
            "the category-pointer focus task reaches its native roundtrip"
        );
        assert!(!delayed.ready.get());
        assert_eq!(harness.focus_count.get(), before_focus);
        dispatch_filter_event(&harness.dom, "click", harness.reset, ());
        delayed.ready.set(true);
        delayed
            .waiter
            .borrow_mut()
            .take()
            .expect("pending native read waker")
            .wake();
        harness.dom.render_immediate_to_vec();
        assert_eq!(
            harness.focus_count.get(),
            before_focus,
            "a reset delivered during the read invalidates the pending focus"
        );
        assert!(harness.html().contains("data-open=\"false\""));
        assert!(!harness.html().contains("ix-category-filter-preview"));
    }

    #[test]
    fn category_reset_veto_preserves_committed_preview_text_and_operator_without_accepted_events() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterResetHarness::new(false, false, true);
        let operator = harness.preview();
        dispatch_filter_event(&harness.dom, "click", operator, ());
        harness.dom.render_immediate_to_vec();
        harness.text("Kept draft");
        let before = harness.html();
        harness.config.borrow_mut().events.clear();
        assert!(
            dispatch_filter_event(&harness.dom, "click", harness.reset, ()),
            "source veto precedes stopPropagation"
        );
        harness.dom.render_immediate_to_vec();
        assert_eq!(harness.config.borrow().events, ["before"]);
        assert_eq!(harness.html(), before);
    }

    #[test]
    fn category_reset_controlled_parent_can_accept_or_reject_empty_state_proposal() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for accept in [false, true] {
            let mut harness = FilterResetHarness::new(true, accept, false);
            harness.text("Controlled draft");
            harness.config.borrow_mut().events.clear();
            dispatch_filter_event(&harness.dom, "click", harness.reset, ());
            harness.dom.render_immediate_to_vec();
            let html = harness.html();
            assert_eq!(html.contains("Committed text"), !accept);
            assert_eq!(html.contains("Existing product"), !accept);
            assert!(html.contains("value=\"Controlled draft\""));
            assert!(html.contains("data-open=\"false\""));
            assert_eq!(
                harness.config.borrow().events,
                ["before", "change:true", "accepted"]
            );
        }
    }

    type PendingFilterScroll = Rc<RefCell<Option<FilterScrollRequest>>>;
    #[derive(Clone, Default)]
    pub(super) struct FilterPendingCapture(pub(super) Rc<RefCell<Option<PendingFilterScroll>>>);

    #[derive(Clone)]
    struct FilterProgrammaticConfig {
        controller: CategoryFilterController,
        prop: Option<FilterState>,
        initial: FilterState,
        controlled: bool,
        accept: bool,
        state_override: Option<FilterState>,
        pending_capture: FilterPendingCapture,
        callback_signal: Option<Signal<bool>>,
        visible: bool,
        unique: bool,
        static_operator: Option<LogicalFilterOperator>,
        nested: Option<FilterState>,
        input_command: Option<FilterState>,
        changes: Vec<FilterState>,
        events: Vec<&'static str>,
    }
    fn programmatic_filter_fixture(config: Rc<RefCell<FilterProgrammaticConfig>>) -> Element {
        let settings = config.borrow().clone();
        let capture = settings.pending_capture.clone();
        use_context_provider(|| capture);
        let mut controlled = use_signal(|| settings.initial.clone());
        let callback_signal = use_signal(|| false);
        config.borrow_mut().callback_signal = Some(callback_signal);
        let change = config.clone();
        let input = config.clone();
        let category = config.clone();
        let clear = config.clone();
        let before = config.clone();
        let change_controller = settings.controller.clone();
        let input_controller = settings.controller.clone();
        rsx! { if settings.visible { CategoryFilter {
            categories:categories(),controller:settings.controller,filter_state:settings.prop,default_state:settings.initial,
            state: if settings.controlled { Some(match settings.state_override { Some(value) => value, None => controlled() }) } else { None },
            unique_categories:settings.unique,static_operator:settings.static_operator,
            onchange:move |state:FilterState|{
                let _ = callback_signal();
                let (accept,nested)={let mut config=change.borrow_mut();config.events.push("change");config.changes.push(state.clone());(config.accept,config.nested.take())};
                if accept {controlled.set(state);}
                if let Some(nested)=nested {change_controller.set_filter_state(nested).unwrap();}
            },
            oninput:move |_|{
                let command={let mut config=input.borrow_mut();config.events.push("input");config.input_command.take()};
                if let Some(command)=command {input_controller.set_filter_state(command).unwrap();}
            },
            oncategory:move |_|category.borrow_mut().events.push("category"),
            onclear:move |_|clear.borrow_mut().events.push("clear"),
            onbeforeclear:move |_|before.borrow_mut().events.push("before"),
        } } }
    }
    struct FilterProgrammaticHarness {
        dom: VirtualDom,
        config: Rc<RefCell<FilterProgrammaticConfig>>,
        input: dioxus::core::ElementId,
        focus_count: Rc<Cell<usize>>,
        token_list: dioxus::core::ElementId,
        scroll: FilterTokenMount,
    }
    impl FilterProgrammaticHarness {
        fn new(
            controller: CategoryFilterController,
            prop: Option<FilterState>,
            controlled: bool,
            accept: bool,
        ) -> Self {
            let config = Rc::new(RefCell::new(FilterProgrammaticConfig {
                controller,
                prop,
                initial: FilterState {
                    tokens: vec!["Initial token".into()],
                    categories: vec![],
                },
                controlled,
                accept,
                state_override: None,
                pending_capture: FilterPendingCapture::default(),
                callback_signal: None,
                visible: true,
                unique: false,
                static_operator: None,
                nested: None,
                input_command: None,
                changes: vec![],
                events: vec![],
            }));
            dioxus::html::set_event_converter(Box::new(FilterTestConverter));
            let mut dom = VirtualDom::new_with_props(programmatic_filter_fixture, config.clone());
            let mutations = dom.rebuild_to_vec();
            let mounts = filter_listener_ids_ref(&mutations, "mounted");
            let token_list = mounts[mounts.len() - 2];
            let ids = filter_listener_ids(mutations, "click");
            let input = ids[ids.len() - 2];
            Self {
                dom,
                config,
                input,
                focus_count: Rc::new(Cell::new(0)),
                token_list,
                scroll: FilterTokenMount::default(),
            }
        }
        fn settle(&mut self) {
            self.dom.render_immediate_to_vec();
            self.dom.render_immediate_to_vec();
        }
        fn mount(&mut self) {
            dispatch_filter_event(&self.dom, "mounted", self.token_list, self.scroll.clone());
            dispatch_filter_event(&self.dom, "mounted", self.input, self.focus_count.clone());
            self.settle();
        }
        fn command(&mut self, state: FilterState) -> Result<(), CategoryFilterError> {
            let controller = self.config.borrow().controller.clone();
            let result = self.dom.in_runtime(|| controller.set_filter_state(state));
            self.settle();
            result
        }
        fn rerender(&mut self) -> Vec<dioxus::core::ElementId> {
            self.dom.mark_dirty(ScopeId::APP);
            let ids = filter_listener_ids(self.dom.render_immediate_to_vec(), "click");
            self.dom.render_immediate_to_vec();
            ids
        }
        fn html(&self) -> String {
            dioxus_ssr::render(&self.dom)
        }
        fn preview(&mut self) -> dioxus::core::ElementId {
            dispatch_filter_event(&self.dom, "click", self.input, ());
            let ids = filter_listener_ids(self.dom.render_immediate_to_vec(), "click");
            dispatch_filter_event(&self.dom, "click", ids[0], ());
            let ids = filter_listener_ids(self.dom.render_immediate_to_vec(), "click");
            ids[0]
        }
        fn text(&mut self, text: &str) {
            dispatch_filter_event(&self.dom, "input", self.input, text.to_string());
            self.settle();
        }
        fn clear_log(&self) {
            let mut config = self.config.borrow_mut();
            config.events.clear();
            config.changes.clear();
        }
    }
    fn plain_filter_state(text: &str) -> FilterState {
        FilterState {
            tokens: vec![text.into()],
            categories: vec![],
        }
    }

    #[test]
    fn category_programmatic_prop_waits_for_native_mount_and_observes_value_changes_only() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let controller = CategoryFilterController::default();
        assert_eq!(
            controller.set_filter_state(FilterState::default()),
            Err(CategoryFilterError::NotMounted)
        );
        let mut harness = FilterProgrammaticHarness::new(
            controller.clone(),
            Some(plain_filter_state(" prop initial ")),
            false,
            false,
        );
        harness.settle();
        assert!(
            harness.config.borrow().events.is_empty(),
            "SSR/default_state are silent"
        );
        assert!(harness.html().contains("Initial token"));
        assert_eq!(
            controller.set_filter_state(FilterState::default()),
            Err(CategoryFilterError::NotMounted)
        );
        harness.mount();
        assert_eq!(
            harness.config.borrow().changes,
            [plain_filter_state("prop initial")]
        );
        assert!(!harness.html().contains("Initial token"));
        harness.config.borrow_mut().unique = true;
        harness.rerender();
        assert_eq!(
            harness.config.borrow().changes.len(),
            1,
            "unrelated renders do not reapply an equal prop"
        );
        harness.config.borrow_mut().prop = Some(plain_filter_state("later"));
        harness.rerender();
        assert_eq!(harness.config.borrow().changes.len(), 2);
        harness.config.borrow_mut().prop = None;
        harness.rerender();
        assert_eq!(
            harness.config.borrow().changes.len(),
            2,
            "None is unset, not an invalid source undefined call"
        );
        assert!(harness.html().contains("later"));
        harness.config.borrow_mut().prop = Some(plain_filter_state("later"));
        harness.rerender();
        assert_eq!(harness.config.borrow().changes.len(), 3);
        harness.command(plain_filter_state("later")).unwrap();
        harness.command(plain_filter_state("later")).unwrap();
        assert_eq!(
            harness.config.borrow().changes.len(),
            5,
            "equal explicit commands notify once each"
        );
        assert_eq!(
            harness.focus_count.get(),
            0,
            "programmatic application never requests focus"
        );
    }

    #[test]
    fn category_programmatic_prop_ignores_signals_read_by_user_callbacks_and_preserves_nested_command()
     {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterProgrammaticHarness::new(
            CategoryFilterController::default(),
            Some(plain_filter_state("prop value")),
            false,
            false,
        );
        harness.config.borrow_mut().nested = Some(plain_filter_state("nested final value"));
        harness.mount();
        assert_eq!(
            harness.config.borrow().changes,
            [
                plain_filter_state("prop value"),
                plain_filter_state("nested final value")
            ]
        );
        let mut callback_signal = harness.config.borrow().callback_signal.unwrap();
        harness.dom.in_runtime(|| callback_signal.set(true));
        harness.settle();
        assert_eq!(
            harness.config.borrow().changes.len(),
            2,
            "an incidental effect dependency cannot reapply the unchanged prop"
        );
        assert!(
            harness.html().contains("nested final value"),
            "the prop watcher must not overwrite the reentrant command"
        );
    }

    #[test]
    fn category_programmatic_source_pipeline_preserves_duplicate_empty_id_and_reserved_id_quirks() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            FilterProgrammaticHarness::new(CategoryFilterController::default(), None, false, false);
        harness.config.borrow_mut().unique = true;
        harness.config.borrow_mut().static_operator = Some(LogicalFilterOperator::Equal);
        harness.rerender();
        harness.mount();
        let token = |id: &str, value: &str, operator| CategoryToken {
            id: id.into(),
            value: value.into(),
            operator,
        };
        let ne = LogicalFilterOperator::NotEqual;
        let eq = LogicalFilterOperator::Equal;
        harness
            .command(FilterState {
                tokens: vec![
                    " A ".into(),
                    "A".into(),
                    " ".into(),
                    "\u{feff}BOM\u{feff}".into(),
                    "\u{0085}NEL\u{0085}".into(),
                ],
                categories: vec![
                    token("one", "A", ne),
                    token("one", "B", ne),
                    token("one", "B", eq),
                    token("two", "B", eq),
                    token("normal", "Q", ne),
                    token("", "Q", eq),
                    token("later", "Q", ne),
                    token(CUSTOM_FILTER_CATEGORY, " R ", ne),
                    token("later", "R", eq),
                ],
            })
            .unwrap();
        let expected = FilterState {
            tokens: vec![
                "A".into(),
                "BOM".into(),
                "\u{0085}NEL\u{0085}".into(),
                "R".into(),
            ],
            categories: vec![
                token("one", "B", ne),
                token("one", "B", eq),
                token("two", "B", eq),
                token("normal", "Q", ne),
                token("", "Q", eq),
            ],
        };
        assert_eq!(harness.config.borrow().changes, [expected]);
        assert_eq!(
            harness.config.borrow().events,
            ["change"],
            "no input/category/reset notifications"
        );
        assert!(!harness.html().contains("Initial token"));
    }

    #[test]
    fn category_programmatic_empty_retains_draft_but_accepted_tokens_and_reentrant_input_clear_it()
    {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            FilterProgrammaticHarness::new(CategoryFilterController::default(), None, false, false);
        harness.mount();
        let operator = harness.preview();
        dispatch_filter_event(&harness.dom, "click", operator, ());
        harness.settle();
        harness.text("retained draft");
        harness.clear_log();
        let before_focus = harness.focus_count.get();
        harness.command(FilterState::default()).unwrap();
        harness.command(plain_filter_state(" ")).unwrap();
        let html = harness.html();
        assert!(html.contains("value=\"retained draft\""));
        assert!(html.contains("ix-category-filter-preview"));
        assert!(html.contains("aria-pressed=\"true\""));
        assert!(
            html.contains("data-open=\"true\""),
            "setter leaves popup visibility alone"
        );
        assert_eq!(harness.config.borrow().events, ["change", "change"]);
        assert_eq!(harness.focus_count.get(), before_focus);
        harness.config.borrow_mut().input_command = Some(plain_filter_state("command from input"));
        harness.clear_log();
        harness.text("input delivered before callback");
        let html = harness.html();
        assert_eq!(harness.config.borrow().events, ["input", "change"]);
        assert!(html.contains("command from input"));
        assert!(html.contains("value=\"\""));
        assert!(
            !html.contains("ix-category-filter-preview"),
            "input callback's setter is not overwritten by stale draft"
        );
        assert_eq!(harness.focus_count.get(), before_focus);
    }

    #[test]
    fn category_programmatic_controller_lifecycle_shared_mounts_swaps_and_reentrant_notifications()
    {
        let _event_converter_guard = crate::test_event_converter_guard();
        let controller = CategoryFilterController::default();
        let mut first = FilterProgrammaticHarness::new(controller.clone(), None, false, false);
        first.mount();
        let mut second = FilterProgrammaticHarness::new(controller.clone(), None, false, false);
        first.command(plain_filter_state("sole mount")).unwrap();
        second.mount();
        assert_eq!(
            first.command(plain_filter_state("ambiguous")),
            Err(CategoryFilterError::MultipleMounted)
        );
        drop(second);
        first.config.borrow_mut().nested = Some(plain_filter_state("nested final"));
        first.clear_log();
        first.command(plain_filter_state("outer")).unwrap();
        assert_eq!(
            first.config.borrow().changes,
            [
                plain_filter_state("outer"),
                plain_filter_state("nested final")
            ]
        );
        assert!(first.html().contains("nested final"));
        assert!(!first.html().contains(">outer<"));
        let replacement = CategoryFilterController::default();
        first.config.borrow_mut().controller = replacement.clone();
        first.rerender();
        assert_eq!(
            controller.set_filter_state(FilterState::default()),
            Err(CategoryFilterError::NotMounted)
        );
        first.command(plain_filter_state("replacement")).unwrap();
        first.config.borrow_mut().visible = false;
        first.rerender();
        assert_eq!(
            replacement.set_filter_state(FilterState::default()),
            Err(CategoryFilterError::NotMounted)
        );
        first.config.borrow_mut().visible = true;
        let ids = first.rerender();
        first.input = ids[ids.len() - 2];
        assert_eq!(
            replacement.set_filter_state(FilterState::default()),
            Err(CategoryFilterError::NotMounted)
        );
        first.mount();
        first.command(plain_filter_state("remounted")).unwrap();
    }

    #[test]
    fn category_programmatic_controlled_parent_can_accept_or_reject_normalized_proposals() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for accept in [false, true] {
            let mut harness = FilterProgrammaticHarness::new(
                CategoryFilterController::default(),
                None,
                true,
                accept,
            );
            harness.mount();
            harness.command(plain_filter_state(" normalized ")).unwrap();
            assert_eq!(
                harness.config.borrow().changes,
                [plain_filter_state("normalized")]
            );
            assert_eq!(harness.html().contains("Initial token"), !accept);
            assert_eq!(harness.html().contains(">normalized<"), accept);
        }
    }

    #[test]
    fn category_mounted_token_scroll_is_cancelled_by_clear_and_component_teardown() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            FilterProgrammaticHarness::new(CategoryFilterController::default(), None, false, false);
        harness.mount();
        harness.scroll.blocked.set(true);
        harness
            .command(plain_filter_state("pending append"))
            .unwrap();
        assert!(harness.scroll.started.get());
        assert!(harness.scroll.scrolls.borrow().is_empty());
        harness.command(FilterState::default()).unwrap();
        harness.scroll.blocked.set(false);
        if let Some(waker) = harness.scroll.waker.borrow_mut().take() {
            waker.wake();
        }
        harness.settle();
        assert!(
            harness.scroll.scrolls.borrow().is_empty(),
            "clear must cancel a suspended append scroll"
        );
        harness
            .command(plain_filter_state("accepted append"))
            .unwrap();
        assert_eq!(*harness.scroll.scrolls.borrow(), [200.0]);
        let focus = harness.focus_count.get();
        harness.scroll.blocked.set(true);
        harness.scroll.started.set(false);
        harness
            .command(plain_filter_state("removed while measuring"))
            .unwrap();
        assert!(harness.scroll.started.get());
        harness.config.borrow_mut().visible = false;
        harness.rerender();
        harness.scroll.blocked.set(false);
        if let Some(waker) = harness.scroll.waker.borrow_mut().take() {
            waker.wake();
        }
        harness.settle();
        assert_eq!(
            *harness.scroll.scrolls.borrow(),
            [200.0],
            "an unmounted list cannot receive an old append scroll"
        );
        assert_eq!(
            harness.focus_count.get(),
            focus,
            "chip scrolling never requests focus"
        );
    }

    #[test]
    fn category_programmatic_scroll_waits_for_deferred_controlled_values_not_only_chip_count() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            FilterProgrammaticHarness::new(CategoryFilterController::default(), None, true, false);
        harness.mount();
        let scrolls = harness.scroll.scrolls.borrow().len();
        harness
            .command(plain_filter_state("first proposal"))
            .unwrap();
        let capture = harness.config.borrow().pending_capture.clone();
        let pending = capture.0.borrow().clone().unwrap();
        assert_eq!(
            pending.borrow().as_ref().unwrap().expected,
            plain_filter_state("first proposal")
        );
        assert!(
            harness.html().contains("Initial token"),
            "same-count rejection keeps parent values"
        );
        harness
            .command(plain_filter_state("latest proposal"))
            .unwrap();
        harness.config.borrow_mut().state_override = Some(plain_filter_state("first proposal"));
        harness.rerender();
        assert_eq!(
            pending.borrow().as_ref().unwrap().expected,
            plain_filter_state("latest proposal"),
            "a stale parent acknowledgement does not consume the newest scroll request"
        );
        harness.config.borrow_mut().state_override = Some(plain_filter_state("latest proposal"));
        harness.rerender();
        assert!(harness.html().contains("latest proposal"));
        assert_eq!(
            harness.scroll.scrolls.borrow().len(),
            scrolls + 1,
            "only matching acknowledged values scroll the mounted list"
        );
        assert!(
            pending.borrow().is_none(),
            "matching deferred values consume the mounted scroll request"
        );
        assert_eq!(
            harness.config.borrow().events,
            ["change", "change"],
            "acknowledging props is silent"
        );
    }

    #[test]
    fn category_programmatic_command_cancels_internal_focus_pending_in_native_read() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            FilterProgrammaticHarness::new(CategoryFilterController::default(), None, false, false);
        harness.mount();
        harness.preview();
        harness.settle();
        let delayed = FilterDelayedMount {
            focus_count: harness.focus_count.clone(),
            ..Default::default()
        };
        dispatch_filter_event(&harness.dom, "mounted", harness.input, delayed.clone());
        harness.settle();
        let before_focus = harness.focus_count.get();
        filter_key_at(&harness.dom, harness.input, Key::Backspace);
        let vendor = filter_choice_target(&harness.dom.render_immediate_to_vec(), "vendor");
        dispatch_filter_event(&harness.dom, "click", vendor, ());
        harness.settle();
        assert!(delayed.started.get());
        harness.command(FilterState::default()).unwrap();
        delayed.ready.set(true);
        delayed.waiter.borrow_mut().take().unwrap().wake();
        harness.settle();
        assert_eq!(harness.focus_count.get(), before_focus);
        assert!(
            harness.html().contains("data-open=\"true\""),
            "setter itself neither opens nor closes options"
        );
    }

    fn filter_labeled_ids(
        mutations: &dioxus::core::Mutations,
        prefix: &str,
    ) -> Vec<dioxus::core::ElementId> {
        let mut current = dioxus::core::ElementId::ROOT;
        let mut ids = Vec::new();
        for edit in &mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = *id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "aria-label" && value.starts_with(prefix) => ids.push(current),
                _ => {}
            }
        }
        ids
    }
    fn filter_new_close_ids(mutations: &dioxus::core::Mutations) -> Vec<dioxus::core::ElementId> {
        filter_labeled_ids(mutations, "Remove ")
    }
    fn filter_key_at(dom: &VirtualDom, id: dioxus::core::ElementId, key: Key) -> (bool, bool) {
        filter_key_modified(dom, id, key, Modifiers::empty())
    }
    fn filter_key_modified(
        dom: &VirtualDom,
        id: dioxus::core::ElementId,
        key: Key,
        modifiers: Modifiers,
    ) -> (bool, bool) {
        let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
            (key, modifiers),
        )));
        let event = Event::new(data, true);
        dom.runtime().handle_event("keydown", event.clone(), id);
        (!event.default_action_enabled(), event.propagates())
    }
    fn filter_chip_labels(html: &str) -> Vec<String> {
        html.split("class=\"ix-category-filter-chip\"")
            .skip(1)
            .map(|part| {
                part.split_once("<span>")
                    .unwrap()
                    .1
                    .split_once("</span>")
                    .unwrap()
                    .0
                    .to_string()
            })
            .collect()
    }
    fn filter_occurrences(html: &str) -> Vec<usize> {
        html.split("data-filter-occurrence=\"")
            .skip(1)
            .map(|part| part.split('"').next().unwrap().parse().unwrap())
            .collect()
    }
    #[derive(Clone)]
    struct FilterOrderConfig {
        controller: CategoryFilterController,
        initial: FilterState,
        controlled: bool,
        accept: bool,
        external: Option<FilterState>,
        disabled: bool,
        readonly: bool,
        unique: bool,
        static_operator: Option<LogicalFilterOperator>,
        nested: Option<FilterState>,
        changes: Vec<FilterState>,
        events: Vec<&'static str>,
        pending: FilterPendingCapture,
    }
    fn filter_order_fixture(config: Rc<RefCell<FilterOrderConfig>>) -> Element {
        let settings = config.borrow().clone();
        let capture = settings.pending.clone();
        use_context_provider(|| capture);
        let mut controlled = use_signal(|| settings.initial.clone());
        let change = config.clone();
        let input = config.clone();
        let category = config.clone();
        let controller = settings.controller.clone();
        rsx! {CategoryFilter {
            categories:categories(),controller:settings.controller,default_state:settings.initial,
            state:if settings.controlled {Some(match settings.external {Some(state)=>state,None=>controlled()})} else {None},
            disabled:settings.disabled,readonly:settings.readonly,unique_categories:settings.unique,static_operator:settings.static_operator,
            onchange:move |state:FilterState|{
                let (accept,nested)={let mut config=change.borrow_mut();config.events.push("change");config.changes.push(state.clone());(config.accept,config.nested.take())};
                if accept {controlled.set(state);}
                if let Some(nested)=nested {controller.set_filter_state(nested).unwrap();}
            },
            oninput:move |_|input.borrow_mut().events.push("input"),
            oncategory:move |_|category.borrow_mut().events.push("category"),
        }}
    }
    struct FilterOrderHarness {
        dom: VirtualDom,
        config: Rc<RefCell<FilterOrderConfig>>,
        input: dioxus::core::ElementId,
        focus: Rc<Cell<usize>>,
    }
    impl FilterOrderHarness {
        fn new(initial: FilterState, controlled: bool, accept: bool) -> Self {
            let config = Rc::new(RefCell::new(FilterOrderConfig {
                controller: CategoryFilterController::default(),
                initial,
                controlled,
                accept,
                external: None,
                disabled: false,
                readonly: false,
                unique: false,
                static_operator: None,
                nested: None,
                changes: vec![],
                events: vec![],
                pending: FilterPendingCapture::default(),
            }));
            dioxus::html::set_event_converter(Box::new(FilterTestConverter));
            let mut dom = VirtualDom::new_with_props(filter_order_fixture, config.clone());
            let mutations = dom.rebuild_to_vec();
            let mounts = filter_listener_ids_ref(&mutations, "mounted");
            let token_list = mounts[mounts.len() - 2];
            let ids = filter_listener_ids(mutations, "input");
            let input = ids[0];
            let focus = Rc::new(Cell::new(0_usize));
            dispatch_filter_event(&dom, "mounted", token_list, FilterTokenMount::default());
            dispatch_filter_event(&dom, "mounted", input, focus.clone());
            let mut harness = Self {
                dom,
                config,
                input,
                focus,
            };
            harness.settle();
            harness
        }
        fn settle(&mut self) -> Vec<dioxus::core::ElementId> {
            let first = self.dom.render_immediate_to_vec();
            let mut ids = filter_new_close_ids(&first);
            ids.extend(filter_new_close_ids(&self.dom.render_immediate_to_vec()));
            ids
        }
        fn html(&self) -> String {
            dioxus_ssr::render(&self.dom)
        }
        fn labels(&self) -> Vec<String> {
            filter_chip_labels(&self.html())
        }
        fn text(&mut self, text: &str) {
            dispatch_filter_event(&self.dom, "input", self.input, text.to_string());
            self.settle();
        }
        fn key(&mut self, key: Key) -> Vec<dioxus::core::ElementId> {
            filter_key_at(&self.dom, self.input, key);
            self.settle()
        }
        fn append(&mut self, text: &str) -> dioxus::core::ElementId {
            self.text(text);
            let ids = self.key(Key::Enter);
            assert_eq!(ids.len(), 1, "one new occurrence close target");
            ids[0]
        }
        fn select(&mut self, index: usize) -> Option<dioxus::core::ElementId> {
            self.text("");
            self.key(Key::Escape);
            dispatch_filter_event(&self.dom, "click", self.input, ());
            let ids = filter_listener_ids(self.dom.render_immediate_to_vec(), "click");
            dispatch_filter_event(&self.dom, "click", ids[index], ());
            let ids = filter_labeled_ids(
                &self.dom.render_immediate_to_vec(),
                "Toggle comparison operator",
            );
            ids.first().copied()
        }
        fn update(&mut self) {
            self.dom.mark_dirty(ScopeId::APP);
            self.settle();
        }
        fn command(&mut self, state: FilterState) -> Vec<dioxus::core::ElementId> {
            let controller = self.config.borrow().controller.clone();
            self.dom
                .in_runtime(|| controller.set_filter_state(state))
                .unwrap();
            self.settle()
        }
        fn clear_log(&self) {
            let mut config = self.config.borrow_mut();
            config.events.clear();
            config.changes.clear();
        }
        fn pending(&self) -> PendingFilterScroll {
            self.config.borrow().pending.0.borrow().clone().unwrap()
        }
    }

    #[test]
    fn category_order_interleaves_occurrences_and_backspace_removes_in_reverse_chronology() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterOrderHarness::new(FilterState::default(), false, false);
        harness.append("A-plain");
        let _ = harness.select(0);
        harness.append("Vendor-one");
        harness.append("B-plain");
        let _ = harness.select(1);
        harness.append("Product-last");
        assert_eq!(
            harness.labels(),
            [
                "A-plain",
                "Vendor = Vendor-one",
                "B-plain",
                "Product = Product-last"
            ]
        );
        harness.clear_log();
        let focus = harness.focus.get();
        for expected in [
            vec!["A-plain", "Vendor = Vendor-one", "B-plain"],
            vec!["A-plain", "Vendor = Vendor-one"],
            vec!["A-plain"],
            vec![],
        ] {
            harness.key(Key::Backspace);
            assert_eq!(harness.labels(), expected);
        }
        assert_eq!(
            harness.config.borrow().events,
            ["change", "change", "change", "change"]
        );
        assert_eq!(harness.focus.get(), focus, "removal never requests focus");
    }

    #[test]
    fn category_order_programmatic_reserved_entries_and_equal_occurrences_remove_exactly_once() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterOrderHarness::new(FilterState::default(), false, false);
        let token = |id: &str, value: &str| CategoryToken {
            id: id.into(),
            value: value.into(),
            operator: LogicalFilterOperator::Equal,
        };
        let closes = harness.command(FilterState {
            tokens: vec!["First".into()],
            categories: vec![
                token("vendor", "Equal"),
                token(CUSTOM_FILTER_CATEGORY, "Middle"),
                token("vendor", "Equal"),
                token("product", "Last"),
            ],
        });
        assert_eq!(
            harness.labels(),
            [
                "First",
                "Vendor = Equal",
                "Middle",
                "Vendor = Equal",
                "Product = Last"
            ]
        );
        let original = filter_occurrences(&harness.html());
        assert_eq!(original.len(), 5);
        assert_ne!(original[1], original[3]);
        harness.clear_log();
        let focus = harness.focus.get();
        let flags = filter_key_at(&harness.dom, closes[3], Key::Enter);
        assert_eq!(flags, (true, false));
        // Even a queued duplicate native click before patching cannot remove the
        // remaining equal occurrence because removal is by stable occurrence.
        dispatch_filter_event(&harness.dom, "click", closes[3], ());
        harness.settle();
        assert_eq!(
            harness.labels(),
            ["First", "Vendor = Equal", "Middle", "Product = Last"]
        );
        let remaining = filter_occurrences(&harness.html());
        assert_eq!(
            remaining,
            [original[0], original[1], original[2], original[4]]
        );
        assert_eq!(harness.config.borrow().events, ["change"]);
        assert_eq!(harness.focus.get(), focus);
        for key in [
            Key::Backspace,
            Key::Delete,
            Key::ArrowLeft,
            Key::ArrowRight,
            Key::ArrowUp,
            Key::ArrowDown,
            Key::Home,
            Key::End,
            Key::PageUp,
            Key::PageDown,
            Key::Escape,
            Key::Character("x".into()),
        ] {
            assert_eq!(filter_key_at(&harness.dom, closes[1], key), (true, false));
        }
        assert_eq!(
            filter_key_at(&harness.dom, closes[1], Key::Tab),
            (false, true)
        );
        harness.settle();
        assert_eq!(harness.config.borrow().events, ["change"]);
        assert_eq!(
            filter_key_at(&harness.dom, closes[1], Key::Character(" ".into())),
            (true, false)
        );
        harness.settle();
        assert_eq!(harness.labels(), ["First", "Middle", "Product = Last"]);
        harness.config.borrow_mut().disabled = true;
        harness.update();
        assert!(!harness.html().contains("aria-label=\"Remove "));
        harness.config.borrow_mut().disabled = false;
        harness.config.borrow_mut().readonly = true;
        harness.update();
        assert!(!harness.html().contains("aria-label=\"Remove "));
    }

    #[test]
    fn category_order_preview_backspace_and_static_unique_follow_source_append_policy() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterOrderHarness::new(FilterState::default(), false, false);
        let operator = harness.select(0).unwrap();
        dispatch_filter_event(&harness.dom, "click", operator, ());
        harness.settle();
        harness.clear_log();
        harness.key(Key::Backspace);
        assert!(harness.config.borrow().events.is_empty());
        let _ = harness.select(0);
        assert!(
            harness.html().contains("aria-pressed=\"true\""),
            "preview exit retains != operator"
        );
        harness.config.borrow_mut().static_operator = Some(LogicalFilterOperator::Equal);
        harness.update();
        harness.append("Retained-ne");
        assert_eq!(
            harness.config.borrow().changes.last().unwrap().categories[0].operator,
            LogicalFilterOperator::NotEqual
        );
        harness.config.borrow_mut().static_operator = Some(LogicalFilterOperator::NotEqual);
        harness.update();
        let _ = harness.select(0);
        harness.config.borrow_mut().unique = true;
        harness.update();
        harness.append("Fresh-eq");
        let state = harness.config.borrow().changes.last().unwrap().clone();
        assert_eq!(state.categories.len(), 2);
        assert_eq!(
            state.categories[1].operator,
            LogicalFilterOperator::Equal,
            "static != is display-only; unique does not restrict retained preview commit"
        );
        harness.clear_log();
        harness.text("\u{feff}BOM\u{feff}");
        harness.key(Key::Escape);
        harness.key(Key::Enter);
        assert!(harness.html().contains("data-open=\"false\""));
        assert_eq!(
            harness.config.borrow().changes.last().unwrap().tokens,
            ["BOM"]
        );
        harness.clear_log();
        harness.text("BOM");
        harness.key(Key::Escape);
        harness.key(Key::Enter);
        assert_eq!(
            harness.config.borrow().events,
            ["input"],
            "rejected append is silent"
        );
        assert!(harness.html().contains("data-open=\"false\""));
    }

    #[test]
    fn category_order_controlled_acknowledges_latest_order_and_preserves_accepted_order_on_rejection()
     {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterOrderHarness::new(FilterState::default(), true, true);
        harness.append("A");
        let _ = harness.select(0);
        harness.append("X");
        harness.append("B");
        assert_eq!(harness.labels(), ["A", "Vendor = X", "B"]);
        let accepted = filter_occurrences(&harness.html());
        harness.config.borrow_mut().accept = false;
        harness.update();
        harness.text("Rejected");
        harness.key(Key::Enter);
        assert_eq!(harness.labels(), ["A", "Vendor = X", "B"]);
        assert_eq!(filter_occurrences(&harness.html()), accepted);
        let rejected = harness.config.borrow().changes.last().unwrap().clone();
        let token = |id: &str, value: &str| CategoryToken {
            id: id.into(),
            value: value.into(),
            operator: LogicalFilterOperator::Equal,
        };
        harness.command(FilterState {
            tokens: vec!["P".into()],
            categories: vec![
                token("vendor", "Y"),
                token(CUSTOM_FILTER_CATEGORY, "M"),
                token("product", "Z"),
            ],
        });
        let latest = harness.config.borrow().changes.last().unwrap().clone();
        assert_eq!(harness.labels(), ["A", "Vendor = X", "B"]);
        harness.config.borrow_mut().external = Some(rejected);
        harness.update();
        assert_eq!(
            harness.labels(),
            ["A", "B", "Rejected", "Vendor = X"],
            "unknown older acknowledgement reconstructs split order"
        );
        harness.config.borrow_mut().external = Some(latest);
        harness.update();
        assert_eq!(
            harness.labels(),
            ["P", "Vendor = Y", "M", "Product = Z"],
            "bounded latest proposal retains its known interleaving"
        );
        harness.config.borrow_mut().external = Some(FilterState {
            tokens: vec!["External".into()],
            categories: vec![token("product", "Canonical")],
        });
        harness.update();
        assert_eq!(harness.labels(), ["External", "Product = Canonical"]);
    }

    #[test]
    fn category_order_same_turn_commits_are_lossless_and_pending_scroll_survives_removal() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterOrderHarness::new(plain_filter_state("Anchor"), false, false);
        harness.clear_log();
        for index in 0..30 {
            dispatch_filter_event(
                &harness.dom,
                "input",
                harness.input,
                format!("batch-{index}"),
            );
            filter_key_at(&harness.dom, harness.input, Key::Enter);
        }
        harness.settle();
        assert_eq!(harness.labels().len(), 31);
        assert_eq!(harness.labels().last().unwrap(), "batch-29");
        harness.clear_log();
        let focus = harness.focus.get();
        dispatch_filter_event(
            &harness.dom,
            "input",
            harness.input,
            "Transient".to_string(),
        );
        filter_key_at(&harness.dom, harness.input, Key::Enter);
        let pending = harness.pending();
        assert!(pending.borrow().is_some());
        filter_key_at(&harness.dom, harness.input, Key::Backspace);
        assert_eq!(
            pending.borrow().as_ref().unwrap().expected.tokens.len(),
            31,
            "unconsumed append scroll retargets remaining criteria"
        );
        harness.settle();
        let before = harness.labels().len();
        harness.key(Key::Backspace);
        assert_eq!(harness.labels().len(), before - 1);
        assert!(
            pending.borrow().is_none(),
            "settled removal creates no new scroll request"
        );
        assert_eq!(harness.focus.get(), focus);
    }

    #[test]
    fn category_order_nested_commands_keep_final_entries_after_append_and_removal_callbacks() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = FilterOrderHarness::new(FilterState::default(), false, false);
        let token = |id: &str, value: &str| CategoryToken {
            id: id.into(),
            value: value.into(),
            operator: LogicalFilterOperator::Equal,
        };
        let nested = FilterState {
            tokens: vec!["Nested P".into()],
            categories: vec![
                token("vendor", "Nested X"),
                token(CUSTOM_FILTER_CATEGORY, "Nested M"),
            ],
        };
        harness.config.borrow_mut().nested = Some(nested.clone());
        harness.text("Outer");
        harness.key(Key::Enter);
        assert_eq!(
            harness.labels(),
            ["Nested P", "Vendor = Nested X", "Nested M"]
        );
        let closes = harness.command(nested.clone());
        harness.config.borrow_mut().nested = Some(plain_filter_state("Final command"));
        harness.clear_log();
        dispatch_filter_event(&harness.dom, "click", closes[1], ());
        harness.settle();
        assert_eq!(harness.labels(), ["Final command"]);
        assert_eq!(harness.config.borrow().events, ["change", "change"]);
    }

    #[derive(Clone)]
    struct CategoryModeConfig {
        categories: Option<Vec<FilterCategory>>,
        suggestions: Vec<String>,
        initial: FilterState,
        input_label: Option<String>,
        reset_label: Option<String>,
        operator_label: Option<String>,
        events: Vec<String>,
        changes: Vec<FilterState>,
        keyboard: FilterKeyboardCapture,
        top_layer: bool,
        blocked: bool,
        neighbors: Rc<RefCell<Vec<&'static str>>>,
    }
    type FilterKeyboardSignals = (
        Signal<Option<FilterKeyboardFocus>>,
        Signal<u64>,
        Signal<bool>,
    );
    #[derive(Clone, Default)]
    pub(super) struct FilterKeyboardCapture(pub(super) Rc<RefCell<Option<FilterKeyboardSignals>>>);
    impl FilterKeyboardCapture {
        fn request(&self) -> Option<FilterKeyboardFocus> {
            self.0.borrow().as_ref().unwrap().0.peek().clone()
        }
        fn generation(&self) -> u64 {
            *self.0.borrow().as_ref().unwrap().1.peek()
        }
    }
    fn category_mode_fixture(config: Rc<RefCell<CategoryModeConfig>>) -> Element {
        let settings = config.borrow().clone();
        let capture = settings.keyboard.clone();
        use_context_provider(|| capture);
        let change = config.clone();
        let category = config.clone();
        let input = config.clone();
        let clear = config.clone();
        let previous = settings.neighbors.clone();
        let next = settings.neighbors.clone();
        rsx! { CategoryFilter {
            onfocusprevious:move |_|previous.borrow_mut().push("previous"),onfocusnext:move |_|next.borrow_mut().push("next"),disabled:settings.blocked,
            categories: settings.categories, suggestions: settings.suggestions, default_state: settings.initial,enable_top_layer:settings.top_layer,
            label:"Legacy input",label_categories:"Category menu",label_plain_text:"Plain menu",placeholder:"Search criteria",
            aria_label_filter_input:settings.input_label,aria_label_reset_button:settings.reset_label,aria_label_operator_button:settings.operator_label,
            non_selectable_categories:BTreeMap::from([("vendor".into(),"Legacy vendor fallback".into())]),
            onchange:move |state|{let mut config=change.borrow_mut();config.events.push("change".into());config.changes.push(state);},
            oncategory:move |_|category.borrow_mut().events.push("category".into()),
            oninput:move |_|input.borrow_mut().events.push("input".into()),
            onclear:move |_|clear.borrow_mut().events.push("clear".into()),
        } }
    }
    struct CategoryModeHarness {
        dom: VirtualDom,
        config: Rc<RefCell<CategoryModeConfig>>,
        input: dioxus::core::ElementId,
        reset: dioxus::core::ElementId,
        mounts: BTreeMap<String, Rc<Cell<usize>>>,
    }
    impl CategoryModeHarness {
        fn mount_buttons(&mut self, mutations: &dioxus::core::Mutations) {
            for (element, id) in filter_focus_button_mounts(mutations) {
                let count = Rc::new(Cell::new(0));
                self.mounts.insert(id, count.clone());
                dispatch_filter_event(&self.dom, "mounted", element, count);
            }
        }
        fn render(&mut self) -> dioxus::core::Mutations {
            let mutations = self.dom.render_immediate_to_vec();
            self.mount_buttons(&mutations);
            mutations
        }
        fn settle(&mut self) {
            for _ in 0..4 {
                self.render();
            }
        }
        fn new(categories: Option<Vec<FilterCategory>>) -> Self {
            Self::with_state(
                categories,
                FilterState {
                    tokens: vec!["Committed mode token".into()],
                    categories: vec![],
                },
            )
        }
        fn with_state(categories: Option<Vec<FilterCategory>>, initial: FilterState) -> Self {
            let config = Rc::new(RefCell::new(CategoryModeConfig {
                categories,
                suggestions: vec!["Shared".into(), "Blocked".into(), "  Trimmed  ".into()],
                initial,
                input_label: None,
                reset_label: None,
                operator_label: None,
                events: vec![],
                changes: vec![],
                keyboard: FilterKeyboardCapture::default(),
                top_layer: false,
                blocked: false,
                neighbors: Rc::new(RefCell::new(Vec::new())),
            }));
            let mut dom = VirtualDom::new_with_props(category_mode_fixture, config.clone());
            dioxus::html::set_event_converter(Box::new(FilterTestConverter));
            let clicks = filter_listener_ids(dom.rebuild_to_vec(), "click");
            let mut harness = Self {
                dom,
                config,
                input: clicks[clicks.len() - 2],
                reset: *clicks.last().unwrap(),
                mounts: BTreeMap::new(),
            };
            harness.settle();
            harness
        }
        fn html(&self) -> String {
            dioxus_ssr::render(&self.dom)
        }
        fn open(&mut self) -> Vec<dioxus::core::ElementId> {
            dispatch_filter_event(&self.dom, "click", self.input, ());
            filter_listener_ids(self.render(), "click")
        }
        fn click(&mut self, id: dioxus::core::ElementId) -> Vec<dioxus::core::ElementId> {
            dispatch_filter_event(&self.dom, "click", id, ());
            filter_listener_ids(self.render(), "click")
        }
        fn text(&mut self, text: &str) {
            dispatch_filter_event(&self.dom, "input", self.input, text.to_string());
            self.render();
        }
        fn key(&mut self, key: Key) -> bool {
            let data: Rc<dyn std::any::Any> =
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(key)));
            let event = Event::new(data, true);
            self.dom
                .runtime()
                .handle_event("keydown", event.clone(), self.input);
            self.render();
            !event.default_action_enabled()
        }
        fn update(&mut self) -> Vec<dioxus::core::ElementId> {
            self.dom.mark_dirty(ScopeId::APP);
            filter_listener_ids(self.render(), "click")
        }
    }

    fn filter_native_option_ids(html: &str) -> Vec<String> {
        html.split("<button ")
            .filter_map(|button| {
                let attributes = button.split_once('>')?.0;
                if !attributes.contains("class=\"ix-category-filter-option\"") {
                    return None;
                }
                Some(
                    attributes
                        .split_once("id=\"")?
                        .1
                        .split('"')
                        .next()?
                        .to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn category_top_layer_option_request_waits_for_rendered_layout_readiness() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(Some(categories()));
        harness.config.borrow_mut().top_layer = true;
        harness.update();
        let capture = harness.config.borrow().keyboard.clone();
        let mut ready = capture.0.borrow().as_ref().unwrap().2;
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        harness.settle();
        let first = capture
            .request()
            .expect("opening focus must wait for mounted native layout");
        assert!(matches!(first.target, FilterFocusTarget::Element(_)));
        harness.settle();
        assert_eq!(
            capture.request(),
            Some(first.clone()),
            "waiting does not consume the request"
        );
        filter_key_at(&harness.dom, harness.input, Key::Escape);
        harness.settle();
        assert!(capture.request().is_none());
        harness.dom.in_runtime(|| ready.set(true));
        harness.settle();
        assert!(
            capture.request().is_none(),
            "a late layout-ready notification cannot revive canceled focus"
        );
        harness.dom.in_runtime(|| ready.set(false));
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        harness.settle();
        assert!(capture.request().is_some());
        harness.dom.in_runtime(|| ready.set(true));
        harness.settle();
        assert!(
            capture.request().is_none(),
            "post-render readiness releases the current request once"
        );
    }

    #[test]
    fn category_keyboard_focuses_mounted_options_and_calls_explicit_neighbors() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(Some(categories()));
        let neighbors = harness.config.borrow().neighbors.clone();
        harness.open();
        harness.settle();
        let ids = filter_native_option_ids(&harness.html());
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        harness.settle();
        assert_eq!(
            harness.mounts[&ids[0]].get(),
            1,
            "opening navigation uses the current mounted option"
        );
        assert_eq!(harness.mounts[&ids[1]].get(), 0);
        filter_key_at(&harness.dom, harness.input, Key::ArrowUp);
        harness.settle();
        assert_eq!(*neighbors.borrow(), ["previous"]);
        harness.config.borrow_mut().categories = None;
        harness.update();
        harness.settle();
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        harness.settle();
        assert_eq!(*neighbors.borrow(), ["previous", "next"]);
        filter_key_at(&harness.dom, harness.input, Key::ArrowUp);
        harness.config.borrow_mut().blocked = true;
        harness.update();
        harness.settle();
        assert_eq!(
            *neighbors.borrow(),
            ["previous", "next"],
            "blocking before delivery cancels the queued neighbor request"
        );
    }

    #[test]
    fn category_keyboard_input_uses_mounted_focus_requests_and_commits_draft_not_choices() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(Some(categories()));
        let capture = harness.config.borrow().keyboard.clone();
        dispatch_filter_event(&harness.dom, "focus", harness.input, ());
        harness.settle();
        assert!(harness.html().contains("data-open=\"false\""));
        for key in [Key::Home, Key::End] {
            assert_eq!(
                filter_key_at(&harness.dom, harness.input, key),
                (false, false)
            );
        }
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::ArrowUp),
            (true, true)
        );
        assert_eq!(
            capture.request().unwrap().target,
            FilterFocusTarget::PreviousControl
        );
        harness.settle();
        assert!(harness.html().contains("data-open=\"false\""));
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::ArrowDown),
            (true, false)
        );
        let first = capture.request().unwrap().target;
        harness.settle();
        let ids = filter_native_option_ids(&harness.html());
        assert_eq!(ids.len(), 2);
        assert_eq!(first, FilterFocusTarget::Element(ids[0].clone()));
        assert!(!harness.html().contains("aria-activedescendant"));
        assert!(!harness.html().contains("role=\"listbox\""));
        assert!(!harness.html().contains("role=\"option\""));
        assert!(!harness.html().contains("aria-selected"));
        for _ in 0..2 {
            assert_eq!(
                filter_key_at(&harness.dom, harness.input, Key::ArrowDown),
                (false, false)
            );
            assert_eq!(
                capture.request().unwrap().target,
                first,
                "open Down never cycles"
            );
            harness.settle();
        }
        harness.text("Ven");
        assert_eq!(filter_native_option_ids(&harness.html()).len(), 1);
        harness.config.borrow_mut().events.clear();
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::Enter),
            (true, true)
        );
        harness.settle();
        assert_eq!(harness.config.borrow().events, ["change"]);
        assert_eq!(
            harness.config.borrow().changes.last().unwrap().tokens,
            ["Committed mode token", "Ven"]
        );
        assert!(
            harness
                .config
                .borrow()
                .changes
                .last()
                .unwrap()
                .categories
                .is_empty()
        );
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::Home),
            (false, false)
        );
        assert_eq!(
            filter_key_modified(&harness.dom, harness.input, Key::End, Modifiers::SHIFT),
            (false, false)
        );
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::Escape),
            (false, true)
        );
        harness.settle();
        assert!(harness.html().contains("data-open=\"false\""));
    }

    #[test]
    fn category_keyboard_space_suggestions_and_unmatched_down_preserve_source_distinctions() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(None);
        let capture = harness.config.borrow().keyboard.clone();
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::Character(" ".into())),
            (true, true)
        );
        let first = capture.request().unwrap().target;
        harness.settle();
        let ids = filter_native_option_ids(&harness.html());
        assert_eq!(first, FilterFocusTarget::Element(ids[0].clone()));
        assert!(harness.html().contains("value=\"\""));
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::Character(" ".into())),
            (true, true)
        );
        assert!(
            capture.request().is_none(),
            "open Space does not refocus or insert"
        );
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::ArrowDown),
            (true, true)
        );
        assert_eq!(
            capture.request().unwrap().target,
            FilterFocusTarget::NextControl
        );
        harness.settle();
        assert!(harness.html().contains("data-open=\"true\""));
        harness.config.borrow_mut().categories = Some(categories());
        harness.update();
        harness.text("no matching category");
        assert!(filter_native_option_ids(&harness.html()).is_empty());
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::ArrowDown),
            (true, true)
        );
        assert_eq!(
            capture.request().unwrap().target,
            FilterFocusTarget::NextControl
        );
        harness.settle();
        assert!(harness.html().contains("data-open=\"true\""));
        assert!(harness.config.borrow().changes.is_empty());
    }

    #[test]
    fn category_keyboard_native_buttons_cancel_activation_and_wrap_only_popup_boundaries() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(Some(categories()));
        let capture = harness.config.borrow().keyboard.clone();
        let options = harness.open();
        let ids = filter_native_option_ids(&harness.html());
        harness.config.borrow_mut().events.clear();
        for key in [
            Key::Enter,
            Key::Character(" ".into()),
            Key::ArrowUp,
            Key::ArrowDown,
            Key::Home,
            Key::End,
        ] {
            assert_eq!(filter_key_at(&harness.dom, options[0], key), (true, true));
            assert!(
                capture.request().is_none(),
                "native option navigation has no ix item target"
            );
        }
        assert_eq!(
            filter_key_at(&harness.dom, options[0], Key::ArrowRight),
            (false, true)
        );
        assert!(harness.config.borrow().events.is_empty());
        assert_eq!(
            filter_key_at(&harness.dom, harness.input, Key::Tab),
            (false, true)
        );
        assert_eq!(
            filter_key_at(&harness.dom, harness.reset, Key::Tab),
            (false, true)
        );
        assert_eq!(
            filter_key_at(&harness.dom, options[0], Key::Tab),
            (false, true)
        );
        assert_eq!(
            filter_key_modified(&harness.dom, options[0], Key::Tab, Modifiers::SHIFT),
            (true, false)
        );
        assert_eq!(
            capture.request().unwrap().target,
            FilterFocusTarget::Element(ids[1].clone())
        );
        harness.settle();
        assert_eq!(
            harness.mounts[&ids[1]].get(),
            1,
            "reverse boundary focuses the mounted last option"
        );
        assert_eq!(
            filter_key_at(&harness.dom, options[1], Key::Tab),
            (true, false)
        );
        assert_eq!(
            capture.request().unwrap().target,
            FilterFocusTarget::Element(ids[0].clone())
        );
        harness.settle();
        assert_eq!(
            harness.mounts[&ids[0]].get(),
            1,
            "forward boundary focuses the mounted first option"
        );
        assert_eq!(
            filter_key_at(&harness.dom, options[0], Key::Escape),
            (false, false)
        );
        harness.settle();
        assert!(harness.html().contains("data-open=\"false\""));
        assert!(harness.config.borrow().events.is_empty());

        let options = harness.open();
        let native = harness.click(options[0]);
        let operator = native[0];
        let focus = Rc::new(Cell::new(0_usize));
        dispatch_filter_event(&harness.dom, "mounted", harness.input, focus.clone());
        harness.settle();
        let before_focus = focus.get();
        harness.config.borrow_mut().events.clear();
        harness.click(operator);
        assert_eq!(
            focus.get(),
            before_focus,
            "pointer operator does not request input focus"
        );
        assert!(harness.html().contains("aria-pressed=\"true\""));
        assert!(harness.config.borrow().events.is_empty());
        for key in [Key::Enter, Key::Character(" ".into())] {
            assert_eq!(filter_key_at(&harness.dom, operator, key), (true, true));
        }
        harness.settle();
        assert!(harness.html().contains("aria-pressed=\"true\""));
        assert_eq!(
            filter_key_at(&harness.dom, operator, Key::Escape),
            (false, false)
        );
        harness.settle();
        assert_eq!(
            focus.get(),
            before_focus,
            "Escape requests no input restoration"
        );
        assert!(harness.html().contains("data-open=\"false\""));
    }

    #[test]
    fn category_keyboard_focus_requests_use_current_identity_and_reset_invalidates_pending_work() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(Some(categories()));
        let capture = harness.config.borrow().keyboard.clone();
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        let pending = capture.request().unwrap();
        // A mode update replaces the actual option identity even when the first
        // ordinal remains zero. A mounted request cannot follow a replacement ID.
        harness.config.borrow_mut().categories = None;
        harness.update();
        let current = filter_native_option_ids(&harness.html());
        if let FilterFocusTarget::Element(id) = pending.target {
            assert!(!current.contains(&id));
            assert!(!harness.html().contains(&format!("id=\"{id}\"")));
        } else {
            panic!("closed Down selects an option target");
        }
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        let generation = capture.generation();
        dispatch_filter_event(&harness.dom, "click", harness.reset, ());
        assert!(capture.generation() > generation);
        harness.settle();
        assert!(capture.request().is_none());
        assert!(harness.html().contains("data-open=\"false\""));
        // The following intentional open/focus request is not suppressed.
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        assert!(matches!(
            capture.request().unwrap().target,
            FilterFocusTarget::Element(_)
        ));
        harness.settle();
        assert!(harness.html().contains("data-open=\"true\""));
    }

    #[test]
    fn category_keyboard_unmount_drops_pending_mounted_option_focus() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(Some(categories()));
        let options = harness.open();
        harness.settle();
        let delayed = FilterDelayedFocus::default();
        dispatch_filter_event(&harness.dom, "mounted", options[0], delayed.clone());
        harness.settle();
        filter_key_at(&harness.dom, harness.input, Key::ArrowDown);
        harness.settle();
        assert!(
            delayed.0.started.get(),
            "the current option receives its mounted focus attempt"
        );
        assert!(!delayed.0.read_dropped.get());
        drop(harness);
        assert!(
            delayed.0.read_dropped.get(),
            "unmount drops the outstanding mounted focus future"
        );
        delayed.0.ready.set(true);
        if let Some(waiter) = delayed.0.waiter.borrow_mut().take() {
            waiter.wake();
        }
        assert_eq!(delayed.0.focus_count.get(), 0);
    }

    #[test]
    fn category_modes_omitted_empty_and_populated_keep_distinct_options_and_free_enter() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for (mode, category_menu, option_count) in [
            (None, false, 3),
            (Some(vec![]), true, 0),
            (Some(categories()), true, 2),
        ] {
            let mut harness = CategoryModeHarness::new(mode);
            let options = harness.open();
            assert_eq!(options.len(), option_count);
            let html = harness.html();
            assert_eq!(html.contains("Category menu"), category_menu);
            assert_eq!(html.contains("Plain menu"), !category_menu);
            assert_eq!(
                html.contains("Shared"),
                !category_menu,
                "suggestions appear only with omitted categories"
            );
            harness.text("ad hoc text");
            assert!(harness.key(Key::Enter));
            let state = harness.config.borrow().changes.last().unwrap().clone();
            assert!(state.tokens.contains(&"ad hoc text".into()));
            assert!(state.categories.is_empty());
            harness.click(harness.reset);
            assert!(harness.config.borrow().changes.last().unwrap().is_empty());
            assert!(harness.html().contains("data-open=\"false\""));
        }
    }

    #[test]
    fn category_modes_dynamic_props_preserve_preview_and_live_native_labels_without_events() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = CategoryModeHarness::new(Some(categories()));
        assert!(harness.html().contains("aria-label=\"Legacy input\""));
        assert!(harness.html().contains("aria-label=\"Clear filters\""));
        let options = harness.open();
        let ids = harness.click(options[0]);
        let operator = ids[0];
        assert!(
            harness
                .html()
                .contains("aria-label=\"Toggle comparison operator\"")
        );
        harness.click(operator);
        // Retain a native operator mount while switching it out of the menu.
        let focus_count = Rc::new(Cell::new(0_usize));
        dispatch_filter_event(&harness.dom, "mounted", operator, focus_count.clone());
        harness.dom.render_immediate_to_vec();
        harness.config.borrow_mut().events.clear();
        {
            let mut config = harness.config.borrow_mut();
            config.input_label = Some("Search assets".into());
            config.reset_label = Some("Reset criteria".into());
            config.operator_label = Some("Switch comparison".into());
        }
        harness.update();
        let html = harness.html();
        for label in ["Search assets", "Reset criteria", "Switch comparison"] {
            assert!(html.contains(&format!("aria-label=\"{label}\"")));
        }
        harness.config.borrow_mut().categories = None;
        harness.update();
        let html = harness.html();
        assert!(html.contains("Plain menu"));
        assert!(html.contains("Shared"));
        assert!(!html.contains("ix-category-filter-preview"));
        assert!(!html.contains("Switch comparison"));
        assert!(
            html.contains("placeholder=\"\""),
            "retained preview still hides placeholder"
        );
        assert!(html.contains("Reset criteria"));
        assert!(
            !harness.key(Key::Tab),
            "hidden operator must not trap Tab through its stale native mount"
        );
        assert_eq!(focus_count.get(), 0);
        {
            let mut config = harness.config.borrow_mut();
            config.categories = Some(vec![]);
            config.input_label = Some(String::new());
            config.reset_label = Some(String::new());
            config.operator_label = Some(String::new());
        }
        harness.update();
        let html = harness.html();
        assert!(html.contains("ix-category-filter-preview"));
        assert!(!html.contains("Vendor"));
        assert!(!html.contains("Legacy vendor fallback"));
        assert!(!html.contains("Plain menu"));
        assert!(!html.contains("Category menu"));
        assert!(!html.contains("Shared"));
        assert_eq!(
            html.matches("aria-label=\"\"").count(),
            3,
            "explicit empty labels reach input/reset/operator"
        );
        harness.config.borrow_mut().categories = Some(vec![FilterCategory::new(
            "vendor",
            "Vendor restored",
            vec!["Zeta".into()],
        )]);
        harness.update();
        assert!(harness.html().contains("Vendor restored"));
        assert!(harness.html().contains("Zeta"));
        assert!(
            harness.html().contains("aria-pressed=\"true\""),
            "mode changes preserve != operator"
        );
        assert!(
            harness.config.borrow().events.is_empty(),
            "prop changes and native Tab emit no filter events"
        );
        // Omitted labels return to compatible defaults on the actual controls.
        {
            let mut config = harness.config.borrow_mut();
            config.input_label = None;
            config.reset_label = None;
            config.operator_label = None;
        }
        harness.update();
        let html = harness.html();
        for label in [
            "Legacy input",
            "Clear filters",
            "Toggle comparison operator",
        ] {
            assert!(html.contains(&format!("aria-label=\"{label}\"")));
        }
        harness.config.borrow_mut().categories = None;
        harness.update();
        harness.text("typed retained category");
        harness.key(Key::Enter);
        let state = harness.config.borrow().changes.last().unwrap().clone();
        assert_eq!(state.categories[0].id, "vendor");
        assert_eq!(state.categories[0].value, "typed retained category");
        assert_eq!(
            state.categories[0].operator,
            LogicalFilterOperator::NotEqual
        );
    }

    #[test]
    fn category_modes_plain_suggestion_click_uses_retained_duplicate_context_but_inserts_free_text()
    {
        let _event_converter_guard = crate::test_event_converter_guard();
        let state = FilterState {
            tokens: vec!["Shared".into()],
            categories: vec![
                CategoryToken {
                    id: "vendor".into(),
                    value: "Blocked".into(),
                    operator: LogicalFilterOperator::Equal,
                },
                CategoryToken {
                    id: "vendor".into(),
                    value: "Trimmed".into(),
                    operator: LogicalFilterOperator::Equal,
                },
            ],
        };
        let mut harness = CategoryModeHarness::with_state(Some(categories()), state);
        let original = harness.open();
        harness.click(original[0]);
        harness.config.borrow_mut().categories = None;
        harness.config.borrow_mut().events.clear();
        harness.update();
        let html = harness.html();
        assert!(
            html.contains("  Trimmed  "),
            "suggestion filtering checks its original untrimmed value"
        );
        assert!(html.contains("Shared"));
        // The source index-based option list retains its first two native nodes
        // while switching category values to plain suggestions.
        harness.click(original[1]);
        assert!(
            harness.config.borrow().changes.is_empty(),
            "trimmed categorized duplicate blocks the click"
        );
        harness.click(original[0]);
        let state = harness.config.borrow().changes.last().unwrap().clone();
        assert_eq!(
            state.tokens,
            ["Shared", "Shared"],
            "retained category permits source's duplicate plain-token corner"
        );
        assert_eq!(
            state.categories.len(),
            2,
            "a plain suggestion never becomes a category token"
        );
        assert!(
            harness.html().contains("placeholder=\"Search criteria\""),
            "accepted suggestion clears retained preview"
        );
        assert_eq!(harness.config.borrow().events, ["change"]);
    }

    #[test]
    fn category_popup_layout_flips_clamps_and_preserves_inline_mode() {
        let metrics = FilterPopupMetrics {
            anchor_x: 950.0,
            anchor_y: 550.0,
            anchor_width: 200.0,
            anchor_height: 40.0,
            content_height: 200.0,
            viewport: FilterViewport {
                left: 0.0,
                top: 0.0,
                width: 1000.0,
                height: 600.0,
            },
        };
        let layout = place_filter_popup(Some(metrics), true);
        assert_eq!(layout.placement, "top");
        assert!(
            layout.style.contains("left:792px")
                && layout.style.contains("top:346px")
                && layout.style.contains("width:200px")
        );
        let narrow = place_filter_popup(
            Some(FilterPopupMetrics {
                anchor_x: 20.0,
                anchor_y: 10.0,
                anchor_width: 300.0,
                anchor_height: 40.0,
                content_height: 500.0,
                viewport: FilterViewport {
                    left: 0.0,
                    top: 0.0,
                    width: 100.0,
                    height: 200.0,
                },
            }),
            true,
        );
        assert_eq!(narrow.placement, "bottom");
        assert!(
            narrow.style.contains("left:8px")
                && narrow.style.contains("width:84px")
                && narrow.style.contains("max-height:140px")
        );
        assert!(
            layout.style.contains("visibility:visible"),
            "placed layout must explicitly clear the initial hidden declaration"
        );
        let inline = place_filter_popup(Some(metrics), false);
        assert!(
            inline.style.contains("visibility:visible")
                && inline.style.contains("position:absolute")
                && inline.style.contains("width:auto")
                && inline.style.contains("top:100%"),
            "inline adaptation must overwrite prior native layout declarations"
        );
        assert_eq!(place_filter_popup(None, true).style, "visibility:hidden");
    }

    fn top_layer_fixture() -> Element {
        let anchor = use_signal(|| None::<Rc<MountedData>>);
        let ready = use_signal(|| false);
        rsx! {
            FilterOptionsSurface {ready,anchor,anchor_id:"first-input",enable_top_layer:true,onnativeerror:move |_|{},onclose:move |_|{},div {role:"listbox","First choices"}}
            FilterOptionsSurface {ready,anchor,anchor_id:"second-input",enable_top_layer:true,onnativeerror:move |_|{},onclose:move |_|{},div {role:"listbox","Second choices"}}
            FilterOptionsSurface {ready,anchor,anchor_id:"inline-input",enable_top_layer:false,onnativeerror:move |_|{},onclose:move |_|{},div {role:"listbox","Inline choices"}}
        }
    }
    #[test]
    fn category_options_keep_scoped_native_surfaces_and_an_inline_mode() {
        let mut dom = VirtualDom::new(top_layer_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("popover=\"manual\"").count(), 2);
        assert_eq!(html.matches("data-top-layer=\"true\"").count(), 2);
        assert_eq!(html.matches("data-top-layer=\"false\"").count(), 1);
        let ids: Vec<_> = html
            .split("id=\"ix-category-surface-")
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap())
            .collect();
        assert_eq!(ids.len(), 3);
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        assert!(html.contains("Inline choices"));
    }
    #[test]
    fn category_clear_cancellation_is_shared_across_dispatched_event_copies() {
        let event = FilterClearEvent::default();
        let delivered = event.clone();
        assert!(!event.default_prevented());
        delivered.prevent_default();
        assert!(event.default_prevented());
    }
    fn categories() -> Vec<FilterCategory> {
        vec![
            FilterCategory::new(
                "vendor",
                "Vendor",
                vec!["Apple".into(), "MS".into(), "APS".into()],
            ),
            FilterCategory::new(
                "product",
                "Product",
                vec!["iPhone X".into(), "Windows".into()],
            ),
        ]
    }
    #[test]
    fn category_tokens_trim_and_deduplicate_by_identity_without_losing_operator() {
        let mut state = FilterState::default();
        assert!(state.add_text(" custom "));
        assert!(!state.add_text("custom"));
        assert!(!state.add_text("  "));
        assert!(state.add_category("vendor", " Apple ", LogicalFilterOperator::NotEqual, false));
        assert_eq!(
            state.categories[0].operator,
            LogicalFilterOperator::NotEqual
        );
        assert!(!state.add_category("vendor", "Apple", LogicalFilterOperator::Equal, false));
        assert!(state.add_category("product", "Apple", LogicalFilterOperator::Equal, false));
        assert!(!state.add_category("vendor", "MS", LogicalFilterOperator::Equal, true));
    }
    #[test]
    fn category_append_uses_retained_operator_and_resets_draft() {
        let mut entries = Vec::new();
        let mut draft = FilterDraft {
            input: FilterInputState {
                token: String::new(),
                category: Some("vendor".into()),
            },
            operator: LogicalFilterOperator::NotEqual,
        };
        let operator = draft.operator;
        assert!(append_filter_entry(
            &mut entries,
            &mut draft,
            "MS",
            "vendor".into(),
            operator
        ));
        assert_eq!(
            project_filter_entries(&entries).categories[0].operator,
            LogicalFilterOperator::NotEqual
        );
        assert_eq!(draft, FilterDraft::default());
    }
    #[test]
    fn suggestion_filtering_handles_case_unique_categories_and_duplicate_values() {
        let mut state = FilterState::default();
        state.add_category("vendor", "Apple", LogicalFilterOperator::Equal, false);
        assert_eq!(
            filter_choices(
                Some(&categories()),
                &[],
                &state,
                &FilterDraft::default(),
                true
            ),
            vec![FilterChoice::Category("product".into(), "Product".into())]
        );
        let draft = FilterDraft {
            input: FilterInputState {
                token: "aP".into(),
                category: Some("vendor".into()),
            },
            ..Default::default()
        };
        assert_eq!(
            filter_choices(Some(&categories()), &[], &state, &draft, false),
            vec![FilterChoice::Value("APS".into())]
        );
        state.add_text("Item 1");
        assert_eq!(
            filter_choices(
                None,
                &["Item 1".into(), "Item 2".into()],
                &state,
                &FilterDraft::default(),
                false
            ),
            vec![FilterChoice::Suggestion("Item 2".into())]
        );
    }
    #[test]
    fn grid_templates_reject_injection_ragged_and_nonrectangular_areas() {
        assert_eq!(
            GridTemplate::from_rows(&[&["a", "b"], &["a", "b"]])
                .unwrap()
                .css(),
            "\"a b\" \"a b\""
        );
        assert_eq!(
            GridTemplate::from_rows(&[&["a", "b"], &["a"]]),
            Err(GridTemplateError::RaggedRows)
        );
        assert!(matches!(
            GridTemplate::from_rows(&[&["a", "a"], &["a", "b"]]),
            Err(GridTemplateError::NonRectangularArea(_))
        ));
        assert!(matches!(
            GridTemplate::from_rows(&[&["a;display:none"]]),
            Err(GridTemplateError::InvalidArea(_))
        ));
    }
    #[test]
    fn responsive_templates_fall_back_and_layout_breakpoints_reject_css_injection() {
        let templates = GridTemplates {
            lg: Some(GridTemplate::from_rows(&[&["a", "b"]]).unwrap()),
            ..Default::default()
        };
        assert_eq!(
            templates.css("grid").matches("grid-template-areas").count(),
            3
        );
        let css = auto_layout_css(
            "layout",
            &[
                AutoLayoutBreakpoint::new("0", 1),
                AutoLayoutBreakpoint::new("800px", 4),
                AutoLayoutBreakpoint::new("1px){body{display:none}}", 3),
            ],
        );
        assert!(css.contains("min-width:800px"));
        assert!(!css.contains("body"));
    }
    #[test]
    fn flip_indices_handle_empty_single_and_out_of_range_pages() {
        assert_eq!(next_flip_index(0, 0), 0);
        assert_eq!(next_flip_index(9, 1), 0);
        assert_eq!(next_flip_index(1, 2), 0);
        assert_eq!(flip_index(99, 3), 2);
        assert_eq!(FlipTileDimension::Rem(f64::NAN).css(16.0), "16rem");
    }
    fn fixture() -> Element {
        rsx! {CategoryFilter{disabled:true,default_state:FilterState{tokens:vec!["Token".into()],categories:vec![]}}FlipTile{index:1,pages:vec![rsx!{FlipTileContent{"Hidden side"}},rsx!{FlipTileContent{"Visible side"}}]}InfoPage{title_text:"Report could not be generated",instructions:"Try again",actions:rsx!{button{"Return to overview"}}}}
    }
    #[test]
    fn composites_render_committed_filter_visible_flip_side_and_recovery_action() {
        let mut dom = VirtualDom::new(fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("class=\"ix-category-filter-text\""));
        assert!(!html.contains("role=\"combobox\""));
        assert!(html.contains("Token"));
        assert!(html.contains("Visible side"));
        assert!(!html.contains("Hidden side"));
        assert!(html.contains("Return to overview"));
    }
}
