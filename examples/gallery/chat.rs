use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, PartialEq)]
struct LocalExchange {
    prompt: String,
    response: String,
    chunks: usize,
    streaming: bool,
}

const RESPONSE_CHUNKS: [&str; 3] = [
    "Review the acknowledged alarms from the last shift. ",
    "Confirm that spare parts are available before the service window. ",
    "Share the summary with the maintenance lead.",
];

#[component]
pub fn ChatExamples() -> Element {
    let mut prompt =
        use_signal(|| "Create a concise follow-up plan for the service team.".to_string());
    let mut exchanges = use_signal(Vec::<LocalExchange>::new);
    let mut feedback = use_signal(|| {
        "The transcript below reproduces the upstream example. New responses are a local interaction demonstration.".to_string()
    });
    let mut vote = use_signal(|| None::<bool>);
    let mut editing = use_signal(|| false);
    let mut user_message = use_signal(|| {
        "Summarize the asset status and include the attached maintenance files.".to_string()
    });
    let mut edit_text = use_signal(String::new);
    let mut sources = use_signal(|| false);
    let mut files = use_signal(Vec::<dioxus::html::FileData>::new);
    let processing = exchanges.read().last().is_some_and(|entry| entry.streaming);
    let submit = use_callback(move |value: String| {
        exchanges.write().push(LocalExchange {
            prompt: value,
            response: String::new(),
            chunks: 0,
            streaming: true,
        });
        prompt.set(String::new());
        feedback.set(
            "Prompt submitted locally. Use Append response chunk to review incremental rendering."
                .into(),
        );
    });
    let input = rsx! { ChatInput { value: prompt(), name: "service-prompt", state: if processing { ChatInputState::Processing } else { ChatInputState::Input },
        onvaluechange: move |value| prompt.set(value), onpromptsubmit: move |value| submit.call(value),
        attachments: rsx! {
            for (index, file) in files.read().iter().cloned().enumerate() {
                ChatAttachment { key: "{index}-{file.name()}", file_name: file.name(), preview_supported: true,
                    onpreview: move |_| feedback.set(format!("Selected attachment: {} ({} bytes)", file.name(), file.size())),
                    onremove: move |_| { files.write().remove(index); } }
            }
        },
        start: rsx! { label { class: "chat-example-file-picker", title: "Attach files",
            Icon { name: "plus", size: 24 }
            input { r#type: "file", multiple: true, aria_label: "Attach files", onchange: move |event| files.write().extend(event.files()) }
        } },
        follow_up: rsx! {
            Button { variant: ButtonVariant::SubtleSecondary, onclick: move |_| prompt.set("Check open alarms and suggest the next maintenance step.".into()), "Check open alarms" }
            Button { variant: ButtonVariant::SubtleSecondary, onclick: move |_| prompt.set("Compare the current report with additional customer feedback.".into()), "Compare reports" }
        }
    } };
    rsx! {
        section { class: "demo-section", id: "chat",
            h2 { "Chat" }
            p { "Messages, actions, attachments, and prompt layout from the upstream chat example. No service is connected." }
            div { class: "chat-example-conversation",
                Chat { prompt: input, busy: processing,
                    ChatUserMessage { message: if editing() { None } else { Some(user_message()) },
                        attachments: rsx! {
                            ChatAttachment { file_name: "equipment_status.pdf", icon: "pdf-document", hide_remove_button: true }
                            ChatAttachment { file_name: "maintenance_notes.txt", hide_remove_button: true }
                        },
                        actions: rsx! {
                            IconButton { icon: "copy", label: "Copy message", variant: ButtonVariant::SubtleTertiary, size: ButtonIconSize::TwentyFour,
                                onclick: move |_| feedback.set("Copy message requested. The host application supplies clipboard access.".into()) }
                            IconButton { icon: "pen", label: "Edit message", variant: ButtonVariant::SubtleTertiary, size: ButtonIconSize::TwentyFour,
                                onclick: move |_| { edit_text.set(user_message()); editing.set(true); } }
                            IconButton { icon: "more-menu", label: "Show more actions", variant: ButtonVariant::SubtleTertiary, size: ButtonIconSize::TwentyFour,
                                onclick: move |_| feedback.set("Message action requested.".into()) }
                        },
                        if editing() {
                            div { class: "chat-example-edit",
                                Textarea { label: "Edit message", value: edit_text(), rows: 3, onvaluechange: move |value| edit_text.set(value) }
                                div { class: "demo-row",
                                    Button { disabled: edit_text().trim().is_empty(), onclick: move |_| { user_message.set(edit_text()); editing.set(false); feedback.set("Message edit saved.".into()); }, "Save edit" }
                                    Button { variant: ButtonVariant::Tertiary, onclick: move |_| editing.set(false), "Cancel" }
                                }
                            }
                        }
                    }
                    ChatAiMessage {
                        actions: rsx! {
                            IconButton { icon: "copy", label: "Copy response", variant: ButtonVariant::SubtleTertiary, size: ButtonIconSize::TwentyFour,
                                onclick: move |_| feedback.set("Copy response requested. The host application supplies clipboard access.".into()) }
                            IconToggleButton { icon: "thumb-up", label: "Helpful response", variant: ButtonVariant::SubtleTertiary, size: ButtonIconSize::TwentyFour,
                                pressed: vote() == Some(true), onpressedchange: move |selected| { vote.set(if selected { Some(true) } else { None }); feedback.set("Response feedback updated.".into()); } }
                            IconToggleButton { icon: "thumb-down", label: "Not helpful response", variant: ButtonVariant::SubtleTertiary, size: ButtonIconSize::TwentyFour,
                                pressed: vote() == Some(false), onpressedchange: move |selected| { vote.set(if selected { Some(false) } else { None }); feedback.set("Response feedback updated.".into()); } }
                            IconButton { icon: "refresh", label: "Regenerate response", disabled: processing, variant: ButtonVariant::SubtleTertiary, size: ButtonIconSize::TwentyFour,
                                onclick: move |_| submit.call("Regenerate the asset performance summary.".into()) }
                        },
                        sources: rsx! { Chip { icon: "globe", outline: true, variant: StatusVariant::Neutral, onclick: move |_| sources.toggle(), "Sources" } },
                        h3 { "Asset performance summary" }
                        p { "The compressor train is operating inside the expected range. Vibration increased slightly after the last shift change, but all values remain below the warning threshold." }
                        ul { li { "Temperature trend is stable over the last 24 hours." } li { "Two alarms were acknowledged by the operator." } li { "Next service window is scheduled for Friday." } }
                        if sources() { ul { aria_label: "Source files", li { "equipment_status.pdf" } li { "maintenance_notes.txt" } } }
                    }
                    ChatUserMessage { message: "Compare the current report with additional customer feedback.",
                        attachments: rsx! { ChatAttachment { file_name: "customer_feedback.pdf", icon: "pdf-document", hide_remove_button: true } } }
                    ChatAiMessage { p { "Customer feedback confirms the same maintenance priorities. The next step is to validate spare-part availability and confirm the planned downtime window." } }
                    for (index, exchange) in exchanges().into_iter().enumerate() {
                        Fragment { key: "exchange-{index}",
                            ChatUserMessage { message: exchange.prompt }
                            ChatAiMessage { streaming: exchange.streaming,
                                if exchange.response.is_empty() { p { "Waiting for the next local response chunk…" } } else { p { "{exchange.response}" } }
                            }
                        }
                    }
                }
            }
            div { class: "demo-row",
                Button { disabled: !processing, onclick: move |_| {
                    if let Some(exchange) = exchanges.write().last_mut() {
                        if let Some(chunk) = RESPONSE_CHUNKS.get(exchange.chunks) { exchange.response.push_str(chunk); exchange.chunks += 1; }
                        if exchange.chunks == RESPONSE_CHUNKS.len() { exchange.streaming = false; }
                    }
                }, "Append response chunk" }
                Button { variant: ButtonVariant::Secondary, disabled: !processing, onclick: move |_| {
                    if let Some(exchange) = exchanges.write().last_mut() { exchange.response = RESPONSE_CHUNKS.join(""); exchange.chunks = RESPONSE_CHUNKS.len(); exchange.streaming = false; }
                }, "Complete local response" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { exchanges.clear(); files.clear(); feedback.set("Local conversation reset.".into()); }, "Reset local conversation" }
            }
            p { class: "chat-example-feedback", role: "status", "{feedback}" }
        }
        section { class: "demo-section", id: "chat-ai-message",
            h2 { "AI message" }
            ChatAiMessage {
                h3 { "Recommended maintenance actions" }
                p { "The compressor is operating within the expected range. Schedule the next inspection during the Friday service window and keep monitoring the vibration trend." }
                ul { li { "Review the acknowledged alarms from the last shift." } li { "Confirm that spare parts are available before the service window." } li { "Share the summary with the maintenance lead." } }
            }
        }
        section { class: "demo-section", id: "chat-input",
            h2 { "Chat input" }
            ChatInputExample { onfeedback: move |value| feedback.set(value) }
        }
        section { class: "demo-section", id: "chat-attachment",
            h2 { "Chat attachment states" }
            div { class: "demo-row",
                ChatAttachment { file_name: "meeting_notes_summary_01.txt", preview_supported: true, onpreview: move |_| feedback.set("Attachment preview requested.".into()), onremove: move |_| feedback.set("Remove attachment requested.".into()) }
                ChatAttachment { file_name: "uploading_file.txt", status: ChatAttachmentStatus::Loading, hide_remove_button: true }
                ChatAttachment { file_name: "failed_file.txt", status: ChatAttachmentStatus::Failed, hide_remove_button: true }
                ChatAttachment { file_name: "Long File Name of chat attachment with many details.txt", hide_remove_button: true }
            }
        }
    }
}

#[component]
fn ChatInputExample(onfeedback: EventHandler<String>) -> Element {
    let mut value =
        use_signal(|| "Check open alarms and suggest the next maintenance step.".to_string());
    let mut attachment = use_signal(|| true);
    let mut soft = use_signal(|| "123456789".to_string());
    let mut hard = use_signal(|| "1234567890".to_string());
    rsx! { div { class: "chat-example-inputs",
        ChatInput { value: value(), placeholder: "Ask about asset status...", textarea_label: "Service assistant prompt", max_length: 180, character_limit: 180,
            onvaluechange: move |next| value.set(next), onpromptsubmit: move |prompt| onfeedback.call(format!("Prompt submitted: {prompt}")),
            attachments: rsx! { if attachment() { ChatAttachment { file_name: "shift_report.pdf", icon: "pdf-document", onremove: move |_| attachment.set(false) } } },
            start: rsx! { IconButton { icon: "plus", label: "Attach file", size: ButtonIconSize::TwentyFour, variant: ButtonVariant::SubtleTertiary, onclick: move |_| attachment.set(true) } },
            end: rsx! {
                IconButton { icon: "image", label: "Add image", size: ButtonIconSize::TwentyFour, variant: ButtonVariant::SubtleTertiary, onclick: move |_| onfeedback.call("Add image requested by the host action slot.".into()) }
                IconButton { icon: "microphone", label: "Record voice input", size: ButtonIconSize::TwentyFour, variant: ButtonVariant::SubtleTertiary, onclick: move |_| onfeedback.call("Voice input requested by the host action slot.".into()) }
            }
        }
        h3 { "Character limit feedback" }
        ChatInput { value: soft(), max_length: 10, onvaluechange: move |next| soft.set(next), disclaimer: "" }
        ChatInput { value: hard(), max_length: 10, onvaluechange: move |next| hard.set(next), disclaimer: "" }
        h3 { "Read-only, disabled, and processing" }
        ChatInput { value: "Read-only prompt", readonly: true, disclaimer: "" }
        ChatInput { value: "Disabled prompt", disabled: true, disclaimer: "" }
        ChatInput { value: "Processing prompt", state: ChatInputState::Processing, disclaimer: "" }
    } }
}

pub(crate) const STYLES: &str = r#"
.chat-example-conversation{max-width:784px;height:760px;max-height:85vh;min-height:480px;border:1px solid var(--ix-color-border);background:var(--ix-color-surface);margin-block:16px 24px}.chat-example-feedback{max-width:784px;overflow-wrap:anywhere}.chat-example-file-picker{position:relative;display:inline-flex;align-items:center;justify-content:center;width:32px;height:32px;color:var(--ix-color-text);border-radius:var(--ix-radius);cursor:pointer;overflow:hidden}.chat-example-file-picker:hover{background:var(--ix-color-surface-2)}.chat-example-file-picker input{position:absolute;inset:0;width:100%;height:100%;opacity:0;cursor:pointer}.chat-example-file-picker:focus-within{outline:2px solid var(--ix-color-focus);outline-offset:2px}.chat-example-edit{min-width:220px;white-space:normal}.chat-example-inputs{display:grid;gap:20px;max-width:720px}.chat-example-inputs h3{font-size:16px;margin:8px 0 0}@media(max-width:600px){.chat-example-conversation{height:640px;min-height:400px;max-height:85vh}}
"#;
