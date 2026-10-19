//! A local, controlled composer workflow using the public chat components.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const WRAPPED_PROMPT: &str = "Review the maintenance notes for the production line and summarize the open alarms, recent inspections, available spare parts, and recommended next actions before the next scheduled service window. Keep the report concise.";

#[derive(Clone, Copy, PartialEq)]
struct ExampleAttachment {
    id: &'static str,
    name: &'static str,
    status: ChatAttachmentStatus,
}

fn illustrative_files() -> Vec<ExampleAttachment> {
    vec![
        ExampleAttachment {
            id: "notes",
            name: "inspection_notes.txt",
            status: ChatAttachmentStatus::Default,
        },
        ExampleAttachment {
            id: "uploading",
            name: "uploading_log.txt",
            status: ChatAttachmentStatus::Loading,
        },
        ExampleAttachment {
            id: "failed",
            name: "failed_report.pdf",
            status: ChatAttachmentStatus::Failed,
        },
    ]
}

#[component]
pub fn ChatComposerExample() -> Element {
    let mut prompt = use_signal(String::new);
    let mut mode = use_signal(|| "input".to_string());
    let mut narrow = use_signal(|| false);
    let mut newline = use_signal(|| false);
    let mut files = use_signal(illustrative_files);
    let mut submit_count = use_signal(|| 0usize);
    let mut last_submit = use_signal(String::new);
    let mut change_count = use_signal(|| 0usize);
    let mut blur_count = use_signal(|| 0usize);
    let mut preview_count = use_signal(|| 0usize);
    let mut remove_count = use_signal(|| 0usize);
    let mut attachment_feedback =
        use_signal(|| "Preview the notes or remove an attachment.".to_string());
    let width = if narrow() { 280 } else { 620 };
    let file_count = files.read().len();
    rsx! {
        section {
            id: "chat-composer-workflow",
            class: "demo-section chat-composer-workflow",
            style: "width:{width}px;max-width:100%",
            h3 { "Compose a prompt" }
            p { "Try a wrapped prompt, multiple lines, and attachment actions. These illustrative files are local examples; no upload or chat service is connected." }
            div { class: "demo-row",
                Button {
                    id: "chat-composer-fill-wrap",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| prompt.set(WRAPPED_PROMPT.into()),
                    "Load wrapped prompt"
                }
                Button {
                    id: "chat-composer-fill-lines",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| prompt.set((1..=12).map(|line| format!("Inspection note {line}")).collect::<Vec<_>>().join("\n")),
                    "Load 12 lines"
                }
                Button {
                    id: "chat-composer-clear",
                    variant: ButtonVariant::Tertiary,
                    onclick: move |_| prompt.set(String::new()),
                    "Clear prompt"
                }
                Button {
                    id: "chat-composer-toggle-width",
                    variant: ButtonVariant::Tertiary,
                    onclick: move |_| narrow.toggle(),
                    if narrow() { "Use wider composer" } else { "Use narrow composer" }
                }
            }
            div { class: "demo-row chat-composer-options",
                label {
                    r#for: "chat-composer-mode",
                    "Composer state "
                    select {
                        id: "chat-composer-mode",
                        value: mode(),
                        onchange: move |event| mode.set(event.value()),
                        option { value: "input", "Input" }
                        option { value: "readonly", "Read-only" }
                        option { value: "disabled", "Disabled" }
                        option { value: "processing", "Processing" }
                    }
                }
                label {
                    input {
                        id: "chat-composer-toggle-newline",
                        r#type: "checkbox",
                        checked: newline(),
                        onchange: move |event| newline.set(event.checked()),
                    }
                    " Enter inserts a new line"
                }
            }
            ChatInput {
                id: "chat-composer-prompt",
                name: "workflow-prompt",
                textarea_label: "Workflow prompt",
                placeholder: "Ask about the next maintenance step…",
                value: prompt(),
                min_rows: 2,
                max_rows: 5,
                max_length: 240,
                readonly: mode() == "readonly",
                disabled: mode() == "disabled",
                state: if mode() == "processing" { ChatInputState::Processing } else { ChatInputState::Input },
                insert_line_break_on_enter: newline(),
                disclaimer: "Local demonstration. Submitting keeps your prompt so you can try it again.",
                onvaluechange: move |value| prompt.set(value),
                onpromptsubmit: move |value| { submit_count += 1; last_submit.set(value); },
                onchange: move |_| change_count += 1,
                onblur: move |_| blur_count += 1,
                attachments: rsx! {
                    for file in files() {
                        ChatAttachment {
                            key: "{file.id}",
                            file_name: file.name,
                            status: file.status,
                            preview_supported: true,
                            remove_label: format!("Remove {}", file.name),
                            onpreview: move |_| {
                                preview_count += 1;
                                attachment_feedback.set(format!("Preview requested for {}. A host application would open the file.", file.name));
                            },
                            onremove: move |_| {
                                files.write().retain(|entry| entry.id != file.id);
                                remove_count += 1;
                                attachment_feedback.set(format!("Removed {} from this prompt.", file.name));
                            },
                        }
                    }
                },
                start: rsx! {
                    Button {
                        id: "chat-composer-restore-files",
                        variant: ButtonVariant::SubtleSecondary,
                        onclick: move |_| {
                            files.set(illustrative_files());
                            attachment_feedback.set("Restored all three illustrative files.".into());
                        },
                        "Restore files"
                    }
                },
                follow_up: rsx! {
                    Button {
                        id: "chat-composer-follow-up",
                        variant: ButtonVariant::SubtleSecondary,
                        onclick: move |_| prompt.set("Suggest the next maintenance step.".into()),
                        "Suggest the next maintenance step"
                    }
                },
            }
            p { "{attachment_feedback}" }
            p {
                id: "chat-composer-status",
                role: "status",
                "data-submit-count": submit_count(),
                "data-last-submit": last_submit(),
                "data-change-count": change_count(),
                "data-blur-count": blur_count(),
                "data-preview-count": preview_count(),
                "data-remove-count": remove_count(),
                "data-files": file_count,
                "data-narrow": narrow(),
                "data-newline": newline(),
                "data-mode": mode(),
                "Submitted {submit_count} · Changes committed {change_count} · Blurs {blur_count} · Previews {preview_count} · Removed {remove_count} · Files {file_count}"
            }
            if !last_submit().is_empty() {
                p { class: "chat-composer-last-submit", "Last submitted: {last_submit}" }
            }
        }
    }
}

pub const STYLES: &str = r#"
.chat-composer-workflow{min-width:0}.chat-composer-workflow>.demo-row{margin-block:12px}.chat-composer-options label{display:inline-flex;align-items:center;gap:8px}.chat-composer-workflow>p{overflow-wrap:anywhere}.chat-composer-last-submit{white-space:pre-wrap}
"#;
