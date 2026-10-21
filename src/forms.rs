//! Form controls adapted from the iX component contracts using native form elements.
//!
//! Values are controlled: update `value` or `checked` from the corresponding event.
//! Text controls retain native constraint validation and form submission semantics.
//! The optional `form` ID associates each successful native control with an
//! external form; omission preserves normal ancestor association.
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

// Header, Popover and Pane groups are scope-owned. Capture participation once so conditional
// registration never changes hook order during ordinary control rerenders.
// Other form uses keep their existing controller-owned mounted lifecycle.
fn use_surface_focus_mount(id: String) -> Option<EventHandler<MountedEvent>> {
    let in_surface = use_hook(|| {
        try_consume_context::<crate::header_control_scope::HeaderControlGroup>().is_some()
            || try_consume_context::<crate::application::PaneFocusScope>().is_some()
            || try_consume_context::<crate::overlay_runtime::PopoverFocusScope>()
                .and_then(|scope| scope.owner)
                .is_some_and(|owner| owner.is_registered())
    });
    in_surface.then(|| crate::use_focus_mount(id))
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
    tooltip: crate::TooltipController,
    show_text_as_tooltip: bool,
) -> Element {
    rsx! {
        if let Some(feedback) = feedback {
            if show_text_as_tooltip {
                crate::TooltipFor { id, controller: tooltip, placement: crate::TooltipPlacement::Bottom, show_delay: 500,
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
    tooltip: crate::TooltipController,
    feedback_id: String,
    required: bool,
    state: FieldState,
    feedback: Option<FieldFeedback>,
    invalid_alert: bool,
    show_text_as_tooltip: bool,
    max_length: Option<u32>,
    length: usize,
    #[props(default)] onpresentationclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "ix-text-field-wrapper",
            onclick: onpresentationclick,
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
                if !show_text_as_tooltip { TextControlFeedback { feedback: feedback.clone(), id: feedback_id.clone(), tooltip: tooltip.clone(), show_text_as_tooltip: false } }
                div { class: "ix-text-field-bottom-right",
                    if let Some(limit) = max_length.filter(|limit| *limit > 0) {
                        crate::Typography { format: crate::TypographyFormat::Body, text_color: crate::TypographyColor::Soft,
                            class: "ix-text-field-counter ix-field-counter", "{length}/{limit}"
                        }
                    }
                }
            }
            if show_text_as_tooltip { TextControlFeedback { feedback, id: feedback_id, tooltip, show_text_as_tooltip: true } }
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
/// Composed `children` take precedence over plain `text`.
#[component]
pub fn FieldLabel(
    #[props(default)] text: String,
    #[props(default)] children: Option<Element>,
    #[props(default)] html_for: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] invalid: bool,
) -> Element {
    rsx! {
        label { class: if invalid { "ix-field-label ix-field-invalid" } else { "ix-field-label" },
            r#for: html_for,
            if let Some(content) = children { {content} } else { "{text}" }
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
    /// Rich label content, taking precedence over `label`.
    #[props(default)]
    label_content: Option<Element>,
    #[props(default)] html_for: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    /// ID of the rendered helper/validation text. Controls can reference this
    /// explicitly with `aria-describedby` and `aria-errormessage`.
    #[props(default)]
    helper_id: Option<String>,
    /// Independent content beside the helper text, outside the native label.
    #[props(default)]
    bottom_right: Option<Element>,
    /// Show nonblank helper/validation feedback on hover or focus of this field,
    /// including its composed label and footer. Native controls retain their
    /// caller-provided descriptions; `helper_id` identifies the feedback in
    /// either presentation mode.
    #[props(default)]
    show_text_as_tooltip: bool,
    #[props(default)] state: FieldState,
) -> Element {
    let root_id = field_id(None);
    let panel_id = format!("{root_id}-feedback-tooltip");
    let tooltip = crate::use_tooltip(panel_id.clone());
    let anchor = crate::use_tooltip_anchor(tooltip.clone(), root_id.clone());
    let feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    let helper = if !show_text_as_tooltip {
        rsx! { if let Some(feedback) = feedback.clone() { {render_field_feedback(feedback, helper_id.clone())} } }
    } else {
        rsx! {}
    };
    rsx! {
        div { id: root_id, class: "ix-field {state.class()}",
            onmounted: anchor.onmounted, onresize: anchor.onresize, onvisible: anchor.onvisible,
            onmouseenter: anchor.onmouseenter, onmouseleave: anchor.onmouseleave,
            onfocusin: anchor.onfocusin, onfocusout: anchor.onfocusout, onkeydown: anchor.onkeydown,
            if let Some(content) = label_content {
                FieldLabel { html_for, required, invalid: state == FieldState::Invalid, {content} }
            } else if let Some(text) = label {
                FieldLabel { text, html_for, required, invalid: state == FieldState::Invalid }
            }
            {children}
            if let Some(content) = bottom_right {
                div { class: "ix-field-composed-footer",
                    {helper}
                    div { class: "ix-field-bottom-right", {content} }
                }
            } else { {helper} }
            if show_text_as_tooltip {
                if let Some(feedback) = feedback {
                    crate::TooltipFor { id: panel_id, controller: tooltip, placement: crate::TooltipPlacement::Bottom, show_delay: 500,
                        class: "ix-field-feedback-tooltip",
                        {render_field_feedback(feedback, helper_id)}
                    }
                }
            }
        }
    }
}

/// Field wrapper for application-defined native controls. Custom Dioxus controls
/// call [`crate::use_custom_field_state`] to report validation, or callers pass
/// `state` explicitly. Unregistered DOM class changes are not observed.
#[component]
pub fn CustomField(
    children: Element,
    #[props(default)] label: Option<String>,
    #[props(default)] label_content: Option<Element>,
    #[props(default)] html_for: Option<String>,
    #[props(default)] required: bool,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] helper_id: Option<String>,
    #[props(default)] bottom_right: Option<Element>,
    #[props(default)] show_text_as_tooltip: bool,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
) -> Element {
    let id = field_id(None);
    let state = crate::form_control::use_custom_field_validation(state, &class);
    rsx! { div { id, class: "ix-custom-field {class}",
        FieldWrapper { label, label_content, html_for, required, helper_text, invalid_text, valid_text, info_text, warning_text, helper_id, bottom_right, show_text_as_tooltip, state,
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
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
    #[props(default = "text".to_string())] input_type: String,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] required: bool,
    /// Request native focus when mounted. Floating Panes also prioritize this
    /// registered control when they open.
    #[props(default)]
    autofocus: bool,
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
    /// Actual native control mount, after internal lifecycle registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
    #[props(default)] onixblur: EventHandler<()>,
    #[props(default)] onvaliditystatechange: EventHandler<ValidityStateChangeEvent>,
    #[props(default)] controller: Option<TextControlController>,
    #[props(default)] class: String,
) -> Element {
    let id = field_id(id);
    let surface_focus_mount = use_surface_focus_mount(id.clone());
    let (focus_commit, commit) = use_text_commit(&value, onchange);
    let has_value = if input_type == "number" {
        number_has_valid_value(&value, required)
    } else {
        !value.is_empty()
    };
    let lifecycle = crate::form_control::use_control(
        &id,
        has_value,
        !disabled && !input_type.eq_ignore_ascii_case("hidden"),
        controller,
        onvaliditystatechange,
    );
    let supplied = crate::form_control::validation_class_state(&class);
    let state = if state != FieldState::Default {
        state
    } else if lifecycle.invalid {
        FieldState::Invalid
    } else {
        supplied
    };
    crate::form_control::use_custom_field_state(crate::form_control::control_validation(
        state, &class,
    ));
    let invalid_text = invalid_text.or_else(|| {
        (lifecycle.invalid && !lifecycle.message.is_empty()).then_some(lifecycle.message)
    });
    let observe = lifecycle.observe;
    let mounted = lifecycle.mounted;
    let focus_state = lifecycle.focus_state;
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
    let adornments = crate::input_geometry::use_adornments();
    let padding = adornments.facts;
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
    let tooltip = crate::use_tooltip(help_id.clone());
    let tooltip_anchor = crate::use_tooltip_anchor(tooltip.clone(), anchor_id.clone());
    let describedby = has_feedback.then(|| help_id.clone());
    rsx! {
        div { id: geometry_id, class: "ix-field ix-text-field-native ix-input-native {state.class()} {class}",
            "data-disabled": disabled.to_string(), "data-readonly": readonly.to_string(),
            TextFieldPresentation { label, control_id: id.clone(), tooltip: tooltip.clone(), feedback_id: help_id.clone(),
                required, state, feedback, invalid_alert, show_text_as_tooltip, max_length, length,
                div { id: anchor_id, class: "ix-input-wrap",
                    onmounted: tooltip_anchor.onmounted, onresize: tooltip_anchor.onresize, onvisible: tooltip_anchor.onvisible,
                    onmouseenter: tooltip_anchor.onmouseenter, onmouseleave: tooltip_anchor.onmouseleave,
                    onfocusin: tooltip_anchor.onfocusin, onfocusout: tooltip_anchor.onfocusout, onkeydown: tooltip_anchor.onkeydown,
                    "data-padding-ready": padding.is_some().to_string(),
                    "data-padding-start": padding.map(|facts| facts.start.to_string()),
                    "data-padding-end": padding.map(|facts| facts.end.to_string()),
                    span { class: "ix-input-start", onresize: adornments.start_resized, if let Some(start) = start { {start} } }
                input { id, class: "ix-input", name, form, r#type: native_type, value, placeholder,
                    required, disabled, readonly, autofocus, pattern, minlength: min_length, maxlength: max_length,
                    min, max, step, autocomplete,
                    style: input_style,
                    "aria-label": aria_label,
                    "aria-invalid": (state == FieldState::Invalid).to_string(),
                    "aria-describedby": describedby,
                    "aria-errormessage": (state == FieldState::Invalid && has_feedback).then(||help_id.clone()),
                    "data-ix-touched": lifecycle.touched.to_string(),
                    onmounted: move |event| { mounted.call(event.data()); if let Some(mount) = surface_focus_mount { mount.call(event.clone()); } onmounted.call(event); },
                    oninput: move |event| { onvaluechange.call(event.value()); observe.call(false); },
                    onfocus: move |event| { focus_state.call(true); focus_commit.call(()); onfocus.call(event); },
                    onblur: move |event| { focus_state.call(false); onixblur.call(()); observe.call(true); commit.call(()); onblur.call(event); },
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
                    span { class: "ix-input-end", onresize: adornments.end_resized,
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
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
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
    /// Actual native control mount, after internal lifecycle registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
    #[props(default)] onkeydown: EventHandler<KeyboardEvent>,
    #[props(default)] onixblur: EventHandler<()>,
    #[props(default)] onvaliditystatechange: EventHandler<ValidityStateChangeEvent>,
    #[props(default)] controller: Option<TextControlController>,
    #[props(default)] class: String,
) -> Element {
    let id = field_id(id);
    let surface_focus_mount = use_surface_focus_mount(id.clone());
    let (focus_commit, commit) = use_text_commit(&value, onchange);
    let lifecycle = crate::form_control::use_control(
        &id,
        !value.is_empty(),
        !disabled,
        controller,
        onvaliditystatechange,
    );
    let supplied = crate::form_control::validation_class_state(&class);
    let state = if state != FieldState::Default {
        state
    } else if lifecycle.invalid {
        FieldState::Invalid
    } else {
        supplied
    };
    crate::form_control::use_custom_field_state(crate::form_control::control_validation(
        state, &class,
    ));
    let invalid_text = invalid_text.or_else(|| {
        (lifecycle.invalid && !lifecycle.message.is_empty()).then_some(lifecycle.message)
    });
    let observe = lifecycle.observe;
    let mounted = lifecycle.mounted;
    let focus_state = lifecycle.focus_state;
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
    let tooltip = crate::use_tooltip(help_id.clone());
    let tooltip_anchor = crate::use_tooltip_anchor(tooltip.clone(), id.clone());
    let describedby = has_feedback.then(|| help_id.clone());
    rsx! {
        div { class: "ix-field ix-text-field-native ix-textarea-native {state.class()} {class}",
            "data-disabled": disabled.to_string(), "data-readonly": readonly.to_string(),
            TextFieldPresentation { label, control_id: id.clone(), tooltip: tooltip.clone(), feedback_id: help_id.clone(),
                required, state, feedback, invalid_alert, show_text_as_tooltip, max_length, length,
                div { class: "ix-input-wrap",
            textarea { id, class: "ix-input ix-textarea", name, form, value, placeholder,
                required, disabled, readonly, rows, cols, minlength: min_length, maxlength: max_length,
                style: dimensions, "aria-label": aria_label, "aria-invalid": (state == FieldState::Invalid).to_string(),
                "aria-describedby": describedby,
                "aria-errormessage": (state == FieldState::Invalid && has_feedback).then(||help_id.clone()),
                "data-ix-touched": lifecycle.touched.to_string(),
                onmounted: move |event| { mounted.call(event.data()); if let Some(mount) = surface_focus_mount { mount.call(event.clone()); } tooltip_anchor.onmounted.call(event.clone()); onmounted.call(event); },
                onresize: tooltip_anchor.onresize, onvisible: tooltip_anchor.onvisible,
                onmouseenter: tooltip_anchor.onmouseenter, onmouseleave: tooltip_anchor.onmouseleave,
                onfocusin: tooltip_anchor.onfocusin, onfocusout: tooltip_anchor.onfocusout,
                oninput: move |event| { onvaluechange.call(event.value()); observe.call(false); },
                onfocus: move |event| { focus_state.call(true); focus_commit.call(()); onfocus.call(event); },
                onkeydown: move |event:KeyboardEvent| { onkeydown.call(event.clone()); tooltip_anchor.onkeydown.call(event); },
                onblur: move |event| { focus_state.call(false); onixblur.call(()); observe.call(true); commit.call(()); onblur.call(event); },
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
    /// Optional noninteractive visible label content, rendered by Dioxus in the
    /// existing typography. Replaces `label` visually; a supplied `aria_label`
    /// wins, otherwise nonempty `label` names composed content explicitly.
    /// Without either string, the native checkbox uses its visible child text.
    #[props(default)]
    children: Option<Element>,
    /// Stable ID of the native interactive control, not its form mirror.
    /// Keep the ID stable for this mounted component scope.
    #[props(default)]
    id: Option<String>,
    #[props(default)] name: Option<String>,
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
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
    /// Receives the interactive native control after internal mounted registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
) -> Element {
    let id = field_id(id);
    let surface_focus_mount = use_surface_focus_mount(id.clone());
    let mut button = use_signal(|| None::<Rc<MountedData>>);
    let mut attempted = use_signal(|| false);
    let label_less = label.is_empty() && children.is_none();
    let aria_label =
        aria_label.or_else(|| (children.is_some() && !label.is_empty()).then(|| label.clone()));
    let invalid = invalid || state == FieldState::Invalid || (attempted() && required && !checked);
    crate::form_control::use_custom_field_state(if invalid {
        FieldState::Invalid
    } else {
        crate::form_control::control_validation(state, &class)
    });
    rsx! {
        span { class: "ix-checkbox-field ix-checkbox-native {state.class()} {class}",
            "data-checked": checked.to_string(), "data-mixed": indeterminate.to_string(),
            "data-invalid": invalid.to_string(), "data-label-less": label_less.to_string(),
            input { class: "ix-check-native", r#type: "checkbox", name, form, value, checked, disabled, required,
                "aria-hidden": "true", tabindex: "-1",
                oninvalid: move |event| {
                    event.prevent_default(); attempted.set(true);
                    let node = button.peek().clone();
                    spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } });
                },
            }
            button { id, class: "ix-check ix-checkbox {state.class()}", r#type: "button", role: "checkbox", disabled,
                "data-mixed": indeterminate.to_string(), "data-checked": checked.to_string(),
                "aria-checked": if indeterminate { "mixed" } else if checked { "true" } else { "false" },
                "aria-required": required.to_string(), "aria-invalid": invalid.to_string(),
                "aria-disabled": disabled.to_string(), aria_label,
                onmounted: move |event| { button.set(Some(event.data())); if let Some(mount) = surface_focus_mount { mount.call(event.clone()); } onmounted.call(event); }, onclick: move |_| { if !disabled { oncheckedchange.call(!checked); } }, onblur,
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
                    class: "ix-checkbox-label", if let Some(children) = children { {children} } else { "{label}" }
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
    feedback_id: String,
    show_text_as_tooltip: bool,
) -> Element {
    let tooltip = crate::use_tooltip(feedback_id.clone());
    let anchor_id = format!("{feedback_id}-choices");
    let anchor = crate::use_tooltip_anchor(tooltip.clone(), anchor_id.clone());
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
            div { id:anchor_id,class: "ix-group-slot",
                onmounted:anchor.onmounted,onresize:anchor.onresize,onvisible:anchor.onvisible,
                onmouseenter:anchor.onmouseenter,onmouseleave:anchor.onmouseleave,
                onfocusin:anchor.onfocusin,onfocusout:anchor.onfocusout,onkeydown:anchor.onkeydown,
                {children}
            }
            div { class: "ix-group-bottom",
                if !show_text_as_tooltip {if let Some(feedback) = feedback.clone() {
                    GroupFeedback { feedback, id:Some(feedback_id.clone()) }
                }}
                div { class: "ix-group-bottom-right" }
            }
            if show_text_as_tooltip {if let Some(feedback) = feedback {
                crate::TooltipFor {id:feedback_id,controller:tooltip,placement:crate::TooltipPlacement::Bottom,show_delay:500,
                    GroupFeedback {feedback,inherit_color:true}
                }
            }}
        }
    }
}

#[component]
fn GroupFeedback(
    feedback: FieldFeedback,
    id: Option<String>,
    #[props(default)] inherit_color: bool,
) -> Element {
    let (icon, color) = match feedback.state {
        FieldState::Default => (None, crate::TypographyColor::Soft),
        FieldState::Invalid => (Some("error"), crate::TypographyColor::Alarm),
        FieldState::Valid => (Some("success"), crate::TypographyColor::Std),
        FieldState::Info => (Some("info"), crate::TypographyColor::Std),
        FieldState::Warning => (Some("warning"), crate::TypographyColor::Std),
    };
    rsx! {
        crate::Typography { id, format: crate::TypographyFormat::BodyParagraph,
            text_color: (!inherit_color).then_some(color), class: "ix-group-bottom-text {feedback.state.class()}",
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
/// Feedback is inline by default; `show_text_as_tooltip` uses the existing
/// bottom tooltip on choice hover/focus without changing child keyboard behavior.
#[component]
pub fn CheckboxGroup(
    children: Element,
    #[props(default)] label: String,
    #[props(default)] horizontal: bool,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    /// Move current nonblank feedback from the inline row to a shared tooltip.
    #[props(default)]
    show_text_as_tooltip: bool,
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
    crate::form_control::use_custom_field_state(crate::form_control::control_validation(
        state, &class,
    ));
    let feedback_id = format!("{}-help", field_id(None));
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
    let describedby = feedback.is_some().then(|| feedback_id.clone());
    rsx! {
        fieldset { class: "ix-group-native ix-checkbox-group {state.class()} {class}", disabled,
            "aria-label": accessible_label, "aria-labelledby": aria_labelledby,"aria-describedby":describedby,
            GroupLayout { label, required, state, feedback, invalid_alert,feedback_id,show_text_as_tooltip,
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
    /// Optional noninteractive visible label content owned by Dioxus, replacing
    /// `label` in the existing typography. Supply `label` or `aria_label` as its
    /// semantic name, including when only composed content is visible.
    #[props(default)]
    children: Option<Element>,
    /// Stable ID of the native interactive control, not its form mirror.
    /// Keep the ID stable for this mounted component scope.
    #[props(default)]
    id: Option<String>,
    #[props(default)] name: Option<String>,
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
    #[props(default)] checked: bool,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    #[props(default)] state: FieldState,
    #[props(default)] class: String,
    aria_label: Option<String>,
    #[props(default)] oncheckedchange: EventHandler<bool>,
    #[props(default)] onvaluechange: EventHandler<String>,
    /// Receives the interactive native control after internal mounted registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
) -> Element {
    let id = field_id(id);
    let surface_focus_mount = use_surface_focus_mount(id.clone());
    crate::form_control::use_custom_field_state(crate::form_control::control_validation(
        state, &class,
    ));
    let selected = value.clone();
    let label_less = label.is_empty() && children.is_none();
    let accessible_label = aria_label.unwrap_or_else(|| label.clone());
    rsx! {
        label { class: "ix-check ix-radio ix-radio-native {state.class()} {class}",
            "data-checked": checked.to_string(), "data-disabled": disabled.to_string(),
            "data-label-less": label_less.to_string(),
            input { id, r#type: "radio", name, form, value, checked, disabled, required,
                onmounted: move |event| { if let Some(mount) = surface_focus_mount { mount.call(event.clone()); } onmounted.call(event); },
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
            if !label_less {
                crate::Typography { format: crate::TypographyFormat::Body,
                    text_color: if disabled { crate::TypographyColor::Weak } else { crate::TypographyColor::Std },
                    class: "ix-radio-label", if let Some(children) = children { {children} } else { "{label}" }
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
/// Explicit `state` selects feedback; `show_text_as_tooltip` moves it into the
/// existing bottom tooltip. Native options keep their selection and constraints.
#[component]
pub fn RadioGroup(
    options: Vec<SelectOption>,
    /// Optional noninteractive visible option labels. Each current option keeps
    /// its semantic `label`, value and native selection behavior. Return a child
    /// component when the content needs hooks; do not call hooks in this callback.
    #[props(default)]
    render_label: Option<Callback<SelectOption, Element>>,
    #[props(default)] value: String,
    #[props(default)] label: String,
    #[props(default)] name: Option<String>,
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
    #[props(default)] horizontal: bool,
    #[props(default)] disabled: bool,
    #[props(default)] required: bool,
    /// Move current nonblank feedback from the inline row to a shared tooltip.
    #[props(default)]
    show_text_as_tooltip: bool,
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
    /// Receives each option value and its mounted native radio input.
    #[props(default)]
    onoptionmounted: EventHandler<(String, MountedEvent)>,
) -> Element {
    crate::form_control::use_custom_field_state(crate::form_control::control_validation(
        state, &class,
    ));
    let name = field_id(name);
    let feedback_id = format!("{}-help", field_id(None));
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
    let describedby = feedback.is_some().then(|| feedback_id.clone());
    rsx! {
        fieldset { class: "ix-group-native ix-radio-group {state.class()} {class}", disabled, role: "radiogroup",
            "aria-label": accessible_label, "aria-labelledby": aria_labelledby,"aria-describedby":describedby,
            GroupLayout { label, required, state, feedback, invalid_alert,feedback_id,show_text_as_tooltip,
                div { class: if horizontal { "ix-group-choices ix-group-row" } else { "ix-group-choices" },
                    for option in options {
                        Radio { key: "{option.value}", children: render_label.map(|renderer| renderer.call(option.clone())),
                            name: name.clone(), form:form.clone(), checked: option.value == value,
                            value: option.value.clone(), label: option.label, disabled: disabled || option.disabled, required, state, onvaluechange,
                            onmounted: move |event| onoptionmounted.call((option.value.clone(), event)) }
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
    /// Stable ID of the native interactive control, not its form mirror.
    /// Keep the ID stable for this mounted component scope.
    #[props(default)]
    id: Option<String>,
    #[props(default)] name: Option<String>,
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
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
    /// Receives the interactive native control after internal mounted registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
) -> Element {
    let id = field_id(id);
    let surface_focus_mount = use_surface_focus_mount(id.clone());
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
    crate::form_control::use_custom_field_state(if invalid {
        FieldState::Invalid
    } else {
        crate::form_control::control_validation(state, &class)
    });
    rsx! {
        span { class: "ix-toggle-field ix-toggle-native {state.class()} {class}",
            "data-checked": checked.to_string(), "data-mixed": indeterminate.to_string(), "data-invalid": invalid.to_string(),
            input { class: "ix-check-native", r#type: "checkbox", name, form, value, checked, disabled, required,
                "aria-hidden": "true", tabindex: "-1",
                oninvalid: move |event| {
                    event.prevent_default(); attempted.set(true);
                    let node = button.peek().clone();
                    spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } });
                },
            }
            button { id, class: "ix-toggle ix-toggle-control {state.class()} {class}", r#type: "button", role: "switch", disabled,
                "data-mixed": indeterminate.to_string(), "data-checked": checked.to_string(),
                "aria-label": accessible_label, "aria-labelledby": aria_labelledby, "aria-disabled": disabled.to_string(),
                "aria-checked": if indeterminate { "mixed" } else if checked { "true" } else { "false" },
                "aria-required": required.to_string(), "aria-invalid": invalid.to_string(),
                onmounted: move |event| { button.set(Some(event.data())); if let Some(mount) = surface_focus_mount { mount.call(event.clone()); } onmounted.call(event); }, onclick: move |_| { if !disabled { oncheckedchange.call(!checked); } }, onblur,
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
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
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
    /// Actual native control mount, after internal lifecycle registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
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
            Input { value: draft(), label, id, name, form, placeholder, aria_label, pattern,
                allowed_characters_pattern, controller, input_type: "number", autocomplete: "off",
                min: min.map(|v| v.to_string()), max: max.map(|v| v.to_string()),
                step: step.to_string(), disabled, readonly, required,
                helper_text, invalid_text, valid_text, info_text, warning_text, state,
                class: number_class, show_text_as_tooltip,
                onvaluechange: emit_value, text_alignment_end, suppress_submit_on_enter,
                onixblur, onvaliditystatechange, onmounted, start,
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
/// marker captions and optional value output are Rust adapters. The floating
/// value tooltip uses the shared Dioxus tooltip anchored to the moving thumb.
#[component]
pub fn Slider(
    #[props(default)] value: f64,
    #[props(default)] min: f64,
    #[props(default = 100.0)] max: f64,
    #[props(default = 1.0)] step: f64,
    #[props(default)] label: String,
    /// Stable ID of the native range input. Keep stable while mounted.
    #[props(default)]
    id: Option<String>,
    #[props(default)] name: Option<String>,
    /// Overrides the visible label as the range input's accessible name.
    #[props(default)]
    aria_label: Option<String>,
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] trace: bool,
    #[props(default)] trace_reference: f64,
    #[props(default)] markers: Vec<SliderMarker>,
    #[props(default)] show_value: bool,
    #[props(default = true)] show_tooltip: bool,
    #[props(default)] label_start: Option<String>,
    #[props(default)] label_end: Option<String>,
    /// Composed start endpoint, taking precedence over `label_start`, even when empty.
    /// Consumers own events and disabled behavior of controls placed here.
    #[props(default)]
    label_start_content: Option<Element>,
    /// Composed end endpoint, taking precedence over `label_end`, even when empty.
    /// Consumers own events and disabled behavior of controls placed here.
    #[props(default)]
    label_end_content: Option<Element>,
    #[props(default)] helper_text: Option<String>,
    #[props(default)] invalid_text: Option<String>,
    #[props(default)] valid_text: Option<String>,
    #[props(default)] info_text: Option<String>,
    #[props(default)] warning_text: Option<String>,
    #[props(default)] show_text_as_tooltip: bool,
    #[props(default)] class: String,
    #[props(default)] state: FieldState,
    #[props(default)] onvaluechange: EventHandler<f64>,
    /// Receives the native range input after scoped focus registration.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
) -> Element {
    let id = field_id(id);
    let surface_focus_mount = use_surface_focus_mount(id.clone());
    let anchor_id = format!("{id}-anchor");
    let feedback_id = format!("{id}-feedback");
    let value_tooltip_id = format!("{id}-value-tooltip");
    let thumb_id = format!("{id}-thumb");
    let (min, max) = slider_bounds(min, max);
    let value = bounded_slider_value(value, min, max);
    let span = (max - min).max(f64::EPSILON);
    let fraction = (value - min) / span;
    let reference = (bounded_slider_value(trace_reference, min, max) - min) / span;
    let start = fraction.min(reference);
    let end = fraction.max(reference);
    let has_endpoints = label_start.is_some()
        || label_end.is_some()
        || label_start_content.is_some()
        || label_end_content.is_some();
    let label_start = label_start.unwrap_or_default();
    let label_end = label_end.unwrap_or_default();
    let state = if state == FieldState::Default {
        crate::form_control::validation_class_state(&class)
    } else {
        state
    };
    crate::form_control::use_custom_field_state(crate::form_control::control_validation(
        state, &class,
    ));
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
    let tooltip = crate::use_tooltip(feedback_id.clone());
    let tooltip_anchor = crate::use_tooltip_anchor(tooltip.clone(), anchor_id.clone());
    let value_tooltip = crate::use_tooltip(value_tooltip_id.clone());
    let value_anchor = crate::use_tooltip_anchor(value_tooltip.clone(), thumb_id.clone());
    let describedby = value_anchor.describedby(if feedback_present { &feedback_id } else { "" });
    let refreshed_value_anchor = value_anchor.clone();
    use_effect(use_reactive!(|value, fraction| {
        // The controlled value moves the proxy without resizing it. Refresh
        // after Dioxus patches that position, rather than polling while idle.
        let _ = (value, fraction);
        refreshed_value_anchor.refresh();
    }));
    let pointer_enter = value_anchor.clone();
    let pointer_leave = value_anchor.clone();
    let pointer_down = value_anchor.clone();
    let focused_value = value_anchor.clone();
    let blurred_value = value_anchor.clone();
    let input_value = value_anchor.clone();
    let keyed_value = value_anchor.clone();
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
                tooltip: tooltip.clone(), feedback_id: feedback_id.clone(), required: false, state,
                feedback, invalid_alert, show_text_as_tooltip,
                max_length: None, length: 0,
                div { id: anchor_id, class: "ix-slider-container",
                    onmounted: tooltip_anchor.onmounted, onresize: tooltip_anchor.onresize, onvisible: tooltip_anchor.onvisible,
                    onmouseenter: tooltip_anchor.onmouseenter, onmouseleave: tooltip_anchor.onmouseleave,
                    onfocusin: tooltip_anchor.onfocusin, onfocusout: tooltip_anchor.onfocusout, onkeydown: tooltip_anchor.onkeydown,
                    div { class: "ix-slider-control",
                        div { class: "ix-slider-track",
                            div { id: thumb_id, class: "ix-slider-thumb-anchor", style: "left:calc({fraction} * (100% - var(--thumb-size)))", aria_hidden: true,
                                onmounted: value_anchor.onmounted, onresize: value_anchor.onresize, onvisible: value_anchor.onvisible,
                            }
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
                            id: id.clone(), r#type: "range", name, form,
                            min, max, step: step.max(f64::EPSILON), value: value.to_string(), disabled, tabindex: if disabled { -1 } else { 0 },
                            role: "slider", "aria-label": aria_label.unwrap_or_else(|| if label.is_empty() { "Slider".to_string() } else { label }),
                            onmounted: move |event| { if let Some(mount) = surface_focus_mount { mount.call(event.clone()); } onmounted.call(event); },
                            "aria-valuenow": value.to_string(), "aria-valuemin": min.to_string(), "aria-valuemax": max.to_string(),
                            "aria-invalid": (state == FieldState::Invalid).to_string(),
                            "aria-describedby": describedby,
                            list: has_markers.then_some("markers"),
                            style: "--value:{fraction};--trace-reference:{reference};--trace-start:{start};--trace-end:{end}",
                            oninput: move |event| if !disabled {
                                if show_tooltip { input_value.show(); }
                                if let Ok(value) = event.value().parse::<f64>() { onvaluechange.call(bounded_slider_value(value, min, max)); }
                            },
                            onfocus: move |_| { if show_tooltip && !disabled { focused_value.show(); } },
                            onblur: move |_| blurred_value.hide(),
                            onpointerenter: move |_| { if show_tooltip && !disabled { pointer_enter.show(); } },
                            onpointerleave: move |_| pointer_leave.hide(),
                            onpointerdown: move |_| { if show_tooltip && !disabled { pointer_down.show(); } },
                            onkeydown: move |event: KeyboardEvent| {
                                if show_tooltip && !disabled && matches!(event.key(), Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End | Key::PageUp | Key::PageDown) { keyed_value.show(); }
                                keyed_value.onkeydown.call(event);
                            },
                        }
                        if show_tooltip && !disabled {
                            crate::TooltipFor { id: value_tooltip_id, controller: value_tooltip, placement: crate::TooltipPlacement::Top,
                                show_delay: 0, hide_delay: 0, class: "ix-slider-value-tooltip", "{value}"
                            }
                        }
                    }
                    if has_endpoints {
                        div { class: "ix-slider-endpoints",
                            div {
                                if let Some(content) = label_start_content { {content} }
                                else { span { "{label_start}" } }
                            }
                            div {
                                if let Some(content) = label_end_content { {content} }
                                else { span { "{label_end}" } }
                            }
                        }
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
/// `id`, `class`, `aria_label` and `onmounted` apply to the existing group div;
/// child controls keep their own labels and native mounted ownership.
#[component]
pub fn RangeField(
    children: Element,
    #[props(default)] hide_arrow: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
) -> Element {
    crate::datetime::provide_range_navigation();
    let mut classes = if hide_arrow {
        "ix-range-field ix-range-no-arrow"
    } else {
        "ix-range-field"
    }
    .to_string();
    if !class.is_empty() {
        classes.push(' ');
        classes.push_str(&class);
    }
    rsx! { div { id,class:classes,role:"group",aria_label,onmounted,{children} } }
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
    focused: Signal<bool>,
    input: Rc<std::cell::RefCell<Option<Rc<MountedData>>>>,
}

/// Rust descriptor for the native combobox input. It is scoped to this mount:
/// retaining it after unmount returns [`SelectMethodError::NotMounted`].
#[derive(Clone)]
pub struct SelectNativeInput {
    id: String,
    alive: Rc<std::cell::Cell<bool>>,
    focused: Signal<bool>,
    node: Rc<MountedData>,
}
impl SelectNativeInput {
    /// DOM id for accessible labels and associations. Use [`Self::focus`] to
    /// focus this registered mounted input directly.
    pub fn id(&self) -> &str {
        &self.id
    }
    pub async fn focus(&self) -> Result<(), SelectMethodError> {
        if !self.alive.get() {
            return Err(SelectMethodError::NotMounted);
        }
        let result = self.node.set_focus(true).await;
        if !self.alive.get() {
            return Err(SelectMethodError::NotMounted);
        }
        if let Err(error) = result {
            // Pinned desktop alpha.2 queries setFocus as bool, but its native
            // interpreter returns void. The typed focus/blur handlers already
            // maintain this state; only current confirmed focus can normalize
            // that operation error. No second focus command or DOM query runs.
            if !self.focused.try_peek().is_ok_and(|focused| *focused) {
                return Err(SelectMethodError::Dom(error.to_string()));
            }
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
            focused: binding.focused,
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
        let result = crate::form_control::read_associated_form(
            &binding.id,
            crate::form_control::NativeFormControl::Select,
        )
        .await
        .map_err(SelectMethodError::Dom)?;
        if !binding.alive.get() {
            return Err(SelectMethodError::NotMounted);
        }
        Ok(result.map(|form| SelectFormElement {
            id: form.id,
            name: form.name,
            method: form.method,
            action: form.action,
            no_validate: form.no_validate,
        }))
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

#[derive(Clone, Default)]
struct MountedAnchor(Option<Rc<MountedData>>);

impl PartialEq for MountedAnchor {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SelectPopupMetrics {
    anchor_x: f64,
    anchor_y: f64,
    anchor_width: f64,
    anchor_height: f64,
    popup_width: f64,
    popup_height: f64,
    content_height: f64,
    viewport_left: f64,
    viewport_top: f64,
    viewport_width: f64,
    viewport_height: f64,
    rtl: bool,
}

impl SelectPopupMetrics {
    fn approx_eq(self, other: Self) -> bool {
        const EPSILON: f64 = 0.25;
        self.rtl == other.rtl
            && (self.anchor_x - other.anchor_x).abs() <= EPSILON
            && (self.anchor_y - other.anchor_y).abs() <= EPSILON
            && (self.anchor_width - other.anchor_width).abs() <= EPSILON
            && (self.anchor_height - other.anchor_height).abs() <= EPSILON
            && (self.popup_width - other.popup_width).abs() <= EPSILON
            && (self.popup_height - other.popup_height).abs() <= EPSILON
            && (self.content_height - other.content_height).abs() <= EPSILON
            && (self.viewport_left - other.viewport_left).abs() <= EPSILON
            && (self.viewport_top - other.viewport_top).abs() <= EPSILON
            && (self.viewport_width - other.viewport_width).abs() <= EPSILON
            && (self.viewport_height - other.viewport_height).abs() <= EPSILON
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectPopupLayout {
    style: String,
    placement: &'static str,
    direction: &'static str,
}

fn place_select_popup(
    metrics: Option<SelectPopupMetrics>,
    dropdown_width: Option<&str>,
    dropdown_max_width: Option<&str>,
) -> SelectPopupLayout {
    const SOURCE_CAP: &str =
        "calc(var(--ix-dropdown-overflow--max-height) - var(--ix-dropdown--offset))";
    let Some(metrics) = metrics else {
        return SelectPopupLayout {
            style: format!(
                "width:{};max-width:{};max-height:{SOURCE_CAP}",
                dropdown_width.unwrap_or("100%"),
                dropdown_max_width.unwrap_or("100%")
            ),
            placement: "bottom-start",
            direction: "ltr",
        };
    };
    let room_width = metrics.viewport_width.max(0.0);
    let min_width = metrics.anchor_width.max(0.0).min(room_width);
    let width_css = match dropdown_width {
        Some(width) => format!("min({width}, {room_width}px)"),
        None => format!("{min_width}px"),
    };
    let max_width_css = match dropdown_max_width {
        Some(max_width) => format!("min({max_width}, {room_width}px)"),
        None => "100%".to_string(),
    };
    // CSS resolves authored lengths (including %, rem, calc and tokens).
    // Position against the actual mounted width rather than parsing CSS in Rust.
    let popup_width = metrics.popup_width.max(0.0).min(room_width);
    let desired = metrics.content_height.max(metrics.popup_height).max(0.0);
    let anchor_bottom = metrics.anchor_y + metrics.anchor_height;
    let anchor_right = metrics.anchor_x + metrics.anchor_width;
    let below = (metrics.viewport_top + metrics.viewport_height - anchor_bottom).max(0.0);
    let above = (metrics.anchor_y - metrics.viewport_top).max(0.0);
    let flipped = below < desired && above > below;
    let room = if flipped { above } else { below };
    let start = if metrics.rtl {
        anchor_right - popup_width
    } else {
        metrics.anchor_x
    };
    let left = start
        .min(metrics.viewport_left + room_width - popup_width)
        .max(metrics.viewport_left);
    let top = if flipped {
        metrics.anchor_y - metrics.popup_height.max(0.0).min(room)
    } else {
        anchor_bottom
    };
    SelectPopupLayout {
        style: format!(
            "min-width:{min_width}px;width:{width_css};max-width:{max_width_css};max-height:min({room}px, {SOURCE_CAP});left:{left}px;top:{top}px"
        ),
        placement: if flipped { "top-start" } else { "bottom-start" },
        direction: if metrics.rtl { "rtl" } else { "ltr" },
    }
}

// The hidden control probe keeps an 8px border box. Its logical padding maps to
// the left in LTR (8px content) and right in RTL (4px content), so Dioxus's normal
// content-box resize event reports inherited direction, including CSS changes.
// Hidden controls can report zero; retain the last direction for unresolved facts.
fn select_rtl_from_probe_width(width: f64) -> Option<bool> {
    const EPSILON: f64 = 0.25;
    if (width - 8.0).abs() <= EPSILON {
        Some(false)
    } else if (width - 4.0).abs() <= EPSILON {
        Some(true)
    } else {
        None
    }
}

#[component]
fn SelectOptionsSurface(
    rtl: bool,
    input_id: String,
    anchor: MountedAnchor,
    content_revision: u64,
    enable_top_layer: bool,
    dropdown_width: Option<String>,
    dropdown_max_width: Option<String>,
    onnativeerror: EventHandler<crate::NativeSurfaceError>,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    let surface_id = field_id(None);
    let viewport = crate::document_viewport::use_document_viewport(true);
    let mut native = use_signal(|| enable_top_layer);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut metrics = use_signal(|| None::<SelectPopupMetrics>);
    let alive = use_hook(|| Rc::new(std::cell::Cell::new(true)));
    let generation = use_hook(|| Rc::new(std::cell::Cell::new(0_u64)));
    let lifetime = alive.clone();
    use_drop(move || {
        lifetime.set(false);
    });
    let anchor_slot = use_hook(|| Rc::new(std::cell::RefCell::new(None::<Rc<MountedData>>)));
    *anchor_slot.borrow_mut() = anchor.0.clone();
    let measure_anchor = anchor_slot.clone();
    let measure_alive = alive.clone();
    let measure_generation = generation.clone();
    let measure_viewport = viewport.clone();
    let measure = use_callback(move |()| {
        let Some(surface) = mounted.peek().clone() else {
            return;
        };
        let Some(anchor_node) = measure_anchor.borrow().clone() else {
            return;
        };
        let Some(viewport) = measure_viewport.facts() else {
            return;
        };
        measure_generation.set(measure_generation.get().wrapping_add(1));
        let revision = measure_generation.get();
        let generation = measure_generation.clone();
        let alive = measure_alive.clone();
        spawn(async move {
            let (Ok(anchor_rect), Ok(popup_rect)) = (
                anchor_node.get_client_rect().await,
                surface.get_client_rect().await,
            ) else {
                return;
            };
            let Ok(content) = surface.get_scroll_size().await else {
                return;
            };
            if !alive.get() || generation.get() != revision {
                return;
            }
            let next = SelectPopupMetrics {
                anchor_x: anchor_rect.origin.x,
                anchor_y: anchor_rect.origin.y,
                anchor_width: anchor_rect.size.width,
                anchor_height: anchor_rect.size.height,
                popup_width: popup_rect.size.width,
                popup_height: popup_rect.size.height,
                content_height: content.height,
                viewport_left: viewport.visual_left,
                viewport_top: viewport.visual_top,
                viewport_width: viewport.visual_width,
                viewport_height: viewport.visual_height,
                rtl,
            };
            if !metrics
                .peek()
                .is_some_and(|current| current.approx_eq(next))
            {
                metrics.set(Some(next));
            }
        });
    });
    use_effect(use_reactive!(
        |anchor, rtl, dropdown_width, dropdown_max_width, content_revision| {
            let _ = (
                anchor,
                rtl,
                dropdown_width,
                dropdown_max_width,
                content_revision,
            );
            measure.call(());
        }
    ));
    use_effect(move || {
        let _ = viewport.layout_revision();
        measure.call(());
    });
    let layout = place_select_popup(
        metrics(),
        dropdown_width.as_deref(),
        dropdown_max_width.as_deref(),
    );
    let mounted_id = surface_id.clone();
    rsx! {
        if !native() {
            button { r#type: "button", class: "ix-select-backdrop", tabindex: "-1", "aria-label": "Close selection list", onclick: move |_| onclose.call(()) }
        }
        div { id: surface_id, class: "ix-select-popup", style: layout.style,
            "popover": native().then_some("auto"), "data-top-layer": native().to_string(),
            "data-placement": layout.placement, "data-direction": layout.direction,
            "data-requested-width": dropdown_width.clone(), "data-requested-max-width": dropdown_max_width.clone(),
            onmounted: move |event| {
                mounted.set(Some(event.data()));
                if !enable_top_layer {
                    measure.call(());
                    return;
                }
                let alive = alive.clone();
                let id = mounted_id.clone();
                let input = input_id.clone();
                spawn(async move {
                    let presentation =
                        crate::native_surface::present(&id, true, None, Some(&input)).await;
                    if alive.get() {
                        if let Err(error) = presentation {
                            native.set(false);
                            onnativeerror.call(error);
                        }
                        measure.call(());
                    }
                });
            },
            onresize: move |_| measure.call(()),
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

#[derive(serde::Serialize)]
struct ChipMeasurements {
    values: Vec<String>,
    widths: Vec<f64>,
    badges: Vec<f64>,
    available: f64,
    gap: f64,
    chips_width: f64,
}

// Dioxus owns every measuring element and resize subscription. Adjacent
// positions include the real CSS margins without computed-style DOM queries.
// A zero-width final marker also includes the final chip's trailing margin.
#[component]
fn SelectChipMeasurementRail(
    chips: Vec<(String, String)>,
    disabled: bool,
    width: f64,
    available: f64,
    remove_label: String,
    onmeasure: EventHandler<ChipMeasurements>,
) -> Element {
    let mut nodes = use_signal(std::collections::BTreeMap::<String, Rc<MountedData>>::new);
    let mut end = use_signal(|| None::<Rc<MountedData>>);
    let generation = use_hook(|| Rc::new(std::cell::Cell::new(0_u64)));
    let alive = use_hook(|| Rc::new(std::cell::Cell::new(true)));
    let drop_alive = alive.clone();
    use_drop(move || drop_alive.set(false));
    let measure_chips = chips.clone();
    let measure = use_callback(move |()| {
        generation.set(generation.get().wrapping_add(1));
        let revision = generation.get();
        let generation = generation.clone();
        let alive = alive.clone();
        let values: Vec<_> = measure_chips
            .iter()
            .map(|(value, _)| value.clone())
            .collect();
        let keys = values
            .iter()
            .map(|value| format!("chip:{value}"))
            .chain((1..=values.len()).map(|count| format!("badge:{count}")));
        let handles: Option<Vec<_>> = keys.map(|key| nodes.peek().get(&key).cloned()).collect();
        let marker = end.peek().clone();
        spawn(async move {
            let (Some(handles), Some(marker)) = (handles, marker) else {
                return;
            };
            let mut positions = Vec::with_capacity(handles.len() + 1);
            for node in handles {
                let Ok(bounds) = node.get_client_rect().await else {
                    return;
                };
                positions.push(bounds.origin.x);
            }
            let Ok(bounds) = marker.get_client_rect().await else {
                return;
            };
            positions.push(bounds.origin.x);
            if !alive.get() || generation.get() != revision {
                return;
            }
            let extents: Vec<_> = positions
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).max(0.0))
                .collect();
            let (widths, badges) = extents.split_at(values.len());
            onmeasure.call(ChipMeasurements {
                values,
                widths: widths.to_vec(),
                badges: badges.to_vec(),
                available,
                gap: 0.0,
                chips_width: width,
            });
        });
    });
    use_effect(use_reactive!(|chips, disabled, width, available| {
        let live: std::collections::BTreeSet<_> = chips
            .iter()
            .map(|(value, _)| format!("chip:{value}"))
            .chain((1..=chips.len()).map(|count| format!("badge:{count}")))
            .collect();
        nodes.write().retain(|key, _| live.contains(key));
        let _ = (disabled, width, available);
        measure.call(());
    }));
    let count = chips.len();
    rsx! {
        div { class: "ix-select-measure", style: "width:{width}px", "aria-hidden": "true", inert: true,
            onresize: move |_| measure.call(()),
            for (value, text) in chips {
                {
                    let node_key = format!("chip:{value}");
                    rsx! { SelectChipPresentation { key: "{value}", text: text.clone(), disabled,
                        measure_value: Some(value),
                        onmounted: move |event: MountedEvent| { nodes.write().insert(node_key.clone(), event.data()); measure.call(()); },
                        onresize: move |_| measure.call(()),
                        close: rsx! { crate::IconButton { icon: "close-small", label: select_remove_label(&remove_label, &text),
                            variant: crate::ButtonVariant::Tertiary, size: crate::ButtonIconSize::Sixteen, oval: true, class: "ix-select-chip-remove", tabindex: -1 } }
                    } }
                }
            }
            for count in 1..=count {
                button { key: "badge:{count}", r#type: "button", class: "ix-select-chip ix-select-overflow ix-select-chip-overflow",
                    "data-chip-disabled": disabled.to_string(), "data-hide-close": "true", "data-measure-badge": count.to_string(), tabindex: "-1",
                    onmounted: move |event| { nodes.write().insert(format!("badge:{count}"), event.data()); measure.call(()); },
                    onresize: move |_| measure.call(()),
                    span { class: "ix-select-chip-label", "+{count}" }
                }
            }
            span { style: "display:block;flex:0 0 0px;width:0;height:0;margin-left:var(--ix-select-chip-horizontal-margin)", onmounted: move |event| { end.set(Some(event.data())); measure.call(()); } }
        }
    }
}

#[component]
fn SelectChipPresentation(
    text: String,
    disabled: bool,
    measure_value: Option<String>,
    close: Element,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
    #[props(default)] onresize: EventHandler<ResizeEvent>,
) -> Element {
    rsx! { span { class: "ix-select-chip", "data-chip-disabled": disabled.to_string(),
        "data-hide-close": disabled.to_string(), "data-measure-value": measure_value,
        onmounted, onresize,
        span { class: "ix-select-chip-label", title: text.clone(), "{text}" }
        if !disabled { span { class: "ix-select-chip-close-host", {close} } }
    } }
}

fn select_visual_chip(
    text: String,
    disabled: bool,
    measure_value: Option<String>,
    close: Element,
) -> Element {
    rsx! { SelectChipPresentation { text, disabled, measure_value, close } }
}

// Every keyed option owns its mounted handle. Dioxus disposes it when a
// filtered option disappears and retains it when the same value is reordered.
// No DOM lookup or separate option registry is needed.
fn use_select_option_scroll(active: bool) -> Signal<Option<Rc<MountedData>>> {
    let mounted = use_signal(|| None::<Rc<MountedData>>);
    use_effect(use_reactive((&active,), move |(active,)| {
        let node = mounted();
        if active {
            if let Some(node) = node {
                spawn(async move {
                    let _ = node
                        .scroll_to_with_options(dioxus::html::ScrollToOptions {
                            behavior: dioxus::html::ScrollBehavior::Instant,
                            vertical: dioxus::html::ScrollLogicalPosition::Nearest,
                            horizontal: dioxus::html::ScrollLogicalPosition::Nearest,
                        })
                        .await;
                });
            }
        }
    }));
    mounted
}

#[component]
fn SelectAddPresentation(
    id: String,
    label: String,
    aria_label: String,
    active: bool,
    onmouseenter: EventHandler<MouseEvent>,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut mounted = use_select_option_scroll(active);
    rsx! {
        button { id, r#type: "button", role: "option", tabindex: "-1", "aria-selected": "false",
            "aria-label": aria_label,
            class: if active { "ix-select-option ix-select-add ix-select-active" } else { "ix-select-option ix-select-add" },
            onmounted: move |event| mounted.set(Some(event.data())),
            onmousedown: move |event| event.prevent_default(), onmouseenter, onclick,
            crate::Icon { name: "plus", size: 16 } "{label}"
        }
    }
}

#[component]
fn SelectOptionPresentation(
    id: String,
    #[props(default)] value: Option<String>,
    label: String,
    selected: bool,
    disabled: bool,
    active: bool,
    #[props(default)] onmouseenter: EventHandler<MouseEvent>,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut mounted = use_select_option_scroll(active);
    rsx! { span { class: "ix-select-item-host", "data-disabled": disabled.to_string(),
        div { class: "ix-select-option-host", "data-selected": selected.to_string(), "data-disabled": disabled.to_string(),
            button { id, value, class: if active { "ix-select-option ix-select-active" } else { "ix-select-option" },
                r#type: "button", role: "option", tabindex: "-1", disabled,
                "aria-selected": selected.to_string(),
                onmounted: move |event| mounted.set(Some(event.data())),
                onmousedown: move |event| event.prevent_default(), onmouseenter, onclick,
                span { class: "ix-select-choice", "aria-hidden": "true",
                    if selected { crate::Icon { name: "single-check", size: 16 } }
                }
                span { class: "ix-select-option-label", "{label}" }
                span { class: "ix-select-option-end" }
            }
        }
    } }
}

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
    #[props(default)] class: String,
    #[props(default)] id: Option<String>,
    #[props(default)] name: Option<String>,
    /// Explicit native form owner; omission keeps normal ancestor association.
    #[props(default)]
    form: Option<String>,
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
    /// Receives the visible native combobox after internal mounted-handle registration.
    /// Use its Dioxus handle for focus, measurement and scrolling.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
) -> Element {
    let id = field_id(id);
    let surface_focus_mount = use_surface_focus_mount(id.clone());
    let list_id = format!("{id}-list");
    let control_id = format!("{id}-control");
    let help_id = format!("{id}-help");
    let mut open = use_signal(|| false);
    let mut rtl = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let mut measured_width = use_signal(|| 0.0_f64);
    let mut measured_available = use_signal(|| 0.0_f64);
    let mut chips_ref = use_signal(|| None::<Rc<MountedData>>);
    let mut reservation_ref = use_signal(|| None::<Rc<MountedData>>);
    let mut clear_ref = use_signal(|| None::<Rc<MountedData>>);
    let layout_generation = use_hook(|| Rc::new(std::cell::Cell::new(0_u64)));
    let mut measured_revision = use_signal(|| 0_u64);
    let mut measured_facts = use_signal(String::new);
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
    let mut control_ref = use_signal(|| None::<Rc<MountedData>>);
    let mut control_revision = use_signal(|| 0_u64);
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
                alive: alive.clone(),
                focused,
                input: method_input.clone(),
            },
        );
    }
    let state = if state == FieldState::Default {
        crate::form_control::validation_class_state(&class)
    } else {
        state
    };
    let state = select_field_state(state, required, invalid_attempted(), !selected.is_empty());
    crate::form_control::use_custom_field_state(crate::form_control::control_validation(
        state, &class,
    ));
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
    let select_text_id = id.clone();
    let focus_alive = alive.clone();
    use_effect(move || {
        let is_open = open();
        if is_open {
            let node = trigger_ref.peek().clone();
            let alive = focus_alive.clone();
            let id = select_text_id.clone();
            let select_text = !*query_dirty.peek();
            spawn(async move {
                if !alive.get() || !*open.peek() {
                    return;
                }
                if let Some(node) = node {
                    // Dioxus owns focus. A close needs no restoration: options
                    // prevent mousedown's focus transfer, and keyboard actions
                    // originate from the combobox. Outside clicks/Tab therefore
                    // cannot be overridden by an older close-focus job.
                    let _ = node.set_focus(true).await;
                    if !alive.get() || !*open.peek() || !select_text {
                        return;
                    }
                    #[cfg(any(feature = "web", feature = "desktop"))]
                    {
                        let _ = crate::input_selection::select_focused_input_text(&id, true).await;
                    }
                    #[cfg(not(any(feature = "web", feature = "desktop")))]
                    let _ = id;
                }
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
    let root_id = format!("{id}-select-root");
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
    let input_placeholder = if disabled || readonly {
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
    let invalid_alert =
        state == FieldState::Invalid && invalid_text.as_ref().is_some_and(|text| !text.is_empty());
    let visual_feedback = field_feedback(
        state,
        helper_text,
        invalid_text,
        valid_text,
        info_text,
        warning_text,
    );
    let tooltip = crate::use_tooltip(help_id.clone());
    let tooltip_anchor = crate::use_tooltip_anchor(tooltip.clone(), control_id.clone());
    let describedby = visual_feedback.is_some().then(|| help_id.clone());
    let focus_outline = focused() && active_index == usize::MAX;
    let has_clear = allow_clear && (!selected.is_empty() || !query_text.is_empty()) && !read_only;
    let layout_alive = alive.clone();
    let measure_layout = use_callback(move |()| {
        layout_generation.set(layout_generation.get().wrapping_add(1));
        let revision = layout_generation.get();
        let generation = layout_generation.clone();
        let alive = layout_alive.clone();
        let chips = chips_ref.peek().clone();
        let reservation = reservation_ref.peek().clone();
        let clear = has_clear.then(|| clear_ref.peek().clone()).flatten();
        spawn(async move {
            let (Some(chips), Some(reservation)) = (chips, reservation) else {
                return;
            };
            let (Ok(chips), Ok(reservation)) = (
                chips.get_client_rect().await,
                reservation.get_client_rect().await,
            ) else {
                return;
            };
            let clear_width = if let Some(clear) = clear {
                let Ok(bounds) = clear.get_client_rect().await else {
                    return;
                };
                bounds.size.width
            } else {
                0.0
            };
            if !alive.get() || generation.get() != revision {
                return;
            }
            let width = chips.size.width;
            let available = (width - reservation.size.width - clear_width).max(0.0);
            if *measured_width.peek() != width {
                measured_width.set(width);
            }
            if *measured_available.peek() != available {
                measured_available.set(available);
            }
        });
    });
    use_effect(use_reactive!(|has_clear, read_only, multiple| {
        let _ = (read_only, multiple);
        if !has_clear {
            clear_ref.set(None);
        }
        measure_layout.call(());
    }));
    let measurement_values = selected.clone();
    let accept_measurement = use_callback(move |measured: ChipMeasurements| {
        if measured.values != measurement_values {
            return;
        }
        let facts = serde_json::to_string(&measured).expect("Dioxus layout serialization");
        if *measured_facts.peek() == facts {
            return;
        }
        let count = fitting_chips(
            &measured.widths,
            &measured.badges,
            measured.available,
            measured.gap,
        );
        let hidden = measured.values.into_iter().skip(count).collect::<Vec<_>>();
        if *hidden_values.peek() != hidden {
            hidden_values.set(hidden);
        }
        measured_facts.set(facts);
        measured_revision += 1;
    });
    rsx! {
        div { id: root_id, class: "ix-select ix-select-visual ix-text-field-native {state.class()} {class}",
            class: if focus_outline { "show-focus-outline" },
            "data-disabled": disabled.to_string(), "data-readonly": readonly.to_string(),
            "data-select-measured-width": measured_width().to_string(),
            "data-select-measured-revision": measured_revision().to_string(),
            "data-select-measured-facts": measured_facts(),
            "data-feedback-tooltip": tooltip_available.to_string(),
            onfocusout: move |_| {
                // FocusData in pinned dioxus-html carries neither relatedTarget
                // nor target containment. One read distinguishes internal focus
                // movement from leaving the whole control (web and desktop).
                let id = blur_root_id.clone();
                spawn(async move {
                    if matches!(crate::focus_containment::read(&id, None).await, Ok(facts) if !facts.inside) {
                        open.set(false); overflow_open.set(false);
                        query.set(String::new()); query_dirty.set(false); active.set(usize::MAX);
                    }
                });
            },
            onkeydown: move |event| {
                if read_only { return; }
                match event.key() {
                    Key::Escape => { event.prevent_default(); open.set(false); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); }
                    Key::Tab => { open.set(false); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); }
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
            TextFieldPresentation { label, control_id: id.clone(), tooltip: tooltip.clone(),
                feedback_id: help_id.clone(), required, state, invalid_alert,
                feedback: if tooltip_available { None } else { visual_feedback.clone() },
                show_text_as_tooltip: false, max_length: None, length: 0,
                // Blur feedback in a preceding field can move the input between
                // press and release. Its presentation remains the click target.
                onpresentationclick: move |_| if focused() && !read_only {
                    overflow_open.set(false);
                    if !open() { active.set(usize::MAX); open.set(true); }
                },
            div { id: control_id.clone(), class: "ix-select-control", "data-readonly": readonly.to_string(), "data-disabled": disabled.to_string(), "data-open": open().to_string(),
                onmounted: move |event| { control_ref.set(Some(event.data())); control_revision += 1; tooltip_anchor.onmounted.call(event); },
                onresize: move |event| { control_revision += 1; measure_layout.call(()); tooltip_anchor.onresize.call(event); },
                onvisible: tooltip_anchor.onvisible,
                onmouseenter: tooltip_anchor.onmouseenter, onmouseleave: tooltip_anchor.onmouseleave,
                onfocusin: tooltip_anchor.onfocusin, onfocusout: tooltip_anchor.onfocusout, onkeydown: tooltip_anchor.onkeydown,
                span { class: "ix-select-direction-probe", "aria-hidden": "true",
                    onresize: move |event| {
                        if let Ok(size) = event.get_content_box_size()
                            && let Some(next) = select_rtl_from_probe_width(size.width)
                            && next != *rtl.peek()
                        {
                            rtl.set(next);
                        }
                    },
                }
                div { class: "ix-select-input-container",
                    div { class: "ix-select-values ix-select-chips",
                        onmounted: move |event| { chips_ref.set(Some(event.data())); measure_layout.call(()); },
                        onresize: move |_| measure_layout.call(()),
                      if multiple && !selected.is_empty() {
                        if collapsed_all {
                            {select_visual_chip(format!("{all_selected_text} ({})", selected.len()), read_only, None, rsx! {
                                crate::IconButton { icon: "close-small", label: select_remove_label(&remove_selected_item_text, &all_selected_text),
                                    variant: crate::ButtonVariant::Tertiary, size: crate::ButtonIconSize::Sixteen, oval: true, class: "ix-select-chip-remove",
                                    onkeydown: move |event: KeyboardEvent| event.stop_propagation(),
                                    onclick: move |event: MouseEvent| { event.stop_propagation(); publish_selection(vec![], onvaluechange, onvalueschange); }
                                }
                            })}
                        }
                        for (value, text) in selected_chips.into_iter().filter(|(value, _)| !collapsed_all && !hidden_values.read().contains(value)) {
                            {
                                let next = selected.iter().filter(|item| **item != value).cloned().collect::<Vec<_>>();
                                rsx! { Fragment { key: "{value}",
                                  {select_visual_chip(text.clone(), read_only, None, rsx! {
                                    crate::IconButton { icon: "close-small", label: select_remove_label(&remove_selected_item_text, &text),
                                        variant: crate::ButtonVariant::Tertiary, size: crate::ButtonIconSize::Sixteen, oval: true, class: "ix-select-chip-remove",
                                        onkeydown: move |event: KeyboardEvent| if matches!(event.key(), Key::Enter | Key::Character(_)) { event.stop_propagation(); },
                                        onclick: move |event: MouseEvent| { event.stop_propagation(); publish_selection(next.clone(), onvaluechange, onvalueschange); let mounted = trigger_ref.peek().clone(); spawn(async move { if let Some(mounted) = mounted { let _ = mounted.set_focus(true).await; } }); }
                                    }
                                  })}
                                } }
                            }
                        }
                        if !collapsed_all && !hidden_chips.is_empty() {
                            button { r#type: "button", class: "ix-select-chip ix-select-overflow ix-select-chip-overflow", disabled: read_only, "data-chip-disabled": read_only.to_string(), "data-hide-close": "true",
                                "aria-label": select_more_label(&more_items_text, hidden_chips.len()), "aria-haspopup": "listbox", "aria-expanded": overflow_open().to_string(), "aria-controls": overflow_id.clone(),
                                onmounted: move |event| overflow_ref.set(Some(event.data())),
                                onclick: move |event| { event.stop_propagation(); open.set(false); overflow_open.set(!overflow_open()); },
                                onkeydown: move |event| { event.stop_propagation(); if matches!(event.key(), Key::Enter | Key::ArrowDown) || event.key() == Key::Character(" ".into()) { event.prevent_default(); open.set(false); overflow_open.set(true); } },
                                span { class: "ix-select-chip-label", "+{hidden_chips.len()}" }
                            }
                        }
                      }
                div { class: "ix-select-trigger-rail",
                input { id: id.clone(), form:form.clone(), r#type: "text", class: if multiple && !selected.is_empty() { "ix-select-trigger ix-select-trigger-multiple" } else { "ix-select-trigger" }, disabled, readonly,
                    value: input_text, placeholder: input_placeholder, autocomplete: "off",
                    role: "combobox", "aria-haspopup": "listbox", "aria-expanded": open().to_string(),
                    "aria-autocomplete": "list",
                    "aria-controls": list_id.clone(), "aria-readonly": readonly.to_string(),
                    "aria-required": required.to_string(), "aria-invalid": (state == FieldState::Invalid).to_string(),
                    "aria-describedby": describedby,
                    "aria-activedescendant": active_descendant,
                    onmounted: move |event| { let node = event.data(); *method_input.borrow_mut() = Some(node.clone()); trigger_ref.set(Some(node)); if let Some(mount) = surface_focus_mount { mount.call(event.clone()); } onmounted.call(event); },
                    oninput: move |event| if !read_only { let text = event.value(); query.set(text.clone()); query_dirty.set(true); active.set(usize::MAX); overflow_open.set(false); open.set(true); oninputchange.call(text); },
                    onfocus: move |_| focused.set(true),
                    onblur: move |_| {
                        focused.set(false);
                        touched.set(true);
                        onixblur.call(());
                        if required && !read_only {
                            let id = validation_id.clone();
                            spawn(async move {
                                // Follow the native owner, including an external `form`
                                // association. noValidate also clears old automatic feedback.
                                if let Ok(form) = crate::form_control::read_associated_form(
                                    &id,
                                    crate::form_control::NativeFormControl::Select,
                                ).await {
                                    invalid_attempted.set(!form.is_some_and(|form| form.no_validate));
                                }
                            });
                        }
                    },
                }
                if allow_clear && (!clear_selected.is_empty() || !query_text.is_empty()) && !read_only {
                    span { class: "ix-select-action-host ix-select-clear-host",
                        onmounted: move |event| { clear_ref.set(Some(event.data())); measure_layout.call(()); },
                        onresize: move |_| measure_layout.call(()),
                    crate::IconButton { icon: "clear", label: aria_label_clear, class: "ix-select-clear", variant: crate::ButtonVariant::SubtleTertiary, size: crate::ButtonIconSize::Sixteen, oval: true,
                        onmousedown: move |event: MouseEvent| event.prevent_default(),
                        onkeydown: move |event: KeyboardEvent| if matches!(event.key(), Key::Enter | Key::Character(_)) { event.stop_propagation(); },
                        onclick: move |event: MouseEvent| { event.stop_propagation(); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); open.set(false); publish_selection(vec![], onvaluechange, onvalueschange); let node = trigger_ref.peek().clone(); spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } }); },
                    } }
                }
                if !read_only {
                    span { class: "ix-select-action-host ix-select-chevron-host", "data-open": open().to_string(),
                    crate::IconButton { class: "ix-select-chevron", tabindex: -1, aria_hidden: true, variant: crate::ButtonVariant::SubtleTertiary,
                        icon: if open() { "chevron-up-small" } else { "chevron-down-small" },
                        label: if open() { "Close select dropdown" } else { "Open select dropdown" },
                        onmousedown: move |event: MouseEvent| event.prevent_default(),
                        onclick: move |event: MouseEvent| { event.stop_propagation(); overflow_open.set(false); if open() { query.set(String::new()); query_dirty.set(false); } active.set(usize::MAX); open.set(!open()); },
                    } }
                }
                } } }
                span { class: "ix-select-trigger-size", "aria-hidden": "true",
                    onmounted: move |event| { reservation_ref.set(Some(event.data())); measure_layout.call(()); },
                    onresize: move |_| measure_layout.call(()),
                }
            }
            }
            if multiple && !selected.is_empty() {
                SelectChipMeasurementRail { chips: measurement_chips, disabled: read_only,
                    width: measured_width(), available: measured_available(),
                    remove_label: remove_selected_item_text.clone(), onmeasure: accept_measurement,
                }
            }
            if overflow_open() && !hidden_chips.is_empty() && !read_only {
                button { r#type: "button", class: "ix-select-backdrop", tabindex: "-1", "aria-label": "Close hidden selected items", onclick: move |_| overflow_open.set(false) }
                div { id: overflow_id, class: "ix-select-popup ix-select-overflow-popup", role: "listbox", "aria-label": "Hidden selected items",
                    for (index, (value, text)) in hidden_chips.iter().cloned().enumerate() {
                        {
                            let remaining = selected.iter().filter(|item| **item != value).cloned().collect::<Vec<_>>();
                            let count = hidden_chips.len();
                            rsx! { span { key: "{value}", class: "ix-select-chip", "data-chip-disabled": "false", "data-hide-close": "false", role: "option", "aria-selected": "true",
                                span { class: "ix-select-chip-label", "{text}" }
                                span { class: "ix-select-chip-close-host",
                                crate::IconButton { class: "ix-select-chip-remove", icon: "close-small", label: select_remove_label(&remove_selected_item_text, &text), variant: crate::ButtonVariant::Tertiary, size: crate::ButtonIconSize::Sixteen, oval: true,
                                    onmounted: move |event: MountedEvent| { let node = event.data(); { let mut refs = hidden_refs.write(); if refs.len() <= index { refs.resize(index + 1, None); } refs[index] = Some(node.clone()); } if index == 0 { spawn(async move { let _ = node.set_focus(true).await; }); } },
                                    onclick: move |event: MouseEvent| { event.stop_propagation(); overflow_open.set(false); publish_selection(remaining.clone(), onvaluechange, onvalueschange); let node = trigger_ref.peek().clone(); spawn(async move { if let Some(node) = node { let _ = node.set_focus(true).await; } }); },
                                    onkeydown: move |event: KeyboardEvent| {
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
                                } }
                            } }
                        }
                    }
                }
            }
            select { class: "ix-select-native", name, form, multiple, required: required && !readonly, disabled, tabindex: "-1", "aria-hidden": "true",
                oninvalid: move |event| { event.prevent_default(); if !read_only { invalid_attempted.set(true); open.set(true); } },
                if !multiple { option { value: "", selected: selected.is_empty(), "" } }
                for option in &all_options {
                    option { value: option.value.clone(), disabled: option.disabled, selected: selected.contains(&option.value), "{option.label}" }
                }
            }
            if open() && !read_only {
                SelectOptionsSurface { rtl: rtl(), input_id: id.clone(),
                    anchor: MountedAnchor(control_ref()),
                    content_revision: (visible.len() as u64)
                        ^ (u64::from(can_add) << 32)
                        ^ (u64::from(no_matches) << 33)
                        ^ (control_revision() << 34),
                    enable_top_layer,
                    dropdown_width, dropdown_max_width, onnativeerror,
                    onclose: move |_| { open.set(false); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); },
                    div { class: "ix-select-list-header", hidden: hide_list_header || no_matches, "aria-hidden": "true", title: list_header.clone(), "{list_header}" }
                    div { id: list_id.clone(), role: "listbox", "aria-multiselectable": multiple.to_string(), class: "ix-select-list",
                        for (index, option) in visible.iter().cloned().enumerate() {
                            {
                                let next_values = update_selection(selected.clone(), option.value.clone(), multiple);
                                let is_selected = selected.contains(&option.value);
                                rsx! {
                                    SelectOptionPresentation { key: "{option.value}", id: format!("{list_id}-{index}"), value: option.value.clone(),
                                        label: option.label, selected: is_selected, disabled: option.disabled, active: index == active_index,
                                        onmouseenter: move |_| if !option.disabled { active.set(index); },
                                        onclick: move |event: MouseEvent| { event.stop_propagation(); publish_selection(next_values.clone(), onvaluechange, onvalueschange); query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); if !multiple { open.set(false); } }
                                    }
                                }
                            }
                        }
                        if no_matches && !editable { div { class: "ix-select-empty", "{no_matches_text}" } }
                        if can_add {
                            SelectAddPresentation { id: format!("{list_id}-{}", visible.len()),
                                aria_label: format!("{aria_label_add}: {}", query_text.trim()),
                                label: query_text.trim().to_string(), active: active_index == visible.len(),
                                onmouseenter: move |_| active.set(add_index),
                                onclick: move |_| {
                                let text = query.read().trim().to_string();
                                if !select_add_allowed(text.clone(), onbeforeadd) { return; }
                                additions.write().push(SelectOption::new(text.clone(), text.clone())); onadd.call(text.clone());
                                publish_selection(update_selection(add_selected.clone(), text, multiple), onvaluechange, onvalueschange);
                                query.set(String::new()); query_dirty.set(false); active.set(usize::MAX); if !multiple { open.set(false); }
                            } }
                        }
                    }
                }
            }
            if tooltip_available {
                crate::TooltipFor { id: help_id.clone(), controller: tooltip.clone(),
                    placement: crate::TooltipPlacement::Bottom, show_delay: 500,
                    class: "ix-select-feedback-tooltip {state.class()}",
                    onnativeerror: move |message: String| onnativeerror.call(crate::NativeSurfaceError::Rejected(message)),
                    if let Some(feedback) = visual_feedback { {render_text_field_feedback(feedback, None)} }
                }
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
/// the retained native input after rendering. Values remain consumer-controlled.
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
    let focus_input = use_callback(move |()| {
        spawn(async move {
            if disabled || !state.peek().expanded {
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
                    onclick: move |_| {
                        state.write().expand();
                        dioxus::core::queue_effect(move || focus_input.call(()));
                    },
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
                            onclick: move |_| {
                                state.write().clear();
                                onvaluechange.call(String::new());
                                dioxus::core::queue_effect(move || focus_input.call(()));
                            },
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

// The real file input owns picker activation on web and desktop. The pinned
// desktop interpreter's native.ts handles actual file-input clicks itself,
// including accept/directory/multiple/name. No click or input-reset bridge is needed.
/// File/folder picker and drop target. `FileData` keeps content reads portable.
/// Folder inputs use the host's directory-picker support; `onentrieschanged`
/// retains browser relative paths and expands native desktop folder selections.
/// No network upload occurs. The application owns transfer and upload state.
/// This component selects files; transfer and progress are managed by the caller.
#[component]
pub fn Upload(
    #[props(default)] name: Option<String>,
    #[props(default)] class: String,
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
    let alive = use_hook(|| Rc::new(std::cell::Cell::new(true)));
    let focused = use_hook(|| Rc::new(std::cell::Cell::new(false)));
    let restore_focus = use_hook(|| Rc::new(std::cell::Cell::new(None::<(u64, u64)>)));
    let mut picker_revision = use_signal(|| 0_u64);
    let current_generation = generation.clone();
    let blocked_focus = restore_focus.clone();
    let blocked = disabled || busy;
    use_effect(use_reactive!(|blocked| {
        if blocked {
            current_generation.set(current_generation.get().wrapping_add(1));
            blocked_focus.set(None);
            processing.set(false);
            drag_depth.set(0);
        }
    }));
    let choose_generation = generation.clone();
    let choose_alive = alive.clone();
    let choose = use_callback(move |files: Vec<FileData>| {
        // A canceled native picker (including desktop's empty change delivery)
        // leaves selection, errors and controlled transfer state unchanged.
        if blocked || files.is_empty() || !choose_alive.get() {
            return;
        }
        failure.set(None);
        drag_depth.set(0);
        processing.set(true);
        let sequence = choose_generation.get().wrapping_add(1);
        choose_generation.set(sequence);
        let generation = choose_generation.clone();
        let alive = choose_alive.clone();
        spawn(async move {
            let result = prepare_upload(files, multiple, directory_upload).await;
            if !alive.get() || generation.get() != sequence {
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
    let root_id = format!("{reset_id}-upload");
    let cleanup_alive = alive.clone();
    let cleanup_generation = generation.clone();
    let cleanup_focus = restore_focus.clone();
    use_drop(move || {
        cleanup_alive.set(false);
        cleanup_generation.set(cleanup_generation.get().wrapping_add(1));
        cleanup_focus.set(None);
    });
    let input_focus = focused.clone();
    let input_blur = focused.clone();
    let blur_restore = restore_focus.clone();
    let change_focus = focused.clone();
    let change_restore = restore_focus.clone();
    let change_generation = generation.clone();
    let change_alive = alive.clone();
    let input_change = use_callback(move |event: FormEvent| {
        if blocked || !change_alive.get() {
            return;
        }
        let files = event.files();
        if files.is_empty() {
            return;
        }
        // Retain portable file handles before replacing just the native input.
        // A fresh framework key permits another selection of the same file.
        choose.call(files);
        let revision = picker_revision.peek().wrapping_add(1);
        change_restore.set(
            change_focus
                .get()
                .then_some((revision, change_generation.get())),
        );
        picker_revision.set(revision);
    });
    let mounted_focus = focused.clone();
    let mounted_restore = restore_focus.clone();
    let mounted_generation = generation.clone();
    let mounted_alive = alive.clone();
    let picker_mount = use_callback(move |event: MountedEvent| {
        let Some((revision, selection)) = mounted_restore.take() else {
            return;
        };
        if blocked
            || !mounted_alive.get()
            || !mounted_focus.get()
            || revision != *picker_revision.peek()
            || selection != mounted_generation.get()
        {
            return;
        }
        let node = event.data();
        let alive = mounted_alive.clone();
        let focused = mounted_focus.clone();
        let generation = mounted_generation.clone();
        spawn(async move {
            if alive.get() && focused.get() && generation.get() == selection {
                let _ = node.set_focus(true).await;
            }
        });
    });
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
    let state_class = match state {
        UploadState::Failed => "ix-upload ix-upload-failed",
        UploadState::Success => "ix-upload ix-upload-success",
        _ => "ix-upload",
    };
    let state_name = match state {
        UploadState::SelectFile => "select",
        UploadState::Loading => "loading",
        UploadState::Success => "success",
        UploadState::Failed => "failed",
    };
    rsx! {
        div { id: root_id, class: "{state_class} ix-upload-visual {class}",
            "aria-busy": (busy || processing()).to_string(), "aria-disabled": blocked.to_string(),
            "data-disabled": disabled.to_string(), "data-state": state_name,
            "data-multiline": multiline.to_string(), "data-error": failure().is_some().to_string(),
            div { class: "ix-upload-area", "data-disabled": disabled.to_string(), "data-checking": busy.to_string(),
                "data-file-over": (drag_depth() > 0).to_string(), "data-multiline": multiline.to_string(),
                ondragenter: move |event: DragEvent| { if !blocked { event.prevent_default(); drag_depth += 1; } },
                ondragleave: move |_| drag_depth.set(drag_depth().saturating_sub(1)),
                ondragover: move |event: DragEvent| { event.prevent_default(); if !blocked { event.data_transfer().set_drop_effect("copy"); } else { event.data_transfer().set_drop_effect("none"); } },
                ondrop: move |event: DragEvent| { event.prevent_default(); event.stop_propagation(); drag_depth.set(0); if !blocked { choose.call(event.data_transfer().files()); } },
                span { class: "ix-upload-state", role: "status",
                    if !disabled {
                        if busy { span { class: "ix-upload-spinner-host", crate::Spinner { size: crate::SpinnerSize::Md, variant: crate::SpinnerVariant::Primary } } }
                        else if matches!(state, UploadState::Failed | UploadState::Success) {
                            crate::Icon { name: if state == UploadState::Failed { "error" } else { "success" }, size: 24,
                                class: if state == UploadState::Failed { "ix-upload-status-icon icon-error" } else { "ix-upload-status-icon icon-success" } }
                        }
                    }
                    span { class: "ix-upload-text", "{text}" }
                }
                div { class: "ix-upload-actions",
                    label { class: "ix-upload-button-host", r#for: reset_id.clone(), "data-disabled": blocked.to_string(),
                        // A keyed root VNode replaces the physical native control,
                        // while the surrounding label/button presentation stays put.
                        {rsx! { input { key: "{picker_revision()}", id: reset_id.clone(), class: "ix-upload-browser", r#type: "file",
                            name, accept, multiple: multiple || directory_upload, disabled: blocked,
                            "aria-label": aria_label, "aria-disabled": blocked.to_string(),
                            "webkitdirectory": directory_upload.then_some("true"), "directory": directory_upload.then_some("true"),
                            onmounted:picker_mount, onchange:input_change,
                            onfocus:move |_|input_focus.set(true),
                            onblur:move |_|{input_blur.set(false);blur_restore.set(None);},
                        } }}
                        span { class: "ix-upload-button ix-button ix-button-tokenized ix-button-standard ix-button-secondary ix-align-center",
                            "aria-hidden":"true", "aria-disabled":blocked.to_string(),
                            span { class:"ix-button-content", "{button_text}" }
                        }
                    }
                }
            }
            if let Some(error) = failure() { span { class: "ix-upload-error", role: "alert", "{error}" } }
        }
    }
}

#[cfg(test)]
#[path = "upload_visual_tests.rs"]
mod upload_visual_tests;

#[cfg(test)]
#[path = "checkbox_token_tests.rs"]
mod checkbox_token_tests;

#[cfg(test)]
#[path = "radio_token_tests.rs"]
mod radio_token_tests;

#[cfg(test)]
#[path = "select_visual_tests.rs"]
mod select_visual_tests;

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

pub(crate) const STYLES: &str = include_str!("styles/forms.css");

#[cfg(test)]
#[path = "forms/field_composition_tests.rs"]
mod field_composition_tests;

#[cfg(test)]
#[path = "forms/field_tooltip_tests.rs"]
mod field_tooltip_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "forms/choice_mount_tests.rs"]
mod choice_mount_tests;

#[cfg(test)]
#[path = "forms/range_field_tests.rs"]
mod range_field_tests;

#[cfg(test)]
mod choice_label_tests;
