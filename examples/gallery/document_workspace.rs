//! Retained inspection-document drafts with owner-controlled close decisions.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
struct InspectionDocument {
    key: String,
    title: String,
    draft: String,
    saved: String,
    reference: bool,
}

fn document(key: &str, title: &str, text: &str, reference: bool) -> InspectionDocument {
    InspectionDocument {
        key: key.into(),
        title: title.into(),
        draft: text.into(),
        saved: text.into(),
        reference,
    }
}

fn initial_documents() -> Vec<InspectionDocument> {
    vec![
        document("motor", "Motor 001", "Inspect bearing housing", false),
        document("pump", "Pump 002", "Check inlet pressure", false),
        document(
            "reference",
            "Inspection reference",
            "Use equipment-specific maintenance instructions",
            true,
        ),
    ]
}

#[component]
fn DocumentPreparation() -> Element {
    let mut note = use_signal(|| "Prepare inspection documents".to_string());
    rsx! {
        label { class: "documents-preparation", r#for: "documents-preparation",
            span { "Inspection preparation" }
            input { id: "documents-preparation", r#type: "text", value: note(), maxlength: "160",
                oninput: move |event: FormEvent| note.set(event.value()),
            }
        }
    }
}

/// Edit illustrative local documents without losing drafts when switching tabs.
#[component]
pub fn DocumentWorkspaceExample() -> Element {
    let mut documents = use_signal(initial_documents);
    let mut selected = use_signal(|| Some("motor".to_string()));
    let mut pending_close = use_signal(|| None::<String>);
    let mut manual = use_signal(|| false);
    let mut top = use_signal(|| false);
    let mut all_tabs = use_signal(|| false);
    let mut german = use_signal(|| false);
    let mut icon_only = use_signal(|| false);
    let mut disable_reference = use_signal(|| false);
    let mut next_id = use_signal(|| 4_usize);
    let mut saves = use_signal(|| 0_usize);
    let mut closes = use_signal(|| 0_usize);
    let mut additions = use_signal(|| 0_usize);
    let mut close_requests = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    // Explicit references to this application's known fields, never DOM discovery.
    let mut fields = use_signal(BTreeMap::<String, Rc<MountedData>>::new);
    let mut add_button = use_signal(|| None::<Rc<MountedData>>);
    let mut focus_request = use_signal(|| None::<String>);
    use_effect(move || {
        let wanted = focus_request();
        let selection = selected();
        let mounted = fields();
        let toolbar = add_button();
        if let Some(key) = wanted {
            if key == "__toolbar" {
                if let Some(node) = toolbar {
                    focus_request.set(None);
                    spawn(async move {
                        if selected.peek().is_none() {
                            let _ = node.set_focus(true).await;
                        }
                    });
                }
            } else if selection.as_deref() == Some(key.as_str()) {
                if let Some(node) = mounted.get(&key).cloned() {
                    focus_request.set(None);
                    spawn(async move {
                        if selected.peek().as_deref() == Some(key.as_str()) {
                            let _ = node.set_focus(true).await;
                        }
                    });
                }
            }
        }
    });

    let remove = use_callback(move |key: String| {
        let mut next = documents.peek().clone();
        let Some(index) = next.iter().position(|document| document.key == key) else {
            return;
        };
        if next[index].reference && *disable_reference.peek() {
            return;
        }
        next.remove(index);
        let current = selected.peek().clone();
        let enabled =
            |document: &&InspectionDocument| !(document.reference && *disable_reference.peek());
        let selection = current
            .filter(|current| {
                *current != key
                    && next.iter().any(|document| {
                        document.key == *current
                            && !(document.reference && *disable_reference.peek())
                    })
            })
            .or_else(|| {
                next.iter()
                    .skip(index)
                    .filter(enabled)
                    .chain(next.iter().take(index).rev().filter(enabled))
                    .next()
                    .map(|document| document.key.clone())
            });
        documents.set(next);
        selected.set(selection.clone());
        pending_close.set(None);
        fields.write().remove(&key);
        focus_request.set(Some(selection.unwrap_or_else(|| "__toolbar".into())));
        let count = closes.peek().saturating_add(1);
        closes.set(count);
    });
    let add = use_callback(move |_| {
        let index = *next_id.peek();
        let key = format!("note-{index}");
        let mut next = documents.peek().clone();
        next.push(document(
            &key,
            &format!("Inspection note {index}"),
            "New inspection note",
            false,
        ));
        documents.set(next);
        selected.set(Some(key.clone()));
        focus_request.set(Some(key));
        next_id.set(index + 1);
        let count = additions.peek().saturating_add(1);
        additions.set(count);
    });
    let save = use_callback(move |key: String| {
        let mut next = documents.peek().clone();
        if let Some(document) = next.iter_mut().find(|document| {
            document.key == key && !(document.reference && *disable_reference.peek())
        }) {
            if document.saved != document.draft {
                document.saved = document.draft.clone();
                documents.set(next);
                let count = saves.peek().saturating_add(1);
                saves.set(count);
            }
        }
    });
    let current_documents = documents();
    let items: Vec<_> = current_documents
        .iter()
        .map(|document| {
            let mut item = TabItem::new(&document.key, &document.title);
            item.closable = true;
            item.disabled = document.reference && disable_reference();
            item.icon_only = icon_only();
            item.icon = icon_only().then(|| {
                (if document.reference {
                    "info"
                } else {
                    "document"
                })
                .to_string()
            });
            item
        })
        .collect();
    let enabled_count = items.iter().filter(|item| !item.disabled).count();
    let dirty_count = current_documents
        .iter()
        .filter(|document| document.draft != document.saved)
        .count();
    let order = current_documents
        .iter()
        .map(|document| document.key.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let pending_document = pending_close().and_then(|key| {
        current_documents
            .iter()
            .find(|document| document.key == key)
            .cloned()
    });
    let samples_missing = (1..=8).any(|index| {
        !current_documents
            .iter()
            .any(|document| document.key == format!("sample-{index}"))
    });

    rsx! {
        section { id: "document-workspace", class: "document-workspace-example",
            h2 { "Inspection documents" }
            p { class: "documents-intro", "Edit illustrative local inspection documents and save the text you want to keep. Switching or reordering tabs preserves drafts. Closing unsaved work asks for a decision; documents are not written to local files." }
            div { class: "documents-toolbar",
                Button { id: "documents-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        documents.set(initial_documents()); selected.set(Some("motor".into())); pending_close.set(None);
                        manual.set(false); top.set(false); all_tabs.set(false); german.set(false); disable_reference.set(false);
                        icon_only.set(false);
                        next_id.set(4); saves.set(0); closes.set(0); additions.set(0); close_requests.set(0);
                        focus_request.set(None); fields.write().clear(); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore documents"
                }
                button { id: "documents-add", class: "ix-button ix-button-secondary", r#type: "button",
                    onmounted: move |event: MountedEvent| add_button.set(Some(event.data())), onclick: move |_| add.call(()), "Add document"
                }
                button { id: "documents-samples", class: "ix-button ix-button-secondary", r#type: "button", "aria-disabled": (!samples_missing).to_string(),
                    onclick: move |_| {
                        let mut next = documents.peek().clone(); let mut added = 0;
                        for index in 1..=8 {
                            let key = format!("sample-{index}");
                            if !next.iter().any(|document| document.key == key) {
                                next.push(document(&key, &format!("Sample inspection {index}"), &format!("Illustrative inspection reference {index}"), false)); added += 1;
                            }
                        }
                        if added > 0 { documents.set(next); let count = additions.peek().saturating_add(added); additions.set(count); }
                    }, "Open sample documents"
                }
                Button { id: "documents-reverse", variant: ButtonVariant::Secondary,
                    onclick: move |_| { let mut next = documents.peek().clone(); next.reverse(); documents.set(next); }, "Reverse documents"
                }
            }
            details { id: "documents-options", class: "documents-options",
                summary { "Workspace options" }
                div { class: "documents-option-fields",
                    label { input { id: "documents-icon-only", r#type: "checkbox", checked: icon_only(), onchange: move |event: FormEvent| icon_only.set(event.checked()) } "Icon-only document tabs" }
                    label { input { id: "documents-manual", r#type: "checkbox", checked: manual(), onchange: move |event: FormEvent| manual.set(event.checked()) } "Manual tab activation" }
                    label { input { id: "documents-top", r#type: "checkbox", checked: top(), onchange: move |event: FormEvent| top.set(event.checked()) } "Top indicator" }
                    label { input { id: "documents-all-tabs", r#type: "checkbox", checked: all_tabs(), onchange: move |event: FormEvent| all_tabs.set(event.checked()) } "Always show all-documents menu" }
                    label { input { id: "documents-german", r#type: "checkbox", checked: german(), onchange: move |event: FormEvent| german.set(event.checked()) } "German tab labels" }
                    label { input { id: "documents-disable-reference", r#type: "checkbox", checked: disable_reference(),
                        onchange: move |event: FormEvent| {
                            let disabled = event.checked(); disable_reference.set(disabled);
                            if disabled && selected.peek().as_deref() == Some("reference") {
                                let first = documents.peek().iter().find(|document| !document.reference).map(|document| document.key.clone()); selected.set(first);
                            }
                        },
                    } "Disable reference tab" }
                }
            }
            for generation in [epoch()] { DocumentPreparation { key: "document-preparation-{generation}" } }
            div { id: "documents-tabs", class: "documents-tabs",
                for generation in [epoch()] {
                    TabSet { key: "document-tabs-{generation}", items: items.clone(), selection: selected().map_or(TabKeySelection::Empty, TabKeySelection::Key), label: "Inspection documents",
                        keyboard_navigation: if manual() { TabKeyboardNavigation::Manual } else { TabKeyboardNavigation::Automatic },
                        placement: if top() { TabPlacement::Top } else { TabPlacement::Bottom }, overflow_menu: all_tabs(),
                        aria_label_more_tabs: if german() { "Alle Dokumente" } else { "Show all documents" },
                        close_label: move |item: TabItem| if german() { format!("Dokument {} schließen", item.label) } else { format!("Close {}", item.label) },
                        onselectionchange: move |key| selected.set(key),
                        onclose: move |key: String| {
                            let current = documents.peek().iter().find(|document| document.key == key).cloned();
                            if let Some(document) = current.filter(|document| !(document.reference && *disable_reference.peek())) {
                                let count = close_requests.peek().saturating_add(1); close_requests.set(count);
                                if document.draft != document.saved { pending_close.set(Some(key)); } else { remove.call(key); }
                            }
                        },
                        for document in &current_documents {
                            TabPanel { key: "{document.key}", tab_key: document.key.clone(), class: "document-editor-panel",
                                div { id: format!("document-editor-{}", document.key), class: "document-editor",
                                    label { r#for: format!("document-draft-{}", document.key),
                                        span { "Document text for {document.title}" }
                                        textarea { id: format!("document-draft-{}", document.key), value: document.draft.clone(), rows: "4", maxlength: "2000",
                                            disabled: document.reference && disable_reference(),
                                            onmounted: { let key = document.key.clone(); move |event: MountedEvent| { fields.write().insert(key.clone(), event.data()); } },
                                            oninput: { let key = document.key.clone(); move |event: FormEvent| {
                                                let mut next = documents.peek().clone();
                                                if let Some(document) = next.iter_mut().find(|document| document.key == key && !(document.reference && *disable_reference.peek())) {
                                                    document.draft = event.value(); documents.set(next);
                                                }
                                            } },
                                        }
                                    }
                                    div { class: "document-editor-actions",
                                        button { id: format!("document-save-{}", document.key), class: "ix-button ix-button-primary", r#type: "button",
                                            "aria-disabled": (document.draft == document.saved || (document.reference && disable_reference())).to_string(),
                                            onclick: { let key = document.key.clone(); move |_| save.call(key.clone()) }, "Save document"
                                        }
                                        span { id: format!("document-status-{}", document.key), if document.draft == document.saved { "Saved" } else { "Unsaved changes" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(document) = pending_document {
                div { id: "documents-close-decision", class: "documents-close-decision", role: "status",
                    p { "Discard unsaved changes to {document.title} and close it?" }
                    div { class: "documents-decision-actions",
                        Button { id: "documents-keep", variant: ButtonVariant::Secondary,
                            onclick: { let key = document.key.clone(); move |_| { pending_close.set(None); selected.set(Some(key.clone())); focus_request.set(Some(key.clone())); } }, "Keep document"
                        }
                        Button { id: "documents-discard", variant: ButtonVariant::DangerPrimary,
                            onclick: move |_| remove.call(document.key.clone()), "Discard and close"
                        }
                    }
                }
            }
            if enabled_count == 0 {
                div { id: "documents-empty", EmptyState { header: "No editable documents", subheader: "Add a document to continue the local inspection workspace.", action: "Add document", onaction: add, layout: EmptyStateLayout::CompactBreak } }
            }
            table { id: "documents-saved", class: "documents-saved",
                caption { "Saved document text" }
                thead { tr { th { scope: "col", "Document" } th { scope: "col", "Saved text" } } }
                tbody {
                    for document in &current_documents {
                        tr { key: "{document.key}", id: format!("document-saved-{}", document.key), "data-document": document.key.clone(),
                            th { scope: "row", "data-field": "title", "{document.title}" } td { "data-field": "saved", "{document.saved}" }
                        }
                    }
                }
            }
            output { id: "documents-result", class: "documents-result", role: "status",
                "data-selected": selected().unwrap_or_else(|| "none".into()), "data-count": current_documents.len().to_string(), "data-dirty": dirty_count.to_string(),
                "data-pending-close": pending_close().unwrap_or_else(|| "none".into()), "data-saves": saves().to_string(), "data-closes": closes().to_string(),
                "data-close-requests": close_requests().to_string(), "data-additions": additions().to_string(), "data-order": order,
                "data-manual": manual().to_string(), "data-top": top().to_string(), "data-all-tabs": all_tabs().to_string(), "data-german": german().to_string(), "data-disable-reference": disable_reference().to_string(),
                "{current_documents.len()} documents · {dirty_count} unsaved · Saved changes: {saves} · Closed documents: {closes}"
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/document_workspace.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn document_workspace_renders_retained_native_panels_and_actual_saved_records() {
        let mut dom = VirtualDom::new(DocumentWorkspaceExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for key in ["motor", "pump", "reference"] {
            assert!(
                html.contains(&format!("id=\"document-draft-{key}\""))
                    && html.contains(&format!("id=\"document-saved-{key}\""))
            );
        }
        assert_eq!(html.matches("role=\"tabpanel\"").count(), 3);
        assert!(html.matches("hidden=true").count() + html.matches("hidden=\"true\"").count() >= 2);
        assert!(
            html.contains("Inspect bearing housing")
                && html.contains("Check inlet pressure")
                && html.contains("Use equipment-specific maintenance instructions")
        );
        assert!(
            html.contains("Prepare inspection documents") && html.contains("Inspection documents")
        );
        assert!(
            html.contains("data-selected=\"motor\"")
                && html.contains("data-count=\"3\"")
                && html.contains("data-dirty=\"0\"")
        );
        assert!(html.contains("Close Motor 001") && html.contains("Close Pump 002"));
    }
}
