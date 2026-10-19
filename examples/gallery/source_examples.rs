//! Searchable, individually mounted upstream preview scenarios.
use super::{
    action_scenarios, application_scenarios, chat_scenarios, datetime_scenarios, dialog_scenarios,
    feedback_scenarios, form_scenarios, layout_scenarios, navigation::GalleryLocation,
    pattern_scenarios,
};
use dioxus::prelude::*;
use dioxus_ix::{Button, ButtonVariant, PORT_CATALOG, UPSTREAM_EXAMPLE_NAMES, UPSTREAM_REVISION};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    AdvancedCharts,
    Grid,
    Charts,
    Icons,
    Actions,
    Forms,
    Layout,
    Feedback,
    Application,
    Chat,
    Dialogs,
    Patterns,
    DateTime,
}
fn family(name: &str) -> Option<Family> {
    if crate::advanced_chart_scenarios::SCENARIOS.contains(&name) {
        return Some(Family::AdvancedCharts);
    }
    if crate::grid_scenarios::SCENARIOS.contains(&name) {
        return Some(Family::Grid);
    }
    if crate::chart_scenarios::SCENARIOS.contains(&name) {
        return Some(Family::Charts);
    }
    if crate::icon_scenarios::SCENARIOS.contains(&name) {
        return Some(Family::Icons);
    }
    if action_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Actions)
    } else if form_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Forms)
    } else if layout_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Layout)
    } else if feedback_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Feedback)
    } else if application_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Application)
    } else if chat_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Chat)
    } else if dialog_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Dialogs)
    } else if pattern_scenarios::SCENARIOS.contains(&name) {
        Some(Family::Patterns)
    } else if datetime_scenarios::SCENARIOS.contains(&name) {
        Some(Family::DateTime)
    } else {
        None
    }
}
fn normalized_name(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

fn source_has_component(name: &str, upstream: &str) -> bool {
    name == upstream
        || name
            .strip_prefix(upstream)
            .is_some_and(|suffix| suffix.starts_with('-'))
        || (name == "buttons" && upstream == "button")
}

fn rust_mapping(name: &str) -> Option<&'static str> {
    PORT_CATALOG
        .iter()
        .filter(|entry| source_has_component(name, entry.upstream) && entry.rust_api.is_some())
        .max_by_key(|entry| entry.upstream.len())
        .and_then(|entry| entry.rust_api)
}

fn names_matching(query: &str, implemented_only: bool) -> Vec<&'static str> {
    let normalized = normalized_name(query);
    let needle = normalized.strip_prefix("ix").unwrap_or(&normalized);
    UPSTREAM_EXAMPLE_NAMES
        .iter()
        .copied()
        .filter(|name| {
            let matches = normalized_name(name).contains(needle)
                || PORT_CATALOG.iter().any(|entry| {
                    source_has_component(name, entry.upstream)
                        && entry
                            .rust_api
                            .is_some_and(|api| normalized_name(api).contains(needle))
                });
            matches && (!implemented_only || family(name).is_some())
        })
        .collect()
}

fn active_name(matches: &[&'static str], requested: &str) -> Option<&'static str> {
    matches
        .iter()
        .copied()
        .find(|name| *name == requested)
        .or_else(|| matches.first().copied())
}

fn rust_source_path(name: &str) -> Option<&'static str> {
    Some(match family(name)? {
        Family::AdvancedCharts => "advanced_chart_scenarios.rs",
        Family::Grid => "grid_scenarios.rs",
        Family::Charts => "chart_scenarios.rs",
        Family::Icons => "icon_scenarios.rs",
        Family::Actions => "action_scenarios.rs",
        Family::Forms => "form_scenarios.rs",
        Family::Layout => "layout_scenarios.rs",
        Family::Feedback => "feedback_scenarios.rs",
        Family::Application => "application_scenarios.rs",
        Family::Chat => "chat_scenarios.rs",
        Family::Dialogs => "dialog_scenarios.rs",
        Family::Patterns => "pattern_scenarios.rs",
        Family::DateTime => "datetime_scenarios.rs",
    })
}

fn usage_hint(name: &str) -> &'static str {
    match family(name) {
        Some(Family::Forms | Family::DateTime) => {
            "Try entering or selecting a value in the example below."
        }
        Some(Family::Actions | Family::Dialogs | Family::Application | Family::Chat) => {
            "Try the available controls below. The example keeps its own local state."
        }
        Some(Family::Grid | Family::Charts | Family::AdvancedCharts) => {
            "Explore the sample data and the controls provided by this example."
        }
        _ => "Inspect the example below. Switching examples starts a fresh instance.",
    }
}

#[component]
pub fn SourceExamples(mut location: Signal<GalleryLocation>) -> Element {
    let mut implemented_only = use_signal(|| true);
    use_effect(move || {
        let current = location.read().clone();
        let matches = names_matching(&current.query, implemented_only());
        if let Some(active) = active_name(&matches, &current.example)
            && current.example != active
        {
            location.write().example = active.into();
        }
    });
    let current = location.read().clone();
    let matches = names_matching(&current.query, implemented_only());
    let active = active_name(&matches, &current.example);
    let index = active.and_then(|name| matches.iter().position(|candidate| *candidate == name));
    let previous = index
        .and_then(|index| index.checked_sub(1))
        .and_then(|index| matches.get(index))
        .copied();
    let next = index.and_then(|index| matches.get(index + 1)).copied();
    let implemented = UPSTREAM_EXAMPLE_NAMES
        .iter()
        .filter(|name| family(name).is_some())
        .count();
    rsx! {
        style { {STYLES} }
        style { {action_scenarios::STYLES} }
        style { {form_scenarios::STYLES} }
        style { {layout_scenarios::STYLES} }
        style { {feedback_scenarios::STYLES} }
        style { {application_scenarios::STYLES} }
        style { {chat_scenarios::STYLES} }
        style { {dialog_scenarios::STYLES} }
        style { {pattern_scenarios::STYLES} }
        style { {datetime_scenarios::STYLES} }
        style { {crate::icon_scenarios::STYLES} }
        style { {crate::grid_scenarios::STYLES} }
        style { {crate::chart_scenarios::STYLES} }
        style { {crate::advanced_chart_scenarios::STYLES} }
        p { class: "source-introduction", "Explore {implemented} working Rust examples from {UPSTREAM_EXAMPLE_NAMES.len()} upstream preview sources. These examples aim for recognizable iX appearance and reliable core interactions. Each starts with fresh local state." }
        div { class: "source-controls",
            label { "Find an example"
                input { r#type: "search", maxlength: super::navigation::MAX_QUERY_CHARS as i64, value: current.query.clone(), placeholder: "Try Button, ix-spinner, or ProgressIndicator", oninput: move |event| location.write().query = event.value() }
            }
            label { "Source example"
                select { value: active.unwrap_or_default(), disabled: active.is_none(), onchange: move |event| location.write().example = event.value(),
                    if active.is_none() { option { value: "", "No matching examples" } }
                    for name in matches.iter().copied() { option { value: name, selected: active == Some(name), "{name}" } }
                }
            }
            if implemented < UPSTREAM_EXAMPLE_NAMES.len() {
                label { class: "source-checkbox", input { r#type: "checkbox", checked: implemented_only(), onchange: move |event| implemented_only.set(event.checked()) } "Show implemented examples only" }
            }
        }
        div { class: "source-navigation",
            Button { variant: ButtonVariant::Secondary, disabled: previous.is_none(), onclick: move |_| { if let Some(previous) = previous { location.write().example = previous.into(); } }, "Previous example" }
            Button { variant: ButtonVariant::Secondary, disabled: next.is_none(), onclick: move |_| { if let Some(next) = next { location.write().example = next.into(); } }, "Next example" }
            span { role: "status", "{matches.len()} matching sources" }
        }
        // A keyed dynamic collection gives each source a fresh component owner.
        // A key inside a static `if` slot alone does not remount on this Dioxus pin.
        for name in active.into_iter() {
            section { key: "{name}", class: "source-preview",
            { let mut permalink = current.clone(); permalink.page = "sources".into(); permalink.example = name.into();
                rsx! { a {
                    class: "source-permalink", href: permalink.to_hash(),
                    onclick: move |event| super::follow_gallery_link(event, location, permalink.clone()),
                    "Link to this example"
                } }
            }
            div { class: "source-heading", h2 { code { "{name}.html" } }
                a { href: format!("https://github.com/siemens/ix/blob/{UPSTREAM_REVISION}/examples/html-examples/src/preview-examples/{name}.html"), target: "_blank", rel: "noopener noreferrer", "View reference source" }
            }
            div { class: "source-usage",
                if let Some(api) = rust_mapping(name) {
                    p { "Rust API: " code { "{api}" } }
                } else {
                    p { "Browse the Rust example to see the components used in this scenario." }
                }
                p { "{usage_hint(name)}" }
                if let Some(path) = rust_source_path(name) {
                    a { href: format!("https://github.com/jme-1308/Dioxus_iX/blob/main/examples/gallery/{path}"), target: "_blank", rel: "noopener noreferrer", "View Rust example" }
                }
                a { href: "https://github.com/jme-1308/Dioxus_iX/blob/main/IMPLEMENTATION.md#parity-ledger", target: "_blank", rel: "noopener noreferrer", "Implementation notes and limitations" }
            }
            div { class: if family(name) == Some(Family::Feedback) { "source-scenario-frame source-feedback-frame" } else { "source-scenario-frame" }, "data-source-example": name,
                match family(name) {
                    Some(Family::AdvancedCharts) => rsx! { crate::advanced_chart_scenarios::AdvancedChartScenario { name: name.to_string() } },
                    Some(Family::Grid) => rsx! { crate::grid_scenarios::GridScenario { name: name.to_string() } },
                    Some(Family::Charts) => rsx! { crate::chart_scenarios::ChartScenario { name: name.to_string() } },
                    Some(Family::Icons) => rsx! { crate::icon_scenarios::IconScenario { name: name.to_string() } },
                    Some(Family::Actions) => rsx! { action_scenarios::ActionScenario { name: name.to_string() } },
                    Some(Family::Forms) => rsx! { form_scenarios::FormScenario { name: name.to_string() } },
                    Some(Family::Layout) => rsx! { layout_scenarios::LayoutScenario { name: name.to_string() } },
                    Some(Family::Feedback) => rsx! { feedback_scenarios::FeedbackScenario { name: name.to_string() } },
                    Some(Family::Application) => rsx! { application_scenarios::ApplicationScenario { name: name.to_string() } },
                    Some(Family::Chat) => rsx! { chat_scenarios::ChatScenario { name: name.to_string() } },
                    Some(Family::Dialogs) => rsx! { dialog_scenarios::DialogScenario { name: name.to_string() } },
                    Some(Family::Patterns) => rsx! { pattern_scenarios::PatternScenario { name: name.to_string() } },
                    Some(Family::DateTime) => rsx! { datetime_scenarios::DateTimeScenario { name: name.to_string() } },
                    None => rsx! { p { "This source example has not been reproduced in isolation yet. The component-family galleries may contain related initial implementations." } },
                }
            }
            }
        }
        if active.is_none() {
            div { class: "source-empty",
                h2 { "No examples found" }
                p { role: "status", "Try a component name such as Button or Spinner, or clear the search to browse all examples." }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { location.write().query.clear(); implemented_only.set(true); }, "Clear search" }
            }
        }
    }
}

const STYLES: &str = r#"
.source-introduction{max-width:80ch;font-size:14px;color:var(--ix-color-text-secondary)}
.source-controls{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:16px;max-width:900px;margin:24px 0 16px}
.source-controls>label{display:flex;flex-direction:column;gap:8px;font-size:14px;min-width:0}
.source-controls input[type=search],.source-controls select{min-width:0;width:100%;height:40px;padding:8px 12px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);color:var(--ix-color-text);background:var(--ix-color-surface-1);font:inherit}
.source-controls input::placeholder{color:var(--ix-color-text-secondary);opacity:1}
.source-controls>.source-checkbox{grid-column:1/-1;flex-direction:row;align-items:center}
.source-navigation{display:flex;align-items:center;flex-wrap:wrap;gap:12px;margin-bottom:32px}
.source-navigation>span{font-size:14px;color:var(--ix-color-text-secondary)}
.source-permalink{display:inline-block;margin-bottom:16px;font-size:14px;text-underline-offset:3px}
.source-usage{max-width:75ch;margin-bottom:24px;font-size:14px;color:var(--ix-color-text-secondary)}
.source-usage p{margin:0 0 8px}.source-usage code{color:var(--ix-color-text)}
.source-usage a{display:inline-block;margin-right:20px;font-size:14px;text-underline-offset:3px}
.source-empty{max-width:65ch;padding:24px 0}.source-empty h2{font-size:24px;margin:0 0 12px}.source-empty p{margin:0 0 20px;color:var(--ix-color-text-secondary)}
.source-controls :is(input,select):focus-visible,.source-permalink:focus-visible,.source-usage a:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:3px}
.source-heading{display:flex;align-items:baseline;flex-wrap:wrap;gap:16px;margin:0 0 16px}
.source-heading h2{font-size:18px;margin:0;overflow-wrap:anywhere}
.source-heading a{font-size:14px;text-underline-offset:3px}
.source-scenario-frame{min-height:200px;min-width:0;padding:24px;border:1px solid var(--ix-color-border);isolation:isolate}
.source-scenario-frame[data-source-example^="badge"]{overflow-x:auto}
.source-feedback-frame{min-height:320px;overflow:visible}
.source-event-feedback{font-size:14px;color:var(--ix-color-text-secondary);margin:24px 0 0;overflow-wrap:anywhere}
@media(max-width:640px){.source-controls{grid-template-columns:1fr}.source-scenario-frame{padding:16px}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[component]
    fn ScenarioProbe(name: String) -> Element {
        match family(&name) {
            Some(Family::AdvancedCharts) => {
                rsx! { crate::advanced_chart_scenarios::AdvancedChartScenario { name } }
            }
            Some(Family::Grid) => rsx! { crate::grid_scenarios::GridScenario { name } },
            Some(Family::Charts) => rsx! { crate::chart_scenarios::ChartScenario { name } },
            Some(Family::Icons) => rsx! { crate::icon_scenarios::IconScenario { name } },
            Some(Family::Actions) => rsx! { action_scenarios::ActionScenario { name } },
            Some(Family::Forms) => rsx! { form_scenarios::FormScenario { name } },
            Some(Family::Layout) => rsx! { layout_scenarios::LayoutScenario { name } },
            Some(Family::Feedback) => rsx! { feedback_scenarios::FeedbackScenario { name } },
            Some(Family::Application) => {
                rsx! { application_scenarios::ApplicationScenario { name } }
            }
            Some(Family::Chat) => rsx! { chat_scenarios::ChatScenario { name } },
            Some(Family::Dialogs) => rsx! { dialog_scenarios::DialogScenario { name } },
            Some(Family::Patterns) => rsx! { pattern_scenarios::PatternScenario { name } },
            Some(Family::DateTime) => rsx! { datetime_scenarios::DateTimeScenario { name } },
            None => rsx! { "Unknown registered source" },
        }
    }
    #[test]
    fn every_registered_source_renders_in_an_independent_runtime() {
        for name in names_matching("", true) {
            let mut dom = VirtualDom::new_with_props(
                ScenarioProbe,
                ScenarioProbeProps {
                    name: name.to_string(),
                },
            );
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(
                !html.contains("not registered")
                    && !html.contains("Unknown registered source")
                    && !html.contains("has not been reproduced"),
                "{name}"
            );
            assert!(!html.is_empty(), "Empty source: {name}");
        }
    }
    #[test]
    fn registered_sources_are_real_unique_inventory_members() {
        let mut seen = std::collections::BTreeSet::new();
        for names in [
            action_scenarios::SCENARIOS,
            form_scenarios::SCENARIOS,
            layout_scenarios::SCENARIOS,
            feedback_scenarios::SCENARIOS,
            application_scenarios::SCENARIOS,
            chat_scenarios::SCENARIOS,
            dialog_scenarios::SCENARIOS,
            pattern_scenarios::SCENARIOS,
            datetime_scenarios::SCENARIOS,
            crate::icon_scenarios::SCENARIOS,
            crate::grid_scenarios::SCENARIOS,
            crate::chart_scenarios::SCENARIOS,
            crate::advanced_chart_scenarios::SCENARIOS,
        ] {
            for name in names {
                assert!(
                    UPSTREAM_EXAMPLE_NAMES.contains(name),
                    "Unknown source {name}"
                );
                assert!(seen.insert(name), "Duplicate source {name}");
            }
        }
    }
    #[test]
    fn search_accepts_rust_api_and_upstream_tag_spellings() {
        let expected = names_matching("progress-indicator", true);
        assert!(!expected.is_empty());
        assert_eq!(names_matching("ProgressIndicator", true), expected);
        assert_eq!(names_matching("ix-progress-indicator", true), expected);
        assert_eq!(names_matching("progress_indicator", true), expected);
        assert_eq!(names_matching(" progress indicator ", true), expected);
        assert!(names_matching("Button", true).contains(&"buttons"));
        assert_eq!(
            rust_mapping("progress-indicator"),
            Some("ProgressIndicator")
        );
        assert_eq!(rust_mapping("buttons"), Some("Button"));
    }

    #[test]
    fn filtered_selection_falls_back_and_empty_search_recovers() {
        let matches = names_matching("spinner", true);
        let first = matches.first().copied();
        assert_eq!(active_name(&matches, "buttons"), first);
        assert_eq!(active_name(&[], "buttons"), None);
        assert_eq!(
            active_name(&names_matching("", true), "buttons"),
            Some("buttons")
        );
    }

    #[test]
    fn search_preserves_upstream_order_and_all_inventory_sources_have_fixtures() {
        assert_eq!(
            names_matching("TOAST", true),
            ["toast-custom", "toast-position", "toast"]
        );
        assert_eq!(
            names_matching("echarts-special-toolbox", false),
            ["echarts-special-toolbox"]
        );
        assert_eq!(
            names_matching("echarts-special-toolbox", true),
            ["echarts-special-toolbox"]
        );
        assert_eq!(names_matching("", true), UPSTREAM_EXAMPLE_NAMES);
        assert_eq!(
            names_matching("echarts-bar-simple", true),
            ["echarts-bar-simple"]
        );
        assert!(names_matching("no-such-source-example", false).is_empty());
    }
}
