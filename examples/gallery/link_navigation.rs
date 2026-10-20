//! Local maintenance references using native links and explicit Rust selection.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reference {
    Motor,
    Pump,
}

impl Reference {
    fn slug(self) -> &'static str {
        match self {
            Self::Motor => "motor",
            Self::Pump => "pump",
        }
    }
    fn asset(self) -> &'static str {
        match self {
            Self::Motor => "Motor 001",
            Self::Pump => "Pump 002",
        }
    }
    fn heading(self) -> &'static str {
        match self {
            Self::Motor => "Motor 001 reference",
            Self::Pump => "Pump 002 reference",
        }
    }
    fn body(self) -> &'static str {
        match self {
            Self::Motor => {
                "Record observed noise, vibration and bearing condition in the Motor 001 inspection record. Include the observation location and any supporting maintenance notes."
            }
            Self::Pump => {
                "Record the observed inlet, outlet and seal condition in the Pump 002 inspection record. Include the observation location and any supporting maintenance notes."
            }
        }
    }
}

#[component]
fn ReferencePreparation() -> Element {
    let mut note = use_signal(|| "Prepare Motor 001 review".to_string());
    rsx! {
        label { class: "references-note-field", r#for: "references-note",
            span { "Inspection preparation" }
            input { id: "references-note", r#type: "text", maxlength: "160", value: note(),
                oninput: move |event: FormEvent| note.set(event.value()),
            }
        }
    }
}

/// Two illustrative local references and an ordinary document-fragment link.
#[component]
pub fn MaintenanceReferencesExample() -> Element {
    let mut selected = use_signal(|| Reference::Motor);
    let mut disabled = use_signal(|| false);
    let mut navigations = use_signal(|| 0_usize);
    let mut disabled_requests = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let mut checklist = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let navigate_checklist = use_callback(move |event: MouseEvent| {
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        {
            // Pinned dioxus-interpreter-js native.ts intercepts anchor redirects,
            // including fragments. Use the explicit mounted target on desktop.
            event.prevent_default();
            if let Some(target) = checklist.peek().clone() {
                spawn(async move {
                    if let Err(error) = target.scroll_to(ScrollBehavior::Instant).await {
                        eprintln!("Could not scroll to inspection checklist: {error:?}");
                    }
                    if let Err(error) = target.set_focus(true).await {
                        eprintln!("Could not focus inspection checklist: {error:?}");
                    }
                });
            }
        }
        #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
        let _ = event; // Web retains uncancelled native fragment navigation.
    });
    let reference = selected();
    rsx! {
        section { id: "maintenance-references", class: "maintenance-references-example",
            h2 { "Maintenance references" }
            p { class: "references-intro", "Read two illustrative local inspection references and keep a preparation note while switching between them. Use your equipment-specific documentation for maintenance procedures." }
            div { class: "references-toolbar",
                Button { id: "references-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        selected.set(Reference::Motor); disabled.set(false); navigations.set(0); disabled_requests.set(0);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore maintenance references"
                }
            }
            nav { class: "references-links", "aria-label": "Local maintenance references",
                for destination in [Reference::Motor, Reference::Pump] {
                    span { key: "{destination.slug()}", id: format!("references-link-{}", destination.slug()),
                        LinkButton { url: format!("#references-{}", destination.slug()), disabled: disabled(),
                            onclick: move |event: MouseEvent| {
                                event.prevent_default();
                                if !*disabled.peek() {
                                    selected.set(destination); let next = navigations.peek().saturating_add(1); navigations.set(next);
                                }
                            },
                            "{destination.asset()} reference"
                        }
                    }
                }
                span { id: "references-link-native",
                    LinkButton { url: "#references-checklist", onclick: navigate_checklist, "Jump to inspection checklist" }
                }
                span { id: "references-link-disabled",
                    LinkButton { url: "#references-checklist", disabled: true,
                        onclick: move |_| { let next = disabled_requests.peek().saturating_add(1); disabled_requests.set(next); },
                        "Archived reference"
                    }
                }
            }
            details { id: "references-options", class: "references-options",
                summary { "Reference options" }
                div { class: "references-option-fields",
                    label { input { id: "references-disable", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable article links" }
                    span { id: "references-link-missing", LinkButton { "Reference not assigned" } }
                    span { id: "references-link-blank", LinkButton { url: "#references-checklist", target: "_blank", "Open checklist in new tab" } }
                }
            }
            div { id: "references-article", class: "references-article", "data-reference": reference.slug(),
                article { id: format!("references-{}", reference.slug()), "aria-labelledby": "references-heading",
                    h3 { id: "references-heading", "{reference.heading()}" }
                    p { id: "references-body", "{reference.body()}" }
                    p { "These example records are stored locally and do not replace the equipment's maintenance instructions." }
                }
            }
            for generation in [epoch()] { ReferencePreparation { key: "reference-preparation-{generation}" } }
            section { id: "references-checklist", class: "references-checklist", tabindex: "-1", "aria-labelledby": "references-checklist-heading",
                onmounted: move |event: MountedEvent| checklist.set(Some(event.data())),
                h3 { id: "references-checklist-heading", "Inspection checklist" }
                ul {
                    li { "Confirm the asset identifier in the inspection record." }
                    li { "Capture observations and reference the applicable equipment documentation." }
                    li { "Record follow-up work for the maintenance team." }
                }
            }
            output { id: "references-result", class: "references-result", role: "status",
                "data-selected": reference.slug(), "data-navigations": navigations().to_string(),
                "data-disabled-requests": disabled_requests().to_string(), "data-disabled": disabled().to_string(),
                "Current reference: {reference.heading()}"
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/link_navigation.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintenance_references_renders_actual_article_and_native_destinations() {
        let mut dom = VirtualDom::new(MaintenanceReferencesExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "maintenance-references",
            "references-link-motor",
            "references-link-pump",
            "references-link-native",
            "references-link-disabled",
            "references-link-missing",
            "references-link-blank",
            "references-article",
            "references-heading",
            "references-body",
            "references-note",
            "references-checklist",
            "references-reset",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(html.contains(Reference::Motor.body()) && !html.contains(Reference::Pump.body()));
        assert!(
            html.contains("href=\"#references-motor\"")
                && html.contains("href=\"#references-pump\"")
                && html.contains("href=\"#references-checklist\"")
        );
        assert!(
            html.contains("Prepare Motor 001 review") && html.contains("for=\"references-note\"")
        );
        assert!(
            html.contains("data-selected=\"motor\"") && html.contains("data-navigations=\"0\"")
        );
        assert!(html.contains("target=\"_blank\"") && html.contains("rel=\"noopener noreferrer\""));
        let missing = html
            .split("id=\"references-link-missing\"")
            .nth(1)
            .expect("unassigned link");
        let anchor = missing
            .split("<a ")
            .nth(1)
            .expect("unassigned anchor")
            .split('>')
            .next()
            .unwrap();
        assert!(
            !anchor.contains("href=") && !anchor.contains("tabindex="),
            "unassigned destination remains noninteractive"
        );
        assert!(html.contains("role=\"link\"") && html.contains("tabindex=\"-1\""));
    }
}
