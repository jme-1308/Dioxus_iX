use super::*;
#[derive(Clone, Default)]
struct OwnerCapture(Rc<RefCell<Option<CustomFieldOwner>>>);
#[derive(Clone)]
struct Composition {
    first: crate::FieldState,
    second: crate::FieldState,
    outer: crate::FieldState,
    nested_state: crate::FieldState,
    first_visible: bool,
    visible: bool,
    nested: bool,
    key: u64,
    root_class: String,
    capture: OwnerCapture,
}
impl Default for Composition {
    fn default() -> Self {
        Self {
            first: crate::FieldState::Default,
            second: crate::FieldState::Default,
            outer: crate::FieldState::Default,
            nested_state: crate::FieldState::Default,
            first_visible: true,
            visible: true,
            nested: false,
            key: 0,
            root_class: String::new(),
            capture: OwnerCapture::default(),
        }
    }
}
#[component]
fn ContributedControl(state: crate::FieldState, #[props(default)] capture: bool) -> Element {
    use_custom_field_state(state);
    if capture {
        let output = consume_context::<OwnerCapture>();
        *output.0.borrow_mut() = try_consume_context::<CustomFieldOwner>();
    }
    rsx! {input {"data-custom-contribution":"true"}}
}
fn composition_fixture(config: Rc<RefCell<Composition>>) -> Element {
    let props = config.borrow().clone();
    let capture = props.capture.clone();
    use_context_provider(|| capture);
    rsx! {
        if props.visible {
            crate::CustomField {class:format!("composition-outer {}",props.root_class),state:props.outer,
                helper_text:"Outer helper",invalid_text:"Outer invalid",warning_text:"Outer warning",info_text:"Outer info",valid_text:"Outer valid",
                if props.first_visible {ContributedControl {key:"{props.key}",state:props.first,capture:true}}
                if props.nested {
                    crate::CustomField {class:"composition-inner",state:props.nested_state,
                        invalid_text:"Inner invalid",warning_text:"Inner warning",helper_text:"Inner helper",
                        ContributedControl {state:props.second}
                    }
                } else {ContributedControl {state:props.second}}
                div {class:"ix-invalid","Unregistered DOM content"}
            }
        }
        crate::CustomField {class:"composition-sibling",valid_text:"Sibling valid",ContributedControl {state:crate::FieldState::Valid}}
    }
}
fn wrapper_state(dom: &VirtualDom, name: &str) -> crate::FieldState {
    let html = dioxus_ssr::render(dom);
    let after = html
        .split_once(&format!("class=\"ix-custom-field {name}"))
        .unwrap()
        .1;
    let class = after
        .split_once("class=\"ix-field")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    validation_class_state(class)
}
fn settle_field(dom: &mut VirtualDom) {
    for _ in 0..8 {
        dom.render_immediate_to_vec();
    }
}
fn update_field(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    settle_field(dom);
}
#[test]
fn custom_field_composition_updates_precedence_default_and_dropped_contributions() {
    use crate::FieldState::*;
    let config = Rc::new(RefCell::new(Composition {
        first: Valid,
        second: Warning,
        ..Composition::default()
    }));
    let mut dom = VirtualDom::new_with_props(composition_fixture, config.clone());
    dom.rebuild_in_place();
    settle_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Warning);
    config.borrow_mut().first = Invalid;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Invalid);
    config.borrow_mut().first = Default;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Warning);
    config.borrow_mut().second = Info;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Info);
    config.borrow_mut().first = Invalid;
    update_field(&mut dom);
    config.borrow_mut().first_visible = false;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Info);
    config.borrow_mut().second = Default;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Default);
    {
        let mut config = config.borrow_mut();
        config.first_visible = true;
        config.first = Valid;
        config.key += 1;
    }
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Valid);
    {
        let mut config = config.borrow_mut();
        config.first = Default;
        config.key += 1;
    }
    update_field(&mut dom);
    assert_eq!(
        wrapper_state(&dom, "composition-outer"),
        Default,
        "presentation helpers cannot retain their own old state"
    );
    assert_eq!(wrapper_state(&dom, "composition-sibling"), Valid);
}
#[test]
fn nested_custom_field_reports_effective_explicit_state_without_self_feedback() {
    use crate::FieldState::*;
    let config = Rc::new(RefCell::new(Composition {
        second: Invalid,
        nested: true,
        nested_state: Warning,
        ..Composition::default()
    }));
    let mut dom = VirtualDom::new_with_props(composition_fixture, config.clone());
    dom.rebuild_in_place();
    settle_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-inner"), Warning);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Warning);
    config.borrow_mut().outer = Info;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Info);
    config.borrow_mut().nested_state = Default;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-inner"), Invalid);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Info);
    config.borrow_mut().outer = Default;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Invalid);
    config.borrow_mut().second = Default;
    update_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-inner"), Default);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Default);
    assert_eq!(wrapper_state(&dom, "composition-sibling"), Valid);
}
#[test]
fn custom_field_remount_rejects_old_owner_and_unregistered_dom_has_no_fallback() {
    use crate::FieldState::*;
    let config = Rc::new(RefCell::new(Composition {
        root_class: "ix-warning".into(),
        ..Composition::default()
    }));
    let mut dom = VirtualDom::new_with_props(composition_fixture, config.clone());
    dom.rebuild_in_place();
    settle_field(&mut dom);
    assert_eq!(
        wrapper_state(&dom, "composition-outer"),
        Warning,
        "own Rust class is observed directly"
    );
    config.borrow_mut().root_class.clear();
    update_field(&mut dom);
    assert_eq!(
        wrapper_state(&dom, "composition-outer"),
        Default,
        "unregistered descendant class is not a contribution"
    );
    let old = config.borrow().capture.0.borrow().clone().unwrap();
    config.borrow_mut().visible = false;
    update_field(&mut dom);
    assert!(!old.alive.get());
    {
        let mut config = config.borrow_mut();
        config.visible = true;
        config.first = Valid;
    }
    update_field(&mut dom);
    old.publish(usize::MAX, Invalid);
    settle_field(&mut dom);
    assert_eq!(wrapper_state(&dom, "composition-outer"), Valid);
    assert_eq!(wrapper_state(&dom, "composition-sibling"), Valid);
}
#[derive(Clone)]
struct BuiltinState {
    kind: u8,
    state: crate::FieldState,
}
fn builtin_state_fixture(config: Rc<RefCell<BuiltinState>>) -> Element {
    let props = config.borrow().clone();
    let state = props.state;
    let content = match props.kind {
        0 => rsx! {crate::Input {state}},
        1 => rsx! {crate::Textarea {state}},
        2 => rsx! {crate::NumberInput {state}},
        3 => rsx! {crate::Checkbox {state,required:true,aria_label:"Checkbox"}},
        4 => rsx! {crate::Toggle {state,required:true}},
        5 => rsx! {crate::Radio {state,value:"radio",aria_label:"Radio"}},
        6 => {
            rsx! {crate::CheckboxGroup {state,aria_label:"Group",aria_labelledby:None::<String>,"Choices"}}
        }
        7 => {
            rsx! {crate::RadioGroup {state,options:Vec::new(),aria_label:"Radios",aria_labelledby:None::<String>}}
        }
        8 => rsx! {crate::Select {state,options:Vec::new()}},
        _ => rsx! {crate::Slider {state}},
    };
    rsx! {crate::CustomField {class:"builtin",helper_text:"Default",warning_text:"Warning",{content}}}
}
#[test]
fn builtin_controls_publish_current_effective_state_and_clear_without_observers() {
    for kind in 0..10 {
        let config = Rc::new(RefCell::new(BuiltinState {
            kind,
            state: crate::FieldState::Warning,
        }));
        let mut dom = VirtualDom::new_with_props(builtin_state_fixture, config.clone());
        dom.rebuild_in_place();
        settle_field(&mut dom);
        assert_eq!(
            wrapper_state(&dom, "builtin"),
            crate::FieldState::Warning,
            "builtin {kind}"
        );
        config.borrow_mut().state = crate::FieldState::Default;
        update_field(&mut dom);
        assert_eq!(
            wrapper_state(&dom, "builtin"),
            crate::FieldState::Default,
            "builtin {kind} must not self-retain feedback"
        );
    }
}

#[derive(Clone)]
struct FocusReply {
    attempts: Rc<Cell<usize>>,
    focused: Rc<Cell<bool>>,
    confirms: Rc<Cell<bool>>,
    ready: Rc<Cell<bool>>,
}
impl dioxus::html::RenderedElementBacking for FocusReply {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let state = self.clone();
        Box::pin(async move {
            state.attempts.set(state.attempts.get() + 1);
            std::future::poll_fn(|_| {
                if state.ready.get() {
                    std::task::Poll::Ready(())
                } else {
                    std::task::Poll::Pending
                }
            })
            .await;
            if state.confirms.get() {
                state.focused.set(true);
            }
            Err(dioxus::html::MountedError::OperationFailed(Box::new(
                std::io::Error::other("focus response unavailable"),
            )))
        })
    }
}
fn poll_focus<T>(
    future: std::pin::Pin<&mut impl std::future::Future<Output = T>>,
) -> std::task::Poll<T> {
    future.poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
}
fn ready_focus<T>(future: impl std::future::Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    match poll_focus(future.as_mut()) {
        std::task::Poll::Ready(result) => result,
        std::task::Poll::Pending => panic!("expected one completed focus attempt"),
    }
}
fn response_binding() -> (Binding, FocusReply) {
    let state = FocusReply {
        attempts: Rc::default(),
        focused: Rc::default(),
        confirms: Rc::new(Cell::new(true)),
        ready: Rc::new(Cell::new(true)),
    };
    let binding = Binding {
        id: "input".into(),
        alive: Rc::new(Cell::new(true)),
        touched: Rc::default(),
        has_value: true,
        node: Rc::new(RefCell::new(Some(Rc::new(MountedData::new(state.clone()))))),
        focused: state.focused.clone(),
        focusable: Rc::new(Cell::new(true)),
    };
    (binding, state)
}
#[test]
fn text_focus_uses_one_current_mount_attempt_and_requires_confirmation_of_errors() {
    let (binding, state) = response_binding();
    let descriptor = NativeTextControl {
        node: binding.node.borrow().clone().unwrap(),
        binding: binding.clone(),
    };
    assert_eq!(ready_focus(descriptor.focus()), Ok(()));
    assert_eq!(state.attempts.get(), 1);
    state.confirms.set(false);
    assert_eq!(
        ready_focus(descriptor.focus()),
        Ok(()),
        "already focused input still makes exactly one attempt"
    );
    assert_eq!(state.attempts.get(), 2);
    state.focused.set(false);
    assert!(matches!(
        ready_focus(descriptor.focus()),
        Err(TextControlError::Dom(_))
    ));
    assert_eq!(state.attempts.get(), 3, "unconfirmed error does not retry");
    binding.focusable.set(false);
    state.focused.set(true);
    assert!(matches!(
        ready_focus(descriptor.focus()),
        Err(TextControlError::Dom(_))
    ));
    assert_eq!(
        state.attempts.get(),
        3,
        "disabled/hidden controls do not issue ineffective focus commands"
    );
    binding.focusable.set(true);
    binding
        .node
        .borrow_mut()
        .replace(Rc::new(MountedData::new(())));
    assert_eq!(
        ready_focus(descriptor.focus()),
        Err(TextControlError::NotMounted)
    );
    assert_eq!(state.attempts.get(), 3);
}
#[test]
fn text_focus_pending_response_rejects_replaced_or_dropped_mounts() {
    let (binding, state) = response_binding();
    let descriptor = NativeTextControl {
        node: binding.node.borrow().clone().unwrap(),
        binding: binding.clone(),
    };
    state.ready.set(false);
    let mut future = std::pin::pin!(descriptor.focus());
    assert!(poll_focus(future.as_mut()).is_pending());
    binding
        .node
        .borrow_mut()
        .replace(Rc::new(MountedData::new(())));
    state.ready.set(true);
    assert_eq!(
        poll_focus(future.as_mut()),
        std::task::Poll::Ready(Err(TextControlError::NotMounted))
    );
    assert_eq!(state.attempts.get(), 1);
    let (binding, state) = response_binding();
    let descriptor = NativeTextControl {
        node: binding.node.borrow().clone().unwrap(),
        binding: binding.clone(),
    };
    state.ready.set(false);
    let mut future = std::pin::pin!(descriptor.focus());
    assert!(poll_focus(future.as_mut()).is_pending());
    binding.alive.set(false);
    state.ready.set(true);
    assert_eq!(
        poll_focus(future.as_mut()),
        std::task::Poll::Ready(Err(TextControlError::NotMounted))
    );
    assert_eq!(state.attempts.get(), 1);
}

struct FocusEventData;
impl dioxus::html::HasFocusData for FocusEventData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct FocusConverter;
macro_rules! focus_converter {
        (enum Event {$(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*}) => {
            impl dioxus::html::HtmlEventConverter for FocusConverter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;focus_converter!(@data $group,event)})*
            }
        };
        (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(FocusEventData)};
        (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<FocusReply>().unwrap().clone())};
        (@data $other:ident,$event:ident)=>{panic!("Unexpected text focus test event")};
    }
dioxus::html::with_html_event_groups!(focus_converter);
type ComponentFocusFixture = (u8, TextControlController, Rc<RefCell<Vec<bool>>>);
fn component_focus_fixture((kind, controller, results): ComponentFocusFixture) -> Element {
    let focus_controller = controller.clone();
    let focus_results = results.clone();
    let onfocus = use_callback(move |_: FocusEvent| {
        focus_results
            .borrow_mut()
            .push(ready_focus(focus_controller.focus_input()).is_ok())
    });
    let blur_controller = controller.clone();
    let onixblur = use_callback(move |()| {
        results
            .borrow_mut()
            .push(ready_focus(blur_controller.focus_input()).is_ok())
    });
    match kind {
        0 => {
            rsx! {crate::Input {id:"method-control",controller,readonly:true,onfocus,onixblur}}
        }
        1 => {
            rsx! {crate::Textarea {id:"method-control",controller,readonly:true,onfocus,onixblur}}
        }
        _ => {
            rsx! {crate::NumberInput {id:"method-control",controller,readonly:true,onfocus,onixblur}}
        }
    }
}
fn dispatch_focus_event(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: impl std::any::Any,
) {
    let data: Rc<dyn std::any::Any> =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
    dom.runtime()
        .handle_event(name, Event::new(data, false), id);
}
#[test]
fn input_textarea_number_confirm_typed_focus_before_consumers_and_clear_before_blur() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(FocusConverter));
    for kind in 0..3 {
        let controller = TextControlController::default();
        let results = Rc::new(RefCell::new(Vec::new()));
        let mut dom = VirtualDom::new_with_props(
            component_focus_fixture,
            (kind, controller.clone(), results.clone()),
        );
        let mutations = dom.rebuild_to_vec();
        let mut current = dioxus::core::ElementId::ROOT;
        let id = mutations
            .edits
            .iter()
            .find_map(|edit| match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = *id;
                    None
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "id" && value == "method-control" => Some(current),
                _ => None,
            })
            .expect("current native text element");
        let (_, reply) = response_binding();
        reply.confirms.set(false);
        dispatch_focus_event(&dom, "mounted", id, reply.clone());
        let descriptor = controller.get_native_input_element().unwrap();
        dispatch_focus_event(&dom, "focus", id, ());
        dispatch_focus_event(&dom, "blur", id, ());
        assert_eq!(
            results.borrow().as_slice(),
            [true, false],
            "typed state precedes consumer callbacks for control kind {kind}"
        );
        assert_eq!(
            reply.attempts.get(),
            2,
            "readonly controls retain the one-attempt focus API"
        );
        drop(dom);
        assert_eq!(
            ready_focus(descriptor.focus()),
            Err(TextControlError::NotMounted)
        );
        assert_eq!(reply.attempts.get(), 2);
    }
}

#[test]
fn native_constraints_are_distinct_from_required_nonempty_queries() {
    let controller = TextControlController::default();
    let alive = Rc::new(Cell::new(true));
    let touched = Rc::new(Cell::new(false));
    controller.bindings.borrow_mut().insert(
        1,
        Binding {
            id: "email".into(),
            alive: alive.clone(),
            touched: touched.clone(),
            node: Rc::default(),
            has_value: true,
            focused: Rc::default(),
            focusable: Rc::new(Cell::new(true)),
        },
    );
    assert_eq!(controller.has_valid_value(), Ok(true));
    assert_eq!(controller.is_touched(), Ok(false));
    touched.set(true);
    assert_eq!(controller.is_touched(), Ok(true));
    assert!(matches!(
        controller.get_native_input_element(),
        Err(TextControlError::NotMounted)
    ));
    alive.set(false);
    assert_eq!(controller.is_touched(), Err(TextControlError::NotMounted));
}
#[test]
fn ambiguous_controllers_and_expired_native_descriptors_reject_operations() {
    let controller = TextControlController::default();
    let binding = Binding {
        id: "native".into(),
        alive: Rc::new(Cell::new(true)),
        touched: Rc::default(),
        node: Rc::new(RefCell::new(Some(Rc::new(MountedData::new(()))))),
        has_value: false,
        focused: Rc::default(),
        focusable: Rc::new(Cell::new(true)),
    };
    controller.bindings.borrow_mut().insert(1, binding.clone());
    let descriptor = controller.get_native_input_element().unwrap();
    controller.bindings.borrow_mut().insert(2, binding.clone());
    assert_eq!(
        controller.is_touched(),
        Err(TextControlError::MultipleMounted)
    );
    binding.alive.set(false);
    assert_eq!(
        descriptor.binding.ensure_mounted(),
        Err(TextControlError::NotMounted)
    );
}
#[test]
fn class_feedback_tracks_nested_controls_and_cancelation_is_shared() {
    assert_eq!(
        validation_class_state("ix-valid ix-invalid--required"),
        crate::FieldState::Invalid
    );
    assert_eq!(
        validation_class_state("ix-valid ix-warning"),
        crate::FieldState::Warning
    );
    assert_eq!(validation_class_state("ix-info"), crate::FieldState::Info);
    assert_eq!(
        validation_class_state("ix-invalid-similar"),
        crate::FieldState::Default
    );
    let event = ValidityStateChangeEvent {
        validity: NativeValidityState::default(),
        prevented: Rc::default(),
    };
    let retained = event.clone();
    event.prevent_default();
    assert!(retained.default_prevented());
}
