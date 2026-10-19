//! Native text-control methods and constraint-validation observations.
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static BINDING_ID: AtomicUsize = AtomicUsize::new(1);

/// Copy of the browser's native `ValidityState` flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct NativeValidityState {
    pub bad_input: bool,
    pub custom_error: bool,
    pub pattern_mismatch: bool,
    pub range_overflow: bool,
    pub range_underflow: bool,
    pub step_mismatch: bool,
    pub too_long: bool,
    pub too_short: bool,
    pub type_mismatch: bool,
    pub value_missing: bool,
    pub valid: bool,
}
impl Default for NativeValidityState {
    fn default() -> Self {
        Self {
            bad_input: false,
            custom_error: false,
            pattern_mismatch: false,
            range_overflow: false,
            range_underflow: false,
            step_mismatch: false,
            too_long: false,
            too_short: false,
            type_mismatch: false,
            value_missing: false,
            valid: true,
        }
    }
}

/// Cancelable notification when touched control validity changes between valid
/// and invalid. Call `prevent_default` synchronously to veto automatic feedback.
#[derive(Clone, Debug)]
pub struct ValidityStateChangeEvent {
    pub validity: NativeValidityState,
    prevented: Rc<Cell<bool>>,
}
impl ValidityStateChangeEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextControlError {
    NotMounted,
    MultipleMounted,
    Dom(String),
}
impl std::fmt::Display for TextControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMounted => f.write_str("Text control is not mounted"),
            Self::MultipleMounted => {
                f.write_str("Text controller is attached to multiple controls")
            }
            Self::Dom(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for TextControlError {}

/// Snapshot of the native input's associated form.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct NativeFormElement {
    pub id: String,
    pub name: String,
    pub method: String,
    pub action: String,
    pub no_validate: bool,
}

#[derive(Clone)]
struct Binding {
    id: String,
    alive: Rc<Cell<bool>>,
    touched: Rc<Cell<bool>>,
    node: Rc<RefCell<Option<Rc<MountedData>>>>,
    has_value: bool,
    focused: Rc<Cell<bool>>,
    focusable: Rc<Cell<bool>>,
}

/// Descriptor scoped to a mounted input or textarea. Retained descriptors reject
/// operations after that component unmounts.
#[derive(Clone)]
pub struct NativeTextControl {
    binding: Binding,
    node: Rc<MountedData>,
}
impl NativeTextControl {
    pub fn id(&self) -> &str {
        &self.binding.id
    }
    pub async fn focus(&self) -> Result<(), TextControlError> {
        self.binding.ensure_current(&self.node)?;
        self.binding.ensure_focusable()?;
        let result = self.node.set_focus(true).await;
        self.binding.ensure_current(&self.node)?;
        self.binding.ensure_focusable()?;
        if let Err(error) = result {
            // Pinned desktop alpha.2 asks setFocus for bool; its interpreter
            // returns void. Typed focus/blur confirms the current mounted input
            // only. No retry, native query or alternative focus path runs.
            if !self.binding.focused.get() {
                return Err(dom_error(error));
            }
        }
        Ok(())
    }
    /// Set a native value without dispatching application input/change events.
    /// Controlled props remain authoritative on the next render.
    pub async fn set_value(&self, value: &str) -> Result<(), TextControlError> {
        self.binding.ensure_mounted()?;
        // MountedData has no native input value setter; controlled Rust props
        // remain authoritative, while this explicit native-value API is silent.
        let task = crate::dom_query::eval(
            r#"
            const [id,value]=await dioxus.recv(), input=document.getElementById(id);
            if (!(input instanceof HTMLInputElement || input instanceof HTMLTextAreaElement)) return false;
            input.value=value; return true;
        "#,
        );
        task.send((&self.binding.id, value)).map_err(dom_error)?;
        let found = task.join::<bool>().await.map_err(dom_error)?;
        self.binding.ensure_mounted()?;
        if found {
            Ok(())
        } else {
            Err(TextControlError::NotMounted)
        }
    }
}
impl Binding {
    fn ensure_current(&self, node: &Rc<MountedData>) -> Result<(), TextControlError> {
        self.ensure_mounted()?;
        if self
            .node
            .borrow()
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, node))
        {
            Ok(())
        } else {
            Err(TextControlError::NotMounted)
        }
    }
    fn ensure_focusable(&self) -> Result<(), TextControlError> {
        if self.focusable.get() {
            Ok(())
        } else {
            Err(TextControlError::Dom(
                "Native input could not receive focus".into(),
            ))
        }
    }
    fn ensure_mounted(&self) -> Result<(), TextControlError> {
        if self.alive.get() && self.node.borrow().is_some() {
            Ok(())
        } else {
            Err(TextControlError::NotMounted)
        }
    }
}
fn dom_error(error: impl std::fmt::Display) -> TextControlError {
    TextControlError::Dom(error.to_string())
}

/// Scoped input/textarea/number-input methods. Attach one control at a time.
#[derive(Clone, Default)]
pub struct TextControlController {
    bindings: Rc<RefCell<BTreeMap<usize, Binding>>>,
}
impl PartialEq for TextControlController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bindings, &other.bindings)
    }
}
pub type InputController = TextControlController;
pub type TextareaController = TextControlController;
pub type NumberInputController = TextControlController;
pub fn use_input() -> InputController {
    use_hook(TextControlController::default)
}
pub fn use_textarea() -> TextareaController {
    use_hook(TextControlController::default)
}
pub fn use_number_input() -> NumberInputController {
    use_hook(TextControlController::default)
}
impl TextControlController {
    fn binding(&self) -> Result<Binding, TextControlError> {
        let bindings = self.bindings.borrow();
        if bindings.len() > 1 {
            return Err(TextControlError::MultipleMounted);
        }
        bindings
            .values()
            .next()
            .filter(|b| b.alive.get())
            .cloned()
            .ok_or(TextControlError::NotMounted)
    }
    /// Value-presence query, independent of HTML constraint validity. Text
    /// fields require nonempty text; numeric fields accept optional empty input
    /// and otherwise require a finite parsed number.
    pub fn has_valid_value(&self) -> Result<bool, TextControlError> {
        Ok(self.binding()?.has_value)
    }
    pub fn is_touched(&self) -> Result<bool, TextControlError> {
        Ok(self.binding()?.touched.get())
    }
    pub fn get_native_input_element(&self) -> Result<NativeTextControl, TextControlError> {
        let binding = self.binding()?;
        binding.ensure_mounted()?;
        let node = binding
            .node
            .borrow()
            .clone()
            .ok_or(TextControlError::NotMounted)?;
        Ok(NativeTextControl { binding, node })
    }
    pub async fn focus_input(&self) -> Result<(), TextControlError> {
        self.get_native_input_element()?.focus().await
    }
    pub async fn get_validity_state(&self) -> Result<NativeValidityState, TextControlError> {
        let binding = self.binding()?;
        binding.ensure_mounted()?;
        let snapshot = read_snapshot(&binding.id).await?;
        binding.ensure_mounted()?;
        Ok(snapshot.validity)
    }
    pub async fn get_associated_form_element(
        &self,
    ) -> Result<Option<NativeFormElement>, TextControlError> {
        let binding = self.binding()?;
        binding.ensure_mounted()?;
        // MountedData does not expose an input's associated form/metadata.
        let task = crate::dom_query::eval(
            r#"
            const id=await dioxus.recv(), input=document.getElementById(id);
            if (!(input instanceof HTMLInputElement || input instanceof HTMLTextAreaElement)) throw new Error('Text control is not mounted');
            const f=input.form; return f?{id:f.id,name:f.name,method:f.method,action:f.action,no_validate:f.noValidate}:null;
        "#,
        );
        task.send(&binding.id).map_err(dom_error)?;
        let form = task
            .join::<Option<NativeFormElement>>()
            .await
            .map_err(dom_error)?;
        binding.ensure_mounted()?;
        Ok(form)
    }
}

#[derive(Clone, Default, PartialEq, serde::Deserialize)]
struct Snapshot {
    validity: NativeValidityState,
    no_validate: bool,
    validation_message: String,
}
// Pinned MountedData has no native validity flags, validation message or
// associated-form validation setting API; this adapter only reads those facts.
async fn read_snapshot(id: &str) -> Result<Snapshot, TextControlError> {
    let task = crate::dom_query::eval(
        r#"
        const id=await dioxus.recv(), input=document.getElementById(id);
        if (!(input instanceof HTMLInputElement || input instanceof HTMLTextAreaElement)) throw new Error('Text control is not mounted');
        const v=input.validity;
        return {validity:{bad_input:v.badInput,custom_error:v.customError,pattern_mismatch:v.patternMismatch,
            range_overflow:v.rangeOverflow,range_underflow:v.rangeUnderflow,step_mismatch:v.stepMismatch,
            too_long:v.tooLong,too_short:v.tooShort,type_mismatch:v.typeMismatch,value_missing:v.valueMissing,valid:v.valid},no_validate:!!input.form?.noValidate,validation_message:input.validationMessage};
    "#,
    );
    task.send(id).map_err(dom_error)?;
    task.join::<Snapshot>().await.map_err(dom_error)
}

pub(crate) struct ControlLifecycle {
    pub invalid: bool,
    pub touched: bool,
    pub mounted: Callback<Rc<MountedData>>,
    pub observe: Callback<bool>,
    pub focus_state: Callback<bool>,
    pub message: String,
}
pub(crate) fn use_control(
    id: &str,
    has_value: bool,
    can_focus: bool,
    controller: Option<TextControlController>,
    validity_changed: EventHandler<ValidityStateChangeEvent>,
) -> ControlLifecycle {
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let touched = use_hook(|| Rc::new(Cell::new(false)));
    let node = use_hook(|| Rc::new(RefCell::new(None)));
    let focused = use_hook(|| Rc::new(Cell::new(false)));
    let focusable = use_hook(|| Rc::new(Cell::new(can_focus)));
    focusable.set(can_focus);
    let token = use_hook(|| BINDING_ID.fetch_add(1, Ordering::Relaxed));
    let bound = use_hook(|| Rc::new(RefCell::new(None::<TextControlController>)));
    if *bound.borrow() != controller {
        if let Some(old) = bound.borrow_mut().take() {
            old.bindings.borrow_mut().remove(&token);
        }
        *bound.borrow_mut() = controller.clone();
    }
    if let Some(controller) = controller {
        controller.bindings.borrow_mut().insert(
            token,
            Binding {
                id: id.into(),
                alive: alive.clone(),
                touched: touched.clone(),
                node: node.clone(),
                has_value,
                focused: focused.clone(),
                focusable: focusable.clone(),
            },
        );
    }
    let drop_alive = alive.clone();
    let drop_bound = bound.clone();
    let drop_node = node.clone();
    let drop_focus = focused.clone();
    use_drop(move || {
        drop_alive.set(false);
        drop_focus.set(false);
        drop_node.borrow_mut().take();
        if let Some(controller) = drop_bound.borrow_mut().take() {
            controller.bindings.borrow_mut().remove(&token);
        }
    });
    let mut snapshot = use_signal(Snapshot::default);
    let mut invalid = use_signal(|| false);
    let mut touched_state = use_signal(|| false);
    let epoch = use_hook(|| Rc::new(Cell::new(0u64)));
    let observe_id = id.to_string();
    let observe_touched = touched.clone();
    let observe_node = node.clone();
    let observe = use_callback(move |blur: bool| {
        if blur {
            observe_touched.set(true);
            touched_state.set(true);
        }
        if !alive.get() || observe_node.borrow().is_none() {
            return;
        }
        let tick = epoch.get().wrapping_add(1);
        epoch.set(tick);
        let id = observe_id.clone();
        let alive = alive.clone();
        let epoch = epoch.clone();
        let touched = observe_touched.clone();
        spawn(async move {
            let Ok(next) = read_snapshot(&id).await else {
                return;
            };
            if !alive.get() || epoch.get() != tick {
                return;
            }
            if touched.get() && *invalid.peek() == next.validity.valid {
                let event = ValidityStateChangeEvent {
                    validity: next.validity,
                    prevented: Rc::new(Cell::new(false)),
                };
                validity_changed.call(event.clone());
                if !event.default_prevented() {
                    invalid.set(!next.validity.valid);
                }
            }
            if *snapshot.peek() != next {
                snapshot.set(next);
            }
        });
    });
    let mount_focus = focused.clone();
    let focus_state = use_callback(move |value: bool| focused.set(value));
    let mounted = use_callback(move |data: Rc<MountedData>| {
        if node
            .borrow()
            .as_ref()
            .is_none_or(|current| !Rc::ptr_eq(current, &data))
        {
            mount_focus.set(false);
        }
        *node.borrow_mut() = Some(data);
        observe.call(false);
    });
    ControlLifecycle {
        invalid: *invalid.read() && *touched_state.read() && !snapshot.read().no_validate,
        touched: *touched_state.read(),
        mounted,
        observe,
        focus_state,
        message: snapshot.read().validation_message.clone(),
    }
}

/// Presentation precedence when several validation classes are present.
pub(crate) fn validation_class_state(class: &str) -> crate::FieldState {
    use crate::FieldState;
    let classes: Vec<_> = class.split_whitespace().collect();
    if classes.iter().any(|c| {
        matches!(
            *c,
            "ix-invalid"
                | "ix-invalid--required"
                | "ix-invalid--validity-invalid"
                | "ix-field-invalid"
        )
    }) {
        FieldState::Invalid
    } else if classes
        .iter()
        .any(|c| matches!(*c, "ix-warning" | "ix-field-warning"))
    {
        FieldState::Warning
    } else if classes
        .iter()
        .any(|c| matches!(*c, "ix-info" | "ix-field-info"))
    {
        FieldState::Info
    } else if classes
        .iter()
        .any(|c| matches!(*c, "ix-valid" | "ix-field-valid"))
    {
        FieldState::Valid
    } else {
        FieldState::Default
    }
}

pub(crate) fn use_custom_validation(id: &str) -> (crate::FieldState, Callback<()>) {
    let mut state = use_signal(|| crate::FieldState::Default);
    let task = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let dispose = task.clone();
    use_drop(move || {
        if let Some(task) = dispose.borrow_mut().take() {
            let _ = task.send(());
        }
    });
    let id = id.to_string();
    let mounted = use_callback(move |()| {
        if task.borrow().is_some() {
            return;
        }
        let mut eval = dioxus::document::eval(
            r#"
            const id=await dioxus.recv(), root=document.getElementById(id);
            if(!root)return;
            const content=root.querySelector('[data-custom-control]');
            const report=()=>{
                const classes=[...root.classList];
                for(const node of content?.querySelectorAll('[class]')??[])classes.push(...node.classList);
                dioxus.send(classes.join(' '));
            };
            const observer=new MutationObserver(report);
            observer.observe(root,{attributes:true,attributeFilter:['class']});
            if(content)observer.observe(content,{subtree:true,childList:true,attributes:true,attributeFilter:['class']});
            let removed;
            const gone=new Promise(resolve=>removed=resolve);
            const removal=new MutationObserver(()=>{if(!root.isConnected)removed();});
            removal.observe(document.documentElement,{subtree:true,childList:true});
            report();
            await Promise.race([dioxus.recv(),gone]);observer.disconnect();removal.disconnect();
        "#,
        );
        if eval.send(&id).is_err() {
            return;
        }
        *task.borrow_mut() = Some(eval);
        spawn(async move {
            while let Ok(classes) = eval.recv::<String>().await {
                let next = validation_class_state(&classes);
                if *state.peek() != next {
                    state.set(next);
                }
            }
        });
    });
    (*state.read(), mounted)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
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
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => {
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
}
