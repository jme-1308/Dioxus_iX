//! Consumer-level rendering contracts, independent of web/desktop runtimes.
use dioxus::prelude::*;
use dioxus_ix::*;

fn render_node(build: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(build);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn buttons_keep_native_semantics_and_loading_state() {
    let html = render_node(|| {
        rsx! {
            Button { disabled: true, "Disabled action" }
            Button { loading: true, "Saving" }
        }
    });
    assert!(html.contains("<button"));
    assert!(html.contains("type=\"button\""));
    assert!(html.contains("disabled"));
    assert!(html.contains("aria-busy=true") || html.contains("aria-busy=\"true\""));
    assert!(html.contains("Saving"));
}

#[test]
fn fields_preserve_labels_names_constraints_and_validation() {
    let html = render_node(|| {
        rsx! {
            Input { id: "machine-name", name: "machine", label: "Machine name",
                value: "Pump 01", required: true, state: FieldState::Invalid,
                invalid_text: "Choose a unique name" }
        }
    });
    assert!(html.contains("for=\"machine-name\""));
    assert!(html.contains("name=\"machine\""));
    assert!(html.contains("required"));
    assert!(html.contains("aria-invalid=\"true\""));
    assert!(html.contains("Choose a unique name"));
}

#[test]
fn messages_have_contextual_accessible_feedback() {
    let html = render_node(|| {
        rsx! {
            MessageBar { message_type: MessageType::Alarm, persistent: true, "Check the connection" }
        }
    });
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("data-type=\"alarm\""));
    assert!(html.contains("Check the connection"));
    assert!(!html.contains("Close message"));
}

#[test]
fn theme_applies_scheme_and_density_without_external_assets() {
    let html = render_node(|| {
        rsx! {
            IxTheme { scheme: ColorScheme::Dark, density: Density::Compact, "App content" }
        }
    });
    assert!(html.contains("data-ix-theme=\"brand\""));
    assert!(html.contains("data-ix-color-schema=\"dark\""));
    assert!(html.contains("data-ix-density=\"compact\""));
    assert!(!html.contains("<script"));
    assert!(!html.contains("<link"));
    assert!(html.contains("App content"));
}

#[test]
fn icon_catalog_is_sorted_complete_nonempty_and_excludes_brand_marks() {
    let names: Vec<_> = icon_names().collect();
    assert_eq!(names.len(), 1473);
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    for name in names {
        assert!(!name.contains("logo") && !name.contains("siemens"));
        assert!(!matches!(name, "youtube" | "youtube-filled"));
        let svg = icons::icon_svg(name).expect("every listed icon resolves");
        assert!(!svg.is_empty(), "empty geometry for {name}");
        assert!(!svg.contains("<script"));
    }
    assert!(icons::icon_svg("not-an-upstream-icon").is_none());
    assert!(
        icons::icon_svg("add")
            .unwrap()
            .contains("viewBox='0 0 512 512'")
    );
}

#[test]
fn icons_separate_decorative_and_meaningful_semantics() {
    let meaningful = render_node(|| rsx! { Icon { name: "add", label: "Add item" } });
    assert!(meaningful.contains("role=\"img\""));
    assert!(meaningful.contains("aria-label=\"Add item\""));
    let decorative = render_node(|| rsx! { Icon { name: "close" } });
    assert!(decorative.contains("aria-hidden=\"true\""));
}

#[test]
fn mixed_checkbox_exposes_one_control_and_preserves_native_form_value() {
    let html = render_node(|| {
        rsx! {
            Checkbox { name: "options", label: "Options", checked: true, indeterminate: true, required: true }
        }
    });
    assert!(html.contains("role=\"checkbox\""));
    assert!(html.contains("aria-checked=\"mixed\""));
    assert!(html.contains("name=\"options\""));
    assert!(html.contains("aria-hidden=\"true\""));
    assert_eq!(html.matches("role=\"checkbox\"").count(), 1);
    assert!(html.contains("type=\"button\""));
}

#[test]
fn tooltip_is_explicitly_linked_to_its_focusable_trigger() {
    let html = render_node(|| {
        rsx! {
            Tooltip { id: "machine-help", label: "Additional machine information",
                Button { aria_describedby: "machine-help", "Information" }
            }
        }
    });
    assert!(html.contains("aria-describedby=\"machine-help\""));
    assert!(html.contains("id=\"machine-help\""));
    assert!(html.contains("role=\"tooltip\""));
}
