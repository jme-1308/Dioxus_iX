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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
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
                    Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsVector2D>,
                >,
        >,
    > {
        Box::pin(async { Ok(dioxus::html::geometry::PixelsVector2D::new(0.0, 0.0)) })
    }
    fn scroll(
        &self,
        point: dioxus::html::geometry::PixelsVector2D,
        behavior: dioxus::html::ScrollBehavior,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
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
fn category_programmatic_empty_retains_draft_but_accepted_tokens_and_reentrant_input_clear_it() {
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
fn category_programmatic_controller_lifecycle_shared_mounts_swaps_and_reentrant_notifications() {
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
        let mut harness =
            FilterProgrammaticHarness::new(CategoryFilterController::default(), None, true, accept);
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
    let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new((
        key, modifiers,
    ))));
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
fn category_order_controlled_acknowledges_latest_order_and_preserves_accepted_order_on_rejection() {
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
fn category_keyboard_native_buttons_allow_activation_and_wrap_only_popup_boundaries() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = CategoryModeHarness::new(Some(categories()));
    let capture = harness.config.borrow().keyboard.clone();
    let options = harness.open();
    let ids = filter_native_option_ids(&harness.html());
    harness.config.borrow_mut().events.clear();
    let before_activation = harness.html();
    for key in [Key::Enter, Key::Character(" ".into())] {
        assert_eq!(filter_key_at(&harness.dom, options[0], key), (false, true));
        harness.settle();
        assert_eq!(harness.html(), before_activation);
        assert!(harness.config.borrow().events.is_empty());
        assert!(capture.request().is_none());
    }
    for key in [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End] {
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
    assert_eq!(
        filter_key_at(&harness.dom, options[0], Key::Enter),
        (false, true)
    );
    // SSR dispatch does not emulate the browser's default button click.
    assert!(harness.config.borrow().events.is_empty());
    let native = harness.click(options[0]);
    assert_eq!(harness.config.borrow().events, ["category"]);
    assert!(harness.config.borrow().changes.is_empty());
    let operator = native[0];
    let focus = Rc::new(Cell::new(0_usize));
    dispatch_filter_event(&harness.dom, "mounted", harness.input, focus.clone());
    harness.settle();
    let before_focus = focus.get();
    harness.config.borrow_mut().events.clear();
    for (key, pressed) in [(Key::Enter, true), (Key::Character(" ".into()), false)] {
        let before_activation = harness.html();
        assert_eq!(filter_key_at(&harness.dom, operator, key), (false, true));
        harness.settle();
        assert_eq!(harness.html(), before_activation);
        harness.click(operator);
        assert!(
            harness
                .html()
                .contains(&format!("aria-pressed=\"{pressed}\""))
        );
        assert_eq!(
            focus.get(),
            before_focus,
            "operator activation does not request input focus"
        );
        assert!(harness.config.borrow().events.is_empty());
    }
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

    let native = harness.open();
    assert_eq!(
        filter_key_at(&harness.dom, native[1], Key::Character(" ".into())),
        (false, true)
    );
    harness.settle();
    assert!(harness.config.borrow().events.is_empty());
    assert!(harness.config.borrow().changes.is_empty());
    harness.click(native[1]);
    assert_eq!(harness.config.borrow().events, ["change"]);
    assert_eq!(
        harness.config.borrow().changes,
        [FilterState {
            tokens: vec!["Committed mode token".into()],
            categories: vec![CategoryToken {
                id: "vendor".into(),
                value: "Apple".into(),
                operator: LogicalFilterOperator::Equal,
            }],
        }]
    );
}

#[test]
fn category_keyboard_plain_button_defers_activation_to_one_click() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = CategoryModeHarness::new(None);
    let options = harness.open();
    let before_activation = harness.html();
    for key in [Key::Enter, Key::Character(" ".into())] {
        assert_eq!(filter_key_at(&harness.dom, options[0], key), (false, true));
        harness.settle();
        assert_eq!(harness.html(), before_activation);
        assert!(harness.config.borrow().events.is_empty());
        assert!(harness.config.borrow().changes.is_empty());
    }
    harness.click(options[0]);
    assert_eq!(harness.config.borrow().events, ["change"]);
    assert_eq!(
        harness.config.borrow().changes,
        [FilterState {
            tokens: vec!["Committed mode token".into(), "Shared".into()],
            categories: vec![],
        }]
    );
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
fn category_modes_plain_suggestion_click_uses_retained_duplicate_context_but_inserts_free_text() {
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
