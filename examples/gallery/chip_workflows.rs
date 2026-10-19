//! Controlled asset chips and applied filters using Dioxus-owned composition.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::rc::Rc;

const ASSETS: [(&str, &str, &str); 3] = [
    ("asset-a", "Motor A", "Motor A"),
    ("asset-b", "Compressor B", "Compressor B"),
    (
        "asset-c",
        "Inspection notes",
        "Inspection notes for compressor commissioning and safety review",
    ),
];

#[component]
pub fn ChipWorkflowsExample() -> Element {
    let mut visible = use_signal(|| [true; 3]);
    let mut keep_closed = use_signal(|| false);
    let mut inspections = use_signal(|| 0usize);
    let mut removals = use_signal(|| 0usize);
    let mut last = use_signal(|| "none".to_string());
    let mut details = use_signal(|| "Select an asset chip to inspect its details.".to_string());
    let mut restore = use_signal(|| None::<Rc<MountedData>>);
    let mut filter_visible = use_signal(|| true);
    let mut filter_removals = use_signal(|| 0usize);
    let mut filter_restore = use_signal(|| None::<Rc<MountedData>>);
    // Key the actual list roots so removal preserves each surviving asset's
    // Dioxus identity; conditional fragment wrappers would make this list unkeyed.
    let visible_assets: Vec<_> = ASSETS
        .into_iter()
        .enumerate()
        .filter(|(index, _)| visible()[*index])
        .collect();
    let count = visible_assets.len();
    let statuses = [
        (StatusVariant::Primary, "Primary", "info"),
        (StatusVariant::Neutral, "Neutral", "info"),
        (StatusVariant::Warning, "Warning", "warning"),
        (StatusVariant::Info, "Info", "info"),
        (StatusVariant::Alarm, "Alarm", "alarm"),
        (StatusVariant::Critical, "Critical", "warning-rhomb"),
        (StatusVariant::Success, "Success", "success"),
    ];
    rsx! {
        section { class: "demo-section chip-workflows", id: "chip-workflows",
            h2 { "Pinned assets" }
            p { "Open an asset's details or remove its pin. Compressor B is unavailable for inspection; its pin can still be removed. These assets are illustrative." }
            div { class: "chip-workflow-assets", aria_label: "Pinned assets",
                for (index, (id, label, title)) in visible_assets {
                    div { key: "{id}", id, class: "chip-workflow-asset",
                        div { id: if index == 2 { Some("chip-long") } else { None },
                            class: if index == 2 { "chip-workflow-bounded" } else { "" },
                            Chip { variant: StatusVariant::Neutral, outline: true, icon: if index == 2 { "document" } else { "pin" },
                                inactive: index == 1, closable: true, class: if index == 2 { "chip-workflow-wide" } else { "" },
                                tooltip: title, aria_label: format!("Inspect {label}"), close_label: format!("Remove {label}"),
                                onclick: move |_| {
                                    inspections += 1; last.set(id.into());
                                    details.set(format!("Inspecting {label}. {}", match index {
                                        0 => "Motor A is ready for the next service window.",
                                        1 => "Compressor B is currently unavailable.",
                                        _ => "Commissioning and safety review notes are selected.",
                                    }));
                                },
                                onclose: move |_| {
                                    removals += 1; last.set(id.into());
                                    if keep_closed() {
                                        details.set(format!("Removal requested for {label}; the host kept the pin."));
                                    } else {
                                        visible.write()[index] = false;
                                        details.set(format!("Removed the pin for {label}. Restore pins to bring it back."));
                                        if let Some(node) = restore.peek().clone() {
                                            spawn(async move { let _ = node.set_focus(true).await; });
                                        }
                                    }
                                },
                                "{title}"
                            }
                        }
                    }
                }
            }
            div { class: "chip-workflow-controls",
                Button { id: "chip-restore", variant: ButtonVariant::Secondary,
                    onmounted: move |event: MountedEvent| restore.set(Some(event.data())),
                    onclick: move |_| { visible.set([true; 3]); details.set("All three asset pins restored.".into()); },
                    "Restore pins"
                }
                label { class: "chip-workflow-checkbox", r#for: "chip-keep-closed",
                    input { id: "chip-keep-closed", r#type: "checkbox", checked: keep_closed(),
                        oninput: move |event| keep_closed.set(event.checked())
                    }
                    "Keep pins after a remove request"
                }
            }
            p { id: "chip-workflow-details", class: "chip-workflow-details", "{details}" }
            p { id: "chip-workflow-status", role: "status", "data-count": count.to_string(),
                "data-inspections": inspections().to_string(), "data-removals": removals().to_string(), "data-last": last(),
                "{count} pinned assets · {inspections} inspections · {removals} remove requests"
            }
            h3 { "Applied filters" }
            p { "Remove the running-status filter independently. Read-only, disabled, and hidden-action filters do not offer a remove button." }
            div { class: "chip-workflow-filters", aria_label: "Applied filters",
                if filter_visible() {
                    span { id: "chip-filter-removable",
                        FilterChip { close_label: "Remove Status: Running", onclose: move |_| {
                            filter_visible.set(false); filter_removals += 1;
                            if let Some(node) = filter_restore.peek().clone() {
                                spawn(async move { let _ = node.set_focus(true).await; });
                            }
                        }, "Status: Running" }
                    }
                }
                span { id: "chip-filter-readonly", FilterChip { readonly: true, close_label: "Remove Line: 3", "Line: 3" } }
                span { id: "chip-filter-disabled", FilterChip { disabled: true, close_label: "Remove Locked filter", "Locked filter" } }
                span { id: "chip-filter-hidden", FilterChip { hide_close_button: true, close_label: "Remove Hidden action", "Hidden action" } }
            }
            div { class: "chip-workflow-controls",
                Button { id: "chip-filter-restore", variant: ButtonVariant::Secondary,
                    onmounted: move |event: MountedEvent| filter_restore.set(Some(event.data())),
                    onclick: move |_| filter_visible.set(true), "Restore running filter"
                }
            }
            p { id: "chip-filter-status", role: "status", "data-present": filter_visible().to_string(), "data-removals": filter_removals().to_string(),
                if filter_visible() { "Running-status filter applied. " } else { "Running-status filter removed. " }
                "{filter_removals} filter remove requests"
            }
            h3 { "Centered content" }
            div { id: "chip-centered", class: "chip-workflow-bounded",
                Chip { variant: StatusVariant::Primary, outline: true, center_content: true, icon: "check", class: "chip-workflow-wide", "Ready for service" }
            }
            p { class: "chip-workflow-note", "The notes pin above truncates its long label while preserving the independent remove button. Centered chips retain the same compact height." }
        }
        section { class: "demo-section chip-workflows", id: "chip-status-preview",
            h2 { "Chip colors" }
            p { "Filled and outlined treatments use the same status palette. Custom chips keep their application-supplied colors." }
            h3 { "Filled" }
            div { class: "chip-workflow-palette", aria_label: "Filled chip colors",
                for (variant, label, icon) in statuses {
                    span { id: "chip-filled-{variant.as_str()}", Chip { variant, icon, "{label}" } }
                }
                span { id: "chip-filled-custom", Chip { variant: StatusVariant::Custom, icon: "info",
                    background: "var(--ix-color-primary)", color: "var(--ix-color-on-primary)", "Custom" }
                }
            }
            h3 { "Outlined" }
            div { class: "chip-workflow-palette", aria_label: "Outlined chip colors",
                for (variant, label, icon) in statuses {
                    span { id: "chip-outline-{variant.as_str()}", Chip { variant, outline: true, icon, "{label}" } }
                }
                span { id: "chip-outline-custom", Chip { variant: StatusVariant::Custom, outline: true, icon: "info",
                    background: "var(--ix-color-primary)", color: "var(--ix-color-text)", "Custom" }
                }
            }
        }
    }
}

pub const STYLES: &str = r#"
.chip-workflows>p{max-width:72ch;line-height:1.5}
.chip-workflow-assets,.chip-workflow-filters,.chip-workflow-palette{display:flex;align-items:center;flex-wrap:wrap;gap:12px;margin:16px 0}
.chip-workflow-asset{min-width:0;max-width:100%}
.chip-workflow-bounded{box-sizing:border-box;width:260px;max-width:100%;min-width:0}
.chip-workflows .chip-workflow-wide{width:100%}
.chip-workflow-controls{display:flex;align-items:center;flex-wrap:wrap;gap:16px;margin:20px 0 12px}
.chip-workflow-checkbox{display:inline-flex;align-items:center;gap:8px;max-width:100%;font-size:14px;cursor:pointer}
.chip-workflow-checkbox input{width:16px;height:16px;margin:0;flex:none;accent-color:var(--ix-color-primary)}
.chip-workflow-checkbox input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
.chip-workflow-details{margin:16px 0 8px;font-size:14px;color:var(--ix-color-text)}
.chip-workflows #chip-workflow-status,.chip-workflows #chip-filter-status,.chip-workflow-note{font-size:13px;color:var(--ix-color-text-secondary)}
.chip-workflows h3{margin:28px 0 12px}
.chip-workflow-palette{max-width:960px}
@media(max-width:420px){.chip-workflow-controls{align-items:flex-start;gap:12px}.chip-workflow-controls .ix-button{max-width:100%}}
"#;
