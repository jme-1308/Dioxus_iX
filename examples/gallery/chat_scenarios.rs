//! Individual chat source previews. Actions report host requests without pretending
//! to provide clipboard, file storage, or AI services.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const SCENARIOS: &[&str] = &["chat", "chat-input", "chat-ai-message", "chat-user-message"];

#[component]
pub fn ChatScenario(name: String) -> Element {
    let mut event = use_signal(|| "No action yet".to_string());
    let initial_prompt = if name == "chat-input" {
        "Check open alarms and suggest the next maintenance step."
    } else {
        "Create a concise follow-up plan for the service team."
    };
    let mut prompt = use_signal(|| initial_prompt.to_string());
    let mut attachment = use_signal(|| true);
    let full = name == "chat";
    let input_only = name == "chat-input";
    let user_only = name == "chat-user-message";
    let requested = use_callback(move |label: String| event.set(format!("{label} requested.")));
    let composer = if full || input_only {
        Some(rsx! {
            ChatInput { value: prompt(),
                placeholder: if input_only { "Ask about asset status..." } else { "Enter a command, question or topic..." },
                textarea_label: if input_only { "Service assistant prompt" } else { "Chat input" },
                max_length: if input_only { Some(180) } else { None },
                character_limit: if input_only { Some(180) } else { None },
                attachments: if input_only && attachment() { Some(rsx! { ChatAttachment { file_name: "shift_report.pdf", icon: "pdf-document", onremove: move |_| attachment.set(false) } }) } else { None },
                start: if input_only { Some(rsx! { IconButton { icon: "plus", label: "Attach file", variant: ButtonVariant::SubtleTertiary, onclick: move |_| requested.call("Attach file".into()) } }) } else { None },
                end: if input_only { Some(rsx! {
                    IconButton { icon: "image", label: "Add image", variant: ButtonVariant::SubtleTertiary, onclick: move |_| requested.call("Add image".into()) }
                    IconButton { icon: "microphone", label: "Record voice input", variant: ButtonVariant::SubtleTertiary, onclick: move |_| requested.call("Record voice input".into()) }
                }) } else { None },
                onvaluechange: move |value| prompt.set(value),
                onpromptsubmit: move |value| event.set(format!("Prompt submitted: {value}")),
            }
        })
    } else {
        None
    };
    rsx! {
        p { class: "source-chat-notice", "Demonstration content adapted from the upstream chat examples." }
        Chat { class: if input_only { "source-chat-preview source-chat-input-only" } else { "source-chat-preview" }, prompt: composer,
            if full || user_only { ChatUserMessage {
                message: if full { "Summarize the asset status and include the attached maintenance files." } else { "Summarize the attached shift report." },
                attachments: rsx! {
                    ChatAttachment { file_name: if full { "equipment_status.pdf" } else { "shift_report.pdf" }, icon: "pdf-document", hide_remove_button: true }
                    ChatAttachment { file_name: if full { "maintenance_notes.txt" } else { "operator_notes.txt" }, icon: "txt-document", hide_remove_button: true }
                },
                actions: rsx! { MessageActions { user: true, onaction: requested } },
            } }
            if full || name == "chat-ai-message" { ChatAiMessage {
                actions: rsx! { MessageActions { onaction: requested } },
                sources: rsx! { Chip { icon: "globe", outline: true, variant: StatusVariant::Neutral, "Sources" } },
                if full {
                    h3 { "Asset performance summary" }
                    p { "The compressor train is operating inside the expected range. Vibration increased slightly after the last shift change, but all values remain below the warning threshold." }
                    ul { li { "Temperature trend is stable over the last 24 hours." } li { "Two alarms were acknowledged by the operator." } li { "Next service window is scheduled for Friday." } }
                } else {
                    h3 { "Recommended maintenance actions" }
                    p { "The compressor is operating within the expected range. Schedule the next inspection during the Friday service window and keep monitoring the vibration trend." }
                    ul { li { "Review the acknowledged alarms from the last shift." } li { "Confirm that spare parts are available before the service window." } li { "Share the summary with the maintenance lead." } }
                }
            } }
            if full {
                ChatUserMessage { message: "Compare the current report with additional customer feedback.",
                    attachments: rsx! { ChatAttachment { file_name: "customer_feedback.pdf", icon: "pdf-document", hide_remove_button: true } },
                }
                ChatAiMessage { p { "Customer feedback confirms the same maintenance priorities. The next step is to validate spare-part availability and confirm the planned downtime window." } }
            }
        }
        p { role: "status", class: "source-event-feedback", "{event}" }
    }
}

#[component]
fn MessageActions(#[props(default)] user: bool, onaction: Callback<String>) -> Element {
    let actions = if user {
        vec![
            ("copy", "Copy message"),
            ("pen", "Edit message"),
            ("more-menu", "Show more actions"),
        ]
    } else {
        vec![
            ("copy", "Copy response"),
            ("thumb-up", "Helpful response"),
            ("thumb-down", "Not helpful response"),
            ("refresh", "Regenerate response"),
        ]
    };
    rsx! { for (icon, label) in actions { IconButton { icon, label, variant: ButtonVariant::SubtleTertiary, onclick: move |_| onaction.call(label.into()) } } }
}

pub const STYLES: &str = include_str!("styles/chat_scenarios.css");
