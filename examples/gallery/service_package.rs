//! Illustrative service package with retained disclosure fields and independent actions.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn PlannedPartsFields() -> Element {
    let mut prepared = use_signal(|| false);
    let mut note = use_signal(|| "Stage seal kit beside Motor 001".to_string());
    rsx! {
        div { class: "service-package-fields",
            p { class: "service-package-body-copy", "Prepare the replacement seal kit and bearing grease for the monthly inspection." }
            label { class: "service-package-check", r#for: "package-seal-kit",
                input { id: "package-seal-kit", r#type: "checkbox", checked: prepared(), onchange: move |event: FormEvent| prepared.set(event.checked()) }
                "Seal kit prepared"
            }
            label { class: "service-package-field", r#for: "package-parts-note",
                span { "Parts staging note" }
                input { id: "package-parts-note", class: "service-package-input", r#type: "text", value: note(), oninput: move |event: FormEvent| note.set(event.value()) }
            }
        }
    }
}

#[component]
fn ReadinessFields() -> Element {
    let mut permit = use_signal(|| false);
    let mut note = use_signal(|| "Review permit before service".to_string());
    rsx! {
        div { class: "service-package-fields",
            p { class: "service-package-body-copy", "Review the permit and agree the isolation plan before starting maintenance." }
            label { class: "service-package-check", r#for: "package-permit",
                input { id: "package-permit", r#type: "checkbox", checked: permit(), onchange: move |event: FormEvent| permit.set(event.checked()) }
                "Permit reviewed"
            }
            label { class: "service-package-field", r#for: "package-readiness-note",
                span { "Readiness note" }
                input { id: "package-readiness-note", class: "service-package-input", r#type: "text", value: note(), oninput: move |event: FormEvent| note.set(event.value()) }
            }
        }
    }
}

fn section_variant(value: &str) -> CardVariant {
    match value {
        "filled" => CardVariant::Filled,
        "warning" => CardVariant::Warning,
        "primary" => CardVariant::Primary,
        _ => CardVariant::Outline,
    }
}

fn section_variant_value(variant: CardVariant) -> &'static str {
    match variant {
        CardVariant::Filled => "filled",
        CardVariant::Warning => "warning",
        CardVariant::Primary => "primary",
        _ => "outline",
    }
}

/// Retained service-package notes with controlled parts and local readiness state.
#[component]
pub fn ServicePackageExample() -> Element {
    let mut parts_open = use_signal(|| false);
    let mut hold_parts = use_signal(|| false);
    let mut custom_heading = use_signal(|| true);
    let mut reveal_parts = use_signal(|| true);
    let mut variant = use_signal(|| CardVariant::Outline);
    let mut selected = use_signal(|| false);
    let mut parts_requests = use_signal(|| 0_usize);
    let mut selections = use_signal(|| 0_usize);
    let mut inspections = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let header = custom_heading().then(|| {
        rsx! {
            span { class: "service-package-custom-heading",
                strong { "Motor 001 readiness" }
                span { "Permit and technician checklist" }
            }
        }
    });

    rsx! {
        section { id: "service-package", class: "service-package-example",
            h2 { "Motor 001 service package" }
            p { class: "service-package-intro",
                "Plan an illustrative monthly inspection using local notes and checklists. Closing a section keeps its entries intact. "
                "Package selection, section controls and the inspection action work independently; Restore service package clears this example."
            }
            div { class: "service-package-toolbar",
                Button { id: "package-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        parts_open.set(false); hold_parts.set(false); custom_heading.set(true);
                        reveal_parts.set(true); variant.set(CardVariant::Outline); selected.set(false);
                        parts_requests.set(0); selections.set(0); inspections.set(0);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    },
                    "Restore service package"
                }
                Button { id: "package-toggle-parts", variant: ButtonVariant::Secondary,
                    onclick: move |_| parts_open.toggle(), "Toggle parts externally"
                }
                label { class: "service-package-option", r#for: "package-variant",
                    span { "Section appearance" }
                    select { id: "package-variant", class: "service-package-select", value: section_variant_value(variant()),
                        onchange: move |event: FormEvent| variant.set(section_variant(&event.value())),
                        for (value, text) in [("outline", "Outline"), ("filled", "Filled"), ("warning", "Warning"), ("primary", "Primary")] {
                            option { value, selected: section_variant_value(variant()) == value, "{text}" }
                        }
                    }
                }
            }
            div { class: "service-package-options",
                label { class: "service-package-check", r#for: "package-hold",
                    input { id: "package-hold", r#type: "checkbox", checked: hold_parts(), onchange: move |event: FormEvent| hold_parts.set(event.checked()) }
                    "Hold parts changes"
                }
                label { class: "service-package-check", r#for: "package-custom-header",
                    input { id: "package-custom-header", r#type: "checkbox", checked: custom_heading(), onchange: move |event: FormEvent| custom_heading.set(event.checked()) }
                    "Custom readiness heading"
                }
                label { class: "service-package-check", r#for: "package-reveal",
                    input { id: "package-reveal", r#type: "checkbox", checked: reveal_parts(), onchange: move |event: FormEvent| reveal_parts.set(event.checked()) }
                    "Reveal opened parts"
                }
            }
            p { class: "service-package-options-copy", "Hold parts changes records section requests while keeping the current view. Toggle parts externally changes the view directly." }
            div { class: "service-package-layout",
                div { id: "service-package-card",
                    for generation in [epoch()] {
                        Card { key: "service-card-{generation}", class: "service-package-main-card", aria_label: "Select Motor 001 package", selected: selected(),
                            onselect: move |_| {
                                selected.toggle(); let next = selections.peek().saturating_add(1); selections.set(next);
                            },
                            CardTitle { "Motor 001" }
                            CardContent {
                                p { class: "service-package-body-copy", "Monthly seal and bearing inspection. Check planned parts, then review the permit and technician notes." }
                            }
                            CardAccordion { id: "package-parts", label: "Planned parts", aria_label: "Show planned maintenance parts",
                                expanded: parts_open(), variant: variant(), reveal_content: reveal_parts(),
                                onexpand: move |next| {
                                    let count = parts_requests.peek().saturating_add(1); parts_requests.set(count);
                                    if !hold_parts() { parts_open.set(next); }
                                },
                                PlannedPartsFields {}
                            }
                        }
                    }
                }
                for generation in [epoch()] {
                    Blind { key: "service-readiness-{generation}", id: "package-readiness", class: "service-package-readiness",
                        label: "Service readiness", aria_label: "Service readiness for Motor 001", icon: "info",
                        variant: variant(), default_expanded: false, custom_header: header.clone(),
                        header_actions: rsx! {
                            Button { id: "package-inspect", variant: ButtonVariant::Secondary,
                                onclick: move |_| { let next = inspections.peek().saturating_add(1); inspections.set(next); },
                                "Inspect package"
                            }
                        },
                        ReadinessFields {}
                    }
                }
            }
            output { id: "service-package-result", class: "service-package-result", role: "status",
                "data-parts-requests": parts_requests().to_string(), "data-card-selections": selections().to_string(), "data-inspections": inspections().to_string(),
                if selected() { "Package selected. " } else { "Package not selected. " }
                "Parts requests: {parts_requests} · Selection changes: {selections} · Inspections: {inspections}. "
                if inspections() > 0 { "Package inspection recorded locally." } else { "No package inspection recorded." }
            }
        }
    }
}

pub const STYLES: &str = r#"
.service-package-example{min-width:0;max-width:100%;margin-bottom:48px}.service-package-intro{max-width:72ch;line-height:1.55}.service-package-toolbar,.service-package-options{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;min-width:0;max-width:100%}.service-package-toolbar{margin:20px 0 16px}.service-package-options{margin:0 0 12px}.service-package-option{display:inline-flex;align-items:center;flex-wrap:wrap;gap:8px;font-size:14px;min-width:0;max-width:100%}.service-package-select{box-sizing:border-box;max-width:100%;min-height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.service-package-options-copy{max-width:72ch;margin:0 0 24px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary)}.service-package-layout{display:grid;gap:48px;max-width:640px;min-width:0}.service-package-example .service-package-main-card{box-sizing:border-box;width:100%;min-width:0;max-width:100%}#service-package-card{min-width:0;max-width:100%}.service-package-readiness{box-sizing:border-box;min-width:0;max-width:100%}.service-package-fields{display:flex;flex-direction:column;gap:16px;min-width:0;max-width:100%}.service-package-body-copy{margin:0;line-height:1.5;color:inherit}.service-package-check{position:relative;z-index:1;pointer-events:auto;display:inline-flex;align-items:center;gap:8px;max-width:100%;font-size:14px;line-height:20px;cursor:pointer}.service-package-check input{width:16px;height:16px;flex:none;margin:0;accent-color:var(--ix-color-primary)}.service-package-field{position:relative;z-index:1;pointer-events:auto;display:flex;flex-direction:column;gap:8px;min-width:0;max-width:100%;font-size:14px;line-height:20px}.service-package-input{box-sizing:border-box;width:100%;max-width:480px;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.service-package-input:focus-visible,.service-package-select:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}.service-package-check input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.service-package-custom-heading{display:flex;flex-direction:column;align-items:flex-start;gap:4px;min-width:0;white-space:normal;overflow-wrap:anywhere;color:inherit}.service-package-custom-heading strong{font-weight:600}.service-package-custom-heading>span{font-size:13px;line-height:18px;color:inherit}.service-package-readiness .ix-blind-heading{flex-wrap:wrap}.service-package-readiness .ix-blind-header{flex:1 1 220px;min-width:0}.service-package-readiness .ix-blind-actions{flex:none;max-width:100%;padding:8px}.service-package-readiness .ix-blind-actions .ix-button{max-width:100%}.service-package-result{display:block;max-width:72ch;margin-top:24px;color:var(--ix-color-text-secondary);font-size:14px;line-height:20px;overflow-wrap:anywhere}@media(max-width:420px){.service-package-toolbar,.service-package-options{align-items:flex-start}.service-package-toolbar .ix-button{max-width:100%}}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute_values<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
        html.split(&format!("{name}=\""))
            .skip(1)
            .filter_map(|tail| tail.split('"').next())
            .collect()
    }

    #[test]
    fn service_package_renders_closed_retained_fields_and_actual_disclosure_relationships() {
        let mut dom = VirtualDom::new(ServicePackageExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "service-package",
            "service-package-card",
            "package-parts",
            "package-readiness",
            "package-seal-kit",
            "package-parts-note",
            "package-permit",
            "package-readiness-note",
            "package-inspect",
            "package-reset",
            "package-toggle-parts",
            "package-hold",
            "package-custom-header",
            "package-reveal",
            "package-variant",
            "service-package-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(html.contains("Select Motor 001 package"));
        assert!(html.contains("Show planned maintenance parts"));
        assert!(html.contains("Service readiness for Motor 001"));
        assert!(html.contains("Motor 001 readiness"));
        assert!(html.contains("Stage seal kit beside Motor 001"));
        assert!(html.contains("Review permit before service"));
        let panels = attribute_values(&html, "aria-controls");
        let toggles = attribute_values(&html, "aria-labelledby");
        assert_eq!(panels.len(), 2);
        assert_eq!(toggles.len(), 2);
        for id in panels.into_iter().chain(toggles) {
            assert!(
                html.contains(&format!("id=\"{id}\"")),
                "unresolved relationship {id}"
            );
        }
        assert_eq!(html.matches("role=\"region\"").count(), 2);
        assert_eq!(
            html.matches("aria-expanded=false").count()
                + html.matches("aria-expanded=\"false\"").count(),
            2
        );
        assert!(html.matches("hidden=true").count() + html.matches("hidden=\"true\"").count() >= 2);
        for counter in [
            "data-parts-requests",
            "data-card-selections",
            "data-inspections",
        ] {
            assert!(html.contains(&format!("{counter}=\"0\"")));
        }
    }
}
