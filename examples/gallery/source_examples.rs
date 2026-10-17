//! Searchable, individually mounted upstream preview scenarios.
use super::{
    action_scenarios, application_scenarios, chat_scenarios, datetime_scenarios, dialog_scenarios,
    feedback_scenarios, form_scenarios, layout_scenarios, pattern_scenarios,
};
use dioxus::prelude::*;
use dioxus_ix::{Button, ButtonVariant, UPSTREAM_EXAMPLE_NAMES, UPSTREAM_REVISION};

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
fn names_matching(query: &str, implemented_only: bool) -> Vec<&'static str> {
    let needle = query.to_lowercase();
    UPSTREAM_EXAMPLE_NAMES
        .iter()
        .copied()
        .filter(|name| name.contains(&needle) && (!implemented_only || family(name).is_some()))
        .collect()
}

#[component]
pub fn SourceExamples() -> Element {
    let mut query = use_signal(String::new);
    let mut selected = use_signal(|| "buttons".to_string());
    let mut implemented_only = use_signal(|| true);
    let matches = names_matching(&query(), implemented_only());
    let requested = selected();
    let active = matches
        .iter()
        .copied()
        .find(|name| *name == requested)
        .or_else(|| matches.first().copied());
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
        p { class: "source-introduction", "{implemented} individual Rust examples are available from the {UPSTREAM_EXAMPLE_NAMES.len()} current upstream preview sources. Legacy examples are excluded. Each starts with fresh local state. These initial reproductions still require full visual and behavioral parity review." }
        div { class: "source-controls",
            label { "Find an example"
                input { r#type: "search", value: query(), placeholder: "Search upstream names", oninput: move |event| query.set(event.value()) }
            }
            label { "Source example"
                select { value: active.unwrap_or_default(), disabled: active.is_none(), onchange: move |event| selected.set(event.value()),
                    if active.is_none() { option { value: "", "No matching examples" } }
                    for name in matches.iter().copied() { option { value: name, selected: active == Some(name), "{name}" } }
                }
            }
            if implemented < UPSTREAM_EXAMPLE_NAMES.len() {
                label { class: "source-checkbox", input { r#type: "checkbox", checked: implemented_only(), onchange: move |event| implemented_only.set(event.checked()) } "Show implemented examples only" }
            }
        }
        div { class: "source-navigation",
            Button { variant: ButtonVariant::Secondary, disabled: previous.is_none(), onclick: move |_| { if let Some(previous) = previous { selected.set(previous.into()); } }, "Previous example" }
            Button { variant: ButtonVariant::Secondary, disabled: next.is_none(), onclick: move |_| { if let Some(next) = next { selected.set(next.into()); } }, "Next example" }
            span { role: "status", "{matches.len()} matching sources" }
        }
        if let Some(name) = active {
            section { key: "{name}", class: "source-preview",
            div { class: "source-heading", h2 { code { "{name}.html" } }
                a { href: format!("https://github.com/siemens/ix/blob/{UPSTREAM_REVISION}/examples/html-examples/src/preview-examples/{name}.html"), target: "_blank", rel: "noopener noreferrer", "View reference source" }
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
        } else { p { role: "status", "No examples match this search. Clear or shorten the name to see more sources." } }
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
