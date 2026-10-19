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

fn validation_rank(state: crate::FieldState) -> u8 {
    use crate::FieldState;
    match state {
        FieldState::Default => 0,
        FieldState::Valid => 1,
        FieldState::Info => 2,
        FieldState::Warning => 3,
        FieldState::Invalid => 4,
    }
}
pub(crate) fn combined_validation(
    states: impl IntoIterator<Item = crate::FieldState>,
) -> crate::FieldState {
    states
        .into_iter()
        .max_by_key(|state| validation_rank(*state))
        .unwrap_or_default()
}
pub(crate) fn control_validation(state: crate::FieldState, class: &str) -> crate::FieldState {
    combined_validation([state, validation_class_state(class)])
}

#[derive(Clone)]
struct CustomFieldOwner {
    states: Signal<BTreeMap<usize, crate::FieldState>>,
    alive: Rc<Cell<bool>>,
}
impl CustomFieldOwner {
    fn publish(&self, token: usize, state: crate::FieldState) {
        if !self.alive.get() {
            return;
        }
        let expected = (state != crate::FieldState::Default).then_some(state);
        let Ok(states) = self.states.try_peek() else {
            return;
        };
        let previous = states.get(&token).copied();
        if previous == expected {
            return;
        }
        drop(states);
        let mut writable = self.states;
        if let Ok(mut states) = writable.try_write() {
            if let Some(state) = expected {
                states.insert(token, state);
            } else {
                states.remove(&token);
            }
        };
    }
}
fn use_field_contribution(owner: Option<CustomFieldOwner>, state: crate::FieldState) {
    let token = use_hook(|| BINDING_ID.fetch_add(1, Ordering::Relaxed));
    let cleanup = owner.clone();
    use_drop(move || {
        if let Some(owner) = cleanup {
            owner.publish(token, crate::FieldState::Default);
        }
    });
    if let Some(owner) = owner {
        owner.publish(token, state);
    }
}

/// Contribute this Dioxus control's current validation state to its nearest
/// [`crate::CustomField`]. Call unconditionally during rendering; `Default` and
/// component teardown remove the contribution. Outside a CustomField it is a
/// no-op. Each field aggregates Invalid, Warning, Info, Valid in that order.
///
/// Custom native controls use this hook or the caller sets `CustomField.state`.
/// Arbitrary descendant DOM class mutations are not discovered or observed.
///
/// ```rust,ignore
/// fn MyControl() -> Element {
///     let mut value = use_signal(String::new);
///     use_custom_field_state(if value().is_empty() { FieldState::Invalid } else { FieldState::Valid });
///     rsx! { input { value:value(), oninput:move |event|value.set(event.value()) } }
/// }
/// ```
pub fn use_custom_field_state(state: crate::FieldState) {
    let owner = use_hook(try_consume_context::<CustomFieldOwner>);
    use_field_contribution(owner, state);
}

pub(crate) fn use_custom_field_validation(
    explicit: crate::FieldState,
    class: &str,
) -> crate::FieldState {
    // Capture the parent before providing our own owner. Re-reading context on
    // later renders would make a nested field report its aggregate to itself.
    let parent = use_hook(try_consume_context::<CustomFieldOwner>);
    let owner = use_context_provider(|| CustomFieldOwner {
        states: Signal::new(BTreeMap::new()),
        alive: Rc::new(Cell::new(true)),
    });
    let cleanup = owner.clone();
    use_drop(move || cleanup.alive.set(false));
    let children = combined_validation(owner.states.read().values().copied());
    let effective = if explicit != crate::FieldState::Default {
        explicit
    } else {
        combined_validation([validation_class_state(class), children])
    };
    use_field_contribution(parent, effective);
    effective
}

#[cfg(test)]
mod tests {
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
