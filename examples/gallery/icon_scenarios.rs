//! Rust counterpart of upstream icon name registration and alias examples.
use dioxus::prelude::*;
use dioxus_ix::Icon;
use dioxus_ix::icons::{IconDefinition, IconRegistryProvider, use_icon_registry};

pub const SCENARIOS: &[&str] = &["add-icons"];

#[component]
pub fn IconScenario(name: String) -> Element {
    let registry = use_icon_registry();
    use_hook(move || {
        registry.add_icons([
            ("iconStar", IconDefinition::bundled("star").unwrap()),
            (
                "iconStarFilled",
                IconDefinition::bundled("star-filled").unwrap(),
            ),
        ]);
    });
    match name.as_str() {
        "add-icons" => {
            rsx! { IconRegistryProvider { registry, div { class: "source-icon-registration", "data-registered-icons": registry.len().to_string(),
                Icon { name: "star" }
                Icon { name: "starFilled" }
                Icon { name: "star-filled" }
            } } }
        }
        _ => rsx! { p { "This icon source has not been reproduced yet." } },
    }
}

pub const STYLES: &str = ".source-icon-registration{display:flex;align-items:center;gap:4px}";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_source_renders_all_three_upstream_icon_names() {
        let mut dom = VirtualDom::new_with_props(
            IconScenario,
            IconScenarioProps {
                name: "add-icons".into(),
            },
        );
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("class=\"ix-icon ").count(), 3);
        assert!(html.contains("data-registered-icons=\"2\""));
        assert_ne!(
            dioxus_ix::icons::icon_svg("star"),
            dioxus_ix::icons::icon_svg("starFilled")
        );
        assert_eq!(
            dioxus_ix::icons::icon_svg("starFilled"),
            dioxus_ix::icons::icon_svg("star-filled")
        );
    }
}
