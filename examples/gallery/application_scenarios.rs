//! Isolated application/frame examples adapted from the pinned iX HTML sources.
//! Company logos and decorative logo placeholders are deliberately omitted.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const SCENARIOS: &[&str] = &[
    "about-and-legal",
    "application",
    "application-advanced",
    "application-app-switch",
    "application-breakpoints",
    "application-header",
    "info-page",
    "menu-category",
    "menu-with-bottom-tabs",
    "pane",
    "pane-layout",
    "settings",
    "theme-switcher",
];

#[component]
pub fn ApplicationScenario(name: String) -> Element {
    let content = match name.as_str() {
        "application"
        | "application-advanced"
        | "application-app-switch"
        | "application-breakpoints" => rsx! { FrameExample { kind: name.clone() } },
        "application-header" => rsx! { HeaderExample {} },
        "about-and-legal" | "settings" => {
            rsx! { OverlayExample { kind: name.clone() } }
        }
        "menu-category" => rsx! { CategoryExample {} },
        "menu-with-bottom-tabs" => {
            rsx! { div { class: "application-source-frame application-source-bottom-menu", Menu { bottom: rsx! { MenuItem { label: "Bottom tab", icon: "info" } }, MenuItem { label: "Home", icon: "home", home: true } } } }
        }
        "pane" => rsx! { PaneExample {} },
        "pane-layout" => rsx! { PaneLayoutExample {} },
        "theme-switcher" => rsx! { ThemeExample {} },
        "info-page" => rsx! { InfoPageExample {} },
        _ => rsx! { p { "No application source example is registered for {name}." } },
    };
    rsx! { div { class: "application-source-example", {content} } }
}

fn source_switch_config() -> AppSwitchConfig {
    AppSwitchConfig {
        current_app_id: "demo-app-2".into(),
        label: "Switch to Application".into(),
        apps: [
            ("demo-app-1", "Floor App", "Floor app"),
            ("demo-app-2", "Calculator App", "Calculator app"),
        ]
        .into_iter()
        .map(|(id, name, description)| AppSwitchEntry {
            id: id.into(),
            name: name.into(),
            description: format!("Example description for {description}"),
            icon_src: if id == "demo-app-1" {
                super::demo_images::floor_app_icon()
            } else {
                super::demo_images::calculator_app_icon()
            },
            url: "https://ix.siemens.io/".into(),
            target: "_self".into(),
        })
        .collect(),
        ..Default::default()
    }
}

#[component]
fn SourceHeaderActions(
    #[props(default)] full: bool,
    #[props(default)] onaction: EventHandler<String>,
) -> Element {
    rsx! {
        if full { for number in 1..=3 { IconButton { icon: "checkboxes", label: format!("Button {number}"), variant: ButtonVariant::Tertiary, onclick: move |_| onaction.call(format!("Button {number}")) } } }
        DropdownButton { label: "Select config", variant: ButtonVariant::SubtleTertiary,
            for number in 1..=3 { DropdownItem { label: format!("Config {number}"), onitemclick: move |_| onaction.call(format!("Config {number}")) } }
        }
        div { class: "application-source-header-avatar",
            MenuAvatar { top: if full { "John Doe" } else { "User" }, bottom: full.then(|| "Administrator".to_string()), hide_logout_button: true,
                for number in 1..=if full { 2 } else { 3 } { MenuAvatarItem { label: format!("Action {number}"), onclick: move |_| onaction.call(format!("Action {number}")) } }
            }
        }
    }
}

#[component]
fn HeaderExample() -> Element {
    let mut event = use_signal(|| "No event yet".to_string());
    rsx! {
        ApplicationHeader { name: "My Application", show_menu: false,
            actions: rsx! { SourceHeaderActions { full: true, onaction: move |value| event.set(value) } }
        }
        output { class: "application-source-events", "aria-live": "polite", "Event: {event}" }
    }
}

#[component]
fn FrameExample(kind: String) -> Element {
    let advanced = kind == "application-advanced";
    let switch = kind == "application-app-switch";
    let breakpoints = kind == "application-breakpoints";
    let mut active = use_signal(|| "Home".to_string());
    let mut breakpoint = use_signal(|| "md".to_string());
    let mut event = use_signal(|| "No event yet".to_string());
    let forced = if breakpoints {
        match breakpoint().as_str() {
            "sm" => ApplicationBreakpoint::Small,
            "lg" => ApplicationBreakpoint::Large,
            _ => ApplicationBreakpoint::Medium,
        }
    } else {
        ApplicationBreakpoint::Medium
    };
    let header = rsx! { ApplicationHeader { name: if switch { "Calculator App" } else { "My Application" },
        actions: rsx! {
            if advanced { Avatar { username: "Example User", tooltip: "Example User · Additional info" } }
            else if switch || breakpoints { SourceHeaderActions { onaction: move |value| event.set(value) } }
        }
    } };
    let menu = rsx! { Menu {
        settings: advanced.then(|| rsx! { MenuSettings {} }),
        about: advanced.then(|| rsx! { MenuAbout {} }),
        if advanced {
            MenuItem { label: "Home", home: true, icon: "home", active: active() == "Home", onclick: move |_| active.set("Home".into()) }
            MenuItem { label: "Alarm", icon: "alarm-bell", active: active() == "Alarm", onclick: move |_| active.set("Alarm".into()) }
            MenuCategory { label: "Analysis", icon: "piechart", active: matches!(active().as_str(), "Plant" | "Network"),
                MenuItem { label: "Plant", icon: "plant", active: active() == "Plant", onclick: move |_| active.set("Plant".into()) }
                MenuItem { label: "Network", icon: "network-device", active: active() == "Network", onclick: move |_| active.set("Network".into()) }
            }
        } else { MenuItem { label: "Item 1" } MenuItem { label: "Item 2" } }
    } };
    rsx! {
        div { class: "application-source-frame",
            Application { header, menu, force_breakpoint: (!advanced).then_some(forced),
                app_switch_config: switch.then(source_switch_config),
                Content {
                    ContentHeader { title: if advanced { format!("Example {} content", active().to_lowercase()) } else if breakpoints { "Choose breakpoint".into() } else { "My Content Page".into() } }
                    if breakpoints { RadioGroup { options: vec![SelectOption::new("sm","Small"),SelectOption::new("md","Medium"),SelectOption::new("lg","Large")], value: breakpoint(), name: "source-breakpoint", onvaluechange: move |value| breakpoint.set(value) } }
                }
            }
        }
        if switch || breakpoints { output { class: "application-source-events", "aria-live": "polite", "Event: {event}" } }
    }
}

#[component]
fn OverlayExample(kind: String) -> Element {
    let about = kind.starts_with("about-");
    let mut active = use_signal(|| "tab-1".to_string());
    let tabs = if about {
        vec![
            TabItem::new("tab-1", "Tab 1"),
            TabItem::new("tab-2", "Tab 2"),
        ]
    } else {
        vec![
            TabItem::new("tab-1", "Example Setting 1"),
            TabItem::new("tab-2", "Example Setting 2"),
        ]
    };
    let children = rsx! {
        if about { TabPanel { tab_key: "tab-1", "Content 1" } TabPanel { tab_key: "tab-2", "Content 2" } }
        else { TabPanel { tab_key: "tab-1", "Example Setting 1 content" } TabPanel { tab_key: "tab-2", "Example Setting 2 content" } }
    };
    let body = if about {
        rsx! { MenuAbout { TabSet { items: tabs, active_key: active(), onselectedchange: move |value| active.set(value), {children} } } }
    } else {
        rsx! { MenuSettings { TabSet { items: tabs, active_key: active(), onselectedchange: move |value| active.set(value), {children} } } }
    };
    rsx! { div { class: "application-source-frame",
        Application { header: rsx! { ApplicationHeader { name: "" } },
            menu: rsx! { Menu { initial_overlay: if about { MenuOverlayKind::About } else { MenuOverlayKind::Settings }, about: about.then(|| body.clone()), settings: (!about).then_some(body) } }
        }
    } }
}

#[component]
fn CategoryExample() -> Element {
    rsx! { div { class: "application-source-frame",
        Application { force_breakpoint: ApplicationBreakpoint::Large,
            menu: rsx! { Menu {
                MenuItem { label: "Home", icon: "home", home: true }
                MenuItem { label: "Normal Tab", icon: "globe" }
                MenuCategory { label: "Top level Category", icon: "rocket", disable_tooltip: true,
                    MenuItem { label: "Nested Tab", icon: "globe" }
                    MenuItem { label: "Nested Tab", icon: "globe" }
                }
            } }
        }
    } }
}

#[component]
fn PaneExample() -> Element {
    let mut expanded = use_signal(|| false);
    rsx! { div { class: "application-source-frame application-source-pane",
        Pane { heading: "Pane Popup", composition: PaneComposition::Right, size: "50%", variant: PaneVariant::Floating, hide_on_collapse: true, expanded: expanded(), onexpandedchange: move |value| expanded.set(value), p { "This is a popup pane." } }
        div { class: "application-source-pane-controls", Button { onclick: move |_| expanded.set(!expanded()), "Toggle Expanded" } }
    } }
}

#[component]
fn PaneLayoutExample() -> Element {
    let mut floating = use_signal(|| true);
    let mut horizontal = use_signal(|| true);
    let mut left = use_signal(|| false);
    let mut right = use_signal(|| false);
    let mut top = use_signal(|| false);
    let mut bottom = use_signal(|| false);
    rsx! { div { class: "application-source-frame",
        PaneLayout { variant: if floating() { PaneVariant::Floating } else { PaneVariant::Inline }, borderless: floating(), layout: if horizontal() { PaneLayoutOrder::FullHorizontal } else { PaneLayoutOrder::FullVertical },
            left: rsx! { Pane { heading: "Pane Left", size: "33%", expanded: left(), onexpandedchange: move |value| left.set(value), p { "This is the left pane." } } },
            right: rsx! { Pane { heading: "Pane Right", size: "33%", expanded: right(), onexpandedchange: move |value| right.set(value), p { "This is the right pane." } } },
            top: rsx! { Pane { heading: "Pane Top", size: "33%", expanded: top(), onexpandedchange: move |value| top.set(value), p { "This is the top pane." } } },
            bottom: rsx! { Pane { heading: "Pane Bottom", size: "33%", expanded: bottom(), onexpandedchange: move |value| bottom.set(value), p { "This is the bottom pane." } } },
            div { class: "application-source-pane-controls", Button { onclick: move |_| floating.set(!floating()), "Toggle Variant" } Button { onclick: move |_| horizontal.set(!horizontal()), "Toggle Layout" } }
        }
    } }
}

#[component]
fn ThemeExample() -> Element {
    let mut mode = use_signal(|| ColorScheme::Dark);
    let mut system = use_signal(|| false);
    rsx! { IxTheme { scheme: if system() { ColorScheme::System } else { mode() }, class: "application-source-theme",
        LayoutGrid {
            Row { Col { size: 2, "Light/Dark" } Col { Button { disabled: system(), onclick: move |_| mode.set(if mode() == ColorScheme::Dark { ColorScheme::Light } else { ColorScheme::Dark }), "Toggle mode" } } }
            Row { Col { size: 2, "Theme" } Col { Select { label: "Theme", placeholder: "Select a theme", options: vec![SelectOption::new("light","light"),SelectOption::new("dark","dark")], value: mode().as_str(), disabled: system(), onvaluechange: move |value| mode.set(if value == "light" { ColorScheme::Light } else { ColorScheme::Dark }) } } }
            Row { Col { size: 2 } Col { Checkbox { label: "Use system", checked: system(), oncheckedchange: move |value| system.set(value) } } }
        }
    } }
}

#[component]
fn InfoPageExample() -> Element {
    let mut event = use_signal(|| "No event yet".to_string());
    rsx! { InfoPage { title_text: "Report could not be generated", copy_text: "An error occurred while generating the report.", instructions: "Contact your administrator for more information.", actions: rsx! { Button { onclick: move |_| event.set("Return to overview".into()), "Return to overview" } } }
        output { class: "application-source-events", "aria-live": "polite", "Event: {event}" }
    }
}

pub const STYLES: &str = r#"
.application-source-example{min-width:0}.application-source-frame{height:420px;position:relative;min-width:0;width:100%;isolation:isolate}.application-source-frame>.ix-application,.application-source-frame>.ix-pane-layout{height:100%}.application-source-events{display:block;margin-top:1rem;color:var(--ix-color-text-secondary);font-size:.875rem}.application-source-header-avatar .ix-menu-avatar-trigger{width:2.5rem;min-height:2.5rem;padding:.25rem}.application-source-header-avatar .ix-menu-avatar-text{display:none}.application-source-header-avatar .ix-menu-avatar-popup{top:100%;bottom:auto;inset-inline-start:auto;inset-inline-end:0}.application-source-pane{container-type:size}.application-source-pane>.ix-pane{height:100%}.application-source-pane-controls{display:flex;flex-wrap:wrap;gap:2.5rem;padding:2.5rem}.application-source-bottom-menu{width:16rem;max-width:100%}.application-source-theme{padding:1rem}.application-source-theme .ix-layout-col{display:flex;align-items:center;min-height:2.5rem}.application-source-theme .ix-layout-col>.ix-field{width:100%}.application-source-theme .ix-layout-col:first-child{overflow-wrap:anywhere}@media(max-width:576px){.application-source-frame{height:440px}.application-source-pane-controls{gap:1rem;padding:1.25rem}.application-source-theme{padding:.5rem}.application-source-theme .ix-layout-col:first-child{flex-basis:30%;flex-grow:0}.application-source-theme .ix-layout-col:not(:first-child){flex-basis:70%;flex-grow:0}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_application_sources_render_independently() {
        for name in SCENARIOS {
            let mut dom = VirtualDom::new_with_props(
                ApplicationScenario,
                ApplicationScenarioProps {
                    name: (*name).into(),
                },
            );
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(html.contains("application-source-example"), "{name}");
            assert!(!html.contains("No application source example"), "{name}");
            assert!(!html.contains("placeholder-logo"), "{name}");
        }
    }
    #[test]
    fn application_switch_preserves_the_source_navigation_configuration() {
        let config = source_switch_config();
        assert_eq!(config.current_app_id, "demo-app-2");
        assert_eq!(config.apps.len(), 2);
        assert_eq!(config.apps[0].name, "Floor App");
        assert_eq!(config.apps[1].name, "Calculator App");
        assert!(
            config
                .apps
                .iter()
                .all(|app| app.target == "_self" && app.url == "https://ix.siemens.io/")
        );
    }
}
