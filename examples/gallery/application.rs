//! Logo-free counterparts of the upstream application, menu and pane examples.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn ApplicationExamples() -> Element {
    let mut content = use_signal(|| "home".to_string());
    let mut expanded = use_signal(|| false);
    let mut forced = use_signal(|| "auto".to_string());
    let mut pinned = use_signal(|| false);
    let mut dark = use_signal(|| true);
    let mut notifications = use_signal(|| true);
    let mut status =
        use_signal(|| "Use the application menu to switch the example content.".to_string());
    let breakpoint = match forced().as_str() {
        "sm" => Some(ApplicationBreakpoint::Small),
        "md" => Some(ApplicationBreakpoint::Medium),
        "lg" => Some(ApplicationBreakpoint::Large),
        _ => None,
    };
    rsx! {
        section { class: "demo-section", id: "application-shell",
            h2 { "Application / advanced" }
            p { "The upstream Home, Alarm, Analysis → Plant and Network navigation. Company logos are omitted." }
            div { class: "application-demo-controls",
                Select { label: "Breakpoint", value: forced(), options: vec![SelectOption::new("auto", "Responsive"), SelectOption::new("sm", "Small"), SelectOption::new("md", "Medium"), SelectOption::new("lg", "Large")],
                    onvaluechange: move |value| forced.set(value) }
                Checkbox { label: "Pin expanded menu", checked: pinned(), oncheckedchange: move |value| pinned.set(value) }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| expanded.set(!expanded()), "Toggle menu" }
            }
            div { class: "application-demo-frame",
                IxTheme { scheme: if dark() { ColorScheme::Dark } else { ColorScheme::Light },
                    Application { menu_expanded: expanded, force_breakpoint: breakpoint, pinned_menu: pinned(),
                        header: rsx! { ApplicationHeader { name: "Dioxus iX", name_suffix: "My Application", app_icon: "application-screen",
                            actions: rsx! { Avatar { username: "Example User", initials: "EU" } } } },
                        menu: rsx! {
                            Menu { enable_toggle_theme: true, ontoggletheme: move |_| dark.set(!dark()),
                                avatar: rsx! { MenuAvatar { top: "Example User", bottom: "Additional info", initials: "EU",
                                    onlogout: move |_| status.set("Logout requested".into()),
                                    MenuAvatarItem { label: "My profile", icon: "user", onclick: move |_| status.set("Profile selected".into()) }
                                    MenuAvatarItem { label: "Change password", icon: "lock", onclick: move |_| status.set("Change password selected".into()) }
                                } },
                                settings: rsx! { MenuSettings {
                                    div { class: "application-demo-settings",
                                        h3 { "Notifications" }
                                        Toggle { checked: notifications(), label: "Enable notifications", oncheckedchange: move |value| notifications.set(value) }
                                        p { role: "status", "Notifications enabled: {notifications}" }
                                        h3 { "Appearance" }
                                        Toggle { checked: dark(), text_on: "Dark", text_off: "Light", label: "Dark appearance", oncheckedchange: move |value| dark.set(value) }
                                    }
                                } },
                                about: rsx! { MenuAbout {
                                    h3 { "Dioxus iX" }
                                    p { "Independent Rust component port of Industrial Experience." }
                                    p { "Component library version: 0.1.0. Dioxus: 0.8.0-alpha.2." }
                                    p { "This example does not include vendor logos or claim vendor affiliation." }
                                } },
                                MenuItem { label: "Home", icon: "home", home: true, active: content() == "home", onclick: move |_| content.set("home".into()) }
                                MenuItem { label: "Alarm", icon: "alarm-bell", notifications: 3, active: content() == "alarm", onclick: move |_| content.set("alarm".into()) }
                                MenuCategory { label: "Analysis", icon: "piechart", active: content() == "plant" || content() == "network",
                                    MenuItem { label: "Plant", icon: "plant", active: content() == "plant", onclick: move |_| content.set("plant".into()) }
                                    MenuItem { label: "Network", icon: "network-device", active: content() == "network", onclick: move |_| content.set("network".into()) }
                                }
                                MenuItem { label: "Disabled item", icon: "lock", disabled: true }
                            }
                        },
                        footer: rsx! { div { class: "application-demo-footer", "Example application footer" } },
                        div { class: "application-demo-content",
                            ContentHeader { title: format!("Example {} content", content()), subtitle: "Application navigation example" }
                            p { "Select a menu item, expand Analysis, open Settings or inspect the user dropdown." }
                            p { "On a small viewport the rail collapses, and the header button opens navigation." }
                            p { role: "status", "{status}" }
                        }
                    }
                }
            }
        }
        PaneExamples {}
        section { class: "demo-section", id: "layout-grid",
            h2 { "Layout grid" }
            p { "The 12-column layout with responsive column sizes and the upstream 24px gap." }
            LayoutGrid { no_margin: true,
                Row {
                    Col { size: 8, size_sm: 12, div { class: "application-grid-cell", "8 columns · 12 on small screens" } }
                    Col { size: 4, size_sm: 12, div { class: "application-grid-cell", "4 columns · 12 on small screens" } }
                }
                Row {
                    for _ in 0..3 { Col { size: 4, size_sm: 12, div { class: "application-grid-cell", "4 columns" } } }
                }
                Row { Col { div { class: "application-grid-cell", "Equal column" } } Col { div { class: "application-grid-cell", "Equal column" } } }
            }
        }
    }
}

#[component]
fn PaneExamples() -> Element {
    let mut left = use_signal(|| false);
    let mut right = use_signal(|| false);
    let mut top = use_signal(|| false);
    let mut bottom = use_signal(|| false);
    let mut floating = use_signal(|| true);
    let mut horizontal = use_signal(|| true);
    let mut keep_open = use_signal(|| false);
    let mut note = use_signal(String::new);
    let mut pane_event = use_signal(|| "No pane change requested".to_string());
    let variant = if floating() {
        PaneVariant::Floating
    } else {
        PaneVariant::Inline
    };
    let variant_label = if floating() { "floating" } else { "inline" };
    let layout_label = if horizontal() {
        "full-horizontal"
    } else {
        "full-vertical"
    };
    rsx! {
        section { class: "demo-section", id: "pane-layout",
            h2 { "Pane layout" }
            p { "Left, top, right and bottom panes with controlled expansion, inline/floating variants and both compositions." }
            div { class: "demo-row",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| floating.set(!floating()), "Toggle Variant" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| horizontal.set(!horizontal()), "Toggle Layout" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { left.set(false); right.set(false); top.set(false); bottom.set(false); }, "Collapse all" }
            }
            p { role: "status", "Variant: {variant_label}. Layout: {layout_label}." }
            div { class: "application-pane-demo",
                PaneLayout { variant, borderless: floating(), layout: if horizontal() { PaneLayoutOrder::FullHorizontal } else { PaneLayoutOrder::FullVertical },
                    left: rsx! { Pane { heading: "Pane Left", expanded: left(), size: "33%", close_on_click_outside: true,
                        onbeforeexpandedchange: move |event: PaneExpandedChangeEvent| {
                            if !event.expanded && keep_open() { event.prevent_default(); pane_event.set("Closing prevented while the note is protected".into()); }
                        },
                        onexpandedchange: move |value| { left.set(value); pane_event.set(if value { "Left pane opened" } else { "Left pane closed" }.into()); },
                        p { "This is the left pane." }
                        label { class: "application-pane-note", "Pane note"
                            input { autofocus: true, value: note(), oninput: move |event| note.set(event.value()) }
                        }
                        Checkbox { label: "Keep pane open", checked: keep_open(), oncheckedchange: move |value| keep_open.set(value) }
                    } },
                    top: rsx! { Pane { heading: "Pane Top", expanded: top(), size: "33%", onexpandedchange: move |value| top.set(value), p { "This is the top pane." } } },
                    right: rsx! { Pane { heading: "Pane Right", expanded: right(), size: "33%", onexpandedchange: move |value| right.set(value), p { "This is the right pane." } } },
                    bottom: rsx! { Pane { heading: "Pane Bottom", expanded: bottom(), size: "33%", onexpandedchange: move |value| bottom.set(value), p { "This is the bottom pane." } } },
                    div { class: "application-pane-content",
                        h3 { "Content" }
                        p { "Use a pane header to expand or collapse that side." }
                        div { class: "demo-row",
                            Button { variant: ButtonVariant::Secondary, onclick: move |event: MouseEvent| { event.stop_propagation(); left.set(!left()); }, "Toggle left" }
                            Button { variant: ButtonVariant::Secondary, onclick: move |event: MouseEvent| { event.stop_propagation(); right.set(!right()); }, "Toggle right" }
                        }
                    }
                }
            }
            p { class: "application-pane-events", role: "status", "{pane_event}" }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.application-demo-controls{display:flex;align-items:end;flex-wrap:wrap;gap:16px;margin-block:24px}.application-demo-controls>.ix-field{width:180px}.application-demo-frame{height:540px;border:1px solid var(--ix-color-border);margin-block:24px;isolation:isolate}.application-demo-frame>.ix-theme{height:100%;min-height:0}.application-demo-content{padding:24px}.application-demo-content p{max-width:65ch}.application-demo-footer{padding:8px 24px;border-top:1px solid var(--ix-color-border);font-size:12px;color:var(--ix-color-text-secondary)}.application-demo-settings{max-width:500px}.application-pane-demo{height:400px;border:1px solid var(--ix-color-border);margin-block:24px}.application-pane-content{padding:24px}.application-grid-cell{padding:16px;background:var(--ix-color-surface-2);font-size:14px;color:var(--ix-color-text);min-height:48px}@media(max-width:576px){.application-demo-frame{height:520px}.application-demo-content{padding:16px}.application-pane-content{padding:12px}}
.application-pane-note{display:flex;flex-direction:column;gap:.5rem;font-size:.875rem;margin-block:1rem}.application-pane-note input{box-sizing:border-box;width:100%;min-width:0;min-height:2rem;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);padding:.375rem .5rem;background:var(--ix-color-surface);color:var(--ix-color-text);font:inherit}.application-pane-note input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}
"#;
