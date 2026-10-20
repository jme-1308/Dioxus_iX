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
mod tests;
