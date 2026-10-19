//! Rust chat layouts and form controls adapted from the upstream iX contracts.
//! Message transport, Markdown, clipboard access, and uploads belong to the host.
use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static CHAT_INPUT_ID: AtomicUsize = AtomicUsize::new(1);

/// Conversation layout with a scrollable message region and a separate composer.
#[component]
pub fn Chat(
    children: Element,
    prompt: Option<Element>,
    #[props(default = "Conversation".to_string())] label: String,
    #[props(default)] busy: bool,
    #[props(default)] class: String,
) -> Element {
    rsx! { section { class: "ix-chat {class}",
        div { class: "ix-chat-messages", role: "log", aria_label: label, aria_live: "polite", aria_relevant: "additions text", aria_atomic: false, aria_busy: busy,
            {children}
        }
        if let Some(prompt) = prompt { div { class: "ix-chat-prompt", {prompt} } }
    } }
}

/// Assistant message with independently composed actions and source references.
/// Set `streaming` while the host appends content; no transport is assumed.
#[component]
pub fn ChatAiMessage(
    children: Element,
    actions: Option<Element>,
    sources: Option<Element>,
    #[props(default)] streaming: bool,
    #[props(default = "Assistant message".to_string())] label: String,
    #[props(default)] class: String,
) -> Element {
    let has_meta = actions.is_some() || sources.is_some();
    rsx! { article { class: "ix-chat-ai-message {class}", aria_label: label, aria_busy: streaming,
        div { class: "ix-chat-ai-content", {children} }
        if streaming { span { class: "ix-sr-only", "Response in progress" } }
        if has_meta { div { class: "ix-chat-ai-meta",
            if let Some(actions) = actions { div { class: "ix-chat-ai-actions", {actions} } }
            if let Some(sources) = sources { div { class: "ix-chat-ai-sources", {sources} } }
        } }
    } }
}

/// User message bubble, attachment row, and consumer-supplied message actions.
/// Actions become visible on hover/focus and remain visible on touch devices.
#[component]
pub fn ChatUserMessage(
    message: Option<String>,
    children: Option<Element>,
    attachments: Option<Element>,
    actions: Option<Element>,
    #[props(default = "User message".to_string())] label: String,
    #[props(default)] class: String,
) -> Element {
    let has_actions = actions.is_some();
    rsx! { article { class: "ix-chat-user-message {class}", aria_label: label,
        tabindex: if has_actions { Some(0) } else { None },
        if let Some(attachments) = attachments { div { class: "ix-chat-user-attachments", {attachments} } }
        div { class: "ix-chat-user-bubble", if let Some(message) = message { "{message}" } {children} }
        if let Some(actions) = actions { div { class: "ix-chat-user-actions", {actions} } }
    } }
}

/// Upload presentation states supported by iX chat attachments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatAttachmentStatus {
    #[default]
    Default,
    Loading,
    Failed,
}
impl ChatAttachmentStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Loading => "loading",
            Self::Failed => "failed",
        }
    }
    /// Preview activation is available only after upload has completed.
    pub const fn can_preview(self, supported: bool) -> bool {
        supported && matches!(self, Self::Default)
    }
}

/// Split only the final extension, keeping dotfiles and trailing dots intact.
pub fn split_chat_filename(filename: &str) -> (&str, &str) {
    let filename = filename.trim();
    match filename.rfind('.') {
        Some(index) if index > 0 && index < filename.len() - 1 => {
            (&filename[..index], &filename[index..])
        }
        _ => (filename, ""),
    }
}

/// File attachment with independent preview/removal events, loading and failure states.
/// File names are presentation only; keep portable `FileData` handles in the host.
#[component]
pub fn ChatAttachment(
    file_name: String,
    #[props(default)] status: ChatAttachmentStatus,
    #[props(default = "txt-document".to_string())] icon: String,
    #[props(default)] hide_remove_button: bool,
    #[props(default)] preview_supported: bool,
    #[props(default = "Remove attachment".to_string())] remove_label: String,
    #[props(default)] onpreview: EventHandler<()>,
    #[props(default)] onremove: EventHandler<()>,
) -> Element {
    let preview = status.can_preview(preview_supported);
    let (base, extension) = split_chat_filename(&file_name);
    let icon = match status {
        ChatAttachmentStatus::Loading => None,
        ChatAttachmentStatus::Failed => Some("error".to_string()),
        ChatAttachmentStatus::Default => Some(icon),
    };
    rsx! { span { class: "ix-chat-attachment", "data-status": status.as_str(), aria_busy: status == ChatAttachmentStatus::Loading,
        crate::Chip { variant: if status == ChatAttachmentStatus::Failed { crate::StatusVariant::Alarm } else { crate::StatusVariant::Neutral },
            outline: true, icon, inactive: !preview, closable: !hide_remove_button, close_label: remove_label,
            onclick: move |_| { if preview { onpreview.call(()); } }, onclose: move |_| onremove.call(()),
            span { class: "ix-chat-attachment-content", title: file_name.clone(),
                if status == ChatAttachmentStatus::Loading { crate::Spinner { size: crate::SpinnerSize::Xxs, variant: crate::SpinnerVariant::Primary } }
                if status == ChatAttachmentStatus::Default {
                    span { class: "ix-chat-file-name", span { class: "ix-chat-file-base", "{base}" } span { class: "ix-chat-file-extension", "{extension}" } }
                } else { span { class: "ix-chat-file-base", "{file_name}" } }
                if status == ChatAttachmentStatus::Failed { span { class: "ix-sr-only", "Upload failed" } }
            }
        }
    } }
}

/// Chat input/processing presentation. Processing disables submission, matching iX.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChatInputState {
    #[default]
    Input,
    Processing,
}
impl ChatInputState {
    /// A prompt is submitted verbatim; whitespace-only prompts are rejected.
    pub fn can_submit(self, value: &str, disabled: bool, readonly: bool) -> bool {
        self == Self::Input && !disabled && !readonly && !value.trim().is_empty()
    }
}

/// Character limit feedback state; counts follow browser UTF-16 text length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatCharacterLimitState {
    Soft,
    Hard,
}

/// Calculate soft/hard feedback using iX's normalized threshold and inclusive hard limit.
pub fn chat_character_limit_state(
    value: &str,
    limit: Option<usize>,
    threshold: f64,
) -> Option<ChatCharacterLimitState> {
    let limit = limit.filter(|limit| *limit > 0)?;
    let current = value.encode_utf16().count();
    if current >= limit {
        return Some(ChatCharacterLimitState::Hard);
    }
    let threshold = if threshold.is_finite() {
        threshold.clamp(0.0, 1.0)
    } else {
        0.9
    };
    (current >= (limit as f64 * threshold).ceil() as usize).then_some(ChatCharacterLimitState::Soft)
}
fn enter_submits(state: ChatInputState, insert_line_break: bool, modifiers: Modifiers) -> bool {
    state == ChatInputState::Input
        && !insert_line_break
        && !modifiers.shift()
        && !modifiers.alt()
        && !modifiers.ctrl()
        && !modifiers.meta()
}
fn format_character_message(template: &str, current: usize, limit: usize) -> String {
    template
        .replace("{current}", &current.to_string())
        .replace("{limit}", &limit.to_string())
}

/// Controlled multiline composer with native form names/length limits and iX send semantics.
/// Enter submits; modified Enter keeps the textarea's native handling (including
/// Shift+Enter for a new line). The consumer decides when to clear the value and
/// how to transport the submitted prompt. Enter used
/// during an input-method composition is left to the native textarea.
#[component]
pub fn ChatInput(
    #[props(default)] value: String,
    #[props(default)] state: ChatInputState,
    #[props(default)] disabled: bool,
    #[props(default)] readonly: bool,
    #[props(default = "Enter a command, question or topic...".to_string())] placeholder: String,
    #[props(default = "Chat input".to_string())] textarea_label: String,
    name: Option<String>,
    id: Option<String>,
    max_length: Option<usize>,
    character_limit: Option<usize>,
    #[props(default = 0.9)] character_limit_warning_threshold: f64,
    #[props(default = "Character limit reached ({current} / {limit} characters)".to_string())]
    character_limit_reached_text: String,
    #[props(default = "You're nearing the limit ({current} / {limit} characters)".to_string())]
    character_limit_warning_text: String,
    #[props(default = 1)] min_rows: u32,
    #[props(default = 6)] max_rows: u32,
    #[props(default)] insert_line_break_on_enter: bool,
    #[props(default = "This content is AI-generated. Always verify the information for accuracy.".to_string())]
    disclaimer: String,
    follow_up: Option<Element>,
    attachments: Option<Element>,
    start: Option<Element>,
    end: Option<Element>,
    #[props(default)] onvaluechange: EventHandler<String>,
    #[props(default)] onpromptsubmit: EventHandler<String>,
    #[props(default)] onchange: EventHandler<String>,
    #[props(default)] onblur: EventHandler<()>,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
) -> Element {
    let generated_id = use_hook(|| {
        format!(
            "ix-chat-input-{}",
            CHAT_INPUT_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let id = id.unwrap_or(generated_id);
    let help_id = format!("{id}-limit");
    let mut committed = use_signal(|| value.clone());
    let submit_value = value.clone();
    let submit = use_callback(move |_| {
        if state.can_submit(&submit_value, disabled, readonly) {
            if *committed.peek() != submit_value {
                committed.set(submit_value.clone());
                onchange.call(submit_value.clone());
            }
            onpromptsubmit.call(submit_value.clone());
        }
    });
    let focus_value = value.clone();
    let blur_value = value.clone();
    let limit = character_limit.or(max_length).filter(|limit| *limit > 0);
    let limit_state = chat_character_limit_state(&value, limit, character_limit_warning_threshold);
    let limit_message = limit_state.zip(limit).map(|(state, limit)| {
        format_character_message(
            if state == ChatCharacterLimitState::Hard {
                &character_limit_reached_text
            } else {
                &character_limit_warning_text
            },
            value.encode_utf16().count(),
            limit,
        )
    });
    let min_rows = min_rows.max(1);
    let max_rows = max_rows.max(min_rows);
    let rows = (value.split('\n').count() as u32).clamp(min_rows, max_rows);
    let blocked = !state.can_submit(&value, disabled, readonly);
    let processing = state == ChatInputState::Processing;
    rsx! { div { class: "ix-chat-input", "data-readonly": readonly, "data-disabled": disabled, "data-processing": processing,
        if let Some(follow_up) = follow_up { div { class: "ix-chat-follow-up", {follow_up} } }
        fieldset { class: "ix-chat-input-surface", disabled, aria_label: textarea_label.clone(),
            if let Some(attachments) = attachments { div { class: "ix-chat-input-attachments", {attachments} } }
            textarea { id, name, value, placeholder, disabled, readonly, maxlength: limit, rows,
                class: "ix-chat-textarea", style: "--ix-chat-min-rows:{min_rows};--ix-chat-max-rows:{max_rows}",
                aria_label: textarea_label, aria_describedby: if limit_message.is_some() { Some(help_id.clone()) } else { None },
                onmounted,
                onfocus: move |_| committed.set(focus_value.clone()),
                oninput: move |event| onvaluechange.call(event.value()),
                onkeydown: move |event| { if event.key() == Key::Enter && !event.is_composing() && enter_submits(state, insert_line_break_on_enter, event.modifiers()) { event.prevent_default(); submit.call(()); } },
                onblur: move |_| { onblur.call(()); if *committed.peek() != blur_value { committed.set(blur_value.clone()); onchange.call(blur_value.clone()); } },
            }
            if let Some(message) = limit_message.clone() { div { id: help_id.clone(), class: "ix-chat-character-limit", "data-hard": limit_state == Some(ChatCharacterLimitState::Hard), role: "status",
                crate::Icon { name: if limit_state == Some(ChatCharacterLimitState::Hard) { "warning" } else { "info" }, size: 16 }
                "{message}"
            } }
            div { class: "ix-chat-action-row",
                div { class: "ix-chat-input-start", {start} }
                div { class: "ix-chat-input-end", {end}
                    crate::IconButton { icon: if processing { "circle-stop" } else { "send-right-filled" },
                        label: if processing { "Stop processing" } else { "Submit prompt" }, variant: crate::ButtonVariant::Tertiary,
                        disabled: blocked, onclick: move |_| submit.call(()) }
                }
            }
        }
        if !disclaimer.is_empty() { p { class: "ix-chat-disclaimer", "{disclaimer}" } }
    } }
}

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-chat{display:flex;flex-direction:column;width:100%;height:100%;min-height:0;min-width:0;max-width:784px;margin-inline:auto;box-sizing:border-box}.ix-theme .ix-chat-messages{display:flex;flex:1 1 auto;flex-direction:column;align-items:center;gap:32px;min-height:0;width:100%;padding:32px;overflow-y:auto;overscroll-behavior:contain}.ix-theme .ix-chat-messages>*{width:100%;max-width:720px;flex:0 0 auto}.ix-theme .ix-chat-prompt{display:flex;flex:0 0 auto;justify-content:center;width:100%;padding:32px}.ix-theme .ix-chat-prompt>*{width:100%;max-width:720px}
.ix-theme .ix-chat-ai-message{display:flex;flex-direction:column;align-items:flex-start;width:100%;min-width:0;gap:16px;color:var(--ix-color-text)}.ix-theme .ix-chat-ai-content{display:flex;flex-direction:column;align-items:stretch;gap:16px;width:100%;min-width:0;font:14px/24px var(--ix-font);overflow-wrap:anywhere}.ix-theme .ix-chat-ai-content>*{margin:0;color:inherit;overflow-wrap:anywhere}.ix-theme .ix-chat-ai-content :is(h1,h2,h3,h4,h5,h6){font:700 14px/24px var(--ix-font)}.ix-theme .ix-chat-ai-content ul,.ix-theme .ix-chat-ai-content ol{padding-left:24px}.ix-theme .ix-chat-ai-meta{display:flex;flex-wrap:wrap;align-items:center;gap:16px;width:100%;min-height:32px}.ix-theme .ix-chat-ai-actions,.ix-theme .ix-chat-ai-sources{display:flex;flex-wrap:wrap;align-items:center;gap:4px}
.ix-theme .ix-chat-user-message{display:flex;flex-direction:column;align-items:flex-end;width:100%;min-width:0;outline:none}.ix-theme .ix-chat-user-bubble{width:fit-content;min-width:min(248px,100%);max-width:80%;padding:12px 16px;border-radius:var(--ix-radius);background:var(--ix-color-surface-2);color:var(--ix-color-text);font:14px/20px var(--ix-font);overflow-wrap:anywhere;white-space:pre-wrap}.ix-theme .ix-chat-user-attachments{display:flex;justify-content:flex-end;flex-wrap:wrap;gap:8px;width:100%;max-width:75%;margin-bottom:8px}.ix-theme .ix-chat-user-attachments>.ix-chat-attachment{flex:0 1 96px}.ix-theme .ix-chat-user-actions{display:flex;align-items:center;justify-content:flex-end;gap:8px;min-height:32px;margin-top:8px;opacity:0;transition:opacity .2s ease-in-out}.ix-theme .ix-chat-user-message:is(:hover,:focus,:focus-within) .ix-chat-user-actions{opacity:1}.ix-theme .ix-chat-user-message:focus-visible .ix-chat-user-bubble{outline:2px solid var(--ix-color-focus);outline-offset:2px}
.ix-theme .ix-chat-attachment{position:relative;display:inline-block;min-width:0;max-width:320px;vertical-align:middle}.ix-theme .ix-chat-attachment .ix-chip{display:flex;width:100%}.ix-theme .ix-chat-attachment-content{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;width:100%}.ix-theme .ix-chat-file-name{display:inline-flex;flex:1 1 auto;min-width:0;max-width:100%;overflow:hidden}.ix-theme .ix-chat-file-base{flex:1 1 auto;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-chat-file-extension{flex:0 0 auto;white-space:nowrap}.ix-theme .ix-chat-attachment .ix-chip-main{min-width:0}.ix-theme .ix-chat-attachment .ix-status-label{display:inline-flex;min-width:0;width:100%}
.ix-theme .ix-chat-attachment[data-status=failed] .ix-chip.ix-outline{color:var(--ix-color-danger-text)}
.ix-theme .ix-chat-input{display:block;width:100%;min-width:0}.ix-theme .ix-chat-input-surface{display:flex;flex-direction:column;min-width:0;width:100%;min-height:100px;padding:12px;margin:0;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);color:var(--ix-color-text);font:14px/24px var(--ix-font)}.ix-theme .ix-chat-input-surface:hover{border-color:var(--ix-color-text-secondary)}.ix-theme .ix-chat-input-surface:focus-within{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-theme .ix-chat-follow-up{display:flex;flex-wrap:wrap;align-items:flex-start;gap:8px;width:100%;margin-bottom:8px}.ix-theme .ix-chat-follow-up>.ix-button{max-width:480px;min-width:64px}.ix-theme .ix-chat-input-attachments{display:flex;flex-wrap:nowrap;gap:12px;width:100%;max-width:100%;margin-bottom:12px;overflow-x:auto;overflow-y:hidden;scrollbar-width:thin;padding-bottom:4px}.ix-theme .ix-chat-input-attachments>.ix-chat-attachment{flex:0 0 auto}.ix-theme .ix-chat-textarea{flex:1 1 auto;width:100%;min-height:calc(var(--ix-chat-min-rows)*24px);max-height:calc(var(--ix-chat-max-rows)*24px);padding:0;border:0;outline:0;resize:none;box-sizing:border-box;field-sizing:content;background:transparent;color:inherit;font:14px/24px var(--ix-font);overflow-y:auto}.ix-theme .ix-chat-textarea:focus-visible{outline:0}.ix-theme .ix-chat-textarea::placeholder{color:var(--ix-color-text-secondary);opacity:1}.ix-theme .ix-chat-character-limit{display:flex;align-items:center;gap:8px;margin-top:12px;font:14px/20px var(--ix-font)}.ix-theme .ix-chat-character-limit>.ix-icon{color:var(--ix-color-info)}.ix-theme .ix-chat-character-limit[data-hard=true]>.ix-icon{color:var(--ix-color-warning)}.ix-theme .ix-chat-action-row{display:flex;align-items:center;justify-content:space-between;gap:8px;width:100%;min-height:32px;margin-top:8px}.ix-theme .ix-chat-input-start,.ix-theme .ix-chat-input-end{display:flex;align-items:center;gap:8px}.ix-theme .ix-chat-input-start{flex:1 1 auto;min-width:0}.ix-theme .ix-chat-input-end{flex:0 0 auto;justify-content:flex-end}.ix-theme .ix-chat-disclaimer{margin:8px 0 0;text-align:center;font:12px/16px var(--ix-font);color:var(--ix-color-text-secondary)}.ix-theme .ix-chat-input[data-disabled=true] .ix-chat-input-surface{opacity:.5;background:transparent}.ix-theme .ix-chat-input[data-readonly=true] .ix-chat-input-surface{background:transparent}.ix-theme .ix-chat-input[data-readonly=true] .ix-chat-input-surface:focus-within{outline:none}
@media(hover:none){.ix-theme .ix-chat-user-actions{opacity:1}}@media(max-width:600px){.ix-theme .ix-chat-messages,.ix-theme .ix-chat-prompt{padding:16px}.ix-theme .ix-chat-user-bubble{max-width:90%}.ix-theme .ix-chat-user-attachments{max-width:90%}.ix-theme .ix-chat-ai-meta{gap:8px}}@media(prefers-reduced-motion:reduce){.ix-theme .ix-chat-user-actions{transition:none}}
"#;

#[cfg(test)]
#[path = "chat/composer_events.rs"]
mod composer_events;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filenames_preserve_extensions_without_breaking_dotfiles() {
        assert_eq!(
            split_chat_filename(" report.final.pdf "),
            ("report.final", ".pdf")
        );
        assert_eq!(split_chat_filename(".gitignore"), (".gitignore", ""));
        assert_eq!(split_chat_filename("notes."), ("notes.", ""));
        assert_eq!(split_chat_filename("résumé.txt"), ("résumé", ".txt"));
    }
    #[test]
    fn submission_rejects_empty_disabled_readonly_and_processing_prompts() {
        assert!(ChatInputState::Input.can_submit(" Check alarms ", false, false));
        for value in ["", " \n\t"] {
            assert!(!ChatInputState::Input.can_submit(value, false, false));
        }
        assert!(!ChatInputState::Input.can_submit("Check alarms", true, false));
        assert!(!ChatInputState::Input.can_submit("Check alarms", false, true));
        assert!(!ChatInputState::Processing.can_submit("Check alarms", false, false));
        assert!(enter_submits(
            ChatInputState::Input,
            false,
            Modifiers::empty()
        ));
        for modifiers in [
            Modifiers::SHIFT,
            Modifiers::CONTROL,
            Modifiers::ALT,
            Modifiers::META,
        ] {
            assert!(!enter_submits(ChatInputState::Input, false, modifiers));
        }
        assert!(!enter_submits(
            ChatInputState::Processing,
            false,
            Modifiers::empty()
        ));
        assert!(!enter_submits(
            ChatInputState::Input,
            true,
            Modifiers::empty()
        ));
    }
    #[test]
    fn character_warning_matches_browser_code_units_and_boundary_rules() {
        assert_eq!(chat_character_limit_state("12345678", Some(10), 0.9), None);
        assert_eq!(
            chat_character_limit_state("123456789", Some(10), 0.9),
            Some(ChatCharacterLimitState::Soft)
        );
        assert_eq!(
            chat_character_limit_state("1234567890", Some(10), 0.9),
            Some(ChatCharacterLimitState::Hard)
        );
        assert_eq!(
            chat_character_limit_state("😀", Some(2), 0.9),
            Some(ChatCharacterLimitState::Hard)
        );
        assert_eq!(
            chat_character_limit_state("123456789", Some(10), f64::NAN),
            Some(ChatCharacterLimitState::Soft)
        );
        assert_eq!(chat_character_limit_state("anything", Some(0), 0.9), None);
        assert_eq!(
            format_character_message("{limit} characters, {current} used", 9, 10),
            "10 characters, 9 used"
        );
    }
    #[test]
    fn attachment_preview_requires_default_status_and_opt_in() {
        assert!(ChatAttachmentStatus::Default.can_preview(true));
        assert!(!ChatAttachmentStatus::Default.can_preview(false));
        assert!(!ChatAttachmentStatus::Loading.can_preview(true));
        assert!(!ChatAttachmentStatus::Failed.can_preview(true));
    }
    #[test]
    fn composer_renders_native_form_names_constraints_and_processing_state() {
        fn render(build: fn() -> Element) -> String {
            let mut dom = VirtualDom::new(build);
            dom.rebuild_in_place();
            dioxus_ssr::render(&dom)
        }
        let html = render(
            || rsx! { ChatInput { name: "prompt", value: "123456789", max_length: 10, textarea_label: "Service assistant prompt" } },
        );
        assert!(html.contains("name=\"prompt\""));
        assert!(html.contains("maxlength=\"10\"") || html.contains("maxlength=10"));
        assert!(html.contains("Service assistant prompt"));
        assert!(html.contains("nearing the limit (9 / 10 characters)"));
        let processing = render(
            || rsx! { ChatInput { value: "Check alarms", state: ChatInputState::Processing } },
        );
        assert!(processing.contains("Stop processing"));
        assert!(processing.contains("disabled"));
        let attachment = render(
            || rsx! { ChatAttachment { file_name: "report.pdf", status: ChatAttachmentStatus::Failed } },
        );
        assert!(attachment.contains("Upload failed"));
        assert!(attachment.contains("Remove attachment"));
    }
}
