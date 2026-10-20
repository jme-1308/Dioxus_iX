//! Local equipment conversations with a native externally associated chat form.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SentMessage {
    id: usize,
    text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EquipmentThread {
    id: &'static str,
    asset: &'static str,
    draft: String,
    sent: Vec<SentMessage>,
}

fn initial_threads() -> Vec<EquipmentThread> {
    vec![
        EquipmentThread {
            id: "motor",
            asset: "Motor 001",
            draft: String::new(),
            sent: Vec::new(),
        },
        EquipmentThread {
            id: "pump",
            asset: "Pump 002",
            draft: String::new(),
            sent: Vec::new(),
        },
    ]
}

fn append_message(
    threads: &mut [EquipmentThread],
    selected: &str,
    id: usize,
    text: String,
) -> bool {
    if text.trim().is_empty() {
        return false;
    }
    if let Some(thread) = threads.iter_mut().find(|thread| thread.id == selected) {
        thread.sent.push(SentMessage { id, text });
        thread.draft.clear();
        true
    } else {
        false
    }
}

#[component]
fn ConversationPreparation() -> Element {
    let mut note = use_signal(|| "Prepare local maintenance discussion".to_string());
    rsx! {
        label { class: "chat-workspace-preparation", r#for: "chat-preparation",
            span { "Inspection preparation" }
            input { id: "chat-preparation", r#type: "text", maxlength: "160", value: note(), oninput: move |event: FormEvent| note.set(event.value()) }
        }
    }
}

/// Two illustrative equipment threads sharing one retained native composer.
#[component]
pub fn ChatWorkspaceExample() -> Element {
    let mut threads = use_signal(initial_threads);
    let mut selected = use_signal(|| "motor".to_string());
    let mut disabled = use_signal(|| false);
    let mut read_only = use_signal(|| false);
    let mut processing = use_signal(|| false);
    let mut newlines = use_signal(|| false);
    let mut german = use_signal(|| false);
    let mut composer = use_signal(|| None::<Rc<MountedData>>);
    let mut next_id = use_signal(|| 1_usize);
    let mut sends = use_signal(|| 0_usize);
    let mut form_submits = use_signal(|| 0_usize);
    let mut resets = use_signal(|| 0_usize);
    let mut last_form_prompt = use_signal(String::new);
    let mut feedback = use_signal(|| "No messages sent.".to_string());
    let mut epoch = use_signal(|| 0_u64);

    let send = use_callback(move |prompt: String| {
        if *disabled.peek() || *read_only.peek() || *processing.peek() {
            feedback.set("Sending is unavailable in the current composer state.".into());
            return;
        }
        if prompt.trim().is_empty() {
            feedback.set("Write a message containing more than spaces.".into());
            return;
        }
        if prompt.encode_utf16().count() > 400 {
            feedback.set("Keep the message within 400 characters.".into());
            return;
        }
        let mut next = threads.peek().clone();
        let id = *next_id.peek();
        if append_message(&mut next, &selected.peek(), id, prompt) {
            threads.set(next);
            next_id.set(id + 1);
            let count = sends.peek().saturating_add(1);
            sends.set(count);
            feedback.set("Message recorded in the current local conversation.".into());
        }
    });
    let current_threads = threads();
    let current = current_threads
        .iter()
        .find(|thread| thread.id == selected())
        .expect("known local equipment thread")
        .clone();
    let total = current_threads
        .iter()
        .map(|thread| thread.sent.len())
        .sum::<usize>();
    let blocked = disabled() || read_only() || processing();

    rsx! {
        section { id: "chat-workspace", class: "chat-workspace-example",
            h2 { "Equipment conversations" }
            p { class: "chat-workspace-intro", "Keep a separate draft and conversation for each illustrative local asset. Send with the composer, Enter, or Send with form. Messages are kept exactly as written; replies are fixed local samples, with no assistant service or transport connected." }
            div { class: "chat-workspace-toolbar",
                Button { id: "chat-restore", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        threads.set(initial_threads()); selected.set("motor".into()); disabled.set(false); read_only.set(false); processing.set(false);
                        newlines.set(false); german.set(false); next_id.set(1); sends.set(0); form_submits.set(0); resets.set(0);
                        last_form_prompt.set(String::new()); feedback.set("No messages sent.".into()); composer.set(None);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore conversations"
                }
                label { class: "chat-workspace-select", r#for: "chat-thread", span { "Equipment thread" }
                    select { id: "chat-thread", value: selected(), onchange: move |event: FormEvent| {
                        let value = event.value();
                        if value == "motor" || value == "pump" { selected.set(value); feedback.set("Equipment thread changed; its draft is retained.".into()); }
                    },
                        option { value: "motor", selected: selected() == "motor", "Motor 001" }
                        option { value: "pump", selected: selected() == "pump", "Pump 002" }
                    }
                }
                Button { id: "chat-focus", variant: ButtonVariant::Secondary, disabled: disabled(),
                    onclick: move |_| {
                        if !*disabled.peek() && let Some(node) = composer.peek().clone() {
                            spawn(async move { let _ = node.set_focus(true).await; });
                        }
                    }, "Focus composer"
                }
            }
            details { id: "chat-options", class: "chat-workspace-options",
                summary { "Composer options" }
                div { class: "chat-workspace-option-fields",
                    label { input { id: "chat-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable composer" }
                    label { input { id: "chat-readonly", r#type: "checkbox", checked: read_only(), onchange: move |event: FormEvent| read_only.set(event.checked()) } "Read-only composer" }
                    label { input { id: "chat-processing", r#type: "checkbox", checked: processing(), onchange: move |event: FormEvent| processing.set(event.checked()) } "Processing state" }
                    label { input { id: "chat-newlines", r#type: "checkbox", checked: newlines(), onchange: move |event: FormEvent| newlines.set(event.checked()) } "Enter inserts a new line" }
                    label { input { id: "chat-german", r#type: "checkbox", checked: german(), onchange: move |event: FormEvent| german.set(event.checked()) } "German composer labels" }
                }
            }
            for generation in [epoch()] { ConversationPreparation { key: "chat-preparation-{generation}" } }
            div { id: "chat-transcript", class: "chat-workspace-conversation", "data-thread": current.id,
                Chat { label: format!("{} local conversation", current.asset), busy: processing(),
                    prompt: rsx! {
                        for generation in [epoch()] {
                            ChatInput { key: "equipment-composer-{generation}", id: "chat-prompt", name: "prompt", form: "chat-message-form", required: true,
                                value: current.draft.clone(), disabled: disabled(), readonly: read_only(),
                                state: if processing() { ChatInputState::Processing } else { ChatInputState::Input }, insert_line_break_on_enter: newlines(),
                                textarea_label: if german() { format!("Nachricht für {}", current.asset) } else { format!("Message for {}", current.asset) },
                                submit_label: if german() { "Nachricht senden" } else { "Send message" }, processing_label: if german() { "Bitte warten" } else { "Please wait" },
                                placeholder: if german() { "Lokale Wartungsnotiz schreiben…" } else { "Write a local maintenance note…" },
                                max_length: 400, character_limit: 400, min_rows: 2, max_rows: 6,
                                disclaimer: "Local demonstration: messages and sample replies stay in this workspace.",
                                onmounted: move |event: MountedEvent| composer.set(Some(event.data())),
                                onvaluechange: move |value| {
                                    if *disabled.peek() || *read_only.peek() { return; }
                                    let key = selected.peek().clone(); let mut next = threads.peek().clone();
                                    if let Some(thread) = next.iter_mut().find(|thread| thread.id == key) { thread.draft = value; threads.set(next); }
                                },
                                onpromptsubmit: move |prompt| send.call(prompt),
                            }
                        }
                    },
                    ChatAiMessage { label: "Local workspace guide",
                        p { "This is the {current.asset} local conversation. Write an inspection note or question; the sample reply only acknowledges local storage in this example." }
                    }
                    for message in &current.sent {
                        div { key: "{message.id}", id: format!("chat-message-{}-{}", current.id, message.id), "data-message-id": message.id.to_string(), "data-thread": current.id,
                            ChatUserMessage { message: message.text.clone(), label: format!("Sent message for {}", current.asset) }
                            ChatAiMessage { label: "Local sample reply",
                                p { "Local sample: your message was recorded for {current.asset}. No assistant service is connected." }
                            }
                        }
                    }
                }
            }
            form { id: "chat-message-form", class: "chat-workspace-form",
                onsubmit: move |event: FormEvent| {
                    event.prevent_default(); let count = form_submits.peek().saturating_add(1); form_submits.set(count);
                    let prompt = event.values().into_iter().find_map(|(name, value)| {
                        if name == "prompt" { if let dioxus::html::FormValue::Text(text) = value { return Some(text); } } None
                    });
                    if let Some(prompt) = prompt { last_form_prompt.set(prompt.clone()); send.call(prompt); }
                    else { last_form_prompt.set(String::new()); feedback.set("The form did not include its associated prompt field.".into()); }
                },
                onreset: move |event: FormEvent| {
                    event.prevent_default(); let key = selected.peek().clone(); let mut next = threads.peek().clone();
                    if let Some(thread) = next.iter_mut().find(|thread| thread.id == key) { thread.draft.clear(); threads.set(next); }
                    let count = resets.peek().saturating_add(1); resets.set(count); feedback.set("Current draft cleared; sent messages and other drafts remain.".into());
                },
                button { id: "chat-form-submit", class: "ix-button ix-button-secondary", r#type: "submit", disabled: blocked, "Send with form" }
                button { id: "chat-form-reset", class: "ix-button ix-button-secondary", r#type: "reset", "Clear current draft" }
            }
            p { id: "chat-feedback", class: "chat-workspace-feedback", role: "status", "{feedback}" }
            table { id: "chat-thread-records", class: "chat-workspace-records",
                caption { "Local conversation records" }
                thead { tr { th { scope: "col", "Equipment" } th { scope: "col", "Retained thread data" } } }
                tbody {
                    for thread in &current_threads {
                        tr { key: "{thread.id}", id: format!("chat-thread-record-{}", thread.id), "data-draft": thread.draft.clone(), "data-count": thread.sent.len().to_string(),
                            th { scope: "row", "{thread.asset}" }
                            td {
                                span { "Draft: " span { "data-field": "draft", "{thread.draft}" } if thread.draft.is_empty() { "Empty draft" } }
                                span { "Sent messages: {thread.sent.len()}" }
                                span { "data-field": "last", if let Some(message) = thread.sent.last() { "{message.text}" } else { "No sent messages" } }
                            }
                        }
                    }
                }
            }
            output { id: "chat-result", class: "chat-workspace-result", role: "status",
                "data-thread": selected(), "data-current-count": current.sent.len().to_string(), "data-total": total.to_string(), "data-sends": sends().to_string(),
                "data-form-submits": form_submits().to_string(), "data-last-form-prompt": last_form_prompt(), "data-resets": resets().to_string(),
                "data-disabled": disabled().to_string(), "data-readonly": read_only().to_string(), "data-processing": processing().to_string(), "data-newlines": newlines().to_string(), "data-german": german().to_string(),
                "{total} messages recorded · {current.sent.len()} in {current.asset}"
            }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.chat-workspace-example{min-width:0;max-width:100%;margin-bottom:48px}.chat-workspace-intro{max-width:72ch;line-height:1.55}.chat-workspace-toolbar,.chat-workspace-form{display:flex;align-items:center;flex-wrap:wrap;gap:12px;min-width:0}.chat-workspace-toolbar{margin:20px 0 16px}.chat-workspace-form{margin:16px 0}.chat-workspace-toolbar .ix-button,.chat-workspace-form .ix-button{max-width:100%;white-space:normal}.chat-workspace-select{display:flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;font-size:14px;line-height:20px}.chat-workspace-select select{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.chat-workspace-options{margin:0 0 24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.chat-workspace-options>summary{padding:10px 12px;font-size:14px;font-weight:600;cursor:pointer}.chat-workspace-option-fields{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;padding:12px;min-width:0}.chat-workspace-option-fields>label{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;font-size:14px;line-height:20px}.chat-workspace-option-fields input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.chat-workspace-preparation{display:flex;flex-direction:column;gap:8px;min-width:0;width:100%;max-width:480px;margin-bottom:24px;font-size:14px;line-height:20px;color:var(--ix-color-text)}.chat-workspace-preparation input{box-sizing:border-box;width:100%;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.chat-workspace-conversation{max-width:784px;height:560px;max-height:80vh;min-height:360px;border:1px solid var(--ix-color-border);background:var(--ix-color-surface);min-width:0}.chat-workspace-conversation .ix-chat-user-bubble{white-space:pre-wrap;overflow-wrap:anywhere}.chat-workspace-feedback{max-width:72ch;font-size:14px;line-height:20px;overflow-wrap:anywhere}.chat-workspace-records{width:100%;max-width:784px;table-layout:fixed;border-collapse:collapse;margin-top:24px;font-size:14px;line-height:20px;color:var(--ix-color-text)}.chat-workspace-records caption{text-align:start;font-weight:600;padding-bottom:8px}.chat-workspace-records th,.chat-workspace-records td{padding:8px;border-bottom:1px solid var(--ix-color-border);text-align:start;vertical-align:top;overflow-wrap:anywhere}.chat-workspace-records thead th{background:var(--ix-color-surface-1)}.chat-workspace-records th:first-child{width:30%}.chat-workspace-records td>span{display:block;margin-bottom:4px;white-space:pre-wrap}.chat-workspace-result{display:block;margin-top:24px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}.chat-workspace-preparation input:focus-visible,.chat-workspace-select select:focus-visible,.chat-workspace-options>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.chat-workspace-option-fields input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}@media(max-width:420px){.chat-workspace-conversation{height:520px;min-height:340px}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_send_keeps_payload_verbatim_and_preserves_the_other_draft() {
        let mut threads = initial_threads();
        threads[0].draft = "  Inspect\nbearings  ".into();
        threads[1].draft = "Pump draft".into();
        assert!(append_message(
            &mut threads,
            "motor",
            1,
            "  Inspect\nbearings  ".into()
        ));
        assert_eq!(threads[0].sent[0].text, "  Inspect\nbearings  ");
        assert!(threads[0].draft.is_empty());
        assert_eq!(threads[1].draft, "Pump draft");
        assert!(!append_message(&mut threads, "motor", 2, " \n ".into()));
        assert_eq!(threads[0].sent.len(), 1);
    }
    #[test]
    fn chat_workspace_renders_one_externally_associated_required_composer() {
        let mut dom = VirtualDom::new(ChatWorkspaceExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("id=\"chat-prompt\"").count(), 1);
        for id in [
            "chat-workspace",
            "chat-message-form",
            "chat-form-submit",
            "chat-form-reset",
            "chat-thread",
            "chat-preparation",
            "chat-thread-record-motor",
            "chat-thread-record-pump",
            "chat-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        let textarea = html
            .split("<textarea")
            .nth(1)
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        assert!(
            textarea.contains("form=\"chat-message-form\"")
                && textarea.contains("name=\"prompt\"")
                && textarea.contains("required")
        );
        assert!(html.contains("type=\"submit\"") && html.contains("type=\"reset\""));
        assert!(
            html.contains("Motor 001")
                && html.contains("Pump 002")
                && html.contains("Prepare local maintenance discussion")
        );
        assert!(html.contains("data-total=\"0\"") && html.contains("data-form-submits=\"0\""));
    }
}
