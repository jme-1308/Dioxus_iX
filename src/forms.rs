//! Form controls adapted from the iX component contracts using native form elements.
//!
//! Values are controlled: update `value` or `checked` from the corresponding event.
//! Text controls retain native constraint validation and form submission semantics.
use crate::form_control::{TextControlController, ValidityStateChangeEvent};
use dioxus::html::FileData;
use dioxus::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
struct CharacterPattern {
    source: Option<String>,
    compiled: Option<regex::Regex>,
}

impl PartialEq for CharacterPattern {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl CharacterPattern {
    fn new(source: Option<String>) -> Self {
        let compiled = source
            .as_deref()
            .and_then(|value| regex::Regex::new(value).ok());
        Self { source, compiled }
    }
    fn allows(&self, text: &str) -> bool {
        self.compiled.as_ref().is_none_or(|pattern| {
            text.chars()
                .all(|character| pattern.is_match(character.encode_utf8(&mut [0; 4])))
        })
    }
}

static FIELD_ID: AtomicUsize = AtomicUsize::new(1);

fn field_id(id: Option<String>) -> String {
    let generated = use_hook(|| format!("ix-field-{}", FIELD_ID.fetch_add(1, Ordering::Relaxed)));
    id.unwrap_or(generated)
}

/// Explicit validation presentation. Native HTML constraints remain active independently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldState {
    /// No explicit validation feedback.
    #[default]
    Default,
    /// Invalid input, displayed with error feedback.
    Invalid,
    /// Valid input.
    Valid,
    /// Informational feedback.
    Info,
    /// Warning feedback.
    Warning,
}

impl FieldState {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Invalid => "ix-field-invalid",
            Self::Valid => "ix-field-valid",
            Self::Info => "ix-field-info",
            Self::Warning => "ix-field-warning",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FieldFeedback {
    text: String,
    state: FieldState,
}

fn field_feedback(
    state: FieldState,
    helper: Option<String>,
    invalid: Option<String>,
    valid: Option<String>,
    info: Option<String>,
    warning: Option<String>,
) -> Option<FieldFeedback> {
    let active = match state {
        FieldState::Invalid => invalid,
        FieldState::Valid => valid,
        FieldState::Info => info,
        FieldState::Warning => warning,
        FieldState::Default => None,
    };
    if let Some(text) = active.filter(|text| !text.trim().is_empty()) {
        Some(FieldFeedback { text, state })
    } else {
        helper
            .filter(|text| !text.trim().is_empty())
            .map(|text| FieldFeedback {
                text,
                state: FieldState::Default,
            })
    }
}

fn render_field_feedback(feedback: FieldFeedback, id: Option<String>) -> Element {
    let state = feedback.state;
    let icon = match state {
        FieldState::Invalid => Some("error"),
        FieldState::Valid => Some("success"),
        FieldState::Info => Some("info"),
        FieldState::Warning => Some("warning"),
        FieldState::Default => None,
    };
    rsx! {
        div { id, class: "ix-helper {state.class()}",
            role: if state == FieldState::Invalid { "alert" } else { "status" },
            if let Some(icon) = icon { crate::Icon { name: icon, size: 16, class: "ix-helper-icon" } }
            "{feedback.text}"
        }
    }
}

fn render_text_field_feedback(feedback: FieldFeedback, id: Option<String>) -> Element {
    let (icon, color) = match feedback.state {
        FieldState::Default => (None, crate::TypographyColor::Soft),
        FieldState::Invalid => (Some("error"), crate::TypographyColor::Alarm),
        FieldState::Valid => (Some("success"), crate::TypographyColor::Std),
        FieldState::Info => (Some("info"), crate::TypographyColor::Std),
        FieldState::Warning => (Some("warning"), crate::TypographyColor::Std),
    };
    rsx! {
        crate::Typography { id, format: crate::TypographyFormat::BodyParagraph,
            text_color: color, class: "ix-text-field-bottom-text ix-helper {feedback.state.class()}",
            role: if feedback.state == FieldState::Invalid { "alert" } else { "status" },
            if let Some(icon) = icon {
                crate::Icon { name: icon, size: 16, class: "ix-text-field-text-icon ix-helper-icon {feedback.state.class()}" }
            }
            "{feedback.text}"
        }
    }
}

#[component]
fn TextControlFeedback(
    feedback: Option<FieldFeedback>,
    id: String,
    anchor_id: String,
    show_text_as_tooltip: bool,
) -> Element {
    rsx! {
        if let Some(feedback) = feedback {
            if show_text_as_tooltip {
                crate::TooltipFor { id, anchor_id, placement: crate::TooltipPlacement::Bottom, show_delay: 500,
                    {render_text_field_feedback(feedback, None)}
                }
            } else {
                {render_text_field_feedback(feedback, Some(id))}
            }
        }
    }
}

#[component]
fn TextFieldPresentation(
    children: Element,
    label: Option<String>,
    control_id: String,
    anchor_id: String,
    feedback_id: String,
    required: bool,
    state: FieldState,
    feedback: Option<FieldFeedback>,
    invalid_alert: bool,
    show_text_as_tooltip: bool,
    max_length: Option<u32>,
    length: usize,
) -> Element {
    rsx! {
        div { class: "ix-text-field-wrapper",
            role: invalid_alert.then_some("alert"), "aria-live": invalid_alert.then_some("polite"),
            if let Some(label) = label.filter(|text| !text.is_empty()) {
                div { class: "ix-text-field-top",
                    div { class: "ix-text-field-label",
                        label { r#for: control_id,
                            crate::Typography { format: crate::TypographyFormat::Body,
                                text_color: if state == FieldState::Invalid { crate::TypographyColor::Alarm } else { crate::TypographyColor::Soft },
                                "{label}" if required { span { class: "ix-required", "aria-hidden": "true", "*" } }
                            }
                        }
                    }
                }
            }
            div { class: "ix-text-field-slot", {children} }
            div { class: "ix-text-field-bottom",
                if !show_text_as_tooltip { TextControlFeedback { feedback: feedback.clone(), id: feedback_id.clone(), anchor_id: anchor_id.clone(), show_text_as_tooltip: false } }
                div { class: "ix-text-field-bottom-right",
                    if let Some(limit) = max_length.filter(|limit| *limit > 0) {
                        crate::Typography { format: crate::TypographyFormat::Body, text_color: crate::TypographyColor::Soft,
                            class: "ix-text-field-counter ix-field-counter", "{length}/{limit}"
                        }
                    }
                }
            }
            if show_text_as_tooltip { TextControlFeedback { feedback, id: feedback_id, anchor_id, show_text_as_tooltip: true } }
        }
    }
}

/// The current value is always the latest controlled prop, never an unaccepted
/// native edit. Updating props alone does not emit a commit.
#[derive(Default)]
struct TextCommitState {
    current: String,
    baseline: Option<String>,
}

impl TextCommitState {
    fn focus(&mut self) {
        self.baseline = Some(self.current.clone());
    }

    fn commit(&mut self) -> Option<String> {
        if self.baseline.as_ref() == Some(&self.current) {
            return None;
        }
        let value = self.current.clone();
        self.baseline = Some(value.clone());
        Some(value)
    }
}

fn use_text_commit(
    value: &str,
    onchange: EventHandler<String>,
) -> (EventHandler<()>, EventHandler<()>) {
    let tracker = use_hook(|| Rc::new(std::cell::RefCell::new(TextCommitState::default())));
    tracker.borrow_mut().current = value.to_string();
    let focus_tracker = tracker.clone();
    let focus = use_callback(move |_| focus_tracker.borrow_mut().focus());
    let commit = use_callback(move |_| {
        let changed = tracker.borrow_mut().commit();
        if let Some(value) = changed {
            onchange.call(value);
        }
    });
    (focus, commit)
}

/// Label associated with a native control through `html_for`.
#[component]
pub fn FieldLabel(
    text: String,
    #[props(default)] html_for: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] invalid: bool,
) -> Element {
    rsx! {
        label { class: if invalid { "ix-field-label ix-field-invalid" } else { "ix-field-label" },
            r#for: html_for, "{text}"
            if required { span { class: "ix-required", "aria-hidden": "true", " *" } }
        }
    }
}

/// Helper or validation text. The active state chooses its matching text, with helper fallback.
#[component]
pub fn HelperText(
    #[props(default)] id: Option<String>,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
) -> Element {
    let feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    rsx! { if let Some(feedback) = feedback { {render_field_feedback(feedback, id)} } }
}

/// Layout for a label, arbitrary control content and associated helper text.
#[component]
pub fn FieldWrapper(
    children: Element,
    #[props(default)] label: Option<String>,
    #[props(default)] html_for: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
) -> Element {
    rsx! {
        div { class: "ix-field {state.class()}",
            if let Some(text) = label { FieldLabel { text, html_for, required, invalid: state == FieldState::Invalid } }
            {children}
            HelperText { helper_text, invalid_text, valid_text, info_text, warning_text, state }
        }
    }
}

/// Field wrapper for application-defined native controls.
#[component]
pub fn CustomField(
    children: Element,
    #[props(default)] label: Option<String>,
    #[props(default)] html_for: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
) -> Element {
    let id = field_id(None);
    let (observed, mounted) = crate::form_control::use_custom_validation(&id);
    let state = if state == FieldState::Default {
        observed
    } else {
        state
    };
    rsx! { div { id, class: "ix-custom-field {class}", onmounted: move |_| mounted.call(()),
        FieldWrapper { label, html_for, required, helper_text, invalid_text, valid_text, info_text, warning_text, state,
            div { "data-custom-control": "true", {children} }
        }
    } }
}

/// Branded native text input. `onvaluechange` reports edits; `onchange` commits the
/// changed controlled value on Enter or blur. Focus snapshots the current value;
/// Enter resets that baseline so the following blur does not repeat the commit.
/// `onixblur` runs before blur commits. Programmatic updates emit only when a
/// commit trigger follows; rejected edits never replace the controlled value.
/// `show_text_as_tooltip` moves nonblank feedback into a delayed bottom tooltip.
/// `input_type` accepts native types including text, password, email, number, date and time.
/// `allowed_characters_pattern` uses Rust's regex syntax per inserted character;
/// JavaScript-only features such as lookarounds are unsupported. Invalid patterns
/// disable the character filter; HTML `pattern` remains an independent constraint.
#[component]
pub fn Input(
    #[props(default)] value: String,
    #[props(default)] label: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] name: Option<String>,
    #[props(default = "text".to_string())] input_type: String,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] pattern: Option<String>,
    #[props(default)] allowed_characters_pattern: Option<String>,
    #[props(default)] min_length: Option<u32>,
    #[props(default)] max_length: Option<u32>,
    #[props(default)] min: Option<String>,
    #[props(default)] max: Option<String>,
    #[props(default)] step: Option<String>,
    #[props(default)] autocomplete: Option<String>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
    #[props(default)] show_text_as_tooltip: bool,
    #[props(default)] suppress_submit_on_enter: bool,
    #[props(default)] text_alignment_end: bool,
    #[props(default)] start: Option<Element>,
    #[props(default)] end: Option<Element>,
    #[props(default)] onvaluechange: EventHandler<String>,
    #[props(default)] onchange: EventHandler<String>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(default)] onkeydown: EventHandler<KeyboardEvent>,
    #[props(default)] onfocus: EventHandler<FocusEvent>,
    #[props(default)] onixblur: EventHandler<()>,
    #[props(default)] onvaliditystatechange: EventHandler<ValidityStateChangeEvent>,
    #[props(default)] controller: Option<TextControlController>,
    #[props(default)] class: String,
) -> Element {
    let id = field_id(id);
    let (focus_commit, commit) = use_text_commit(&value, onchange);
    let has_value = if input_type == "number" {
        number_has_valid_value(&value, required)
    } else {
        !value.is_empty()
    };
    let lifecycle =
        crate::form_control::use_control(&id, has_value, controller, onvaliditystatechange);
    let supplied = crate::form_control::validation_class_state(&class);
    let state = if state != FieldState::Default {
        state
    } else if lifecycle.invalid {
        FieldState::Invalid
    } else {
        supplied
    };
    let invalid_text = invalid_text.or_else(|| {
        (lifecycle.invalid && !lifecycle.message.is_empty()).then_some(lifecycle.message)
    });
    let observe = lifecycle.observe;
    let mounted = lifecycle.mounted;
    let constraints = (
        pattern.clone(),
        min_length,
        max_length,
        min.clone(),
        max.clone(),
        step.clone(),
        input_type.clone(),
    );
    use_effect(use_reactive(
        (&value, &required, &disabled, &readonly, &constraints),
        move |_| observe.call(false),
    ));
    let help_id = format!("{id}-help");
    let allowed = use_memo(use_reactive((&allowed_characters_pattern,), |(source,)| {
        CharacterPattern::new(source)
    }));
    let mut password_visible = use_signal(|| false);
    let password = input_type == "password";
    let native_type = if password && password_visible() {
        "text".to_string()
    } else {
        input_type
    };
    let length = value.encode_utf16().count();
    let geometry_id = field_id(None);
    let (padding, measure_padding) = crate::input_geometry::use_adornments(&geometry_id);
    let input_style = format!(
        "text-align:{};{}",
        if text_alignment_end { "end" } else { "start" },
        padding.map(|facts| facts.padding()).unwrap_or_default()
    );
    let invalid_alert =
        state == FieldState::Invalid && invalid_text.as_ref().is_some_and(|text| !text.is_empty());
    let feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    let has_feedback = feedback.is_some();
    let anchor_id = format!("{id}-input-wrap");
    rsx! {
        div { id: geometry_id, class: "ix-field ix-text-field-native ix-input-native {state.class()} {class}",
            "data-disabled": disabled.to_string(), "data-readonly": readonly.to_string(),
            TextFieldPresentation { label, control_id: id.clone(), anchor_id: anchor_id.clone(), feedback_id: help_id.clone(),
                required, state, feedback, invalid_alert, show_text_as_tooltip, max_length, length,
                div { id: anchor_id, class: "ix-input-wrap",
                    onmounted: move |_| measure_padding.call(()),
                    "data-padding-ready": padding.is_some().to_string(),
                    "data-padding-start": padding.map(|facts| facts.start.to_string()),
                    "data-padding-end": padding.map(|facts| facts.end.to_string()),
                    span { class: "ix-input-start", if let Some(start) = start { {start} } }
                input { id, class: "ix-input", name, r#type: native_type, value, placeholder,
                    required, disabled, readonly, pattern, minlength: min_length, maxlength: max_length,
                    min, max, step, autocomplete,
                    style: input_style,
                    "aria-label": aria_label,
                    "aria-invalid": (state == FieldState::Invalid).to_string(),
                    "aria-describedby": has_feedback.then(|| help_id.clone()),
                    "aria-errormessage": (state == FieldState::Invalid && has_feedback).then(||help_id.clone()),
                    "data-ix-touched": lifecycle.touched.to_string(),
                    onmounted: move |event| mounted.call(event.data()),
                    oninput: move |event| { onvaluechange.call(event.value()); observe.call(false); },
                    onfocus: move |event| { focus_commit.call(()); onfocus.call(event); },
                    onblur: move |event| { onixblur.call(()); observe.call(true); commit.call(()); onblur.call(event); },
                    onkeydown: move |event| {
                        onkeydown.call(event.clone());
                        if event.key() == Key::Enter { commit.call(()); }
                        if suppress_submit_on_enter && event.key() == Key::Enter { event.prevent_default(); }
                        if !event.is_composing() && !event.modifiers().intersects(Modifiers::CONTROL | Modifiers::META | Modifiers::ALT) {
                            if let Key::Character(text) = event.key() { if !allowed.read().allows(&text) { event.prevent_default(); } }
                        }
                    },
                    onbeforeinput: move |event| if !event.is_composing() { if let Some(text) = event.data().data() { if !allowed.read().allows(&text) { event.prevent_default(); } } },
                    onpaste: move |event| if let Some(text) = event.data_transfer().get_as_text() { if !allowed.read().allows(&text) { event.prevent_default(); } },
                }
                    span { class: "ix-input-end",
                        if let Some(end) = end { {end} }
                        if password && !disabled {
                            span { class: "ix-input-password-holder", style: "display:contents",
                                onmousedown: move |event| event.prevent_default(),
                                crate::IconButton { class: "ix-input-password-eye", size: crate::ButtonIconSize::Sixteen,
                                    variant: crate::ButtonVariant::Tertiary, icon_color: "--ix-input-password-eye-color",
                                    label: if password_visible() { "Hide password" } else { "Show password" },
                                    aria_pressed: password_visible(),
                                    icon: if password_visible() { "eye-cancelled" } else { "eye" },
                                    onclick: move |_| password_visible.toggle(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Multiline input with native length limits and configurable resize behavior.
/// `onvaluechange` reports edits; `onchange` commits changed controlled values on
/// blur, after `onixblur`. Enter retains its native newline behavior. The focus
/// baseline and programmatic-update rules match [`Input`].
/// `show_text_as_tooltip` moves nonblank feedback into a delayed bottom tooltip.
#[component]
pub fn Textarea(
    #[props(default)] value: String,
    #[props(default)] label: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] name: Option<String>,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] rows: Option<u32>,
    #[props(default)] cols: Option<u32>,
    #[props(default)] width: Option<String>,
    #[props(default)] height: Option<String>,
    #[props(default)] min_length: Option<u32>,
    #[props(default)] max_length: Option<u32>,
    #[props(default)] aria_label: Option<String>,
    #[props(default = "both".to_string())] resize: String,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
    #[props(default)] show_text_as_tooltip: bool,
    #[props(default)] onvaluechange: EventHandler<String>,
    #[props(default)] onchange: EventHandler<String>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(default)] onfocus: EventHandler<FocusEvent>,
    #[props(default)] onkeydown: EventHandler<KeyboardEvent>,
    #[props(default)] onixblur: EventHandler<()>,
    #[props(default)] onvaliditystatechange: EventHandler<ValidityStateChangeEvent>,
    #[props(default)] controller: Option<TextControlController>,
    #[props(default)] class: String,
) -> Element {
    let id = field_id(id);
    let (focus_commit, commit) = use_text_commit(&value, onchange);
    let lifecycle =
        crate::form_control::use_control(&id, !value.is_empty(), controller, onvaliditystatechange);
    let supplied = crate::form_control::validation_class_state(&class);
    let state = if state != FieldState::Default {
        state
    } else if lifecycle.invalid {
        FieldState::Invalid
    } else {
        supplied
    };
    let invalid_text = invalid_text.or_else(|| {
        (lifecycle.invalid && !lifecycle.message.is_empty()).then_some(lifecycle.message)
    });
    let observe = lifecycle.observe;
    let mounted = lifecycle.mounted;
    use_effect(use_reactive(
        (
            &value,
            &required,
            &disabled,
            &readonly,
            &min_length,
            &max_length,
        ),
        move |_| observe.call(false),
    ));
    let help_id = format!("{id}-help");
    let resize = match resize.as_str() {
        "none" | "both" | "horizontal" => resize.as_str(),
        _ => "vertical",
    };
    let dimensions = format!(
        "resize:{resize};{}{}",
        height
            .map(|value| format!("height:{value};"))
            .unwrap_or_default(),
        width
            .map(|value| format!("width:{value};"))
            .unwrap_or_default()
    );
    let length = value.encode_utf16().count();
    let invalid_alert =
        state == FieldState::Invalid && invalid_text.as_ref().is_some_and(|text| !text.is_empty());
    let feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    let has_feedback = feedback.is_some();
    let anchor_id = id.clone();
    rsx! {
        div { class: "ix-field ix-text-field-native ix-textarea-native {state.class()} {class}",
            "data-disabled": disabled.to_string(), "data-readonly": readonly.to_string(),
            TextFieldPresentation { label, control_id: id.clone(), anchor_id, feedback_id: help_id.clone(),
                required, state, feedback, invalid_alert, show_text_as_tooltip, max_length, length,
                div { class: "ix-input-wrap",
            textarea { id, class: "ix-input ix-textarea", name, value, placeholder,
                required, disabled, readonly, rows, cols, minlength: min_length, maxlength: max_length,
                style: dimensions, "aria-label": aria_label, "aria-invalid": (state == FieldState::Invalid).to_string(),
                "aria-describedby": has_feedback.then(|| help_id.clone()),
                "aria-errormessage": (state == FieldState::Invalid && has_feedback).then(||help_id.clone()),
                "data-ix-touched": lifecycle.touched.to_string(),
                onmounted: move |event| mounted.call(event.data()),
                oninput: move |event| { onvaluechange.call(event.value()); observe.call(false); },
                onfocus: move |event| { focus_commit.call(()); onfocus.call(event); },
                onkeydown,
                onblur: move |event| { onixblur.call(()); observe.call(true); commit.call(()); onblur.call(event); },
            }
                }
            }
        }
    }
}

/// Controlled checkbox with a native form mirror and accessible three-state button.
/// Reset `indeterminate` from the parent after a user toggle to leave the mixed state.
///
/// Shape, target geometry, validation colors and label typography consume the
/// source semantic tokens. Checked and indeterminate marks are independent.
/// The source mutates its checked property before emitting; this Rust API instead
/// asks the consumer to update `checked` from `oncheckedchange`.
#[component]
pub fn Checkbox(
    #[props(default)] checked: bool,
    #[props(default)] label: String,
    #[props(default)] name: Option<String>,
    #[props(default = "on".to_string())] value: String,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] indeterminate: bool,
    #[props(default)] invalid: bool,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
    aria_label: Option<String>,
    #[props(default)] oncheckedchange: EventHandler<bool>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
) -> Element {
    let mut button = use_signal(|| None::<Rc<MountedData>>);
    let mut attempted = use_signal(|| false);
    let invalid = invalid || state == FieldState::Invalid || (attempted() && required && !checked);
    rsx! {
        span { class: "ix-checkbox-field ix-checkbox-native {state.class()} {class}",
            "data-checked": checked.to_string(), "data-mixed": indeterminate.to_string(),
            "data-invalid": invalid.to_string(), "data-label-less": label.is_empty().to_string(),
            input { class: "ix-check-native", r#type: "checkbox", name, value, checked, disabled, required,
                "aria-hidden": "true", tabindex: "-1",
                oninvalid: move |event| {
                    event.prevent_default(); attempted.set(true);
                    let node = button.peek().clone();
                    spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } });
                },
            }
            button { class: "ix-check ix-checkbox {state.class()}", r#type: "button", role: "checkbox", disabled,
                "data-mixed": indeterminate.to_string(), "data-checked": checked.to_string(),
                "aria-checked": if indeterminate { "mixed" } else if checked { "true" } else { "false" },
                "aria-required": required.to_string(), "aria-invalid": invalid.to_string(),
                "aria-disabled": disabled.to_string(), aria_label,
                onmounted: move |event| button.set(Some(event.data())), onclick: move |_| { if !disabled { oncheckedchange.call(!checked); } }, onblur,
                span { class: "ix-checkbox-button", "aria-hidden": "true",
                    span { class: "ix-check-shape",
                        svg { view_box: "0 0 18 18", width: "18", height: "18", fill: "none",
                            if indeterminate {
                                rect { width: "18", height: "18", fill: "transparent" }
                                rect { x: "3", y: "8", width: "12", height: "2", fill: "var(--ix-checkbox-checkmark--color)" }
                            }
                            if checked { path { d: "M3.65625 8.15625L8.4375 12.9375L14.625 3.9375", stroke: "var(--ix-checkbox-checkmark--color)", stroke_width: "2" } }
                        }
                    }
                }
                crate::Typography { format: crate::TypographyFormat::Body,
                    text_color: if disabled { crate::TypographyColor::Weak } else { crate::TypographyColor::Std },
                    class: "ix-checkbox-label", "{label}"
                }
            }
            if attempted() && required && !checked { span { class: "ix-helper ix-field-invalid", role: "alert", "Please select this option" } }
        }
    }
}

// Scoped field-wrapper presentation for groups. Other controls retain their
// independently reviewed feedback adapters.
#[component]
fn GroupLayout(
    children: Element,
    label: String,
    required: bool,
    state: FieldState,
    feedback: Option<FieldFeedback>,
    invalid_alert: bool,
) -> Element {
    rsx! {
        div { class: "ix-group-wrapper",
            role: invalid_alert.then_some("alert"),
            "aria-live": invalid_alert.then_some("polite"),
            if !label.is_empty() {
                div { class: "ix-group-top",
                    div { class: "ix-group-label", "aria-hidden": "true",
                        label {
                            crate::Typography { format: crate::TypographyFormat::Body,
                                text_color: if state == FieldState::Invalid { crate::TypographyColor::Alarm } else { crate::TypographyColor::Soft },
                                "{label}"
                                if required { span { "*" } }
                            }
                        }
                    }
                }
            }
            div { class: "ix-group-slot", {children} }
            div { class: "ix-group-bottom",
                if let Some(feedback) = feedback {
                    GroupFeedback { feedback }
                }
                div { class: "ix-group-bottom-right" }
            }
        }
    }
}

#[component]
fn GroupFeedback(feedback: FieldFeedback) -> Element {
    let (icon, color) = match feedback.state {
        FieldState::Default => (None, crate::TypographyColor::Soft),
        FieldState::Invalid => (Some("error"), crate::TypographyColor::Alarm),
        FieldState::Valid => (Some("success"), crate::TypographyColor::Std),
        FieldState::Info => (Some("info"), crate::TypographyColor::Std),
        FieldState::Warning => (Some("warning"), crate::TypographyColor::Std),
    };
    rsx! {
        crate::Typography { format: crate::TypographyFormat::BodyParagraph,
            text_color: color, class: "ix-group-bottom-text {feedback.state.class()}",
            if let Some(icon) = icon {
                crate::Icon { name: icon, size: 16, class: "ix-group-text-icon {feedback.state.class()}" }
            }
            "{feedback.text}"
        }
    }
}

/// Semantic fieldset for related checkboxes; each child controls its own value.
/// `required` presents the group marker; children retain their own constraints.
/// Descendant validation aggregation is not inferred from arbitrary children.
#[component]
pub fn CheckboxGroup(
    children: Element,
    #[props(default)] label: String,
    #[props(default)] horizontal: bool,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
) -> Element {
    let accessible_label = aria_labelledby
        .is_none()
        .then(|| aria_label.unwrap_or_else(|| label.clone()))
        .filter(|label| !label.is_empty());
    let invalid_alert =
        state == FieldState::Invalid && invalid_text.as_ref().is_some_and(|text| !text.is_empty());
    let feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    rsx! {
        fieldset { class: "ix-group-native ix-checkbox-group {state.class()} {class}", disabled,
            "aria-label": accessible_label, "aria-labelledby": aria_labelledby,
            GroupLayout { label, required, state, feedback, invalid_alert,
                div { class: if horizontal { "ix-group-choices ix-group-row" } else { "ix-group-choices" }, {children} }
            }
        }
    }
}

/// Native radio with Brand token geometry, selection dot and label typography.
/// Give controls the same `name` for native arrow navigation and form grouping.
/// The consumer accepts selections by updating `checked` from the callbacks;
/// the source's cancellable event and automatic group mutations are separate APIs.
#[component]
pub fn Radio(
    value: String,
    #[props(default)] label: String,
    #[props(default)] name: Option<String>,
    #[props(default)] checked: bool,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
    aria_label: Option<String>,
    #[props(default)] oncheckedchange: EventHandler<bool>,
    #[props(default)] onvaluechange: EventHandler<String>,
) -> Element {
    let selected = value.clone();
    let accessible_label = aria_label.unwrap_or_else(|| label.clone());
    rsx! {
        label { class: "ix-check ix-radio ix-radio-native {state.class()} {class}",
            "data-checked": checked.to_string(), "data-disabled": disabled.to_string(),
            "data-label-less": label.is_empty().to_string(),
            input { r#type: "radio", name, value, checked, disabled, required,
                "aria-label": accessible_label,
                "aria-invalid": (state == FieldState::Invalid).to_string(),
                // Radio change fires only when becoming checked. FormData::checked
                // parses checkbox booleans; a radio instead returns its value.
                onchange: move |_| { oncheckedchange.call(true); onvaluechange.call(selected.clone()); },
            }
            span { class: "ix-radio-button", "aria-hidden": "true",
                span { class: "ix-check-shape",
                    span { class: "ix-radio-dot" }
                }
            }
            if !label.is_empty() {
                crate::Typography { format: crate::TypographyFormat::Body,
                    text_color: if disabled { crate::TypographyColor::Weak } else { crate::TypographyColor::Std },
                    class: "ix-radio-label", "{label}"
                }
            }
        }
    }
}

/// Value and display text shared by select and radio group options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    /// Submitted value, independent of the display label.
    pub value: String,
    /// Visible and accessible option label.
    pub label: String,
    /// Prevents choosing this option.
    pub disabled: bool,
}

impl SelectOption {
    /// Create an enabled option.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
    /// Mark this option disabled.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Controlled radio group with native form semantics and automatic shared control name.
#[component]
pub fn RadioGroup(
    options: Vec<SelectOption>,
    #[props(default)] value: String,
    #[props(default)] label: String,
    #[props(default)] name: Option<String>,
    #[props(default)] horizontal: bool,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    #[props(default)] onvaluechange: EventHandler<String>,
) -> Element {
    let name = field_id(name);
    let accessible_label = aria_labelledby
        .is_none()
        .then(|| aria_label.unwrap_or_else(|| label.clone()))
        .filter(|label| !label.is_empty());
    let invalid_alert =
        state == FieldState::Invalid && invalid_text.as_ref().is_some_and(|text| !text.is_empty());
    let feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    rsx! {
        fieldset { class: "ix-group-native ix-radio-group {state.class()} {class}", disabled, role: "radiogroup",
            "aria-label": accessible_label, "aria-labelledby": aria_labelledby,
            GroupLayout { label, required, state, feedback, invalid_alert,
                div { class: if horizontal { "ix-group-choices ix-group-row" } else { "ix-group-choices" },
                    for option in options {
                        Radio { key: "{option.value}", name: name.clone(), checked: option.value == value,
                            value: option.value, label: option.label, disabled: disabled || option.disabled, required, state, onvaluechange }
                    }
                }
            }
        }
    }
}

/// Controlled three-state switch with a native form mirror and source token geometry.
/// Accept `oncheckedchange` by updating `checked` and clearing `indeterminate`.
#[component]
pub fn Toggle(
    #[props(default)] checked: bool,
    #[props(default)] name: Option<String>,
    #[props(default = "on".to_string())] value: String,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] indeterminate: bool,
    #[props(default)] hide_text: bool,
    #[props(default = "On".to_string())] text_on: String,
    #[props(default = "Off".to_string())] text_off: String,
    #[props(default = "Mixed".to_string())] text_indeterminate: String,
    #[props(default)] label: Option<String>,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(default)] oncheckedchange: EventHandler<bool>,
) -> Element {
    let mut button = use_signal(|| None::<Rc<MountedData>>);
    let mut attempted = use_signal(|| false);
    let text = if indeterminate {
        text_indeterminate
    } else if checked {
        text_on
    } else {
        text_off
    };
    let aria_labelledby = aria_labelledby.filter(|value| !value.is_empty());
    let accessible_label = if aria_labelledby.is_some() {
        None
    } else {
        aria_label.or(label).or(Some(text.clone()))
    };
    let invalid = state == FieldState::Invalid || (attempted() && required && !checked);
    rsx! {
        span { class: "ix-toggle-field ix-toggle-native {state.class()} {class}",
            "data-checked": checked.to_string(), "data-mixed": indeterminate.to_string(), "data-invalid": invalid.to_string(),
            input { class: "ix-check-native", r#type: "checkbox", name, value, checked, disabled, required,
                "aria-hidden": "true", tabindex: "-1",
                oninvalid: move |event| {
                    event.prevent_default(); attempted.set(true);
                    let node = button.peek().clone();
                    spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } });
                },
            }
            button { class: "ix-toggle ix-toggle-control {state.class()} {class}", r#type: "button", role: "switch", disabled,
                "data-mixed": indeterminate.to_string(), "data-checked": checked.to_string(),
                "aria-label": accessible_label, "aria-labelledby": aria_labelledby, "aria-disabled": disabled.to_string(),
                "aria-checked": if indeterminate { "mixed" } else if checked { "true" } else { "false" },
                "aria-required": required.to_string(), "aria-invalid": invalid.to_string(),
                onmounted: move |event| button.set(Some(event.data())), onclick: move |_| { if !disabled { oncheckedchange.call(!checked); } }, onblur,
                span { class: "ix-toggle-wrapper",
                    span { class: if checked { if indeterminate { "ix-toggle-switch checked indeterminate" } else { "ix-toggle-switch checked" } } else if indeterminate { "ix-toggle-switch indeterminate" } else { "ix-toggle-switch" },
                        "data-checked": checked.to_string(), "data-mixed": indeterminate.to_string(), "aria-hidden": "true",
                        span { class: "ix-toggle-slider" }
                    }
                    if !hide_text {
                        crate::Typography { format: crate::TypographyFormat::Body,
                            class: "ix-toggle-label", aria_hidden: "true", "{text}"
                        }
                    }
                }
            }
            if attempted() && required && !checked { span { class: "ix-helper ix-field-invalid", role: "alert", "Please enable this option" } }
        }
    }
}

/// Native numeric input with branded controls and min/max/step validation.
///
/// `value` and `onvaluechange` retain the editable string (including scientific
/// notation). `onnumericvaluechange` reports its parsed number on each input or
/// step; empty input reports `Some(0.0)` unless `allow_empty_value_change` is set.
/// `oncommit` reports the actual optional number on changed blur or Enter, once
/// per change. Blur normalizes scientific notation without another value event.
#[component]
pub fn NumberInput(
    #[props(default = "0".to_string())] value: String,
    #[props(default)] label: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] name: Option<String>,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] pattern: Option<String>,
    #[props(default)] allowed_characters_pattern: Option<String>,
    #[props(default)] min: Option<f64>,
    #[props(default)] max: Option<f64>,
    #[props(default = 1.0)] step: f64,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
    #[props(default)] show_text_as_tooltip: bool,
    #[props(default)] show_stepper_buttons: bool,
    #[props(default)] allow_empty_value_change: bool,
    #[props(default)] suppress_submit_on_enter: bool,
    #[props(default = true)] text_alignment_end: bool,
    #[props(default)] controller: Option<crate::TextControlController>,
    #[props(default)] start: Option<Element>,
    #[props(default)] end: Option<Element>,
    #[props(default)] onvaluechange: EventHandler<String>,
    #[props(default)] onnumericvaluechange: EventHandler<Option<f64>>,
    #[props(default)] oncommit: EventHandler<Option<f64>>,
    #[props(default)] onfocus: EventHandler<FocusEvent>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(default)] onixblur: EventHandler<()>,
    #[props(default)] onvaliditystatechange: EventHandler<crate::ValidityStateChangeEvent>,
) -> Element {
    let mut draft = use_signal(|| value.clone());
    let mut committed = use_signal(|| NumberCommit::new(&value));
    // The local draft lets blur normalize the native text without inventing a
    // valueChange event. Later external controlled updates still replace it.
    use_effect(use_reactive((&value,), move |(value,)| draft.set(value)));
    let mut emit_value = move |text: String| {
        draft.set(text.clone());
        onnumericvaluechange.call(number_input_event_value(&text, allow_empty_value_change));
        onvaluechange.call(text);
    };
    let allowed_characters_pattern =
        allowed_characters_pattern.or_else(|| Some(r"[0-9Ee+\-.,]".to_string()));
    let number_class = format!(
        "ix-number-input-native{} {class}",
        if show_stepper_buttons {
            " ix-number-has-steppers"
        } else {
            ""
        }
    );
    let steppers_hidden = !show_stepper_buttons || disabled || readonly;
    rsx! {
        div { class: "ix-number-field",
            Input { value: draft(), label, id, name, placeholder, aria_label, pattern,
                allowed_characters_pattern, controller, input_type: "number", autocomplete: "off",
                min: min.map(|v| v.to_string()), max: max.map(|v| v.to_string()),
                step: step.to_string(), disabled, readonly, required,
                helper_text, invalid_text, valid_text, info_text, warning_text, state,
                class: number_class, show_text_as_tooltip,
                onvaluechange: emit_value, text_alignment_end, suppress_submit_on_enter,
                onixblur, onvaliditystatechange, start,
                onfocus: move |event| {
                    committed.write().focus(&draft.peek());
                    onfocus.call(event);
                },
                onblur: move |event| {
                    let current = number_input_value(&draft.peek());
                    if let Some(value) = current { draft.set(number_input_format_value(value)); }
                    onblur.call(event);
                    if committed.write().commit(current) { oncommit.call(current); }
                },
                end: rsx! {
                    {end}
                    span { class: "ix-number-steppers", "data-hidden": steppers_hidden.to_string(),
                        // Preserve the existing Rust focus-retention adapter.
                        onmousedown: move |event| event.prevent_default(),
                        crate::IconButton { class: "ix-number-stepper ix-number-minus",
                            variant: crate::ButtonVariant::SubtleTertiary, size: crate::ButtonIconSize::Sixteen,
                            label: "decrement number", icon: "minus",
                            onclick: move |_| { if !steppers_hidden { let next = stepped_number(&draft.peek(), step, min, max, false); emit_value(next); } },
                        }
                        crate::IconButton { class: "ix-number-stepper ix-number-plus",
                            variant: crate::ButtonVariant::SubtleTertiary, size: crate::ButtonIconSize::Sixteen,
                            label: "increment number", icon: "plus",
                            onclick: move |_| { if !steppers_hidden { let next = stepped_number(&draft.peek(), step, min, max, true); emit_value(next); } },
                        }
                    }
                },
                onkeydown: move |event: KeyboardEvent| if !disabled && !readonly {
                    match event.key() {
                        Key::ArrowUp | Key::ArrowDown => {
                            event.prevent_default();
                            let next = stepped_number(&draft.peek(), step, min, max, event.key() == Key::ArrowUp);
                            emit_value(next);
                        }
                        Key::Enter if !event.is_composing() => {
                            let current = number_input_value(&draft.peek());
                            if committed.write().commit(current) { oncommit.call(current); }
                        }
                        _ => {}
                    }
                },
            }
        }
    }
}

fn number_input_value(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn number_input_format_value(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if value.abs() < 1e-6 || value.abs() >= 1e21 {
        let text = format!("{value:e}");
        if let Some((coefficient, exponent)) = text.split_once('e') {
            if !exponent.starts_with('-') {
                return format!("{coefficient}e+{exponent}");
            }
        }
        return text;
    }
    value.to_string()
}

fn number_has_valid_value(value: &str, required: bool) -> bool {
    if value.is_empty() {
        !required
    } else {
        number_input_value(value).is_some()
    }
}

#[cfg(test)]
mod numeric_presence_tests {
    #[test]
    fn optional_empty_numeric_values_are_present_without_accepting_invalid_numbers() {
        assert!(super::number_has_valid_value("", false));
        assert!(!super::number_has_valid_value("", true));
        assert!(super::number_has_valid_value("0", true));
        assert!(super::number_has_valid_value("1e3", true));
        assert!(!super::number_has_valid_value("1e", false));
        assert!(!super::number_has_valid_value("NaN", false));
    }
}

fn number_input_event_value(value: &str, allow_empty: bool) -> Option<f64> {
    number_input_value(value).or_else(|| (!allow_empty).then_some(0.0))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NumberCommit(Option<f64>);

impl NumberCommit {
    fn new(value: &str) -> Self {
        Self(number_input_value(value))
    }
    fn focus(&mut self, value: &str) {
        self.0 = number_input_value(value);
    }
    fn commit(&mut self, value: Option<f64>) -> bool {
        let changed = self.0 != value;
        self.0 = value;
        changed
    }
}

fn decimal_places(number: f64) -> i32 {
    let text = number.to_string();
    let (coefficient, exponent) = text
        .split_once(['e', 'E'])
        .map(|(value, exponent)| (value, exponent.parse::<i32>().unwrap_or(0)))
        .unwrap_or((&text, 0));
    let fraction = coefficient
        .split_once('.')
        .map(|(_, fraction)| fraction.len() as i32)
        .unwrap_or(0);
    (fraction - exponent).clamp(0, 100)
}

fn stepped_number(value: &str, step: f64, min: Option<f64>, max: Option<f64>, up: bool) -> String {
    let current = value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .unwrap_or(0.0);
    let step = if step.is_finite() { step } else { 1.0 };
    let precision = 10.0_f64.powi(decimal_places(current).max(decimal_places(step)));
    let mut next = current + if up { step } else { -step };
    if next.abs() < 1e21 {
        next = (next * precision).round() / precision;
    }
    if let Some(min) = min.filter(|value| value.is_finite()) {
        next = next.max(min);
    }
    if let Some(max) = max.filter(|value| value.is_finite()) {
        next = next.min(max);
    }
    if next == 0.0 {
        next = 0.0;
    }
    next.to_string()
}

/// Slider tick with a value and optional display label.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderMarker {
    /// Position in the slider's numeric range.
    pub value: f64,
    /// Caption under this position.
    pub label: String,
}

fn bounded_slider_value(value: f64, min: f64, max: f64) -> f64 {
    let (min, max) = slider_bounds(min, max);
    if !value.is_finite() {
        return min;
    }
    value.clamp(min, max)
}

fn slider_bounds(min: f64, max: f64) -> (f64, f64) {
    let min = if min.is_finite() { min } else { 0.0 };
    let max = if max.is_finite() { max } else { 100.0 };
    (min.min(max), min.max(max))
}

/// Native range slider with source track, trace and numeric tick presentation.
/// Values remain controlled. Native disabled/submission, finite-bound safeguards,
/// marker captions and optional value output are Rust adapters. The existing
/// value-tooltip renderer does not certify upstream floating/drag chronology.
#[component]
pub fn Slider(
    #[props(default)] value: f64,
    #[props(default)] min: f64,
    #[props(default = 100.0)] max: f64,
    #[props(default = 1.0)] step: f64,
    #[props(default)] label: String,
    #[props(default)] name: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] trace: bool,
    #[props(default)] trace_reference: f64,
    #[props(default)] markers: Vec<SliderMarker>,
    #[props(default)] show_value: bool,
    #[props(default = true)] show_tooltip: bool,
    #[props(default)] label_start: Option<String>,
    #[props(default)] label_end: Option<String>,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] show_text_as_tooltip: bool,
    #[props(default)] class: String,
    #[props(default)] state: FieldState,
    #[props(default)] onvaluechange: EventHandler<f64>,
) -> Element {
    let id = field_id(None);
    let anchor_id = format!("{id}-anchor");
    let feedback_id = format!("{id}-feedback");
    let (min, max) = slider_bounds(min, max);
    let value = bounded_slider_value(value, min, max);
    let mut focused = use_signal(|| false);
    let mut hovered = use_signal(|| false);
    let span = (max - min).max(f64::EPSILON);
    let fraction = (value - min) / span;
    let reference = (bounded_slider_value(trace_reference, min, max) - min) / span;
    let start = fraction.min(reference);
    let end = fraction.max(reference);
    let has_endpoints = label_start.is_some() || label_end.is_some();
    let label_start = label_start.unwrap_or_default();
    let label_end = label_end.unwrap_or_default();
    let state = if state == FieldState::Default {
        crate::form_control::validation_class_state(&class)
    } else {
        state
    };
    let invalid_alert =
        state == FieldState::Invalid && invalid_text.as_ref().is_some_and(|text| !text.is_empty());
    let feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    let feedback_present = feedback.is_some();
    let captions: Vec<_> = markers
        .iter()
        .filter(|marker| marker.value >= min && marker.value <= max && !marker.label.is_empty())
        .cloned()
        .collect();
    let has_markers = !markers.is_empty();
    rsx! {
        div { class: "ix-slider-field ix-text-field-native {state.class()} {class}",
            "data-disabled": disabled.to_string(),
            if show_value { output { class: "ix-slider-value-output", "for": id.clone(), "{value}" } }
            TextFieldPresentation { label: (!label.is_empty()).then_some(label.clone()), control_id: id.clone(),
                anchor_id: anchor_id.clone(), feedback_id: feedback_id.clone(), required: false, state,
                feedback, invalid_alert, show_text_as_tooltip,
                max_length: None, length: 0,
                div { id: anchor_id, class: "ix-slider-container",
                    div { class: "ix-slider-control",
                        onmouseenter: move |_| hovered.set(true), onmouseleave: move |_| hovered.set(false),
                        div { class: "ix-slider-track",
                            div { class: "ix-slider-thumb-anchor", style: "left:calc({fraction} * (100% - var(--thumb-size)))" }
                            svg { class: "ix-slider-ticks", xmlns: "http://www.w3.org/2000/svg", "aria-hidden": "true",
                                for marker in markers.iter().filter(|marker| marker.value >= min && marker.value <= max) {
                                    circle { class: "ix-slider-tick",
                                        class: if trace && marker.value >= value.min(trace_reference) && marker.value <= value.max(trace_reference) { "ix-slider-tick-active" },
                                        class: if marker.value == min { "ix-slider-tick-at-min" },
                                        class: if marker.value == max && max > min { "ix-slider-tick-at-max" },
                                        cx: "0", cy: "0", style: format!("--tick-value:{}", (marker.value-min)/span),
                                    }
                                }
                            }
                        }
                        input { class: "ix-slider",
                            class: if trace && fraction != reference { "trace" },
                            class: if trace && (trace_reference <= min || trace_reference >= max) { "hide-trace-reference" },
                            id: id.clone(), r#type: "range", name,
                            min, max, step: step.max(f64::EPSILON), value: value.to_string(), disabled, tabindex: if disabled { -1 } else { 0 },
                            role: "slider", "aria-label": if label.is_empty() { "Slider".to_string() } else { label },
                            "aria-valuenow": value.to_string(), "aria-valuemin": min.to_string(), "aria-valuemax": max.to_string(),
                            "aria-invalid": (state == FieldState::Invalid).to_string(),
                            "aria-describedby": feedback_present.then_some(feedback_id),
                            list: has_markers.then_some("markers"),
                            style: "--value:{fraction};--trace-reference:{reference};--trace-start:{start};--trace-end:{end}",
                            oninput: move |event| if !disabled { if let Ok(value) = event.value().parse::<f64>() { onvaluechange.call(bounded_slider_value(value, min, max)); } },
                            onfocus: move |_| focused.set(true), onblur: move |_| focused.set(false),
                        }
                        if show_tooltip && !disabled && (focused() || hovered()) {
                            output { class: "ix-slider-tooltip", style: "left:calc({fraction} * (100% - var(--thumb-size)) + var(--thumb-size) / 2)", "aria-hidden": "true", "{value}" }
                        }
                    }
                    if has_endpoints {
                        div { class: "ix-slider-endpoints", div { span { "{label_start}" } } div { span { "{label_end}" } } }
                    }
                    if !captions.is_empty() {
                        div { class: "ix-slider-marker-captions", "aria-hidden": "true",
                            for marker in captions { span { style: "left:calc({(marker.value-min)/span} * (100% - var(--thumb-size)) + var(--thumb-size) / 2)", "{marker.label}" } }
                        }
                    }
                }
            }
        }
    }
}

/// Combines two arbitrary field controls into the upstream range-field layout.
#[component]
pub fn RangeField(children: Element, #[props(default)] hide_arrow: bool) -> Element {
    crate::datetime::provide_range_navigation();
    rsx! { div { class: if hide_arrow { "ix-range-field ix-range-no-arrow" } else { "ix-range-field" }, {children} } }
}

fn update_selection(mut values: Vec<String>, value: String, multiple: bool) -> Vec<String> {
    if !multiple {
        return vec![value];
    }
    if let Some(index) = values.iter().position(|item| *item == value) {
        values.remove(index);
    } else {
        values.push(value);
    }
    values
}

fn publish_selection(
    values: Vec<String>,
    single: EventHandler<String>,
    multiple: EventHandler<Vec<String>>,
) {
    single.call(values.first().cloned().unwrap_or_default());
    multiple.call(values);
}

/// Number of leading chips that fit, preserving the first chip like iX. Widths
/// include actual rendered glyphs, padding and remove controls; no text estimate.
fn fitting_chips(widths: &[f64], badges: &[f64], available: f64, gap: f64) -> usize {
    if widths.is_empty() {
        return 0;
    }
    let total = widths.iter().sum::<f64>() + gap * widths.len().saturating_sub(1) as f64;
    if total <= available {
        return widths.len();
    }
    let mut used = widths[0];
    let mut visible = 1;
    for (index, width) in widths.iter().enumerate().skip(1) {
        let remaining = widths.len() - index - 1;
        let reserve = if remaining == 0 {
            0.0
        } else {
            gap + badges.get(remaining - 1).copied().unwrap_or(0.0)
        };
        if used + gap + width + reserve > available {
            break;
        }
        used += gap + width;
        visible += 1;
    }
    visible
}

fn select_matches(label: &str, query: &str) -> bool {
    label.to_lowercase().contains(&query.trim().to_lowercase())
}

fn select_can_add(options: &[SelectOption], query: &str, editable: bool) -> bool {
    editable
        && !query.trim().is_empty()
        && !options.iter().any(|option| option.label == query.trim())
}

fn select_remove_label(prefix: &str, label: &str) -> String {
    if label.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {label}")
    }
}

fn select_more_label(template: &str, count: usize) -> String {
    template.replacen("{count}", &count.to_string(), 1)
}

fn select_field_state(
    explicit: FieldState,
    required: bool,
    touched: bool,
    has_value: bool,
) -> FieldState {
    if explicit == FieldState::Default && required && touched && !has_value {
        FieldState::Invalid
    } else {
        explicit
    }
}

/// Failure of a scoped Select DOM method. One mounted Select may use a controller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectMethodError {
    NotMounted,
    MultipleMounted,
    Dom(String),
}
impl std::fmt::Display for SelectMethodError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMounted => formatter.write_str("Select input is not mounted"),
            Self::MultipleMounted => {
                formatter.write_str("Select controller is attached to multiple selects")
            }
            Self::Dom(message) => formatter.write_str(message),
        }
    }
}
impl std::error::Error for SelectMethodError {}

#[derive(Clone)]
struct SelectMethodBinding {
    id: String,
    required: bool,
    has_value: bool,
    touched: Rc<std::cell::Cell<bool>>,
    alive: Rc<std::cell::Cell<bool>>,
    input: Rc<std::cell::RefCell<Option<Rc<MountedData>>>>,
}

/// Rust descriptor for the native combobox input. It is scoped to this mount:
/// retaining it after unmount returns [`SelectMethodError::NotMounted`].
#[derive(Clone)]
pub struct SelectNativeInput {
    id: String,
    alive: Rc<std::cell::Cell<bool>>,
    node: Rc<MountedData>,
}
impl SelectNativeInput {
    /// DOM id suitable for an accessible association or consumer-owned DOM bridge.
    pub fn id(&self) -> &str {
        &self.id
    }
    pub async fn focus(&self) -> Result<(), SelectMethodError> {
        if !self.alive.get() {
            return Err(SelectMethodError::NotMounted);
        }
        if let Err(error) = self.node.set_focus(true).await {
            if !self.alive.get() {
                return Err(SelectMethodError::NotMounted);
            }
            // Desktop alpha.2 can focus successfully but fail to decode the
            // native query's null result as bool. Confirm the actual DOM state
            // through the acknowledged query channel before retrying focus.
            #[cfg(any(feature = "web", feature = "desktop"))]
            {
                let task = crate::dom_query::eval(
                    r#"
                    const id = await dioxus.recv();
                    const input = document.getElementById(id);
                    if (!(input instanceof HTMLInputElement) || !input.isConnected) return false;
                    if (document.activeElement !== input) input.focus();
                    return document.activeElement === input;
                "#,
                );
                task.send(&self.id)
                    .map_err(|failure| SelectMethodError::Dom(failure.to_string()))?;
                let focused = task.join::<bool>().await;
                if !self.alive.get() {
                    return Err(SelectMethodError::NotMounted);
                }
                let focused =
                    focused.map_err(|failure| SelectMethodError::Dom(failure.to_string()))?;
                if !focused {
                    return Err(SelectMethodError::Dom(error.to_string()));
                }
            }
            #[cfg(not(any(feature = "web", feature = "desktop")))]
            return Err(SelectMethodError::Dom(error.to_string()));
        }
        if !self.alive.get() {
            return Err(SelectMethodError::NotMounted);
        }
        Ok(())
    }
}

/// Snapshot of the input's associated native form. Rust does not expose a raw
/// JavaScript `HTMLFormElement`; query again after changing the form's attributes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct SelectFormElement {
    pub id: String,
    pub name: String,
    pub method: String,
    pub action: String,
    pub no_validate: bool,
}

/// Scoped adaptation of iX Select's public input methods and internal form/value
/// queries. Values remain controlled through `value`/`values` and their callbacks.
#[derive(Clone, Default)]
pub struct SelectController {
    bindings: Rc<std::cell::RefCell<std::collections::BTreeMap<usize, SelectMethodBinding>>>,
}
impl PartialEq for SelectController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bindings, &other.bindings)
    }
}
/// Create a stable controller following Dioxus's rules of hooks. Attach it to
/// one Select using its `controller` prop.
pub fn use_select() -> SelectController {
    use_hook(SelectController::default)
}
impl SelectController {
    fn binding(&self) -> Result<SelectMethodBinding, SelectMethodError> {
        let bindings = self.bindings.borrow();
        if bindings.len() > 1 {
            return Err(SelectMethodError::MultipleMounted);
        }
        bindings
            .values()
            .next()
            .filter(|binding| binding.alive.get())
            .cloned()
            .ok_or(SelectMethodError::NotMounted)
    }
    /// Mirrors upstream's internal `hasValidValue`: **required AND nonempty**.
    /// This is not the browser's general constraint-validity result.
    pub fn has_valid_value(&self) -> Result<bool, SelectMethodError> {
        let binding = self.binding()?;
        Ok(binding.required && binding.has_value)
    }
    /// Whether the visible input has emitted blur, including optional fields and
    /// fields inside a `noValidate` form. Validation presentation is independent.
    pub fn is_touched(&self) -> Result<bool, SelectMethodError> {
        Ok(self.binding()?.touched.get())
    }
    pub fn get_native_input_element(&self) -> Result<SelectNativeInput, SelectMethodError> {
        let binding = self.binding()?;
        let node = binding
            .input
            .borrow()
            .clone()
            .ok_or(SelectMethodError::NotMounted)?;
        Ok(SelectNativeInput {
            id: binding.id,
            alive: binding.alive,
            node,
        })
    }
    pub async fn focus_input(&self) -> Result<(), SelectMethodError> {
        self.get_native_input_element()?.focus().await
    }
    /// Native form metadata, or `None` when the input is not in a form.
    pub async fn get_associated_form_element(
        &self,
    ) -> Result<Option<SelectFormElement>, SelectMethodError> {
        let binding = self.binding()?;
        if binding.input.borrow().is_none() {
            return Err(SelectMethodError::NotMounted);
        }
        let task = crate::dom_query::eval(
            r#"
            const id = await dioxus.recv();
            const input = document.getElementById(id);
            if (!(input instanceof HTMLInputElement)) throw new Error('Select input is not mounted');
            const form = input.form;
            return form ? {id:form.id,name:form.name,method:form.method,action:form.action,no_validate:form.noValidate} : null;
        "#,
        );
        task.send(binding.id)
            .map_err(|error| SelectMethodError::Dom(error.to_string()))?;
        let result = task
            .join::<Option<SelectFormElement>>()
            .await
            .map_err(|error| SelectMethodError::Dom(error.to_string()))?;
        if !binding.alive.get() {
            return Err(SelectMethodError::NotMounted);
        }
        Ok(result)
    }
}

fn select_feedback_text(
    state: FieldState,
    helper: Option<String>,
    invalid: Option<String>,
    valid: Option<String>,
    info: Option<String>,
    warning: Option<String>,
) -> Option<String> {
    match state {
        FieldState::Invalid => invalid,
        FieldState::Valid => valid,
        FieldState::Info => info,
        FieldState::Warning => warning,
        FieldState::Default => None,
    }
    .or(helper)
}

/// Cancelable request emitted before an editable select inserts and selects an item.
/// Call `prevent_default()` synchronously to leave options, value and input unchanged.
#[derive(Clone, Debug)]
pub struct SelectAddEvent {
    pub value: String,
    prevented: Rc<std::cell::Cell<bool>>,
}
impl SelectAddEvent {
    fn new(value: String) -> Self {
        Self {
            value,
            prevented: Rc::new(std::cell::Cell::new(false)),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

fn select_add_allowed(value: String, before: EventHandler<SelectAddEvent>) -> bool {
    let event = SelectAddEvent::new(value);
    before.call(event.clone());
    !event.default_prevented()
}

// Native geometry and lifecycle plumbing. Filtering, additions, selection and
// open/close state remain Rust-owned. The input is the invoker so typed keys
// remain in the combobox while the surface enters the top layer.
#[cfg(any(feature = "web", feature = "desktop"))]
const SELECT_POPOVER_BRIDGE: &str = r#"
const [id, anchorId, inputId] = await dioxus.recv();
const panel = document.getElementById(id), anchor = document.getElementById(anchorId), input = document.getElementById(inputId);
if (!panel || !anchor || !input) return {error:'Select surface is not mounted'};
if (typeof panel.showPopover !== 'function') return {error:'Popover API is unavailable'};
let disposed = false, frame = 0;
const events = new AbortController();
const place = () => {
  if (disposed || !panel.isConnected || !anchor.isConnected) { cleanup(); return; }
  const viewport = window.visualViewport;
  const left = viewport?.offsetLeft ?? 0, top = viewport?.offsetTop ?? 0;
  const width = viewport?.width ?? document.documentElement.clientWidth;
  const height = viewport?.height ?? window.innerHeight;
  const rect = anchor.getBoundingClientRect(), roomWidth = Math.max(0, width);
  const requestedWidth = panel.dataset.requestedWidth, requestedMax = panel.dataset.requestedMaxWidth;
  const direction = getComputedStyle(anchor).direction;
  panel.style.minWidth = Math.min(rect.width, roomWidth) + 'px';
  panel.style.width = Math.min(rect.width, roomWidth) + 'px';
  panel.style.width = requestedWidth ? `min(${requestedWidth}, ${roomWidth}px)` : Math.min(rect.width, roomWidth) + 'px';
  panel.style.maxWidth = roomWidth + 'px';
  panel.style.maxWidth = requestedMax ? `min(${requestedMax}, ${roomWidth}px)` : roomWidth + 'px';
  panel.style.maxHeight = 'none';
  const desired = panel.offsetHeight;
  const below = Math.max(0, top + height - rect.bottom), above = Math.max(0, rect.top - top);
  const flipped = below < desired && above >= desired;
  const room = flipped ? above : below;
  panel.style.maxHeight = room + 'px';
  const start = direction === 'rtl' ? rect.right - panel.offsetWidth : rect.left;
  panel.style.left = Math.max(left, Math.min(start, left + width - panel.offsetWidth)) + 'px';
  panel.style.top = (flipped ? rect.top - Math.min(desired, room) : rect.bottom) + 'px';
  panel.dataset.placement = flipped ? 'top-start' : 'bottom-start';
  panel.dataset.direction = direction;
};
const schedule = () => { if (!disposed && !frame) frame = requestAnimationFrame(() => { frame = 0; place(); }); };
const resize = new ResizeObserver(schedule);
const changes = new MutationObserver(schedule);
const ancestry = new MutationObserver(schedule);
const removal = new MutationObserver(() => { if (!panel.isConnected || !anchor.isConnected) cleanup(); });
function cleanup() {
  if (disposed) return;
  disposed = true; events.abort(); resize.disconnect(); changes.disconnect(); ancestry.disconnect(); removal.disconnect();
  cancelAnimationFrame(frame); delete panel.__ixSelectCleanup;
}
panel.__ixSelectCleanup = cleanup;
try {
  panel.showPopover({source:input}); place();
  resize.observe(anchor); resize.observe(panel);
  changes.observe(panel, {childList:true,subtree:true,characterData:true,attributes:true,attributeFilter:['data-requested-width','data-requested-max-width']});
  for (let node = anchor; node; node = node.parentElement) ancestry.observe(node, {attributes:true,attributeFilter:['dir','style','class']});
  removal.observe(document.documentElement, {childList:true,subtree:true});
  window.addEventListener('resize', schedule, {signal:events.signal});
  document.addEventListener('scroll', schedule, {capture:true,signal:events.signal});
  window.visualViewport?.addEventListener('resize', schedule, {signal:events.signal});
  window.visualViewport?.addEventListener('scroll', schedule, {signal:events.signal});
  document.fonts?.addEventListener('loadingdone', schedule, {signal:events.signal});
  panel.addEventListener('toggle', event => { if (event.newState === 'closed') cleanup(); }, {signal:events.signal});
  return {error:null};
} catch(error) { cleanup(); return {error:String(error)}; }
"#;

#[component]
fn SelectOptionsSurface(
    anchor_id: String,
    input_id: String,
    enable_top_layer: bool,
    dropdown_width: Option<String>,
    dropdown_max_width: Option<String>,
    onnativeerror: EventHandler<crate::NativeSurfaceError>,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    let surface_id = field_id(None);
    let mut native = use_signal(|| enable_top_layer);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let alive = use_hook(|| Rc::new(std::cell::Cell::new(true)));
    let lifetime = alive.clone();
    let cleanup_id = surface_id.clone();
    use_drop(move || {
        lifetime.set(false);
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            let encoded = serde_json::to_string(&cleanup_id).expect("string serialization");
            let _ = dioxus::document::eval(&format!(
                "document.getElementById({encoded})?.__ixSelectCleanup?.();"
            ));
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        let _ = cleanup_id;
    });
    let inline_style = format!(
        "width:{};max-width:{}",
        dropdown_width.as_deref().unwrap_or("100%"),
        dropdown_max_width.as_deref().unwrap_or("100%")
    );
    let mounted_id = surface_id.clone();
    rsx! {
        if !native() {
            button { r#type: "button", class: "ix-select-backdrop", tabindex: "-1", "aria-label": "Close selection list", onclick: move |_| onclose.call(()) }
        }
        div { id: surface_id, class: "ix-select-popup", style: if native() { String::new() } else { inline_style },
            "popover": native().then_some("auto"), "data-top-layer": native().to_string(),
            "data-requested-width": dropdown_width.clone(), "data-requested-max-width": dropdown_max_width.clone(),
            onmounted: move |event| {
                mounted.set(Some(event.data()));
                if !enable_top_layer { return; }
                let alive = alive.clone();
                let id = mounted_id.clone(); let anchor = anchor_id.clone(); let input = input_id.clone();
                spawn(async move {
                    #[cfg(any(feature = "web", feature = "desktop"))]
                    let failure = {
                        #[derive(serde::Deserialize)] struct Reply { error: Option<String> }
                        let bridge = crate::dom_query::eval(SELECT_POPOVER_BRIDGE);
                        if let Err(error) = bridge.send((id, anchor, input)) { Some(crate::NativeSurfaceError::Rejected(error.to_string())) }
                        else { match bridge.join::<Reply>().await { Ok(reply) => reply.error.map(crate::NativeSurfaceError::Rejected), Err(error) => Some(crate::NativeSurfaceError::Rejected(error.to_string())) } }
                    };
                    #[cfg(not(any(feature = "web", feature = "desktop")))]
                    let failure = { let _ = (id, anchor, input); Some(crate::NativeSurfaceError::UnsupportedRenderer) };
                    if alive.get() { if let Some(error) = failure { native.set(false); onnativeerror.call(error); } }
                });
            },
            ontoggle: move |_| {
                let node = mounted.peek().clone();
                spawn(async move { if let Some(node) = node { if let Ok(bounds) = node.get_client_rect().await {
                    if *native.peek() && bounds.size.width == 0.0 && bounds.size.height == 0.0 { onclose.call(()); }
                } } });
            },
            {children}
        }
    }
}

// An input can have focus without any option having visual focus. The add row
// participates in the same navigation sequence as enabled source/dynamic items.
fn select_next_active(enabled: &[usize], active: usize, backwards: bool) -> Option<usize> {
    if enabled.is_empty() {
        return None;
    }
    let current = enabled.iter().position(|index| *index == active);
    Some(match current {
        None => {
            if backwards {
                *enabled.last().unwrap()
            } else {
                enabled[0]
            }
        }
        Some(index) => {
            enabled[if backwards {
                (index + enabled.len() - 1) % enabled.len()
            } else {
                (index + 1) % enabled.len()
            }]
        }
    })
}

#[derive(serde::Deserialize)]
struct ChipMeasurements {
    values: Vec<String>,
    widths: Vec<f64>,
    badges: Vec<f64>,
    available: f64,
    gap: f64,
}

// DOM plumbing only: all partitioning, selection and focus state stays in Rust.
// Observe intrinsic chip widths as well as container size (including font/density
// changes). Both observers disconnect when this component leaves the document.
const SELECT_MEASURE: &str = r#"
const id = await dioxus.recv();
const root = document.getElementById(id);
if (!root) return;
let frame = 0, previous = '', done = () => {};
const measure = () => {
  frame = 0;
  if (!root.isConnected) { resize.disconnect(); changes.disconnect(); removal.disconnect(); done(); return; }
  const control = root.querySelector('.ix-select-control');
  const rail = root.querySelector('.ix-select-measure');
  if (!control || !rail || !control.clientWidth) return;
  const trigger = control.querySelector('.ix-select-trigger');
  const clear = control.querySelector('.ix-select-clear');
  const chevron = control.querySelector('.ix-select-chevron');
  const chips = [...rail.querySelectorAll('[data-measure-value]')];
  const style = getComputedStyle(root.querySelector('.ix-select-chips') || rail);
  const payload = {
    values: chips.map(n => n.dataset.measureValue),
    widths: chips.map(n => n.getBoundingClientRect().width),
    badges: [...rail.querySelectorAll('[data-measure-badge]')].map(n => n.getBoundingClientRect().width),
    available: Math.max(0, control.clientWidth - (trigger?.getBoundingClientRect().width || 0) - (clear?.getBoundingClientRect().width || 0) - (chevron?.getBoundingClientRect().width || 0) - parseFloat(style.paddingLeft || 0) - parseFloat(style.paddingRight || 0)),
    gap: parseFloat(style.columnGap) || 0
  };
  const json = JSON.stringify(payload);
  if (json !== previous) { previous = json; dioxus.send(payload); }
};
const schedule = () => { if (!frame) frame = requestAnimationFrame(measure); };
const resize = new ResizeObserver(schedule);
const observe = () => { resize.disconnect(); resize.observe(root); root.querySelectorAll('.ix-select-control,.ix-select-measure > *').forEach(n => resize.observe(n)); schedule(); };
const changes = new MutationObserver(observe);
changes.observe(root, { childList: true, subtree: true, characterData: true, attributes: true });
const removal = new MutationObserver(() => { if (!root.isConnected) { resize.disconnect(); changes.disconnect(); removal.disconnect(); cancelAnimationFrame(frame); done(); } });
removal.observe(document.documentElement, { childList: true, subtree: true });
await new Promise(resolve => { done = resolve; observe(); });
return null;
"#;

/// Branded select with single/multiple selection, clear, filtering, editable additions and
/// Arrow/Home/End/Enter/Escape keyboard handling. `value` controls single selection;
/// `values` and `onvalueschange` control multiple selection. A native select mirrors submission.
#[component]
pub fn Select(
    options: Vec<SelectOption>,
    /// Optional scoped input-method controller created with [`use_select`].
    #[props(default)]
    controller: Option<SelectController>,
    #[props(default)] value: String,
    #[props(default)] values: Vec<String>,
    #[props(default)] multiple: bool,
    #[props(default)] editable: bool,
    /// Retained for compatibility; all selects accept inline filtering like iX.
    #[props(default)]
    filterable: bool,
    #[props(default)] allow_clear: bool,
    /// Collapse all selected options into one removable "All (N)" chip.
    #[props(default)]
    collapse_multiple_selection: bool,
    #[props(default)] label: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] name: Option<String>,
    #[props(default = "Select an option".to_string())] placeholder: String,
    #[props(default = "Type of select option".to_string())] editable_placeholder: String,
    #[props(default = "Select an option".to_string())] list_header: String,
    #[props(default)] hide_list_header: bool,
    #[props(default = "Clear selection".to_string())] aria_label_clear: String,
    #[props(default = "Add item".to_string())] aria_label_add: String,
    #[props(default = "All".to_string())] all_selected_text: String,
    /// Accessible overflow-chip label; the first `{count}` is replaced with its hidden count.
    #[props(default = "{count} more".to_string())]
    more_items_text: String,
    /// Prefix used before each selected chip's label in its remove button name.
    #[props(default = "Remove".to_string())]
    remove_selected_item_text: String,
    #[props(default)] dropdown_width: Option<String>,
    #[props(default)] dropdown_max_width: Option<String>,
    /// Opt into native Popover API rendering to escape clipping ancestors (iX default: false).
    #[props(default)]
    enable_top_layer: bool,
    /// Reports native activation failures; the list remains usable inline.
    #[props(default)]
    onnativeerror: EventHandler<crate::NativeSurfaceError>,
    #[props(default = "No matches".to_string())] no_matches_text: String,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    /// Present helper/active validation feedback below the control as a tooltip.
    /// Pointer hover and keyboard focus reveal it after the source's 500ms delay.
    #[props(default)]
    show_text_as_tooltip: bool,
    #[props(default)] state: FieldState,
    #[props(default)] onvaluechange: EventHandler<String>,
    #[props(default)] onvalueschange: EventHandler<Vec<String>>,
    #[props(default)] onadd: EventHandler<String>,
    /// Cancel insertion and selection synchronously with `event.prevent_default()`.
    #[props(default)]
    onbeforeadd: EventHandler<SelectAddEvent>,
    /// Raw text entered in the visible combobox, independent of selected values.
    #[props(default)]
    oninputchange: EventHandler<String>,
    #[props(default)] onixblur: EventHandler<()>,
) -> Element {
    let id = field_id(id);
    let list_id = format!("{id}-list");
    let control_id = format!("{id}-control");
    let help_id = format!("{id}-help");
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let mut active = use_signal(|| usize::MAX);
    let mut query_dirty = use_signal(|| false);
    let mut invalid_attempted = use_signal(|| false);
    let touched = use_hook(|| Rc::new(std::cell::Cell::new(false)));
    let alive = use_hook(|| Rc::new(std::cell::Cell::new(true)));
    let method_input = use_hook(|| Rc::new(std::cell::RefCell::new(None::<Rc<MountedData>>)));
    let method_token = use_hook(|| FIELD_ID.fetch_add(1, Ordering::Relaxed));
    let bound_controller = use_hook(|| Rc::new(std::cell::RefCell::new(None::<SelectController>)));
    if *bound_controller.borrow() != controller {
        if let Some(old) = bound_controller.borrow_mut().take() {
            old.bindings.borrow_mut().remove(&method_token);
        }
        *bound_controller.borrow_mut() = controller.clone();
    }
    let drop_controller = bound_controller.clone();
    let drop_alive = alive.clone();
    use_drop(move || {
        drop_alive.set(false);
        if let Some(controller) = drop_controller.borrow_mut().take() {
            controller.bindings.borrow_mut().remove(&method_token);
        }
    });
    let mut trigger_ref = use_signal(|| None::<Rc<MountedData>>);
    let mut additions = use_signal(Vec::<SelectOption>::new);
    let mut overflow_open = use_signal(|| false);
    let mut overflow_ref = use_signal(|| None::<Rc<MountedData>>);
    let mut hidden_values = use_signal(Vec::<String>::new);
    let mut hidden_refs = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let all_options: Vec<_> = options
        .iter()
        .cloned()
        .chain(
            additions
                .read()
                .iter()
                .filter(|addition| !options.iter().any(|option| option.value == addition.value))
                .cloned(),
        )
        .collect();
    let selected = if multiple {
        values
    } else if value.is_empty() {
        vec![]
    } else {
        vec![value]
    };
    if let Some(controller) = controller {
        controller.bindings.borrow_mut().insert(
            method_token,
            SelectMethodBinding {
                id: id.clone(),
                required,
                has_value: !selected.is_empty(),
                touched: touched.clone(),
                alive,
                input: method_input.clone(),
            },
        );
    }
    let state = select_field_state(state, required, invalid_attempted(), !selected.is_empty());
    let invalid_text = invalid_text.or_else(|| {
        (invalid_attempted() && required).then(|| "Please select an option".to_string())
    });
    let visible: Vec<_> = all_options
        .iter()
        .filter(|o| select_matches(&o.label, &query.read()))
        .cloned()
        .collect();
    let can_add = select_can_add(&all_options, &query.read(), editable);
    let enabled: Vec<_> = visible
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.disabled)
        .map(|(index, _)| index)
        .chain(can_add.then_some(visible.len()))
        .collect();
    let active_index = if enabled.contains(&active()) {
        active()
    } else {
        usize::MAX
    };
    let mut was_open = use_signal(|| false);
    let mut restore_focus = use_signal(|| true);
    let focus_id = id.clone();
    use_effect(move || {
        let is_open = open();
        if *was_open.peek() && !is_open && *restore_focus.peek() {
            let id = focus_id.clone();
            spawn(async move {
                // Closing and its DOM update are asynchronous. A subsequent
                // click/Tab may already have focused another control; inspect
                // and restore atomically so this older close cannot steal it.
                let task = crate::dom_query::eval(
                    r#"
                    const id = await dioxus.recv();
                    const input = document.getElementById(id), active = document.activeElement;
                    const root = input?.closest('.ix-select');
                    if (input?.isConnected && (root?.contains(active) || active === document.body || active === document.documentElement)) {
                        input.focus({preventScroll:true});
                    }
                    return null;
                "#,
                );
                let _ = task.send(id);
                let _ = task.await;
            });
        }
        if !*was_open.peek() && is_open {
            restore_focus.set(true);
            let id = focus_id.clone();
            let select_text = !*query_dirty.peek();
            spawn(async move {
                let task = crate::dom_query::eval(
                    r#"
                    const [id, selectText] = await dioxus.recv();
                    const input = document.getElementById(id);
                    input?.focus({preventScroll:true});
                    if (selectText) input?.select();
                    return null;
                "#,
                );
                let _ = task.send((id, select_text));
                let _ = task.await;
            });
        }
        was_open.set(is_open);
    });
    let scroll_list_id = list_id.clone();
    use_effect(move || {
        let is_open = open();
        let index = active();
        let _query = query();
        if is_open {
            let option_id = format!("{scroll_list_id}-{index}");
            spawn(async move {
                let task = crate::dom_query::eval(
                    r#"
                    const id = await dioxus.recv();
                    document.getElementById(id)?.scrollIntoView({block:'nearest',inline:'nearest'});
                    return null;
                "#,
                );
                let _ = task.send(option_id);
                let _ = task.await;
            });
        }
    });
    let keys_visible = visible.clone();
    let no_matches = visible.is_empty();
    let keys_selected = selected.clone();
    let key_id = list_id.clone();
    let selected_labels: Vec<_> = selected
        .iter()
        .map(|v| {
            all_options
                .iter()
                .find(|o| &o.value == v)
                .map(|o| o.label.clone())
                .unwrap_or(v.clone())
        })
        .collect();
    let selected_chips: Vec<_> = selected
        .iter()
        .cloned()
        .zip(selected_labels.iter().cloned())
        .collect();
    let collapsed_all = multiple
        && collapse_multiple_selection
        && !all_options.is_empty()
        && all_options
            .iter()
            .all(|option| selected.contains(&option.value));
    let hidden_chips: Vec<_> = selected_chips
        .iter()
        .filter(|(value, _)| hidden_values.read().contains(value))
        .cloned()
        .collect();
    let measurement_chips = selected_chips.clone();
    let measurement_count = selected.len();
    let root_id = format!("{id}-select-root");
    let mount_id = root_id.clone();
    let blur_root_id = root_id.clone();
    let validation_id = id.clone();
    let overflow_id = format!("{id}-overflow");
    use_effect(move || {
        if hidden_values.read().is_empty() || !multiple || collapsed_all {
            overflow_open.set(false);
        }
    });
    let clear_selected = selected.clone();
    let add_selected = selected.clone();
    let query_text = query();
    let input_text = if query_dirty() {
        query_text.clone()
    } else if multiple {
        String::new()
    } else {
        selected_labels.first().cloned().unwrap_or_default()
    };
    let active_descendant = if open() && enabled.contains(&active_index) {
        Some(format!("{key_id}-{active_index}"))
    } else {
        None
    };
    let add_index = visible.len();
    let input_placeholder = if disabled || readonly || (multiple && !selected.is_empty()) {
        String::new()
    } else if editable {
        editable_placeholder
    } else {
        placeholder
    };
    let _ = filterable;
    let read_only = readonly || disabled;
    let feedback = select_feedback_text(
        state,
        helper_text.clone(),
        invalid_text.clone(),
        valid_text.clone(),
        info_text.clone(),
        warning_text.clone(),
    );
    let tooltip_available = show_text_as_tooltip && feedback.is_some();
    rsx! {
        div { id: root_id, class: "ix-field ix-select {state.class()}",
            "data-feedback-tooltip": tooltip_available.to_string(),
            onfocusout: move |_| {
                let id = blur_root_id.clone();
                spawn(async move {
                    let task = crate::dom_query::eval(r#"
                        const id = await dioxus.recv();
                        await new Promise(requestAnimationFrame);
                        const root = document.getElementById(id);
                        return !!root?.contains(document.activeElement);
                    "#);
                    if task.send(id).is_ok() && matches!(task.join::<bool>().await, Ok(false)) {
                        restore_focus.set(false); open.set(false); overflow_open.set(false);
                        query.set(String::new()); query_dirty.set(false); active.set(usize::MAX);
                    }
                });
            },
            onmounted: move |_| {
                let id = mount_id.clone();
                spawn(async move {
                    let mut task = dioxus::document::eval(SELECT_MEASURE);
                    if task.send(id).is_err() { return; }
                    while let Ok(measured) = task.recv::<ChipMeasurements>().await {
                        let count = fitting_chips(&measured.widths, &measured.badges, measured.available, measured.gap);
                        let hidden = measured.values.into_iter().skip(count).collect::<Vec<_>>();
                        if *hidden_values.peek() != hidden { hidden_values.set(hidden); }
                    }
                });
            },
            onkeydown: move |event| {
                if read_only { return; }
                match event.key() {
                    Key::Escape => { event.prevent_default(); open.set(false); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); }
                    Key::Tab => { restore_focus.set(false); open.set(false); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); }
                    Key::ArrowDown | Key::ArrowUp => {
                        event.prevent_default();
                        overflow_open.set(false); open.set(true);
                        if let Some(next) = select_next_active(&enabled, active_index, event.key() == Key::ArrowUp) { active.set(next); }
                    }
                    Key::Home if open() => { event.prevent_default(); if let Some(index) = enabled.first() { active.set(*index); } }
                    Key::End if open() => { event.prevent_default(); if let Some(index) = enabled.last() { active.set(*index); } }
                    Key::Enter => {
                        event.prevent_default();
                        if !open() { active.set(usize::MAX); open.set(true); }
                        else if let Some(option) = keys_visible.get(active_index).filter(|o| !o.disabled) {
                            publish_selection(update_selection(keys_selected.clone(), option.value.clone(), multiple), onvaluechange, onvalueschange);
                            query.set(String::new()); query_dirty.set(false); active.set(usize::MAX);
                            if !multiple { open.set(false); }
                        } else if can_add && active_index == keys_visible.len() {
                            let text = query.read().trim().to_string();
                            if !select_add_allowed(text.clone(), onbeforeadd) { return; }
                            additions.write().push(SelectOption::new(text.clone(), text.clone()));
                            onadd.call(text.clone());
                            publish_selection(update_selection(keys_selected.clone(), text, multiple), onvaluechange, onvalueschange);
                            query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); if !multiple { open.set(false); }
                        }
                    }
                    _ => {}
                }
            },
            if let Some(text) = label { FieldLabel { text, html_for: id.clone(), required, invalid: state == FieldState::Invalid } }
            div { id: control_id.clone(), class: "ix-select-control", "data-readonly": readonly.to_string(), "data-open": open().to_string(),
                if multiple && !selected.is_empty() {
                    div { class: "ix-select-values ix-select-chips",
                        if collapsed_all {
                            span { class: "ix-select-chip", span { class: "ix-select-chip-label", "{all_selected_text} ({selected.len()})" },
                                if !read_only { button { r#type: "button", class: "ix-select-chip-remove", "aria-label": select_remove_label(&remove_selected_item_text, &all_selected_text),
                                    onkeydown: move |event| event.stop_propagation(),
                                    onclick: move |event| { event.stop_propagation(); publish_selection(vec![], onvaluechange, onvalueschange); },
                                    crate::Icon { name: "close".to_string(), size: 12 }
                                } }
                            }
                        }
                        for (value, text) in selected_chips.into_iter().filter(|(value, _)| !collapsed_all && !hidden_values.read().contains(value)) {
                            {
                                let next = selected.iter().filter(|item| **item != value).cloned().collect::<Vec<_>>();
                                rsx! { span { key: "{value}", class: "ix-select-chip", span { class: "ix-select-chip-label", title: text.clone(), "{text}" },
                                    if !read_only { button { r#type: "button", class: "ix-select-chip-remove", "aria-label": select_remove_label(&remove_selected_item_text, &text),
                                        onkeydown: move |event| if matches!(event.key(), Key::Enter | Key::Character(_)) { event.stop_propagation(); },
                                        onclick: move |event| { event.stop_propagation(); publish_selection(next.clone(), onvaluechange, onvalueschange); let mounted = trigger_ref.peek().clone(); spawn(async move { if let Some(mounted) = mounted { let _ = mounted.set_focus(true).await; } }); },
                                        crate::Icon { name: "close".to_string(), size: 12 }
                                    } }
                                } }
                            }
                        }
                        if !collapsed_all && !hidden_chips.is_empty() {
                            button { r#type: "button", class: "ix-select-chip ix-select-overflow", disabled: read_only,
                                "aria-label": select_more_label(&more_items_text, hidden_chips.len()), "aria-haspopup": "listbox", "aria-expanded": overflow_open().to_string(), "aria-controls": overflow_id.clone(),
                                onmounted: move |event| overflow_ref.set(Some(event.data())),
                                onclick: move |event| { event.stop_propagation(); open.set(false); overflow_open.set(!overflow_open()); },
                                onkeydown: move |event| { event.stop_propagation(); if matches!(event.key(), Key::Enter | Key::ArrowDown) || event.key() == Key::Character(" ".into()) { event.prevent_default(); open.set(false); overflow_open.set(true); } },
                                "+{hidden_chips.len()}"
                            }
                        }
                    }
                }
                input { id: id.clone(), r#type: "text", class: if multiple && !selected.is_empty() { "ix-select-trigger ix-select-trigger-multiple" } else { "ix-select-trigger" }, disabled, readonly,
                    value: input_text, placeholder: input_placeholder, autocomplete: "off",
                    role: "combobox", "aria-haspopup": "listbox", "aria-expanded": open().to_string(),
                    "aria-autocomplete": "list",
                    "aria-controls": list_id.clone(), "aria-readonly": readonly.to_string(),
                    "aria-required": required.to_string(), "aria-invalid": (state == FieldState::Invalid).to_string(),
                    "aria-describedby": help_id.clone(),
                    "aria-activedescendant": active_descendant,
                    onmounted: move |event| { let node = event.data(); *method_input.borrow_mut() = Some(node.clone()); trigger_ref.set(Some(node)); },
                    onclick: move |_| if !read_only { overflow_open.set(false); if !open() { active.set(usize::MAX); open.set(true); } },
                    oninput: move |event| if !read_only { let text = event.value(); query.set(text.clone()); query_dirty.set(true); active.set(usize::MAX); overflow_open.set(false); open.set(true); oninputchange.call(text); },
                    onblur: move |_| {
                        touched.set(true);
                        onixblur.call(());
                        if required && !read_only {
                            let id = validation_id.clone();
                            spawn(async move {
                                // `noValidate` suppresses iX's internal touched validation too.
                                // Read the form flag here; required/value/feedback stay Rust-owned.
                                let task = crate::dom_query::eval(r#"
                                    const id = await dioxus.recv();
                                    const input = document.getElementById(id);
                                    return !!input && !input.closest('form')?.noValidate;
                                "#);
                                if task.send(id).is_ok() && matches!(task.join::<bool>().await, Ok(true)) {
                                    invalid_attempted.set(true);
                                }
                            });
                        }
                    },
                }
                if allow_clear && (!clear_selected.is_empty() || !query_text.is_empty()) && !read_only {
                    button { r#type: "button", class: "ix-select-clear", "aria-label": aria_label_clear,
                        onmousedown: move |event| event.prevent_default(),
                        onkeydown: move |event| if matches!(event.key(), Key::Enter | Key::Character(_)) { event.stop_propagation(); },
                        onclick: move |event| { event.stop_propagation(); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); open.set(false); publish_selection(vec![], onvaluechange, onvalueschange); let node = trigger_ref.peek().clone(); spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } }); },
                        crate::Icon { name: "close".to_string(), size: 16 }
                    }
                }
                if !read_only {
                    button { r#type: "button", class: "ix-select-chevron", tabindex: "-1", "aria-hidden": "true",
                        "aria-label": if open() { "Close select dropdown" } else { "Open select dropdown" },
                        onmousedown: move |event| event.prevent_default(),
                        onclick: move |event| { event.stop_propagation(); overflow_open.set(false); if open() { query.set(String::new()); query_dirty.set(false); } active.set(usize::MAX); open.set(!open()); },
                        crate::Icon { name: (if open() { "chevron-up" } else { "chevron-down" }).to_string(), size: 16 }
                    }
                }
            }
            if multiple && !selected.is_empty() {
                div { class: "ix-select-measure", "aria-hidden": "true", inert: true,
                    for (value, text) in measurement_chips {
                        span { class: "ix-select-chip", "data-measure-value": value, span { class: "ix-select-chip-label", "{text}" },
                            if !read_only { span { class: "ix-select-chip-remove", crate::Icon { name: "close".to_string(), size: 12 } } }
                        }
                    }
                    for count in 1..=measurement_count {
                        button { r#type: "button", class: "ix-select-chip ix-select-overflow", "data-measure-badge": count.to_string(), tabindex: "-1", "+{count}" }
                    }
                }
            }
            if overflow_open() && !hidden_chips.is_empty() && !read_only {
                button { r#type: "button", class: "ix-select-backdrop", tabindex: "-1", "aria-label": "Close hidden selected items", onclick: move |_| overflow_open.set(false) }
                div { id: overflow_id, class: "ix-select-popup ix-select-overflow-popup", role: "listbox", "aria-label": "Hidden selected items",
                    for (index, (value, text)) in hidden_chips.iter().cloned().enumerate() {
                        {
                            let remaining = selected.iter().filter(|item| **item != value).cloned().collect::<Vec<_>>();
                            let count = hidden_chips.len();
                            rsx! { span { key: "{value}", class: "ix-select-chip", role: "option", "aria-selected": "true", "{text}",
                                button { r#type: "button", class: "ix-select-chip-remove", "aria-label": select_remove_label(&remove_selected_item_text, &text),
                                    onmounted: move |event| { let node = event.data(); { let mut refs = hidden_refs.write(); if refs.len() <= index { refs.resize(index + 1, None); } refs[index] = Some(node.clone()); } if index == 0 { spawn(async move { let _ = node.set_focus(true).await; }); } },
                                    onclick: move |event| { event.stop_propagation(); overflow_open.set(false); publish_selection(remaining.clone(), onvaluechange, onvalueschange); let node = trigger_ref.peek().clone(); spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } }); },
                                    onkeydown: move |event| {
                                        event.stop_propagation();
                                        let next = match event.key() {
                                            Key::ArrowDown => Some((index + 1) % count),
                                            Key::ArrowUp => Some((index + count - 1) % count),
                                            Key::Tab => Some(if event.modifiers().shift() { (index + count - 1) % count } else { (index + 1) % count }),
                                            Key::Escape => { event.prevent_default(); overflow_open.set(false); let node = overflow_ref.peek().clone(); spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } }); None },
                                            _ => None,
                                        };
                                        if let Some(next) = next { event.prevent_default(); let node = hidden_refs.peek().get(next).cloned().flatten(); spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } }); }
                                    },
                                    crate::Icon { name: "close".to_string(), size: 12 }
                                }
                            } }
                        }
                    }
                }
            }
            select { class: "ix-select-native", name, multiple, required: required && !readonly, disabled, tabindex: "-1", "aria-hidden": "true",
                oninvalid: move |event| { event.prevent_default(); if !read_only { invalid_attempted.set(true); open.set(true); } },
                if !multiple { option { value: "", selected: selected.is_empty(), "" } }
                for option in &all_options {
                    option { value: option.value.clone(), disabled: option.disabled, selected: selected.contains(&option.value), "{option.label}" }
                }
            }
            if open() && !read_only {
                SelectOptionsSurface { anchor_id: control_id.clone(), input_id: id.clone(), enable_top_layer,
                    dropdown_width, dropdown_max_width, onnativeerror,
                    onclose: move |_| { restore_focus.set(false); open.set(false); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); },
                    if !hide_list_header && !no_matches { div { class: "ix-select-list-header", "aria-hidden": "true", title: list_header.clone(), "{list_header}" } }
                    div { id: list_id.clone(), role: "listbox", "aria-multiselectable": multiple.to_string(), class: "ix-select-list",
                        for (index, option) in visible.iter().cloned().enumerate() {
                            {
                                let next_values = update_selection(selected.clone(), option.value.clone(), multiple);
                                let is_selected = selected.contains(&option.value);
                                rsx! {
                                    button { key: "{option.value}", id: format!("{list_id}-{index}"),
                                        class: if index == active_index { "ix-select-option ix-select-active" } else { "ix-select-option" },
                                        r#type: "button", role: "option", tabindex: "-1", disabled: option.disabled,
                                        "aria-selected": is_selected.to_string(),
                                        onmousedown: move |event| event.prevent_default(),
                                        onmouseenter: move |_| if !option.disabled { active.set(index); },
                                        onclick: move |event| { event.stop_propagation(); publish_selection(next_values.clone(), onvaluechange, onvalueschange); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); if !multiple { open.set(false); } },
                                        if multiple { span { class: if is_selected { "ix-select-choice checked" } else { "ix-select-choice" }, "aria-hidden": "true",
                                            if is_selected { crate::Icon { name: "check".to_string(), size: 16 } }
                                        } }
                                        "{option.label}"
                                        if is_selected && !multiple { crate::Icon { name: "check".to_string(), size: 16 } }
                                    }
                                }
                            }
                        }
                        if no_matches && !editable { div { class: "ix-select-empty", "{no_matches_text}" } }
                        if can_add {
                            button { id: format!("{list_id}-{}", visible.len()), r#type: "button", role: "option", tabindex: "-1", "aria-selected": "false",
                                "aria-label": format!("{aria_label_add}: {}", query_text.trim()),
                                class: if active_index == visible.len() { "ix-select-option ix-select-add ix-select-active" } else { "ix-select-option ix-select-add" },
                                onmousedown: move |event| event.prevent_default(),
                                onmouseenter: move |_| active.set(add_index),
                                onclick: move |_| {
                                let text = query.read().trim().to_string();
                                if !select_add_allowed(text.clone(), onbeforeadd) { return; }
                                additions.write().push(SelectOption::new(text.clone(), text.clone())); onadd.call(text.clone());
                                publish_selection(update_selection(add_selected.clone(), text, multiple), onvaluechange, onvalueschange);
                                query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); if !multiple { open.set(false); }
                            }, crate::Icon { name: "plus".to_string(), size: 16 } "{query_text.trim()}" }
                        }
                    }
                }
            }
            if tooltip_available {
                crate::TooltipFor { id: help_id.clone(), anchor_id: control_id.clone(),
                    placement: crate::TooltipPlacement::Bottom, show_delay: 500,
                    class: "ix-select-feedback-tooltip {state.class()}",
                    onnativeerror: move |message: String| onnativeerror.call(crate::NativeSurfaceError::Rejected(message)),
                    "{feedback.as_deref().unwrap_or_default()}"
                }
            } else {
                HelperText { id: help_id.clone(), helper_text, invalid_text, valid_text, info_text, warning_text, state }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ExpandingSearchState {
    expanded: bool,
    changed: bool,
}

impl ExpandingSearchState {
    fn expand(&mut self) {
        self.expanded = true;
    }
    fn input(&mut self, value: &str) {
        self.changed = !self.changed || !value.is_empty();
    }
    fn blur(&mut self) {
        if !self.changed {
            self.expanded = false;
        }
    }
    fn clear(&mut self) {
        self.changed = false;
    }
}

/// Search that starts collapsed and expands when its icon is activated.
///
/// Input history controls the clear button and whether blur collapses the field;
/// an initial or externally supplied value does not expand it. Expansion focuses
/// the retained native input after 300ms. Values remain consumer-controlled.
/// `disabled` and the Enter `onsearch` callback are Rust extensions to source iX.
#[component]
pub fn ExpandingSearch(
    #[props(default)] value: String,
    #[props(default = "Enter text here".to_string())] placeholder: String,
    #[props(default)] full_width: bool,
    #[props(default)] disabled: bool,
    #[props(default)] icon: Option<String>,
    #[props(default = crate::ButtonVariant::Tertiary)] variant: crate::ButtonVariant,
    #[props(default)] class: String,
    #[props(default)] aria_label_search_icon_button: Option<String>,
    #[props(default = "Clear search".to_string())] aria_label_clear_icon_button: String,
    #[props(default = "Search input".to_string())] aria_label_search_input: String,
    #[props(default)] onvaluechange: EventHandler<String>,
    #[props(default)] onsearch: EventHandler<String>,
) -> Element {
    let mut state = use_signal(ExpandingSearchState::default);
    let mut input_ref = use_signal(|| None::<Rc<MountedData>>);
    let alive = use_hook(|| Rc::new(std::cell::Cell::new(true)));
    let cleanup = alive.clone();
    use_drop(move || cleanup.set(false));
    let focus_input = use_callback(move |delay_ms: u64| {
        let alive = alive.clone();
        spawn(async move {
            if delay_ms > 0 {
                futures_timer::Delay::new(std::time::Duration::from_millis(delay_ms)).await;
            }
            if !alive.get() {
                return;
            }
            let node = input_ref.peek().clone();
            if let Some(node) = node {
                let _ = node.set_focus(true).await;
            }
        });
    });
    let expanded = state().expanded;
    let changed = state().changed;
    let search_label = aria_label_search_icon_button.unwrap_or_else(|| {
        if expanded {
            "Close search"
        } else {
            "Open search"
        }
        .to_string()
    });
    let search_value = value.clone();
    rsx! {
        div { class: "ix-expanding-search {class}",
            class: if expanded { "expanded" },
            class: if full_width { "ix-search-full fullWidth" },
            "data-expanded": expanded.to_string(), "data-changed": changed.to_string(),
            "data-disabled": disabled.to_string(),
            span { class: "ix-search-trigger-host", class: if expanded { "ix-search-trigger-expanded" },
                "data-expanded": expanded.to_string(),
                crate::IconButton { class: "ix-search-button ix-search-trigger", disabled,
                    icon: icon.unwrap_or_else(|| "search".to_string()),
                    size: if expanded { crate::ButtonIconSize::Sixteen } else { crate::ButtonIconSize::TwentyFour },
                    variant: if expanded { crate::ButtonVariant::Tertiary } else { variant },
                    icon_color: expanded.then(|| "--si-sys-color-text-primary".to_string()),
                    label: search_label, tabindex: if expanded { -1 } else { 0 }, aria_expanded: expanded,
                    onclick: move |_| { state.write().expand(); focus_input.call(300); },
                }
            }
            div { class: "ix-search-input-container", "data-expanded": expanded.to_string(),
                input { class: "ix-search-input", class: if expanded { "ix-search-input-expanded" },
                    r#type: "text", value, placeholder, disabled,
                    tabindex: if expanded { 0 } else { -1 }, "aria-label": aria_label_search_input,
                    onmounted: move |event| input_ref.set(Some(event.data())),
                    oninput: move |event| { if !disabled { let value = event.value(); state.write().input(&value); onvaluechange.call(value); } },
                    onblur: move |_| state.write().blur(),
                    onkeydown: move |event| {
                        if !disabled && event.key() == Key::Enter && !event.is_composing() {
                            event.prevent_default(); onsearch.call(search_value.clone());
                        }
                    },
                }
                if changed {
                    span { class: "ix-search-clear-host",
                        crate::IconButton { class: "ix-search-button ix-search-clear", disabled,
                            variant: crate::ButtonVariant::SubtleTertiary, size: crate::ButtonIconSize::Sixteen,
                            label: aria_label_clear_icon_button, icon: "clear",
                            onclick: move |_| { state.write().clear(); onvaluechange.call(String::new()); focus_input.call(0); },
                        }
                    }
                }
            }
        }
    }
}

/// Upload status is controlled by the application performing the upload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UploadState {
    /// Ready to select a file.
    #[default]
    SelectFile,
    /// Application upload in progress.
    Loading,
    /// Application upload completed successfully.
    Success,
    /// Application upload failed.
    Failed,
}

/// A selected file and its folder-relative location, without reading its contents.
#[derive(Clone, Debug, PartialEq)]
pub struct UploadEntry {
    pub file: FileData,
    pub relative_path: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UploadError {
    MultipleFilesNotAllowed,
    DirectoryNotAllowed,
    ReadDirectory(String),
}
impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MultipleFilesNotAllowed => f.write_str("Choose one file at a time."),
            Self::DirectoryNotAllowed => f.write_str("Choose a file instead of a folder."),
            Self::ReadDirectory(message) => {
                write!(f, "Could not read the selected folder: {message}")
            }
        }
    }
}
impl std::error::Error for UploadError {}

fn upload_count_allowed(count: usize, multiple: bool, directory: bool) -> Result<(), UploadError> {
    if count > 1 && !multiple && !directory {
        Err(UploadError::MultipleFilesNotAllowed)
    } else {
        Ok(())
    }
}

fn upload_entries(
    files: Vec<FileData>,
    multiple: bool,
    directory: bool,
) -> Result<Vec<UploadEntry>, UploadError> {
    upload_count_allowed(files.len(), multiple, directory)?;
    let mut entries = Vec::new();
    for file in files {
        let path = file.path();
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        if path.is_dir() {
            if !directory {
                return Err(UploadError::DirectoryNotAllowed);
            }
            let prefix = path
                .file_name()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| "folder".into());
            collect_upload_directory(&path, &prefix, &mut entries)?;
            continue;
        }
        let relative_path = if path.is_absolute() {
            file.name().into()
        } else {
            path
        };
        entries.push(UploadEntry {
            file,
            relative_path,
        });
    }
    Ok(entries)
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn collect_upload_directory(
    path: &std::path::Path,
    relative: &std::path::Path,
    files: &mut Vec<UploadEntry>,
) -> Result<(), UploadError> {
    let failure = |error: std::io::Error| UploadError::ReadDirectory(error.to_string());
    let mut children = std::fs::read_dir(path)
        .map_err(failure)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(failure)?;
    children.sort_by_key(|entry| entry.file_name());
    for entry in children {
        let kind = entry.file_type().map_err(failure)?;
        // Browser folder selection does not grant traversal of external link targets.
        if kind.is_symlink() {
            continue;
        }
        let relative_path = relative.join(entry.file_name());
        if kind.is_dir() {
            collect_upload_directory(&entry.path(), &relative_path, files)?;
        } else if kind.is_file() {
            let metadata = entry.metadata().map_err(failure)?;
            let file = FileData::new(dioxus::html::SerializedFileData {
                path: entry.path(),
                size: metadata.len(),
                last_modified: metadata
                    .modified()
                    .ok()
                    .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|value| value.as_millis() as u64)
                    .unwrap_or(0),
                content_type: None,
                contents: None,
            });
            files.push(UploadEntry {
                file,
                relative_path,
            });
        }
    }
    Ok(())
}

async fn prepare_upload(
    files: Vec<FileData>,
    multiple: bool,
    directory: bool,
) -> Result<Vec<UploadEntry>, UploadError> {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    {
        let (sender, receiver) = futures_channel::oneshot::channel();
        std::thread::spawn(move || {
            let _ = sender.send(upload_entries(files, multiple, directory));
        });
        receiver
            .await
            .map_err(|error| UploadError::ReadDirectory(error.to_string()))?
    }
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    {
        upload_entries(files, multiple, directory)
    }
}

/// File/folder picker and drop target. `FileData` keeps content reads portable.
/// Folder inputs use the host's directory-picker support; `onentrieschanged`
/// retains browser relative paths and expands native desktop folder selections.
/// No network upload occurs. The application owns transfer and upload state.
/// This component selects files; transfer and progress are managed by the caller.
#[component]
pub fn Upload(
    #[props(default)] name: Option<String>,
    #[props(default)] accept: Option<String>,
    #[props(default)] multiple: bool,
    #[props(default)] directory_upload: bool,
    #[props(default)] multiline: bool,
    #[props(default)] disabled: bool,
    #[props(default)] state: UploadState,
    select_file_text: Option<String>,
    button_text: Option<String>,
    loading_text: Option<String>,
    aria_label: Option<String>,
    #[props(default = "File upload currently not possible.".to_string())] disabled_text: String,
    #[props(default = "Upload successful".to_string())] success_text: String,
    #[props(default = "Upload failed. Please try again.".to_string())] failed_text: String,
    #[props(default)] onfileschanged: EventHandler<Vec<FileData>>,
    #[props(default)] onentrieschanged: EventHandler<Vec<UploadEntry>>,
    #[props(default)] onerror: EventHandler<UploadError>,
) -> Element {
    let busy = state == UploadState::Loading;
    let mut processing = use_signal(|| false);
    let mut failure = use_signal(|| None::<UploadError>);
    let mut drag_depth = use_signal(|| 0_usize);
    let generation = use_hook(|| Rc::new(std::cell::Cell::new(0_u64)));
    let current_generation = generation.clone();
    let blocked = disabled || busy;
    use_effect(use_reactive!(|blocked| {
        if blocked {
            current_generation.set(current_generation.get().wrapping_add(1));
            processing.set(false);
            drag_depth.set(0);
        }
    }));
    let choose = use_callback(move |files: Vec<FileData>| {
        if blocked {
            return;
        }
        failure.set(None);
        drag_depth.set(0);
        processing.set(true);
        let sequence = generation.get().wrapping_add(1);
        generation.set(sequence);
        let generation = generation.clone();
        spawn(async move {
            let result = prepare_upload(files, multiple, directory_upload).await;
            if generation.get() != sequence {
                return;
            }
            processing.set(false);
            match result {
                Ok(entries) => {
                    onfileschanged.call(entries.iter().map(|entry| entry.file.clone()).collect());
                    onentrieschanged.call(entries);
                }
                Err(error) => {
                    failure.set(Some(error.clone()));
                    onerror.call(error);
                }
            }
        });
    });
    let reset_id = field_id(None);
    let change_id = reset_id.clone();
    let button_text = button_text.unwrap_or_else(|| {
        if directory_upload {
            "Upload folder…"
        } else {
            "Upload file…"
        }
        .into()
    });
    let aria_label = aria_label.unwrap_or_else(|| {
        if directory_upload {
            "Upload folder"
        } else {
            "Upload files"
        }
        .into()
    });
    let text = if disabled {
        disabled_text
    } else {
        match state {
            UploadState::SelectFile => select_file_text.unwrap_or_else(|| {
                if directory_upload {
                    "+ Drag folder here or…"
                } else {
                    "+ Drag files here or…"
                }
                .into()
            }),
            UploadState::Loading => loading_text.unwrap_or_else(|| {
                if directory_upload {
                    "Checking folder…"
                } else {
                    "Checking files…"
                }
                .into()
            }),
            UploadState::Success => success_text,
            UploadState::Failed => failed_text,
        }
    };
    let class = match state {
        UploadState::Failed => "ix-upload ix-upload-failed",
        UploadState::Success => "ix-upload ix-upload-success",
        _ => "ix-upload",
    };
    rsx! {
        label { class, "aria-busy": (busy || processing()).to_string(), "aria-disabled": blocked.to_string(), "data-disabled": disabled.to_string(), "data-multiline": multiline.to_string(), "data-file-over": (drag_depth() > 0).to_string(),
            ondragenter: move |event: DragEvent| { if !blocked { event.prevent_default(); drag_depth += 1; } },
            ondragleave: move |_| drag_depth.set(drag_depth().saturating_sub(1)),
            ondragover: move |event: DragEvent| { event.prevent_default(); if !blocked { event.data_transfer().set_drop_effect("copy"); } else { event.data_transfer().set_drop_effect("none"); } },
            ondrop: move |event: DragEvent| { event.prevent_default(); event.stop_propagation(); drag_depth.set(0); if !blocked { choose.call(event.data_transfer().files()); } },
            input { id: reset_id, r#type: "file", name, accept, multiple: multiple || directory_upload, disabled: blocked, "aria-label": aria_label,
                "webkitdirectory": directory_upload.then_some("true"), "directory": directory_upload.then_some("true"),
                onchange: move |event| {
                    if blocked { return; }
                    choose.call(event.files());
                    // Reset only the native input, after Dioxus has retained the FileData
                    // handles, so choosing the same filename again still emits change.
                    let id = change_id.clone(); spawn(async move {
                        let task = crate::dom_query::eval("const id=await dioxus.recv(); const node=document.getElementById(id); if(node instanceof HTMLInputElement)node.value=''; return null;");
                        let _ = task.send(id); let _ = task.await;
                    });
                },
            }
            span { class: "ix-upload-state", role: "status",
                if state == UploadState::Loading { crate::Spinner { size: crate::SpinnerSize::Sm, variant: crate::SpinnerVariant::Primary } }
                else if matches!(state, UploadState::Failed | UploadState::Success) { crate::Icon { name: (if state == UploadState::Failed { "error" } else { "check" }).to_string(), size: 24 } }
                span { "{text}" }
            }
            if !disabled && !busy { span { class: "ix-upload-button", "{button_text}" } }
            if let Some(error) = failure() { span { class: "ix-upload-error", role: "alert", "{error}" } }
        }
    }
}

#[cfg(test)]
#[path = "checkbox_token_tests.rs"]
mod checkbox_token_tests;

#[cfg(test)]
#[path = "radio_token_tests.rs"]
mod radio_token_tests;

#[cfg(test)]
#[path = "slider_token_tests.rs"]
mod slider_token_tests;

#[cfg(test)]
#[path = "expanding_search_tests.rs"]
mod expanding_search_tests;

#[cfg(test)]
#[path = "number_input_token_tests.rs"]
mod number_input_token_tests;

#[cfg(test)]
#[path = "input_token_tests.rs"]
mod input_token_tests;

#[cfg(test)]
#[path = "group_token_tests.rs"]
mod group_token_tests;

#[cfg(test)]
#[path = "toggle_token_tests.rs"]
mod toggle_token_tests;

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-upload-state{display:flex;align-items:center;gap:12px;flex:1;min-width:0}.ix-theme .ix-upload-state>span{overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.ix-theme .ix-upload[data-multiline=true] .ix-upload-state>span{white-space:normal;overflow-wrap:anywhere}.ix-theme .ix-upload-button{flex:none;white-space:nowrap}.ix-theme .ix-upload[data-file-over=true]{border-style:solid;border-color:var(--ix-color-primary)}.ix-theme .ix-upload[aria-busy=true]{border-style:solid}.ix-theme .ix-upload:has(.ix-upload-error){flex-wrap:wrap}.ix-theme .ix-upload-error{flex-basis:100%;color:var(--ix-color-danger-text);overflow-wrap:anywhere}
.ix-theme .ix-field{display:flex;flex-direction:column;gap:.375rem;min-width:0;width:100%;font:inherit}
.ix-theme .ix-field-label{display:block;color:var(--ix-color-text);font-size:.875rem;line-height:1.5rem;margin:0;padding:0;font-weight:400}
.ix-theme .ix-field-bottom{display:flex;justify-content:space-between;gap:.5rem;align-items:flex-start}.ix-theme .ix-field-bottom>.ix-helper{flex:1;min-width:0}.ix-theme .ix-field-counter{flex:none;margin-inline-start:auto;font-size:.75rem;line-height:1.125rem;color:var(--ix-color-text-secondary);font-variant-numeric:tabular-nums}.ix-theme .ix-password-toggle{display:flex;align-items:center;justify-content:center;flex:none;width:2rem;min-height:2rem;margin-inline-end:.25rem;border:0;background:transparent;color:var(--ix-color-text-secondary);cursor:pointer}.ix-theme .ix-password-toggle:hover{color:var(--ix-color-primary)}.ix-theme .ix-password-toggle:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}
.ix-theme .ix-required{color:inherit}.ix-theme .ix-helper{display:flex;align-items:flex-start;gap:.25rem;font-size:.75rem;line-height:1.125rem;color:var(--ix-color-text-secondary);min-height:1.125rem;overflow-wrap:anywhere}
.ix-theme .ix-helper.ix-field-invalid{color:var(--ix-color-danger-text)}
.ix-theme .ix-helper.ix-field-valid,.ix-theme .ix-helper.ix-field-info,.ix-theme .ix-helper.ix-field-warning{color:var(--ix-color-text)}.ix-theme .ix-helper.ix-field-valid>.ix-helper-icon{color:var(--ix-color-success)}.ix-theme .ix-helper.ix-field-info>.ix-helper-icon{color:var(--ix-color-info-text)}.ix-theme .ix-helper.ix-field-warning>.ix-helper-icon{color:var(--ix-color-warning-text)}
.ix-theme .ix-input-wrap{display:flex;position:relative;align-items:center;min-width:0;width:100%;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);overflow:hidden}
.ix-theme .ix-input{box-sizing:border-box;display:block;min-width:0;width:100%;min-height:2.5rem;padding:.5rem .75rem;color:var(--ix-color-text);background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);font:inherit;font-size:.875rem;line-height:1.375rem;caret-color:var(--ix-color-primary)}
.ix-theme .ix-input-wrap>.ix-input{border:0;border-radius:0;background:transparent}.ix-theme .ix-input::placeholder{color:var(--ix-color-text-secondary);opacity:1}
.ix-theme .ix-input-wrap:hover:not(:has(:disabled)),.ix-theme .ix-input:hover:not(:disabled):not(:read-only){border-color:var(--ix-color-text-secondary)}
.ix-theme .ix-input-wrap:focus-within,.ix-theme .ix-input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-theme .ix-input-wrap>.ix-input:focus-visible{outline:0}
.ix-theme .ix-input:disabled{opacity:.45;cursor:not-allowed}.ix-theme .ix-input[readonly]{background:transparent;color:var(--ix-color-text-secondary)}
.ix-theme .ix-field-invalid .ix-input-wrap,.ix-theme .ix-field-invalid>.ix-input{border-color:var(--ix-color-danger-text)}
.ix-theme .ix-field-warning .ix-input-wrap,.ix-theme .ix-field-warning>.ix-input{border-color:var(--ix-color-warning-text)}
.ix-theme .ix-field-info .ix-input-wrap{border-color:var(--ix-color-info-text)}.ix-theme .ix-field-valid .ix-input-wrap{border-color:var(--ix-color-success)}
.ix-theme .ix-input-start,.ix-theme .ix-input-end{display:flex;align-items:center;flex-shrink:0;padding-inline:.5rem;color:var(--ix-color-text-secondary)}.ix-theme .ix-input-start+.ix-input{padding-inline-start:0}.ix-theme .ix-input:has(+.ix-input-end){padding-inline-end:0}
.ix-theme .ix-textarea{min-height:6rem}.ix-theme .ix-check-group{border:0;padding:0;margin:0;min-inline-size:0}.ix-theme .ix-check-group legend{margin-bottom:.5rem}
.ix-theme .ix-choices{display:flex;flex-direction:column;gap:.75rem}.ix-theme .ix-choices-row{flex-direction:row;flex-wrap:wrap;gap:1.5rem}
.ix-theme .ix-check{display:inline-flex;position:relative;align-items:center;gap:.5rem;min-height:1.5rem;font-size:.875rem;cursor:pointer;color:var(--ix-color-text);width:fit-content}
.ix-theme .ix-check>input{position:absolute;opacity:0;width:1.25rem;height:1.25rem;inset-inline-start:0;margin:0;cursor:inherit}
.ix-theme .ix-check-shape{display:flex;align-items:center;justify-content:center;width:1.125rem;height:1.125rem;box-sizing:border-box;flex-shrink:0;border:1px solid var(--ix-color-text-secondary);border-radius:2px;background:transparent;color:var(--ix-color-on-primary)}
.ix-theme .ix-check:hover .ix-check-shape{border-color:var(--ix-color-primary);background:var(--ix-color-surface-2)}.ix-theme .ix-check>input:checked+.ix-check-shape,.ix-theme .ix-check[data-mixed=true] .ix-check-shape{background:var(--ix-color-primary);border-color:var(--ix-color-primary)}
.ix-theme .ix-check>input:focus-visible+.ix-check-shape{outline:2px solid var(--ix-color-focus);outline-offset:3px}.ix-theme .ix-check:has(:disabled){opacity:.45;cursor:not-allowed}
.ix-theme .ix-radio .ix-check-shape{border-radius:50%}.ix-theme .ix-radio input:checked+.ix-check-shape{box-shadow:inset 0 0 0 3px var(--ix-color-surface-1)}
.ix-theme .ix-check>input[aria-invalid=true]+.ix-check-shape{border-color:var(--ix-color-danger-text)}
.ix-theme .ix-checkbox-field{display:inline-flex;flex-direction:column;gap:.25rem;position:relative;max-width:100%}.ix-theme .ix-check-native{position:absolute;opacity:0;width:1px;height:1px;pointer-events:none;inset:0;overflow:hidden}.ix-theme button.ix-check,.ix-theme button.ix-toggle{font-family:inherit;text-align:start;border:0;background:transparent;padding:0;color:var(--ix-color-text)}.ix-theme button.ix-check:disabled,.ix-theme button.ix-toggle:disabled{opacity:.45;cursor:not-allowed}.ix-theme button.ix-check:focus-visible,.ix-theme button.ix-toggle:focus-visible{outline:0}.ix-theme button.ix-check:focus-visible .ix-check-shape{outline:2px solid var(--ix-color-focus);outline-offset:3px}.ix-theme .ix-check[data-checked=true] .ix-check-shape{background:var(--ix-color-primary);border-color:var(--ix-color-primary)}.ix-theme .ix-check[aria-invalid=true] .ix-check-shape{border-color:var(--ix-color-danger-text)}
.ix-theme .ix-toggle{display:inline-flex;align-items:center;position:relative;gap:.5rem;min-height:1.5rem;font-size:.875rem;cursor:pointer;width:fit-content}
.ix-theme .ix-toggle>input{position:absolute;inset-inline-start:0;width:2.5rem;height:1.5rem;opacity:0;margin:0;cursor:inherit}.ix-theme .ix-toggle-track{position:relative;box-sizing:border-box;width:2.5rem;height:1.25rem;border:1px solid var(--ix-color-text-secondary);border-radius:1rem;background:var(--ix-color-surface-1)}
.ix-theme .ix-toggle-track:before{content:'';position:absolute;top:2px;left:2px;width:.875rem;height:.875rem;border-radius:50%;background:var(--ix-color-text-secondary);transition:transform 120ms ease-out,background 120ms ease-out}
.ix-theme .ix-toggle input:checked+.ix-toggle-track,.ix-theme .ix-toggle[data-mixed=true] .ix-toggle-track{background:var(--ix-color-primary);border-color:var(--ix-color-primary)}.ix-theme .ix-toggle input:checked+.ix-toggle-track:before{transform:translateX(1.25rem);background:var(--ix-color-on-primary)}.ix-theme .ix-toggle[data-mixed=true] .ix-toggle-track:before{transform:translateX(.625rem);background:var(--ix-color-on-primary)}
.ix-theme .ix-toggle>input:focus-visible+.ix-toggle-track{outline:2px solid var(--ix-color-focus);outline-offset:3px}.ix-theme .ix-toggle:has(:disabled){opacity:.45;cursor:not-allowed}
.ix-theme .ix-toggle[data-checked=true] .ix-toggle-track{background:var(--ix-color-primary);border-color:var(--ix-color-primary)}.ix-theme .ix-toggle[data-checked=true] .ix-toggle-track:before{transform:translateX(1.25rem);background:var(--ix-color-on-primary)}.ix-theme .ix-toggle[data-mixed=true] .ix-toggle-track:before{transform:translateX(.625rem)}.ix-theme button.ix-toggle:focus-visible .ix-toggle-track{outline:2px solid var(--ix-color-focus);outline-offset:3px}.ix-theme button.ix-toggle[aria-invalid=true] .ix-toggle-track{border-color:var(--ix-color-danger-text)}




.ix-theme .ix-range-field{display:flex;align-items:end;gap:2.5rem;position:relative;min-width:0;width:100%}.ix-theme .ix-range-field>.ix-temporal-field,.ix-theme .ix-range-field>.ix-field{flex:1;min-width:0}.ix-theme .ix-range-field:before{content:'';position:absolute;left:calc(50% - .5rem);bottom:1.25rem;width:1rem;border-top:1px solid var(--ix-color-text-secondary)}.ix-theme .ix-range-field:after{content:'';position:absolute;left:calc(50% + .125rem);bottom:1.075rem;width:.35rem;height:.35rem;border-top:1px solid var(--ix-color-text-secondary);border-right:1px solid var(--ix-color-text-secondary);transform:rotate(45deg)}.ix-theme .ix-range-no-arrow:before,.ix-theme .ix-range-no-arrow:after{display:none}.ix-theme .ix-range-no-arrow{gap:.75rem}
.ix-theme .ix-field-info{--ix-slider-accent:var(--ix-color-info);--ix-check-accent:var(--ix-color-info)}.ix-theme .ix-field-warning{--ix-slider-accent:var(--ix-color-warning);--ix-check-accent:var(--ix-color-warning)}.ix-theme .ix-field-invalid{--ix-slider-accent:var(--ix-color-danger);--ix-check-accent:var(--ix-color-danger)}.ix-theme .ix-field-valid{--ix-slider-accent:var(--ix-color-primary);--ix-check-accent:var(--ix-color-primary)}.ix-theme .ix-check:is(.ix-field-info,.ix-field-warning,.ix-field-invalid,.ix-field-valid) .ix-check-shape{border-color:var(--ix-check-accent)}.ix-theme .ix-check:is(.ix-field-info,.ix-field-warning,.ix-field-invalid,.ix-field-valid)[data-checked=true] .ix-check-shape,.ix-theme .ix-radio:is(.ix-field-info,.ix-field-warning,.ix-field-invalid,.ix-field-valid)>input:checked+.ix-check-shape{background:var(--ix-check-accent)}.ix-theme .ix-upload-state{display:flex;align-items:center;gap:.5rem;min-width:0}.ix-theme .ix-upload-state>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-upload-button{flex:none}.ix-theme .ix-field-info>.ix-input,.ix-theme .ix-field-info .ix-select-control{border-color:var(--ix-color-info-text)}.ix-theme .ix-field-valid>.ix-input,.ix-theme .ix-field-valid .ix-select-control{border-color:var(--ix-color-success)}.ix-theme .ix-field-warning .ix-select-control{border-color:var(--ix-color-warning-text)}
@media(prefers-reduced-motion:reduce){.ix-theme .ix-toggle-track:before{transition:none}}
.ix-theme .ix-select{position:relative}
.ix-theme .ix-select-feedback-tooltip.ix-field-invalid{color:var(--ix-color-danger-text)}.ix-theme .ix-select-feedback-tooltip.ix-field-valid{color:var(--ix-color-success)}.ix-theme .ix-select-feedback-tooltip.ix-field-info{color:var(--ix-color-info-text)}.ix-theme .ix-select-feedback-tooltip.ix-field-warning{color:var(--ix-color-warning-text)}
.ix-theme .ix-select-control{position:relative;display:flex;align-items:stretch;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1)}
.ix-theme .ix-select-control:focus-within{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-theme .ix-select-control:hover{border-color:var(--ix-color-text-secondary)}.ix-theme .ix-field-invalid .ix-select-control{border-color:var(--ix-color-danger-text)}
.ix-theme .ix-select-trigger{display:flex;align-items:center;justify-content:space-between;gap:.5rem;min-width:0;min-height:2.5rem;padding:.375rem .75rem;width:100%;border:0;background:transparent;color:var(--ix-color-text);font:inherit;font-size:.875rem;cursor:pointer;text-align:start}.ix-theme .ix-select-trigger:focus-visible{outline:0}.ix-theme .ix-select-trigger::placeholder{color:var(--ix-color-text-secondary)}.ix-theme .ix-select-control[data-open=true]{z-index:92}.ix-theme .ix-select-chevron{display:flex;align-items:center;justify-content:center;flex:none;width:2.5rem;border:0;background:transparent;color:var(--ix-color-text);cursor:pointer}.ix-theme .ix-select-chevron:hover{background:var(--ix-color-surface-2)}
.ix-theme .ix-select-trigger:disabled{opacity:.45;cursor:not-allowed}.ix-theme .ix-select-control[data-readonly=true] .ix-select-trigger{cursor:default}.ix-theme .ix-select-placeholder{color:var(--ix-color-text-secondary)}.ix-theme .ix-select-values{display:flex;flex-wrap:wrap;gap:.25rem;min-width:0;overflow-wrap:anywhere}
.ix-theme .ix-select-chip{display:inline-flex;align-items:center;border-radius:1rem;background:var(--ix-color-surface-2);padding:.125rem .5rem;font-size:.75rem;line-height:1.25rem;white-space:nowrap}.ix-theme .ix-select-chip-label{min-width:0;overflow:hidden;text-overflow:ellipsis}
.ix-theme .ix-select-chips{flex-wrap:nowrap;overflow:hidden}.ix-theme .ix-select-chips>.ix-select-chip{flex:0 1 auto;min-width:0;overflow:hidden}.ix-theme .ix-select-chips>.ix-select-overflow{flex:none}.ix-theme .ix-select-chip-remove{flex:none}.ix-theme .ix-select-overflow{border:0;color:var(--ix-color-text);font-family:inherit;cursor:pointer}.ix-theme .ix-select-overflow:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}.ix-theme .ix-select-overflow:disabled{cursor:default}.ix-theme .ix-select-measure{position:fixed;inset:0 auto auto 0;display:flex;width:max-content;visibility:hidden;pointer-events:none;gap:.25rem}.ix-theme .ix-select-measure>.ix-select-chip{flex:none;max-width:none}.ix-theme .ix-select-overflow-popup{display:flex;flex-direction:column;align-items:flex-start;gap:.5rem;padding:.75rem;max-height:18rem;overflow:auto}.ix-theme .ix-select-overflow-popup>.ix-select-chip{max-width:100%;white-space:normal;overflow-wrap:anywhere}
.ix-theme .ix-select-chips{flex:1;align-items:center;padding:.375rem 0 .375rem .75rem}.ix-theme .ix-select-trigger-multiple{width:5rem;flex:0 0 5rem;padding-inline:.5rem}.ix-theme .ix-select-chip-remove{display:inline-flex;align-items:center;justify-content:center;margin-inline-start:.25rem;padding:0;width:1.25rem;height:1.25rem;border:0;background:transparent;color:inherit;border-radius:50%;cursor:pointer}.ix-theme .ix-select-chip-remove:hover{background:var(--ix-color-border)}.ix-theme .ix-select-chip-remove:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}.ix-theme .ix-select-control:has(.ix-select-trigger:disabled) .ix-select-chips{opacity:.45}
.ix-theme .ix-select-clear{display:flex;align-items:center;justify-content:center;border:0;background:transparent;color:var(--ix-color-text);padding:.5rem;cursor:pointer;flex:none}.ix-theme .ix-select-clear:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-select-clear:focus-visible{outline:2px solid var(--ix-color-focus)}
.ix-theme .ix-select-native{position:absolute;width:1px;height:1px;opacity:0;overflow:hidden;pointer-events:none;inset-inline-start:0;bottom:1rem}.ix-theme .ix-select-backdrop{position:fixed;inset:0;border:0;background:transparent;z-index:90;cursor:default}
.ix-theme .ix-select-popup{position:absolute;top:100%;inset-inline:0;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);box-shadow:0 8px 24px #0004;z-index:91;min-width:12rem;padding:.25rem 0;border-radius:var(--ix-radius)}.ix-theme .ix-select-list-header{padding:.5rem .75rem;color:var(--ix-color-text-secondary);font-size:.875rem;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-select-list{max-height:18rem;overflow:auto;overscroll-behavior:contain}
.ix-theme .ix-select-popup[data-top-layer=true]{position:fixed;inset:auto;margin:0;box-sizing:border-box;color:var(--ix-color-text);overflow:hidden}.ix-theme .ix-select-popup[data-top-layer=true]:popover-open{display:flex;flex-direction:column}.ix-theme .ix-select-popup[data-top-layer=true]>.ix-select-list{flex:0 1 auto;min-height:0}.ix-theme .ix-select-popup[data-top-layer=true]>.ix-select-list-header{flex:none}
.ix-theme .ix-select-option{display:flex;align-items:center;gap:.5rem;min-height:2.5rem;width:100%;font:inherit;font-size:.875rem;text-align:start;color:var(--ix-color-text);padding:.5rem .75rem;border:0;background:transparent;cursor:pointer}.ix-theme .ix-select-option>.ix-icon:last-child{margin-inline-start:auto}.ix-theme .ix-select-option:hover,.ix-theme .ix-select-active{background:var(--ix-color-surface-2)}.ix-theme .ix-select-option[aria-selected=true]{color:var(--ix-color-primary)}.ix-theme .ix-select-option:disabled{opacity:.45;cursor:not-allowed}.ix-theme .ix-select-option:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}
.ix-theme .ix-select-choice{width:1rem;height:1rem;border:1px solid var(--ix-color-text-secondary);border-radius:2px;display:flex;align-items:center;justify-content:center;flex-shrink:0}.ix-theme .ix-select-choice.checked{background:var(--ix-color-primary);color:var(--ix-color-on-primary);border-color:var(--ix-color-primary)}.ix-theme .ix-select-empty{padding:.75rem;color:var(--ix-color-text-secondary);font-size:.875rem}
.ix-theme .ix-upload{display:flex;align-items:center;justify-content:space-between;flex-direction:row;gap:1rem;position:relative;min-height:4rem;padding:1rem;box-sizing:border-box;border:1px dashed var(--ix-color-border);color:var(--ix-color-text-secondary);font-size:.875rem;cursor:pointer;background:var(--ix-color-surface-1)}.ix-theme .ix-upload:hover{border-color:var(--ix-color-primary)}.ix-theme .ix-upload>input{position:absolute;inset:0;opacity:0;width:100%;height:100%;cursor:inherit}.ix-theme .ix-upload:has(:focus-visible){outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-theme .ix-upload-button{padding:.5rem 1rem;color:var(--ix-color-primary);border:1px solid var(--ix-color-primary);border-radius:var(--ix-radius)}.ix-theme .ix-upload-success{border-color:var(--ix-color-success)}.ix-theme .ix-upload-failed{border-color:var(--ix-color-danger-text);color:var(--ix-color-danger-text)}.ix-theme .ix-upload[data-disabled=true]{cursor:not-allowed;opacity:.45}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_commit_snapshots_focus_and_deduplicates_enter_then_blur() {
        let mut tracker = TextCommitState {
            current: "initial".into(),
            baseline: None,
        };
        tracker.focus();
        assert_eq!(tracker.commit(), None, "unchanged blur is silent");
        tracker.current = "changed".into();
        assert_eq!(
            tracker.commit().as_deref(),
            Some("changed"),
            "Enter commits"
        );
        assert_eq!(tracker.commit(), None, "repeated Enter and blur are silent");
        tracker.current.clear();
        assert_eq!(
            tracker.commit().as_deref(),
            Some(""),
            "clearing is a change"
        );
        tracker.current = "programmatic before focus".into();
        tracker.focus();
        assert_eq!(tracker.commit(), None, "refocus snapshots latest prop");
        tracker.current = "programmatic while focused".into();
        assert_eq!(
            tracker.commit().as_deref(),
            Some("programmatic while focused")
        );
    }

    #[derive(Default)]
    struct TextCommitTestConfig {
        value: String,
        callback_revision: usize,
        callbacks: Option<(EventHandler<()>, EventHandler<()>)>,
        events: Vec<String>,
    }

    fn text_commit_fixture(config: Rc<std::cell::RefCell<TextCommitTestConfig>>) -> Element {
        let value = config.borrow().value.clone();
        let revision = config.borrow().callback_revision;
        let output = config.clone();
        let onchange = use_callback(move |value| {
            output
                .borrow_mut()
                .events
                .push(format!("{revision}:{value}"))
        });
        let callbacks = use_text_commit(&value, onchange);
        config.borrow_mut().callbacks = Some(callbacks);
        VNode::empty()
    }

    #[test]
    fn text_commit_uses_latest_controlled_props_and_callbacks_without_emitting_on_render() {
        let config = Rc::new(std::cell::RefCell::new(TextCommitTestConfig {
            value: "initial".into(),
            ..Default::default()
        }));
        let mut dom = VirtualDom::new_with_props(text_commit_fixture, config.clone());
        dom.rebuild_in_place();
        let (focus, commit) = config.borrow().callbacks.unwrap();
        dom.in_runtime(|| focus.call(()));
        // A parent-rejected native edit leaves the controlled prop unchanged.
        dom.in_runtime(|| commit.call(()));
        assert!(config.borrow().events.is_empty());
        // Parent transforms an edit before acknowledging it through props.
        config.borrow_mut().value = "ACCEPTED".into();
        config.borrow_mut().callback_revision = 1;
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        assert!(
            config.borrow().events.is_empty(),
            "prop changes alone emit nothing"
        );
        dom.in_runtime(|| commit.call(()));
        dom.in_runtime(|| commit.call(()));
        assert_eq!(config.borrow().events, ["1:ACCEPTED"]);
        config.borrow_mut().value = "programmatic".into();
        config.borrow_mut().callback_revision = 2;
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        assert_eq!(config.borrow().events.len(), 1);
        dom.in_runtime(|| commit.call(()));
        assert_eq!(config.borrow().events, ["1:ACCEPTED", "2:programmatic"]);
    }

    #[test]
    fn field_feedback_ignores_blank_and_inactive_messages_and_keeps_helper_fallback_neutral() {
        for state in [
            FieldState::Invalid,
            FieldState::Warning,
            FieldState::Info,
            FieldState::Valid,
        ] {
            let feedback = field_feedback(
                state,
                Some("Helper".into()),
                Some(" ".into()),
                Some("\t".into()),
                None,
                Some("\n".into()),
            )
            .unwrap();
            assert_eq!(feedback.text, "Helper");
            assert_eq!(feedback.state, FieldState::Default);
        }
        assert!(
            field_feedback(
                FieldState::Default,
                Some(" \t".into()),
                Some("Ignored".into()),
                None,
                None,
                None
            )
            .is_none()
        );
        let feedback = field_feedback(
            FieldState::Invalid,
            Some("Helper".into()),
            Some("Required".into()),
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(feedback.text, "Required");
        assert_eq!(feedback.state, FieldState::Invalid);
    }

    fn text_feedback_fixture() -> Element {
        rsx! {
            Input { id: "input-feedback", value: "a😀", max_length: 4,
                show_text_as_tooltip: true, state: FieldState::Invalid,
                helper_text: "Inactive input helper", invalid_text: "Input error" }
            Textarea { id: "textarea-feedback", value: "a😀", max_length: 4,
                show_text_as_tooltip: true, state: FieldState::Warning,
                helper_text: "Inactive textarea helper", warning_text: "Textarea warning" }
        }
    }

    #[test]
    fn text_feedback_tooltips_describe_controls_keep_counters_and_render_state_icons_once() {
        let mut dom = VirtualDom::new(text_feedback_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in ["input-feedback", "textarea-feedback"] {
            assert!(
                html.contains(&format!("aria-describedby=\"{id}-help\"")),
                "{html}"
            );
            assert!(html.contains(&format!("id=\"{id}-help\"")), "{html}");
        }
        assert!(html.contains("data-anchor-id=\"input-feedback-input-wrap\""));
        assert!(html.contains("data-anchor-id=\"textarea-feedback\""));
        assert_eq!(html.matches("role=\"tooltip\"").count(), 2);
        assert_eq!(html.matches("data-placement=\"bottom\"").count(), 2);
        assert_eq!(html.matches("Input error").count(), 1);
        assert_eq!(html.matches("Textarea warning").count(), 1);
        assert_eq!(
            html.matches("3/4").count(),
            2,
            "counters remain outside tooltips"
        );
        assert_eq!(html.matches("ix-helper-icon").count(), 2);
        assert!(!html.contains("Inactive input helper"));
        assert!(!html.contains("Inactive textarea helper"));
    }

    fn blank_text_feedback_fixture() -> Element {
        rsx! {
            Input { id: "blank-input", show_text_as_tooltip: true, helper_text: " \t", invalid_text: "Ignored inactive error" }
            Textarea { id: "blank-textarea", show_text_as_tooltip: true, state: FieldState::Invalid, invalid_text: " ", helper_text: "\n" }
            Input { id: "inline-input", state: FieldState::Invalid, invalid_text: " ", helper_text: "Inline helper fallback" }
            Textarea { id: "inline-textarea", state: FieldState::Valid, valid_text: "Accepted textarea" }
        }
    }

    #[test]
    fn blank_tooltips_are_absent_and_inline_feedback_has_matching_accessible_ids() {
        let mut dom = VirtualDom::new(blank_text_feedback_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("role=\"tooltip\""));
        assert!(!html.contains("blank-input-help"));
        assert!(!html.contains("blank-textarea-help"));
        assert!(!html.contains("Ignored inactive error"));
        assert!(html.contains("id=\"inline-input-help\""));
        assert!(html.contains("id=\"inline-textarea-help\""));
        assert_eq!(html.matches("Inline helper fallback").count(), 1);
        assert_eq!(html.matches("Accepted textarea").count(), 1);
        assert_eq!(
            html.matches("ix-helper-icon").count(),
            1,
            "helper fallback has no state icon"
        );
    }
    #[test]
    fn upload_policy_rejects_multiple_plain_files_but_allows_folder_contents() {
        assert_eq!(
            upload_count_allowed(2, false, false),
            Err(UploadError::MultipleFilesNotAllowed)
        );
        assert_eq!(upload_count_allowed(2, false, true), Ok(()));
        assert_eq!(upload_count_allowed(2, true, false), Ok(()));
        assert_eq!(upload_count_allowed(0, false, false), Ok(()));
    }

    #[test]
    fn directory_picker_attributes_are_absent_in_file_mode_and_preserve_native_form_submission() {
        fn normal() -> Element {
            rsx! { Upload { name: "attachment" } }
        }
        fn folder() -> Element {
            rsx! { Upload { directory_upload: true, name: "folder", multiline: true } }
        }
        for (app, directory) in [
            (normal as fn() -> Element, false),
            (folder as fn() -> Element, true),
        ] {
            let mut dom = VirtualDom::new(app);
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert_eq!(html.contains("webkitdirectory"), directory, "{html}");
            assert_eq!(html.contains(" multiple"), directory, "{html}");
            assert!(html.contains(if directory {
                "name=\"folder\""
            } else {
                "name=\"attachment\""
            }));
            assert!(html.contains(if directory {
                "Upload folder…"
            } else {
                "Upload file…"
            }));
        }
    }

    #[test]
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    fn native_folder_selection_keeps_nested_relative_names_and_readable_file_handles() {
        let directory = std::env::temp_dir().join(format!(
            "dioxus-ix-upload-{}-{}",
            std::process::id(),
            FIELD_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::create_dir(directory.join("nested")).unwrap();
        std::fs::write(directory.join("same.rs"), "root").unwrap();
        std::fs::write(directory.join("nested/same.rs"), "nested").unwrap();
        let mut entries = Vec::new();
        collect_upload_directory(&directory, std::path::Path::new("Folder"), &mut entries).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0].relative_path,
            std::path::PathBuf::from("Folder/nested/same.rs")
        );
        assert_eq!(
            entries[1].relative_path,
            std::path::PathBuf::from("Folder/same.rs")
        );
        assert_eq!(entries[0].file.name(), entries[1].file.name());
        assert_ne!(entries[0].file.path(), entries[1].file.path());
        assert_eq!(
            std::fs::read_to_string(entries[0].file.path()).unwrap(),
            "nested"
        );
        assert_eq!(entries[1].file.size(), 4);
        std::fs::remove_file(directory.join("nested/same.rs")).unwrap();
        std::fs::remove_file(directory.join("same.rs")).unwrap();
        std::fs::remove_dir(directory.join("nested")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn measured_chip_partition_reacts_to_resize_removal_and_badge_digits() {
        let widths = [80.0, 110.0, 50.0, 70.0];
        let badges = [25.0, 25.0, 25.0, 25.0];
        assert_eq!(fitting_chips(&widths, &badges, 322.0, 4.0), 4);
        assert_eq!(fitting_chips(&widths, &badges, 280.0, 4.0), 3);
        assert_eq!(fitting_chips(&widths, &badges, 210.0, 4.0), 1);
        assert_eq!(fitting_chips(&widths, &badges, 223.0, 4.0), 2);
        assert_eq!(fitting_chips(&widths, &badges, 20.0, 4.0), 1);
        assert_eq!(fitting_chips(&widths[..1], &badges, 20.0, 4.0), 1);
        assert_eq!(fitting_chips(&[], &[], 0.0, 4.0), 0);
        // Remove a selected chip: the remaining values can all reappear.
        assert_eq!(fitting_chips(&[80.0, 50.0], &badges, 134.0, 4.0), 2);
        // A wider two-digit +N indicator changes the number of visible chips.
        let many = [40.0; 12];
        let mut badges = [25.0; 12];
        badges[9..].fill(34.0);
        assert_eq!(fitting_chips(&many, &badges, 117.0, 4.0), 1);
        assert_eq!(fitting_chips(&many, &[25.0; 12], 117.0, 4.0), 2);
    }
    #[test]
    fn select_filter_matches_upstream_substrings_case_and_trimmed_whitespace() {
        assert!(select_matches("Framework Option 10", " OPTION 1 "));
        assert!(select_matches("Überwachung", "ÜBER"));
        assert!(select_matches("Option 1", "  "));
        assert!(!select_matches("Option 1", "Option 10"));
        let options = [
            SelectOption::new("a", "Alpha").disabled(),
            SelectOption::new("b", "Beta"),
        ];
        let enabled: Vec<_> = options
            .iter()
            .filter(|option| select_matches(&option.label, "a") && !option.disabled)
            .map(|option| option.value.as_str())
            .collect();
        assert_eq!(enabled, ["b"]);
    }
    #[test]
    fn select_inline_navigation_starts_from_input_skips_disabled_and_includes_add_row() {
        // Visible source indices 0 and 2 are disabled; index 4 is Add item.
        let enabled = [1, 3, 4];
        assert_eq!(select_next_active(&enabled, usize::MAX, false), Some(1));
        assert_eq!(select_next_active(&enabled, usize::MAX, true), Some(4));
        assert_eq!(select_next_active(&enabled, 1, false), Some(3));
        assert_eq!(select_next_active(&enabled, 3, false), Some(4));
        assert_eq!(select_next_active(&enabled, 4, false), Some(1));
        assert_eq!(select_next_active(&enabled, 1, true), Some(4));
        assert_eq!(select_next_active(&[], usize::MAX, false), None);
        // Removing/filtering the formerly active option restarts at the first.
        assert_eq!(select_next_active(&[3], 1, false), Some(3));
    }
    #[test]
    fn select_add_row_uses_trimmed_exact_label_equality_like_source() {
        let options = [SelectOption::new("existing-id", "Existing")];
        assert!(!select_can_add(&options, "  ", true));
        assert!(!select_can_add(&options, "New", false));
        assert!(!select_can_add(&options, " Existing ", true));
        assert!(select_can_add(&options, "existing", true));
        assert!(select_can_add(&options, "New", true));
    }
    #[test]
    fn select_chip_accessible_labels_accept_translated_prefixes_and_count_templates() {
        assert_eq!(
            select_remove_label("Entfernen", "Überwachung"),
            "Entfernen Überwachung"
        );
        assert_eq!(select_remove_label("إزالة", "مضخة"), "إزالة مضخة");
        assert_eq!(select_remove_label("Remove", ""), "Remove");
        assert_eq!(
            select_more_label("Weitere {count} Elemente", 12),
            "Weitere 12 Elemente"
        );
        assert_eq!(select_more_label("{count} إضافية", 3), "3 إضافية");
        assert_eq!(
            select_more_label("Hidden selections", 3),
            "Hidden selections"
        );
    }
    #[test]
    fn select_required_feedback_waits_for_touch_and_clears_when_selected_or_optional() {
        assert_eq!(
            select_field_state(FieldState::Default, true, false, false),
            FieldState::Default
        );
        assert_eq!(
            select_field_state(FieldState::Default, true, true, false),
            FieldState::Invalid
        );
        assert_eq!(
            select_field_state(FieldState::Default, true, true, true),
            FieldState::Default
        );
        assert_eq!(
            select_field_state(FieldState::Default, false, true, false),
            FieldState::Default
        );
        assert_eq!(
            select_field_state(FieldState::Warning, true, true, false),
            FieldState::Warning
        );
    }
    #[derive(Clone)]
    struct SelectMethodsTestConfig {
        controller: SelectController,
        value: String,
        required: bool,
        visible: bool,
    }
    fn select_methods_fixture(config: Rc<std::cell::RefCell<SelectMethodsTestConfig>>) -> Element {
        let config = config.borrow().clone();
        rsx! {
            if config.visible {
                Select { id: "method-fixture", controller: config.controller, value: config.value,
                    required: config.required, options: vec![SelectOption::new("a", "Alpha")],
                    helper_text: "Choose an item", show_text_as_tooltip: true }
            }
        }
    }
    #[test]
    fn select_methods_track_controlled_values_and_reject_stale_or_ambiguous_mounts() {
        let controller = SelectController::default();
        assert_eq!(
            controller.has_valid_value(),
            Err(SelectMethodError::NotMounted)
        );
        let config = Rc::new(std::cell::RefCell::new(SelectMethodsTestConfig {
            controller: controller.clone(),
            value: "a".into(),
            required: true,
            visible: true,
        }));
        let mut dom = VirtualDom::new_with_props(select_methods_fixture, config.clone());
        dom.rebuild_in_place();
        assert_eq!(controller.has_valid_value(), Ok(true));
        assert_eq!(controller.is_touched(), Ok(false));
        assert!(matches!(
            controller.get_native_input_element(),
            Err(SelectMethodError::NotMounted)
        ));
        config.borrow_mut().required = false;
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        assert_eq!(
            controller.has_valid_value(),
            Ok(false),
            "upstream requires both required and a value"
        );
        config.borrow_mut().required = true;
        config.borrow_mut().value.clear();
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        assert_eq!(controller.has_valid_value(), Ok(false));
        let mut second = VirtualDom::new_with_props(select_methods_fixture, config.clone());
        second.rebuild_in_place();
        assert_eq!(
            controller.is_touched(),
            Err(SelectMethodError::MultipleMounted)
        );
        drop(second);
        assert_eq!(controller.is_touched(), Ok(false));
        config.borrow_mut().visible = false;
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        assert_eq!(controller.is_touched(), Err(SelectMethodError::NotMounted));
    }

    struct SelectFocusMount(Rc<std::cell::Cell<usize>>);
    impl dioxus::html::RenderedElementBacking for SelectFocusMount {
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
    fn ready_select_method<T>(future: impl std::future::Future<Output = T>) -> T {
        use std::{
            pin::pin,
            task::{Context, Poll, Waker},
        };
        match pin!(future).poll(&mut Context::from_waker(Waker::noop())) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("mock native method did not settle"),
        }
    }
    #[test]
    fn select_native_input_descriptor_focuses_its_mount_and_stops_after_unmount() {
        let count = Rc::new(std::cell::Cell::new(0));
        let alive = Rc::new(std::cell::Cell::new(true));
        let input = SelectNativeInput {
            id: "choice".into(),
            alive: alive.clone(),
            node: Rc::new(MountedData::new(SelectFocusMount(count.clone()))),
        };
        assert_eq!(input.id(), "choice");
        assert_eq!(ready_select_method(input.focus()), Ok(()));
        assert_eq!(count.get(), 1);
        alive.set(false);
        assert_eq!(
            ready_select_method(input.focus()),
            Err(SelectMethodError::NotMounted)
        );
        assert_eq!(
            count.get(),
            1,
            "stale descriptor must not focus a new mount"
        );
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        {
            let unsupported = SelectNativeInput {
                id: "unsupported".into(),
                alive: Rc::new(std::cell::Cell::new(true)),
                node: Rc::new(MountedData::new(())),
            };
            assert!(matches!(
                ready_select_method(unsupported.focus()),
                Err(SelectMethodError::Dom(_))
            ));
        }
    }
    #[test]
    fn select_tooltip_exposes_active_feedback_as_the_combobox_description() {
        let controller = SelectController::default();
        let config = Rc::new(std::cell::RefCell::new(SelectMethodsTestConfig {
            controller,
            value: String::new(),
            required: true,
            visible: true,
        }));
        let mut dom = VirtualDom::new_with_props(select_methods_fixture, config);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("aria-describedby=\"method-fixture-help\""));
        assert!(html.contains("id=\"method-fixture-help\""));
        let tooltip = html
            .split('<')
            .find(|fragment| {
                fragment
                    .split('>')
                    .next()
                    .is_some_and(|tag| tag.contains("id=\"method-fixture-help\""))
            })
            .expect("described tooltip element");
        let (attributes, _) = tooltip.split_once('>').expect("tooltip opening tag");
        assert!(attributes.contains("role=\"tooltip\""));
        assert_eq!(html.matches("Choose an item").count(), 1);
        assert!(
            !html.contains("class=\"ix-helper"),
            "tooltip mode must not duplicate feedback below the control"
        );
        assert_eq!(
            select_feedback_text(
                FieldState::Invalid,
                Some("Help".into()),
                Some("Choose an item first".into()),
                None,
                None,
                None
            )
            .as_deref(),
            Some("Choose an item first")
        );
        assert_eq!(
            select_feedback_text(
                FieldState::Info,
                Some("Help".into()),
                None,
                None,
                None,
                None
            )
            .as_deref(),
            Some("Help")
        );
    }

    #[test]
    fn select_controller_reassignment_releases_the_old_handle_without_remounting_the_input() {
        let first = SelectController::default();
        let second = SelectController::default();
        let config = Rc::new(std::cell::RefCell::new(SelectMethodsTestConfig {
            controller: first.clone(),
            value: "a".into(),
            required: true,
            visible: true,
        }));
        let mut dom = VirtualDom::new_with_props(select_methods_fixture, config.clone());
        dom.rebuild_in_place();
        assert_eq!(first.has_valid_value(), Ok(true));
        assert_eq!(second.has_valid_value(), Err(SelectMethodError::NotMounted));
        let first_binding = first.binding().unwrap();
        config.borrow_mut().controller = second.clone();
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        assert_eq!(first.has_valid_value(), Err(SelectMethodError::NotMounted));
        assert_eq!(second.has_valid_value(), Ok(true));
        let second_binding = second.binding().unwrap();
        assert!(
            Rc::ptr_eq(&first_binding.alive, &second_binding.alive),
            "controller replacement must preserve the actual input mount"
        );
        assert!(Rc::ptr_eq(&first_binding.input, &second_binding.input));
        assert_eq!(second_binding.id, "method-fixture");
        drop(dom);
        assert!(!first_binding.alive.get());
        assert!(!second_binding.alive.get());
        assert_eq!(second.is_touched(), Err(SelectMethodError::NotMounted));
    }
    #[test]
    fn select_add_cancellation_is_shared_and_scoped_to_each_request() {
        let rejected = SelectAddEvent::new("Keep typed text".into());
        let listener = rejected.clone();
        assert!(!rejected.default_prevented());
        listener.prevent_default();
        assert!(rejected.default_prevented());
        assert_eq!(rejected.value, "Keep typed text");
        assert!(!SelectAddEvent::new("Another option".into()).default_prevented());
    }
    fn select_add_guard_fixture() -> Element {
        let cancel = EventHandler::new(|event: SelectAddEvent| event.prevent_default());
        let allow = EventHandler::new(|_: SelectAddEvent| {});
        let rejected = select_add_allowed("Rejected".into(), cancel);
        let accepted = select_add_allowed("Accepted".into(), allow);
        rsx! { output { "{rejected}/{accepted}" } }
    }
    #[test]
    fn select_add_callback_can_veto_the_default_before_insertion() {
        let mut dom = VirtualDom::new(select_add_guard_fixture);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains("false/true"));
    }
    fn select_surface_contract_fixture() -> Element {
        rsx! {
            SelectOptionsSurface { anchor_id: "control", input_id: "choice", enable_top_layer: true,
                dropdown_width: Some("320px".into()), dropdown_max_width: None,
                onnativeerror: move |_| {}, onclose: move |_| {}, "Native list" }
            SelectOptionsSurface { anchor_id: "inline-control", input_id: "inline-choice", enable_top_layer: false,
                dropdown_width: Some("280px".into()), dropdown_max_width: Some("90%".into()),
                onnativeerror: move |_| {}, onclose: move |_| {}, "Inline list" }
        }
    }
    #[test]
    fn select_top_layer_is_opt_in_and_inline_fallback_preserves_custom_sizes() {
        let mut dom = VirtualDom::new(select_surface_contract_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("popover=\"auto\"").count(), 1);
        assert_eq!(html.matches("data-top-layer=\"true\"").count(), 1);
        assert_eq!(html.matches("data-top-layer=\"false\"").count(), 1);
        assert_eq!(
            html.matches("aria-label=\"Close selection list\"").count(),
            1
        );
        assert!(html.contains("width:280px;max-width:90%"));
    }
    type SelectSurfaceTestConfig = Rc<std::cell::RefCell<(Option<String>, Option<String>)>>;
    fn select_reactive_surface_fixture(config: SelectSurfaceTestConfig) -> Element {
        let (width, max_width) = config.borrow().clone();
        rsx! {
            SelectOptionsSurface { anchor_id: "reactive-control", input_id: "reactive-choice", enable_top_layer: true,
                dropdown_width: width, dropdown_max_width: max_width,
                onnativeerror: move |_| {}, onclose: move |_| {}, "Stable open list" }
        }
    }
    #[test]
    fn select_open_surface_receives_changed_and_removed_width_constraints_without_remounting() {
        let config = Rc::new(std::cell::RefCell::new((Some("480px".into()), None)));
        let mut dom = VirtualDom::new_with_props(select_reactive_surface_fixture, config.clone());
        dom.rebuild_in_place();
        let initial = dioxus_ssr::render(&dom);
        let id = initial
            .split("id=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_string();
        assert!(initial.contains("data-requested-width=\"480px\""));
        *config.borrow_mut() = (Some("480px".into()), Some("360px".into()));
        // Dioxus 0.8 inserts root error/suspense boundaries. The fixture lives
        // at APP; rerendering ROOT only revisits their memoized child props.
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        let limited = dioxus_ssr::render(&dom);
        assert!(limited.contains(&format!("id=\"{id}\"")));
        assert!(limited.contains("data-requested-max-width=\"360px\""));
        *config.borrow_mut() = (None, None);
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        let reset = dioxus_ssr::render(&dom);
        assert!(reset.contains(&format!("id=\"{id}\"")));
        assert!(!reset.contains("data-requested-width="));
        assert!(!reset.contains("data-requested-max-width="));
    }
    fn inline_select_contract_fixture() -> Element {
        rsx! {
            Select { id: "inline-choice", name: "choice", label: "Choice", value: "b",
                options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta")], allow_clear: true }
            Select { id: "editable-choice", editable: true, editable_placeholder: "Create an option", options: vec![] }
            Select { id: "readonly-choice", readonly: true, value: "a", options: vec![SelectOption::new("a", "Alpha")] }
        }
    }
    #[test]
    fn select_visible_combobox_is_inline_native_text_and_submission_keeps_option_value() {
        let mut dom = VirtualDom::new(inline_select_contract_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let input = html
            .split("<input")
            .find(|tag| {
                tag.split('>')
                    .next()
                    .is_some_and(|tag| tag.contains("id=\"inline-choice\""))
            })
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        assert!(input.contains("type=\"text\""));
        assert!(html.contains("role=\"combobox\""));
        assert!(html.contains("aria-autocomplete=\"list\""));
        assert!(html.contains("value=\"Beta\""));
        assert!(html.contains("value=\"b\" selected"));
        assert!(html.contains("name=\"choice\""));
        assert!(html.contains("placeholder=\"Create an option\""));
        assert!(html.contains("readonly"));
        assert!(!html.contains("Filter options"));
    }
    fn native_contract_fixture() -> Element {
        rsx! {
            Input { input_type: "password", value: "a😀", max_length: 4 }
            Select { name: "choices", multiple: true, values: vec!["a".into(), "b".into()],
                options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta")] }
        }
    }
    #[test]
    fn password_and_multiple_selection_keep_native_and_accessible_contracts() {
        let mut dom = VirtualDom::new(native_contract_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("type=\"password\""));
        assert!(html.contains("aria-label=\"Show password\""));
        assert!(
            html.contains("3/4"),
            "Length counters must match HTML UTF-16 length"
        );
        assert!(html.contains("aria-label=\"Remove Alpha\""));
        assert!(html.contains("aria-label=\"Remove Beta\""));
        assert!(html.contains("name=\"choices\""));
        assert!(html.contains("class=\"ix-select-native\""));
        assert!(html.contains("aria-hidden=\"true\""));
        let chip_start = html.find("ix-select-chips").unwrap();
        let trigger_start = html
            .find("ix-select-trigger ix-select-trigger-multiple")
            .unwrap();
        assert!(
            chip_start < trigger_start,
            "Chip buttons must precede the inline combobox input"
        );
    }
    #[test]
    fn character_filter_checks_every_inserted_unicode_character() {
        let letters = CharacterPattern::new(Some("[a-d]".into()));
        assert!(letters.allows("abcd"));
        assert!(letters.allows(""));
        assert!(!letters.allows("abed"));
        assert!(!letters.allows("a\n"));
        let unicode = CharacterPattern::new(Some("\\p{L}".into()));
        assert!(unicode.allows("å漢é"));
        assert!(!unicode.allows("å1"));
        assert!(CharacterPattern::new(None).allows("anything 123"));
        assert!(CharacterPattern::new(Some("[".into())).allows("anything"));
    }
    #[test]
    fn number_steps_preserve_decimal_precision_and_clamp_bounds() {
        assert_eq!(stepped_number("0.2", 0.1, None, None, true), "0.3");
        assert_eq!(stepped_number("0.3", 0.1, None, None, false), "0.2");
        assert_eq!(stepped_number("0.15", 0.1, None, None, true), "0.25");
        assert_eq!(stepped_number("9", 2.0, None, Some(10.0), true), "10");
        assert_eq!(stepped_number("1", 2.0, Some(0.0), None, false), "0");
        assert_eq!(stepped_number("", 1.0, None, None, true), "1");
        assert_eq!(stepped_number("NaN", f64::NAN, None, None, false), "-1");
        assert_eq!(stepped_number("0.1", 0.1, None, None, false), "0");
        assert_eq!(
            stepped_number("0.0000001", 0.0000001, None, None, true),
            "0.0000002"
        );
    }
    #[test]
    fn slider_values_are_bounded_and_nonfinite_values_do_not_escape() {
        assert_eq!(bounded_slider_value(-4.0, 0.0, 100.0), 0.0);
        assert_eq!(bounded_slider_value(250.0, 0.0, 100.0), 100.0);
        assert_eq!(bounded_slider_value(25.0, 10.0, 5.0), 10.0);
        assert_eq!(bounded_slider_value(f64::NAN, 10.0, 20.0), 10.0);
        assert_eq!(bounded_slider_value(50.0, f64::NAN, f64::NAN), 50.0);
        assert_eq!(bounded_slider_value(f64::INFINITY, 0.0, 100.0), 0.0);
    }
    #[test]
    fn single_selection_replaces_and_multiple_selection_toggles() {
        assert_eq!(
            update_selection(vec!["a".into()], "b".into(), false),
            vec!["b"]
        );
        assert_eq!(
            update_selection(vec!["a".into()], "b".into(), true),
            vec!["a", "b"]
        );
        assert_eq!(
            update_selection(vec!["a".into(), "b".into()], "a".into(), true),
            vec!["b"]
        );
    }
    #[test]
    fn number_input_typed_events_keep_empty_and_zero_distinct() {
        assert_eq!(number_input_event_value("", false), Some(0.0));
        assert_eq!(number_input_event_value(" ", true), None);
        assert_eq!(number_input_event_value("0", true), Some(0.0));
        assert_eq!(number_input_event_value("1E6", true), Some(1_000_000.0));
        assert_eq!(number_input_event_value("1e-6", true), Some(0.000001));
        assert_eq!(number_input_event_value("-2.5E2", false), Some(-250.0));
        assert_eq!(number_input_value("NaN"), None);
        assert_eq!(number_input_value("Infinity"), None);
        assert_eq!(number_input_value("1e999"), None);
    }

    #[test]
    fn number_input_commit_tracks_focus_and_deduplicates_enter_then_blur() {
        let mut commit = NumberCommit::new("123");
        commit.focus("123");
        assert!(!commit.commit(Some(123.0)));
        assert!(
            commit.commit(Some(1_000_000.0)),
            "Enter commits the numeric change"
        );
        assert!(
            !commit.commit(Some(1_000_000.0)),
            "Blur must not repeat Enter"
        );
        assert!(
            commit.commit(None),
            "Clearing commits empty, including with zero-valued input events"
        );
        assert!(!commit.commit(None));
        commit.focus("1e2");
        assert!(
            !commit.commit(Some(100.0)),
            "Equivalent notation is not a numeric change"
        );
        assert!(commit.commit(Some(101.0)));
    }

    #[test]
    fn number_input_blur_format_normalizes_scientific_notation_and_negative_zero() {
        assert_eq!(number_input_format_value(1e6), "1000000");
        assert_eq!(number_input_format_value(1e-6), "0.000001");
        assert_eq!(number_input_format_value(1e-7), "1e-7");
        assert_eq!(number_input_format_value(1e21), "1e+21");
        assert_eq!(number_input_format_value(-0.0), "0");
    }

    #[test]
    fn number_input_step_accepts_current_zero_and_negative_step_contracts() {
        assert_eq!(stepped_number("4", 0.0, None, None, true), "4");
        assert_eq!(stepped_number("4", -0.5, None, None, true), "3.5");
        assert_eq!(stepped_number("4", -0.5, None, None, false), "4.5");
        assert_eq!(stepped_number("1e2", 0.1, None, None, true), "100.1");
        assert_eq!(
            stepped_number("1e-17", 1e-17, None, None, true),
            "0.00000000000000002"
        );
    }

    fn number_input_contract_fixture() -> Element {
        rsx! {
            NumberInput { id: "number-default", label: "Default numeric value" }
            NumberInput { id: "number-empty", value: "", text_alignment_end: false,
                placeholder: "Enter an amount", required: true, suppress_submit_on_enter: true,
                show_stepper_buttons: true, name: "amount" }
            NumberInput { id: "number-readonly", value: "12", readonly: true, show_stepper_buttons: true }
            NumberInput { id: "number-disabled", value: "12", disabled: true, show_stepper_buttons: true }
        }
    }

    #[test]
    fn number_input_native_contract_defaults_to_zero_and_hides_unusable_steppers() {
        let mut dom = VirtualDom::new(number_input_contract_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"number-default\""));
        assert!(html.contains("value=\"0\""));
        assert!(html.contains("text-align:end"));
        assert!(html.contains("text-align:start"));
        assert!(html.contains("placeholder=\"Enter an amount\""));
        assert!(html.contains("name=\"amount\""));
        assert_eq!(html.matches("aria-label=\"increment number\"").count(), 4);
        assert_eq!(html.matches("aria-label=\"decrement number\"").count(), 4);
        assert_eq!(html.matches("data-hidden=\"true\"").count(), 3);
        assert_eq!(html.matches("data-hidden=\"false\"").count(), 1);
    }
}
