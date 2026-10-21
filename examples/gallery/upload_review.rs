//! Native local file/folder selection and resource-owned UTF-8 inspection.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const STYLES: &str = include_str!("styles/upload_review.css");

#[component]
pub fn UploadReview() -> Element {
    let mut entries = use_signal(Vec::<UploadEntry>::new);
    let mut callbacks = use_signal(|| 0_usize);
    let mut source = use_signal(|| "none");
    let mut multiple = use_signal(|| true);
    let mut disabled = use_signal(|| false);
    let mut selection_error = use_signal(|| None::<String>);
    let mut requested = use_signal(|| None::<dioxus::html::FileData>);
    let mut reader = use_resource(move || {
        let file = requested();
        async move {
            let file = file?;
            let result = match file.read_bytes().await {
                Ok(bytes) => String::from_utf8(bytes.to_vec()).map_err(|_| {
                    "This file is not valid UTF-8 text. Choose a text file and try again."
                        .to_string()
                }),
                Err(error) => Err(format!("Could not read this local file: {error}")),
            };
            Some((file, result))
        }
    });
    let current = entries();
    let request = requested();
    // A resource retains its previous result while restarting. Only the current
    // file's completed result is displayed; clearing the request is idle.
    let result = reader
        .read()
        .as_ref()
        .and_then(|value| value.as_ref())
        .filter(|(file, _)| request.as_ref() == Some(file))
        .map(|(_, result)| result.clone());
    let pending = request.is_some()
        && (*reader.state().read() == UseResourceState::Pending || result.is_none());
    let read_state = if request.is_none() {
        "idle"
    } else if pending {
        "loading"
    } else {
        match &result {
            Some(Ok(_)) => "success",
            Some(Err(_)) => "failed",
            None => "loading",
        }
    };
    let content = result
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    let error = result
        .as_ref()
        .and_then(|result| result.as_ref().err())
        .cloned();
    let read_name = request.as_ref().map(|file| file.name()).unwrap_or_default();
    rsx! {
        style { {STYLES} }
        section { id: "upload-workflow", class: "demo-section upload-workflow",
            h2 { "Inspect local files" }
            p { class: "upload-workflow-intro", "Choose text files or a folder, inspect the selection, then read its first file as UTF-8 text. Files stay on this computer; nothing is sent to a server." }
            div { class: "upload-workflow-options",
                label { input { id: "upload-multiple", r#type: "checkbox", checked: multiple(), onchange: move |event: FormEvent| multiple.set(event.checked()) } "Select multiple text files" }
                label { input { id: "upload-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable selection" }
            }
            div { class: "upload-workflow-fields",
                div { id: "upload-text-files",
                    h3 { "Text files" }
                    Upload { name: "inspection-files", aria_label: "Choose text files", accept: ".txt,.md,text/plain", multiple: multiple(), disabled: disabled(),
                        select_file_text: "Choose local text files", button_text: "Choose files", disabled_text: "Text file selection is disabled.",
                        onentrieschanged: move |selected: Vec<UploadEntry>| {
                            reader.clear(); requested.set(None); entries.set(selected); callbacks += 1; source.set("files"); selection_error.set(None);
                        },
                        onerror: move |error: UploadError| selection_error.set(Some(error.to_string())),
                    }
                }
                div { id: "upload-folder",
                    h3 { "Folder" }
                    Upload { name: "inspection-folder", aria_label: "Choose folder", directory_upload: true, disabled: disabled(),
                        select_file_text: "Choose a folder to inspect", button_text: "Choose folder", disabled_text: "Folder selection is disabled.",
                        onentrieschanged: move |selected: Vec<UploadEntry>| {
                            reader.clear(); requested.set(None); entries.set(selected); callbacks += 1; source.set("folder"); selection_error.set(None);
                        },
                        onerror: move |error: UploadError| selection_error.set(Some(error.to_string())),
                    }
                }
            }
            div { class: "upload-workflow-actions",
                Button { id: "upload-read-first", button_type: ButtonType::Button, disabled: current.is_empty() || pending,
                    onclick: move |_| { reader.clear(); requested.set(entries.peek().first().map(|entry| entry.file.clone())); }, "Read first file"
                }
                Button { id: "upload-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                    onclick: move |_| { reader.clear(); requested.set(None); entries.clear(); source.set("none"); selection_error.set(None); }, "Reset selection"
                }
            }
            p { id: "upload-status", class: "upload-workflow-status", role: "status", "aria-live": "polite",
                "data-count": current.len(), "data-callbacks": callbacks(), "data-source": source(), "data-read-state": read_state, "data-read-name": read_name.clone(),
                "{current.len()} file(s) selected · {callbacks} selection update(s). "
                match read_state {
                    "loading" => format!("Reading {read_name} locally…"),
                    "success" => format!("Read {read_name}."),
                    "failed" => format!("Could not read {read_name}."),
                    _ => "Choose files or a folder, then read the first file.".to_string(),
                }
            }
            if let Some(message) = selection_error() { p { id: "upload-selection-error", role: "alert", "{message}" } }
            if let Some(message) = error { p { id: "upload-read-error", role: "alert", "{message}" } }
            ul { id: "upload-selected", class: "upload-workflow-selection",
                for (index, entry) in current.iter().enumerate() {
                    li { key: "{index}", "data-name": entry.file.name(), "data-path": entry.relative_path.to_string_lossy().to_string(), "data-size": entry.file.size(),
                        strong { "{entry.file.name()}" }
                        span { "{entry.relative_path.display()} · {entry.file.size()} bytes" }
                    }
                }
            }
            Textarea { id: "upload-content", label: "File contents", value: content, readonly: true, rows: 8, width: "100%", resize: "vertical", class: "upload-workflow-content" }
        }
    }
}
