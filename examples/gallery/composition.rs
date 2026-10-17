//! Keyed tab content and composed dropdown/settings/about examples.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn CompositionExamples() -> Element {
    let mut active = use_signal(|| "tab-1".to_string());
    let mut settings_open = use_signal(|| false);
    let mut about_open = use_signal(|| false);
    let mut news_open = use_signal(|| true);
    let mut settings_tab = use_signal(|| "general".to_string());
    let mut about_tab = use_signal(|| "about".to_string());
    let mut notifications = use_signal(|| true);
    let mut last_action = use_signal(|| "No action selected".to_string());
    let tabs = vec![
        TabItem::new("tab-1", "Tab 1"),
        TabItem::new("tab-2", "Tab 2"),
        TabItem::new("tab-3", "Tab 3"),
    ];
    rsx! {
        section { class: "demo-section", id: "tab-set",
            h2 { "Tabs and linked panels" }
            p { "The upstream Tab 1, Tab 2, and Tab 3 example. Panels are associated by stable keys and retain their state while hidden." }
            TabSet { items: tabs, active_key: active(), onselectedchange: move |key| active.set(key),
                TabPanel { tab_key: "tab-1", "Content Tab 1" }
                TabPanel { tab_key: "tab-2", PersistentTabDraft {} }
                TabPanel { tab_key: "tab-3", "Content Tab 3" }
            }
            p { role: "status", "Selected key: {active}" }
        }
        section { class: "demo-section", id: "dropdown-buttons",
            h2 { "Dropdown button" }
            div { class: "demo-row",
                DropdownButton { label: "Dropdown button",
                    DropdownItem { label: "Item 1", onclick: move |_| last_action.set("Item 1".into()) }
                    DropdownItem { label: "Item 2", onclick: move |_| last_action.set("Item 2".into()) }
                }
                DropdownButton { label: "With icon", icon: "cut", variant: ButtonVariant::Secondary,
                    DropdownItem { label: "Cut", icon: "cut", onclick: move |_| last_action.set("Cut".into()) }
                    DropdownItem { label: "Delete", icon: "trashcan", onclick: move |_| last_action.set("Delete".into()) }
                }
                DropdownButton { label: "Disabled", disabled: true, DropdownItem { label: "Item 1" } }
                DropdownButton { label: "Loading", loading: true, DropdownItem { label: "Item 1" } }
            }
        }
        section { class: "demo-section", id: "dropdown-quick-actions",
            h2 { "Dropdown quick actions" }
            p { "The upstream cut, bulb, trashcan, star, and document example, with visible action feedback." }
            Dropdown { label: "Open",
                DropdownQuickActions {
                    DropdownItem { icon: "cut", label: "Cut", compact: true, onclick: move |_| last_action.set("Cut".into()) }
                    DropdownItem { icon: "bulb", label: "Light", compact: true, onclick: move |_| last_action.set("Light".into()) }
                    DropdownItem { icon: "trashcan", label: "Delete", compact: true, onclick: move |_| last_action.set("Delete".into()) }
                }
                Divider {}
                DropdownItem { icon: "star", label: "Item 1", onclick: move |_| last_action.set("Item 1".into()) }
                DropdownItem { icon: "document", label: "Item 2", onclick: move |_| last_action.set("Item 2".into()) }
                DropdownItem { icon: "bulb", label: "Item 3", onclick: move |_| last_action.set("Item 3".into()) }
            }
            p { role: "status", "{last_action}" }
        }
        section { class: "demo-section", id: "settings-about",
            h2 { "Settings and about tabs" }
            p { "Settings and about sections use keyed tabs and panels. This preview contains illustrative application information." }
            div { class: "demo-row",
                Button { onclick: move |_| settings_open.set(!settings_open()), "Toggle settings" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| about_open.set(!about_open()), "Toggle about" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| news_open.set(true), "Restore news" }
            }
            MenuSettings { show: settings_open(), onclose: move |_| settings_open.set(false),
                TabSet { items: vec![TabItem::new("general", "General"), TabItem::new("notifications", "Notifications")],
                    active_key: settings_tab(), onselectedchange: move |key| settings_tab.set(key), label: "Settings sections",
                    TabPanel { tab_key: "general", h3 { "General settings" } p { "Example application settings." } }
                    TabPanel { tab_key: "notifications", Toggle { label: "Enable notifications", checked: notifications(), oncheckedchange: move |value| notifications.set(value) } }
                }
            }
            MenuAbout { show: about_open(), onclose: move |_| about_open.set(false),
                TabSet { items: vec![TabItem::new("about", "About"), TabItem::new("news", "What's new"), TabItem::new("legal", "Legal")],
                    active_key: about_tab(), onselectedchange: move |key| about_tab.set(key), label: "About sections",
                    TabPanel { tab_key: "about", h3 { "Dioxus iX examples" } p { "An independent Rust port review application." } }
                    TabPanel { tab_key: "news", h3 { "Example release note" } p { "This is illustrative news content for reviewing the about panel." } }
                    TabPanel { tab_key: "legal", p { "See the repository README for license and upstream attribution." } }
                }
            }
            MenuAboutNews { label: "What's new", show: news_open(), about_item_label: "Example release note", active_about_tab_key: "news",
                onclose: move |_| news_open.set(false),
                onshowmore: move |key: Option<String>| { about_tab.set(key.unwrap_or_else(|| "news".into())); about_open.set(true); },
                "Illustrative content. Open the associated about tab to review the full example."
            }
        }
    }
}

#[component]
fn PersistentTabDraft() -> Element {
    let mut value = use_signal(String::new);
    rsx! { div { p { "Content Tab 2" }
        Input { label: "Tab 2 draft", value: value(), onvaluechange: move |next| value.set(next) }
    } }
}

pub(crate) const STYLES: &str = r#"
.ix-theme #settings-about>.ix-menu-overlay-panel { position:relative;inset:auto;max-width:720px;max-height:none;margin-block:16px; }
.ix-theme #settings-about .ix-about-news { margin-top:20px; }
"#;
