//! Composite filtering, flippable tiles, information pages, and responsive layouts.
use crate::Icon;
use dioxus::prelude::*;
use std::{
    cell::Cell,
    collections::BTreeMap,
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
}
impl FilterChoice {
    fn label(&self) -> &str {
        match self {
            Self::Category(_, label) | Self::Value(label) => label,
        }
    }
}
fn filter_choices(
    categories: &[FilterCategory],
    suggestions: &[String],
    state: &FilterState,
    draft: &FilterDraft,
    unique: bool,
) -> Vec<FilterChoice> {
    let query = draft.input.token.to_lowercase();
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
                            && !state
                                .categories
                                .iter()
                                .any(|token| token.id == *id && token.value == **value)
                    })
                    .cloned()
                    .map(FilterChoice::Value)
                    .collect()
            })
            .unwrap_or_default();
    }
    if !categories.is_empty() {
        categories
            .iter()
            .filter(|category| {
                category.label.to_lowercase().contains(&query)
                    && (!unique || !state.categories.iter().any(|token| token.id == category.id))
            })
            .map(|category| FilterChoice::Category(category.id.clone(), category.label.clone()))
            .collect()
    } else {
        suggestions
            .iter()
            .filter(|value| value.to_lowercase().contains(&query) && !state.tokens.contains(value))
            .cloned()
            .map(FilterChoice::Value)
            .collect()
    }
}
fn apply_filter_choice(
    choice: FilterChoice,
    state: &mut FilterState,
    draft: &mut FilterDraft,
    unique: bool,
    static_operator: Option<LogicalFilterOperator>,
) -> bool {
    match choice {
        FilterChoice::Category(id, _) => {
            draft.input.category = Some(id);
            draft.input.token.clear();
            false
        }
        FilterChoice::Value(value) => {
            let changed = if let Some(id) = &draft.input.category {
                state.add_category(id, value, static_operator.unwrap_or(draft.operator), unique)
            } else {
                state.add_text(value)
            };
            if changed {
                *draft = FilterDraft::default();
            }
            changed
        }
    }
}
fn focus_filter(input: Signal<Option<MountedEvent>>) {
    let node = input.peek().clone();
    if let Some(node) = node {
        spawn(async move {
            let _ = node.set_focus(true).await;
        });
    }
}
fn blur_filter(
    root_id: String,
    mut epoch: Signal<u64>,
    mut open: Signal<bool>,
    mut active: Signal<Option<usize>>,
) {
    let request = epoch.peek().wrapping_add(1);
    epoch.set(request);
    spawn(async move {
        #[cfg(any(feature = "web", feature = "desktop"))]
        let contains_focus = {
            let id = serde_json::to_string(&root_id).expect("string serialization");
            crate::dom_query::eval(&format!(
                "await new Promise(requestAnimationFrame);const root=document.getElementById({id});return !!root&&root.contains(document.activeElement);"
            )).join::<bool>().await.unwrap_or(false)
        };
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        let contains_focus = {
            let _ = root_id;
            false
        };
        if *epoch.peek() == request && !contains_focus {
            open.set(false);
            active.set(None);
        }
    });
}
#[derive(Clone, Copy)]
struct FilterInteraction {
    draft: Signal<FilterDraft>,
    local: Signal<FilterState>,
    open: Signal<bool>,
    active: Signal<Option<usize>>,
    input: Signal<Option<MountedEvent>>,
    unique: bool,
    static_operator: Option<LogicalFilterOperator>,
    onchange: EventHandler<FilterState>,
    oncategory: EventHandler<Option<String>>,
}
fn commit_filter_choice(
    choice: FilterChoice,
    snapshot: &FilterState,
    interaction: FilterInteraction,
) {
    let FilterInteraction {
        mut draft,
        mut local,
        mut open,
        mut active,
        input,
        unique,
        static_operator,
        onchange,
        oncategory,
    } = interaction;
    let mut next = snapshot.clone();
    let mut next_draft = draft();
    let was_category = next_draft.input.category.clone();
    let changed = apply_filter_choice(choice, &mut next, &mut next_draft, unique, static_operator);
    let new_category = next_draft.input.category.clone();
    draft.set(next_draft);
    active.set(None);
    open.set(true);
    if changed {
        local.set(next.clone());
        onchange.call(next);
    }
    if was_category != new_category {
        oncategory.call(new_category);
    }
    focus_filter(input);
}

// A fresh identity per mounted surface prevents an old activation reply from
// opening a subsequently mounted dropdown. All listener/observer ownership is
// tied to this DOM node; neither filter values nor open state live in the bridge.
#[cfg(any(feature = "web", feature = "desktop"))]
const FILTER_POPOVER_BRIDGE: &str = r#"
const [id, anchorId] = await dioxus.recv();
const panel = document.getElementById(id), anchor = document.getElementById(anchorId);
if (!panel || !anchor || !panel.isConnected) return {error:'Surface is not mounted'};
if (typeof panel.showPopover !== 'function') return {error:'Popover API is unavailable'};
let disposed = false, frame = 0;
const events = new AbortController();
const place = () => {
    if (disposed || !panel.isConnected || !anchor.isConnected) { cleanup(); return; }
    const viewport = window.visualViewport;
    const left = viewport?.offsetLeft ?? 0, top = viewport?.offsetTop ?? 0;
    const width = viewport?.width ?? document.documentElement.clientWidth;
    const height = viewport?.height ?? window.innerHeight;
    const edge = 8, gap = 2, rect = anchor.getBoundingClientRect();
    const availableWidth = Math.max(0, width - edge * 2);
    panel.style.width = Math.min(rect.width, availableWidth) + 'px';
    panel.style.maxHeight = '320px';
    const desired = Math.min(320, panel.scrollHeight + 2);
    const below = Math.max(0, top + height - edge - rect.bottom - gap);
    const above = Math.max(0, rect.top - top - edge - gap);
    const flipped = below < desired && above > below;
    const room = Math.min(320, flipped ? above : below);
    panel.style.maxHeight = room + 'px';
    const actual = Math.min(desired, room);
    panel.style.left = Math.max(left + edge, Math.min(rect.left, left + width - edge - panel.offsetWidth)) + 'px';
    panel.style.top = (flipped ? rect.top - gap - actual : rect.bottom + gap) + 'px';
    panel.dataset.placement = flipped ? 'top' : 'bottom';
};
const schedule = () => { if (!frame && !disposed) frame = requestAnimationFrame(() => { frame = 0; place(); }); };
const resize = new ResizeObserver(schedule);
const removal = new MutationObserver(() => { if (!panel.isConnected || !anchor.isConnected) cleanup(); });
function cleanup() {
    if (disposed) return;
    disposed = true; events.abort(); resize.disconnect(); removal.disconnect();
    if (frame) cancelAnimationFrame(frame);
    delete panel.__ixCategoryCleanup;
}
panel.__ixCategoryCleanup = cleanup;
try {
    const source = anchor.querySelector('input[role="combobox"]');
    panel.showPopover(source ? {source} : undefined); place();
    resize.observe(anchor); resize.observe(panel);
    window.addEventListener('resize', schedule, {signal:events.signal});
    document.addEventListener('scroll', schedule, {capture:true,signal:events.signal});
    window.visualViewport?.addEventListener('resize', schedule, {signal:events.signal});
    window.visualViewport?.addEventListener('scroll', schedule, {signal:events.signal});
    panel.addEventListener('toggle', event => { if (event.newState === 'closed') cleanup(); }, {signal:events.signal});
    removal.observe(document.documentElement, {childList:true,subtree:true});
    return {error:null};
} catch (error) { cleanup(); return {error:String(error)}; }
"#;

#[component]
fn FilterOptionsSurface(
    anchor_id: String,
    enable_top_layer: bool,
    onnativeerror: EventHandler<crate::NativeSurfaceError>,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    let surface_id = use_signal(|| pattern_id("category-surface"))();
    let mut native = use_signal(|| enable_top_layer);
    let mut mounted = use_signal(|| None::<MountedEvent>);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let lifetime = alive.clone();
    let cleanup_id = surface_id.clone();
    use_drop(move || {
        lifetime.set(false);
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            let encoded = serde_json::to_string(&cleanup_id).expect("string serialization");
            let _ = dioxus::document::eval(&format!(
                "document.getElementById({encoded})?.__ixCategoryCleanup?.();"
            ));
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        let _ = cleanup_id;
    });
    let mounted_id = surface_id.clone();
    rsx! {
        div { id: surface_id, class: "ix-category-filter-options",
            "popover": native().then_some("auto"), "data-top-layer": native().to_string(),
            onmounted: move |event| {
                mounted.set(Some(event));
                if !enable_top_layer { return; }
                let alive = alive.clone();
                let id = mounted_id.clone();
                let anchor_id = anchor_id.clone();
                spawn(async move {
                    #[cfg(any(feature = "web", feature = "desktop"))]
                    let failure = {
                        #[derive(serde::Deserialize)]
                        struct Reply { error: Option<String> }
                        let bridge = crate::dom_query::eval(FILTER_POPOVER_BRIDGE);
                        let result = bridge.send((id, anchor_id));
                        if let Err(error) = result { Some(crate::NativeSurfaceError::Rejected(error.to_string())) }
                        else { match bridge.join::<Reply>().await {
                            Ok(reply) => reply.error.map(crate::NativeSurfaceError::Rejected),
                            Err(error) => Some(crate::NativeSurfaceError::Rejected(error.to_string())),
                        } }
                    };
                    #[cfg(not(any(feature = "web", feature = "desktop")))]
                    let failure = { let _ = (id, anchor_id); Some(crate::NativeSurfaceError::UnsupportedRenderer) };
                    if alive.get() { if let Some(error) = failure { native.set(false); onnativeerror.call(error); } }
                });
            },
            ontoggle: move |_| {
                // Dioxus alpha.2 does not expose ToggleEvent.newState. Native
                // closure is observed from the event target's mounted bounds.
                let node = mounted.peek().clone();
                spawn(async move {
                    if let Some(node) = node {
                        if let Ok(bounds) = node.get_client_rect().await {
                            if *native.peek() && bounds.size.width == 0.0 && bounds.size.height == 0.0 {
                                onclose.call(());
                            }
                        }
                    }
                });
            },
            {children}
        }
    }
}

/// Native combobox for categorized criteria and free-text tokens. `state` enables
/// controlled usage. Enter commits text; arrows select suggestions; Backspace on
/// an empty input exits a category or removes the last chip. Escape closes options.
/// `enable_top_layer` uses the native Popover API to escape clipping ancestors,
/// with input-anchored viewport placement. Engines without that API retain the
/// inline dropdown and report the limitation through `onnativeerror`.
#[component]
pub fn CategoryFilter(
    #[props(default)] categories: Vec<FilterCategory>,
    #[props(default)] suggestions: Vec<String>,
    #[props(default)] non_selectable_categories: BTreeMap<String, String>,
    #[props(default)] state: Option<FilterState>,
    #[props(default)] default_state: FilterState,
    #[props(default)] onchange: EventHandler<FilterState>,
    #[props(default)] oninput: EventHandler<FilterInputState>,
    #[props(default)] oncategory: EventHandler<Option<String>>,
    #[props(default)] onclear: EventHandler<()>,
    /// Runs before clearing; cancellation preserves committed and draft values.
    #[props(default)]
    onbeforeclear: EventHandler<FilterClearEvent>,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] unique_categories: bool,
    #[props(default)] static_operator: Option<LogicalFilterOperator>,
    #[props(default="Filter by".to_string())] placeholder: String,
    #[props(default="Filter".to_string())] label: String,
    #[props(default="Categories".to_string())] label_categories: String,
    #[props(default="Filter by text".to_string())] label_plain_text: String,
    #[props(default="search".to_string())] icon: String,
    #[props(default)] hide_icon: bool,
    /// Match upstream's opt-in `enableTopLayer` setting.
    #[props(default)]
    enable_top_layer: bool,
    #[props(default)] onnativeerror: EventHandler<crate::NativeSurfaceError>,
) -> Element {
    let mut local = use_signal(|| default_state);
    let snapshot = state.unwrap_or(local());
    let mut draft = use_signal(FilterDraft::default);
    let mut open = use_signal(|| false);
    let mut active = use_signal(|| None::<usize>);
    let mut input = use_signal(|| None::<MountedEvent>);
    let mut operator_button = use_signal(|| None::<MountedEvent>);
    let mut clear_button = use_signal(|| None::<MountedEvent>);
    let mut pointer_focus = use_signal(|| false);
    let mut focus_epoch = use_signal(|| 0u64);
    let mut suppress_focus_open = use_signal(|| false);
    let interaction = FilterInteraction {
        draft,
        local,
        open,
        active,
        input,
        unique: unique_categories,
        static_operator,
        onchange,
        oncategory,
    };
    let list_id = use_signal(|| pattern_id("category-options"))();
    let anchor_id = use_signal(|| pattern_id("category-anchor"))();
    let root_id = format!("{anchor_id}-root");
    let choices = filter_choices(
        &categories,
        &suggestions,
        &snapshot,
        &draft(),
        unique_categories,
    );
    let selected_category = draft().input.category.and_then(|id| {
        categories
            .iter()
            .find(|category| category.id == id)
            .map(|category| category.label.clone())
            .or_else(|| non_selectable_categories.get(&id).cloned())
            .or(Some(id))
    });
    let current_operator = static_operator.unwrap_or(draft().operator);
    let blocked = disabled || readonly;
    let keyboard_choices = choices.clone();
    let keyboard_state = snapshot.clone();
    rsx! {
        div {id:root_id.clone(),class:"ix-category-filter","data-disabled":disabled.to_string(),"data-readonly":readonly.to_string(),
            onfocusin:move |_|{let next=focus_epoch.peek().wrapping_add(1);focus_epoch.set(next);},
            onfocusout:move |_|{pointer_focus.set(false);blur_filter(root_id.clone(),focus_epoch,open,active);},
            onkeydown:move |event|if event.key()==Key::Escape&&open(){event.prevent_default();event.stop_propagation();open.set(false);active.set(None);suppress_focus_open.set(true);focus_filter(input);},
            div {id:anchor_id.clone(),class:"ix-category-filter-input",
                if !hide_icon {Icon {name:icon,size:16}}
                div {class:"ix-category-filter-tokens",
                    for (index,token) in snapshot.tokens.iter().enumerate() {
                        span {class:"ix-category-filter-chip",span {"{token}"}
                            if !readonly {button {r#type:"button",disabled,"aria-label":"Remove {token}",onclick:{let snapshot=snapshot.clone();move |_|{let mut next=snapshot.clone();next.tokens.remove(index);local.set(next.clone());onchange.call(next);focus_filter(input);}},Icon {name:"close",size:12}}}
                        }
                    }
                    for (index,token) in snapshot.categories.iter().enumerate() {
                        span {class:"ix-category-filter-chip",
                            span {"{categories.iter().find(|category|category.id==token.id).map(|category|category.label.as_str()).or_else(||non_selectable_categories.get(&token.id).map(String::as_str)).unwrap_or(&token.id)} {token.operator.symbol()} {token.value}"}
                            if !readonly {button {r#type:"button",disabled,"aria-label":"Remove {token.value}",onclick:{let snapshot=snapshot.clone();move |_|{let mut next=snapshot.clone();next.categories.remove(index);local.set(next.clone());onchange.call(next);focus_filter(input);}},Icon {name:"close",size:12}}}
                        }
                    }
                    if let Some(category)=selected_category.clone() {span {class:"ix-category-filter-preview","{category}"}}
                    input {r#type:"text",role:"combobox",class:"ix-category-filter-text",disabled,readonly,value:draft().input.token,placeholder:if blocked||selected_category.is_some(){String::new()}else{placeholder},"aria-label":label,"aria-expanded":open()&&!blocked,"aria-autocomplete":"list","aria-controls":list_id.clone(),"aria-activedescendant":if open() {active().filter(|index|*index<choices.len()).map(|index|format!("{list_id}-{index}"))} else {None},
                        onmounted:move |event|input.set(Some(event)),
                        // Focus during pointerdown precedes native light-dismiss.
                        // Open after click instead; keyboard/programmatic focus
                        // opens immediately without waiting for a pointer event.
                        onpointerdown:move |_|if !blocked{pointer_focus.set(true);},
                        onpointerup:move |_|pointer_focus.set(false),
                        onpointercancel:move |_|pointer_focus.set(false),
                        onclick:move |_|if !blocked{pointer_focus.set(false);open.set(true);},
                        onfocus:move |_|if !blocked{if suppress_focus_open(){suppress_focus_open.set(false);}else if pointer_focus(){pointer_focus.set(false);}else{open.set(true);}},
                        oninput:move |event|{if blocked{return;}let mut next=draft();next.input.token=event.value();oninput.call(next.input.clone());draft.set(next);active.set(None);open.set(true);},
                        onkeydown:move |event:KeyboardEvent|{
                            if blocked{return;}
                            match event.key(){
                                Key::ArrowDown=>{event.prevent_default();open.set(true);if !keyboard_choices.is_empty(){active.set(Some(active().map_or(0,|index|(index+1)%keyboard_choices.len())));}},
                                Key::ArrowUp=>{event.prevent_default();open.set(true);if !keyboard_choices.is_empty(){active.set(Some(active().map_or(keyboard_choices.len()-1,|index|(index+keyboard_choices.len()-1)%keyboard_choices.len())));}},
                                Key::Escape=>{event.prevent_default();event.stop_propagation();open.set(false);active.set(None);},
                                Key::Tab if !event.modifiers().shift()&&open()&&draft().input.category.is_some()&&static_operator.is_none()&&operator_button.peek().is_some()=>{event.prevent_default();focus_filter(operator_button);},
                                Key::Enter=>{event.prevent_default();let choice=if open(){active().and_then(|index|keyboard_choices.get(index).cloned())}else{None}.unwrap_or_else(||FilterChoice::Value(draft().input.token));commit_filter_choice(choice,&keyboard_state,interaction);},
                                Key::Backspace if draft().input.token.is_empty()=>{if draft().input.category.is_some(){let mut next=draft();next.input.category=None;draft.set(next);oncategory.call(None);}else{let mut next=keyboard_state.clone();if next.remove_last(){local.set(next.clone());onchange.call(next);}}},
                                _=>{}
                            }
                        }
                    }
                }
                if !blocked&&(!snapshot.is_empty()||selected_category.is_some()){button {class:"ix-category-filter-reset",r#type:"button",disabled:blocked,"aria-label":"Clear filters",
                    onmounted:move |event|clear_button.set(Some(event)),
                    onkeydown:move |event|if event.key()==Key::Tab&&event.modifiers().shift()&&open()&&draft().input.category.is_some()&&static_operator.is_none()&&operator_button.peek().is_some(){event.prevent_default();focus_filter(operator_button);},
                    onclick:move |_|{let request=FilterClearEvent::default();onbeforeclear.call(request.clone());if request.default_prevented(){return;}local.set(FilterState::default());draft.set(FilterDraft::default());open.set(false);active.set(None);onchange.call(FilterState::default());oncategory.call(None);onclear.call(());focus_filter(input);},Icon {name:"clear",size:24}}}
            }
            if open()&&!blocked {
                FilterOptionsSurface { anchor_id:anchor_id.clone(), enable_top_layer, onnativeerror, onclose:move |_|{open.set(false);active.set(None);},
                    div {class:"ix-category-filter-options-heading",
                        span {if let Some(category)=selected_category.clone() {"{category}"}else if categories.is_empty(){"{label_plain_text}"}else{"{label_categories}"}}
                        if draft().input.category.is_some()&&static_operator.is_none(){button {r#type:"button","aria-label":"Toggle comparison operator","aria-pressed":current_operator==LogicalFilterOperator::NotEqual,
                            onmounted:move |event|operator_button.set(Some(event)),
                            onkeydown:move |event|if event.key()==Key::Tab{if event.modifiers().shift(){event.prevent_default();focus_filter(input);}else if clear_button.peek().is_some(){event.prevent_default();focus_filter(clear_button);}},
                            onmousedown:move |event|event.prevent_default(),onclick:move |_|{let mut next=draft();next.operator=next.operator.toggled();draft.set(next);focus_filter(input);},"{current_operator.toggled().symbol()}"}}
                    }
                    div {id:list_id.clone(),role:"listbox","aria-label":"Filter options",
                        for (index,choice) in choices.iter().enumerate(){
                            div {id:format!("{list_id}-{index}"),role:"option",class:"ix-category-filter-option","aria-selected":active()==Some(index),onmousedown:move |event|event.prevent_default(),onclick:{let choice=choice.clone();let snapshot=snapshot.clone();move |_|commit_filter_choice(choice.clone(),&snapshot,interaction)},
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
.ix-category-filter{position:relative;min-width:0;width:100%;font:inherit}.ix-category-filter-input{display:flex;align-items:flex-start;gap:8px;min-height:40px;padding:4px 8px;border:1px solid var(--ix-color-border);border-bottom-color:var(--ix-color-text-secondary);border-radius:var(--ix-radius) var(--ix-radius) 0 0;background:var(--ix-color-surface);color:var(--ix-color-text)}.ix-category-filter-input>.ix-icon{margin-top:7px;color:var(--ix-color-primary)}.ix-category-filter-input:focus-within{border-color:var(--ix-color-primary);box-shadow:inset 0 -1px var(--ix-color-primary)}.ix-category-filter-tokens{display:flex;flex:1;min-width:0;flex-wrap:wrap;align-items:center;gap:4px;max-height:60px;overflow-y:auto}.ix-category-filter-text{flex:1;min-width:120px;height:30px;padding:2px 0;border:0;background:transparent;color:inherit;outline:none!important;font:inherit}.ix-category-filter-text::placeholder{color:var(--ix-color-text-secondary);opacity:1}.ix-category-filter-chip{display:inline-flex;align-items:center;gap:4px;padding:2px 8px;max-width:100%;min-height:24px;background:var(--ix-color-surface-2);color:var(--ix-color-text);border-radius:12px;font-size:12px;line-height:20px}.ix-category-filter-chip>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-category-filter-chip>button{display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;min-width:16px;background:transparent;color:inherit;border:0;padding:0;border-radius:50%;cursor:pointer}.ix-category-filter-chip>button:hover{background:color-mix(in srgb,currentColor 15%,transparent)}.ix-category-filter-preview{display:inline-flex;align-items:center;min-height:28px;padding:0 8px;background:var(--ix-color-surface-2);white-space:nowrap}.ix-category-filter-reset{display:inline-flex;align-items:center;justify-content:center;width:30px;height:30px;min-width:30px;padding:0;border:0;background:transparent;color:var(--ix-color-primary);border-radius:var(--ix-radius);cursor:pointer}.ix-category-filter-reset:hover{background:var(--ix-color-surface-2)}.ix-category-filter-options{position:absolute;inset-inline:0;top:100%;z-index:30;max-height:320px;overflow:auto;padding:4px 0;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);box-shadow:0 8px 20px #0003}.ix-category-filter-options[data-top-layer=true]{position:fixed;inset:auto;margin:0;box-sizing:border-box;max-width:calc(100vw - 16px)}.ix-category-filter-options-heading{display:flex;align-items:center;justify-content:space-between;gap:8px;padding:4px 12px;color:var(--ix-color-text-secondary);font-size:12px;line-height:20px}.ix-category-filter-options-heading>button{font:inherit;background:transparent;border:0;border-radius:var(--ix-radius);color:var(--ix-color-primary);cursor:pointer;min-width:32px;min-height:24px}.ix-category-filter-option{padding:6px 12px;min-height:32px;cursor:pointer;color:var(--ix-color-text)}.ix-category-filter-option:hover,.ix-category-filter-option[aria-selected=true]{background:var(--ix-color-surface-2)}.ix-category-filter-no-options{padding:8px 12px;font-size:12px;color:var(--ix-color-text-secondary)}.ix-category-filter[data-disabled=true]{opacity:.45}.ix-category-filter[data-disabled=true] *{cursor:not-allowed}.ix-category-filter[data-readonly=true] .ix-category-filter-input{background:transparent;border-color:var(--ix-color-border)}.ix-category-filter button:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}
.ix-flip-tile{--ix-flip-accent:var(--ix-color-border);--ix-flip-footer-bg:var(--ix-color-surface-1);--ix-flip-footer-color:var(--ix-color-text);perspective:1000px;max-width:100%;min-height:100px;display:flex;flex-direction:column;border:1px solid var(--ix-flip-accent);border-radius:var(--ix-radius) var(--ix-radius) 0 0;overflow:visible}.ix-flip-tile-container{height:100%;display:flex;flex-direction:column;background:var(--ix-color-surface-1);color:var(--ix-color-text);transform-style:preserve-3d;backface-visibility:hidden;border-radius:inherit}.ix-flip-tile-header{display:flex;align-items:center;min-height:40px;padding:0 8px 0 16px;gap:8px}.ix-flip-tile-header>div{flex:1;min-width:0;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-flip-tile-toggle{display:inline-flex;align-items:center;justify-content:center;width:32px;height:32px;flex-shrink:0;border:0;border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-primary);padding:4px;cursor:pointer}.ix-flip-tile-toggle:hover:not(:disabled){background:var(--ix-color-surface-2)}.ix-flip-tile-toggle:disabled{opacity:.4;cursor:default}.ix-flip-tile-toggle:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:0}.ix-flip-tile-content{flex:1;min-height:0;overflow:auto;margin:16px}.ix-flip-tile-side{display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100%;min-width:0}.ix-flip-tile-footer{display:flex;align-items:center;justify-content:center;min-height:48px;padding:4px 8px;border-top:1px solid var(--ix-flip-accent);background:var(--ix-flip-footer-bg);color:var(--ix-flip-footer-color);text-align:center;font-size:14px;line-height:20px}.ix-flip-tile-filled{--ix-flip-accent:transparent}.ix-flip-tile-outline .ix-flip-tile-container{background:transparent}.ix-flip-tile-primary{--ix-flip-accent:var(--ix-color-primary);--ix-flip-footer-bg:var(--ix-color-primary);--ix-flip-footer-color:var(--ix-color-on-primary)}.ix-flip-tile-info{--ix-flip-accent:var(--ix-color-info);--ix-flip-footer-bg:var(--ix-color-info);--ix-flip-footer-color:var(--ix-color-on-info,#000028)}.ix-flip-tile-warning{--ix-flip-accent:var(--ix-color-warning);--ix-flip-footer-bg:var(--ix-color-warning);--ix-flip-footer-color:var(--ix-color-on-warning,#000028)}.ix-flip-tile-alarm{--ix-flip-accent:var(--ix-color-danger);--ix-flip-footer-bg:var(--ix-color-danger);--ix-flip-footer-color:var(--ix-color-on-danger,#fff)}.ix-flip-tile-container[data-phase=outgoing]{animation:ix-pattern-flip-out .15s ease-in forwards}.ix-flip-tile-container[data-phase=incoming]{animation:ix-pattern-flip-in .15s ease-out both}@keyframes ix-pattern-flip-out{from{transform:rotateY(0)}to{transform:rotateY(90deg)}}@keyframes ix-pattern-flip-in{from{transform:rotateY(270deg)}to{transform:rotateY(360deg)}}
.ix-info-page{display:flex;width:100%;min-height:100%;padding:32px 16px}.ix-info-page-content{display:flex;flex-direction:column;align-items:center;max-width:800px;margin:auto;text-align:center}.ix-info-page-image{display:flex;align-items:flex-end;justify-content:center;width:260px;height:196px;max-width:100%;margin-bottom:40px}.ix-info-page-image>*{max-width:100%;max-height:100%;object-fit:contain}.ix-info-page-message{display:flex;flex-direction:column;align-items:center;max-width:640px}.ix-info-page-message h1{margin:0;font-size:32px;line-height:40px;font-weight:600;text-wrap:balance}.ix-info-page-copy{margin:16px 0 0;font-size:16px;line-height:24px}.ix-info-page-instructions{margin:16px 0 0;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary)}.ix-info-page-actions{display:flex;flex-wrap:wrap;justify-content:center;gap:8px;width:100%;margin-top:32px}
.ix-css-grid{display:grid;position:relative;gap:16px;min-width:0}.ix-grid-area{min-width:0}.ix-layout-auto{--ix-layout-columns:1;--ix-layout-gap:24px;display:flex;flex-wrap:wrap;align-items:stretch;gap:var(--ix-layout-gap);margin:0 12px}.ix-layout-auto>*{flex:0 0 auto;min-width:0;width:calc((100% + var(--ix-layout-gap)) / var(--ix-layout-columns) * min(var(--ix-colspan,1),var(--ix-layout-columns)) - var(--ix-layout-gap))}
@media(max-width:576px){.ix-info-page{padding:24px 16px}.ix-info-page-message h1{font-size:28px;line-height:36px}}
@media(prefers-reduced-motion:reduce){.ix-flip-tile-container[data-phase=outgoing],.ix-flip-tile-container[data-phase=incoming]{animation-duration:.01ms}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn top_layer_fixture() -> Element {
        rsx! {
            FilterOptionsSurface {anchor_id:"first-input",enable_top_layer:true,onnativeerror:move |_|{},onclose:move |_|{},div {role:"listbox","First choices"}}
            FilterOptionsSurface {anchor_id:"second-input",enable_top_layer:true,onnativeerror:move |_|{},onclose:move |_|{},div {role:"listbox","Second choices"}}
            FilterOptionsSurface {anchor_id:"inline-input",enable_top_layer:false,onnativeerror:move |_|{},onclose:move |_|{},div {role:"listbox","Inline choices"}}
        }
    }
    #[test]
    fn category_options_keep_scoped_native_surfaces_and_an_inline_mode() {
        let mut dom = VirtualDom::new(top_layer_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("popover=\"auto\"").count(), 2);
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
    fn category_choice_enters_category_then_commits_value_and_resets_draft() {
        let mut state = FilterState::default();
        let mut draft = FilterDraft::default();
        assert!(!apply_filter_choice(
            FilterChoice::Category("vendor".into(), "Vendor".into()),
            &mut state,
            &mut draft,
            false,
            None
        ));
        draft.operator = LogicalFilterOperator::NotEqual;
        assert!(apply_filter_choice(
            FilterChoice::Value("MS".into()),
            &mut state,
            &mut draft,
            false,
            Some(LogicalFilterOperator::Equal)
        ));
        assert_eq!(state.categories[0].operator, LogicalFilterOperator::Equal);
        assert_eq!(draft, FilterDraft::default());
        assert!(state.remove_last());
        assert!(state.is_empty());
    }
    #[test]
    fn suggestion_filtering_handles_case_unique_categories_and_duplicate_values() {
        let mut state = FilterState::default();
        state.add_category("vendor", "Apple", LogicalFilterOperator::Equal, false);
        assert_eq!(
            filter_choices(&categories(), &[], &state, &FilterDraft::default(), true),
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
            filter_choices(&categories(), &[], &state, &draft, false),
            vec![FilterChoice::Value("APS".into())]
        );
        state.add_text("Item 1");
        assert_eq!(
            filter_choices(
                &[],
                &["Item 1".into(), "Item 2".into()],
                &state,
                &FilterDraft::default(),
                false
            ),
            vec![FilterChoice::Value("Item 2".into())]
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
        assert!(html.contains("role=\"combobox\""));
        assert!(html.contains("Token"));
        assert!(html.contains("Visible side"));
        assert!(!html.contains("Hidden side"));
        assert!(html.contains("Return to overview"));
    }
}
