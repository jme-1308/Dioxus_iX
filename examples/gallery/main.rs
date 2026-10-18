//! Interactive, logo-free iX port review examples. All authored assets are Rust.
use dioxus::prelude::*;
use dioxus_ix::*;

mod action_scenarios;
mod actions;
mod advanced_chart_scenarios;
mod application;
mod application_scenarios;
mod button_review;
mod chart_scenarios;
mod chat;
mod chat_scenarios;
mod composition;
mod date_dropdown_review;
mod datetime;
mod datetime_scenarios;
mod dialog_scenarios;
mod dialogs;
mod dropdown_review;
mod feedback_scenarios;
mod filter_keyboard_review;
mod filter_mode_review;
mod filter_order_review;
mod filter_reset_review;
mod filter_state_review;
mod form_scenarios;
mod forms;
mod grid_scenarios;
mod header_layout_review;
mod header_nested_review;
mod header_switch_review;
mod icon_button_review;
mod icon_scenarios;
mod layout;
mod layout_scenarios;
mod menu_category_review;
mod menu_hover_review;
mod menu_navigation_review;
mod menu_overlay_review;
mod native_field_review;
mod number_input_review;
mod pattern_scenarios;
mod patterns;
mod source_examples;
mod tabs_review;
mod text_commit_review;
mod theme_review;
mod toggle_button_review;
mod tree;
mod workflow_review;

fn main() {
    dioxus::launch(Gallery);
}

#[derive(Clone, Copy, PartialEq)]
enum GalleryPage {
    Actions,
    Sources,
    Forms,
    Layout,
    DateTime,
    DataGrid,
    Tree,
    Application,
    Feedback,
    Composition,
    Chat,
    Patterns,
    Dialogs,
    Icons,
    Theme,
    Coverage,
}

impl GalleryPage {
    const ALL: &'static [Self] = &[
        Self::Actions,
        Self::Sources,
        Self::Forms,
        Self::Layout,
        Self::DateTime,
        Self::DataGrid,
        Self::Tree,
        Self::Application,
        Self::Feedback,
        Self::Composition,
        Self::Chat,
        Self::Patterns,
        Self::Dialogs,
        Self::Icons,
        Self::Theme,
        Self::Coverage,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Actions => "Actions & status",
            Self::Sources => "Source examples",
            Self::Forms => "Forms & selection",
            Self::Layout => "Layout & navigation",
            Self::Feedback => "Feedback & menus",
            Self::Composition => "Tabs & composition",
            Self::Chat => "Chat",
            Self::Patterns => "Filters & patterns",
            Self::Dialogs => "Dialogs & popovers",
            Self::DateTime => "Date & time",
            Self::DataGrid => "Data grid",
            Self::Tree => "Tree",
            Self::Application => "Application shell",
            Self::Icons => "Icons",
            Self::Theme => "Theme & typography",
            Self::Coverage => "Port coverage",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::Actions => "play",
            Self::Sources => "list",
            Self::Forms => "text",
            Self::Layout => "editor-grid",
            Self::Feedback => "notification",
            Self::Composition => "list",
            Self::Chat => "comment-alt",
            Self::Patterns => "editor-grid",
            Self::Dialogs => "application-screen",
            Self::Icons => "shapes",
            Self::Theme => "shapes",
            Self::DateTime => "calendar",
            Self::DataGrid => "editor-grid",
            Self::Tree => "hierarchy",
            Self::Application => "application-screen",
            Self::Coverage => "list",
        }
    }
}

#[component]
fn Gallery() -> Element {
    if let Some(app) = header_switch_review::navigation_destination() {
        return rsx! { header_switch_review::HeaderReviewDestination { app } };
    }
    #[cfg(all(target_arch = "wasm32", feature = "web", debug_assertions))]
    use_hook(|| {
        // Install after the web renderer's development hook so failed Rust
        // assertions remain visible in the browser console during review.
        std::panic::set_hook(Box::new(|info| {
            web_sys::console::error_1(&info.to_string().into());
        }));
    });
    let mut page = use_signal(|| GalleryPage::Actions);
    let mut dark = use_signal(|| true);
    let mut compact = use_signal(|| false);
    rsx! {
        document::Title { "Dioxus iX | Component examples" }
        style { {GALLERY_STYLES} }
        style { {actions::STYLES} }
        style { {application::STYLES} }
        style { {chat::STYLES} }
        style { {composition::STYLES} }
        style { {patterns::STYLES} }
        style { {forms::STYLES} }
        style { {layout::STYLES} }
        style { {datetime::STYLES} }
        style { {dialogs::STYLES} }
        style { {tree::STYLES} }
        style { {grid_scenarios::STYLES} }
        IxTheme {
            scheme: if dark() { ColorScheme::Dark } else { ColorScheme::Light },
            density: if compact() { Density::Compact } else { Density::Normal },
            class: "review-app",
            a { class: "skip-link", href: "#review-content", "Skip to examples" }
            header { class: "review-header",
                strong { class: "review-name", "Dioxus iX" }
                span { class: "review-subtitle", "Component examples" }
                div { class: "review-settings",
                    label { input { r#type: "checkbox", checked: compact(), onchange: move |event| compact.set(event.checked()) } "Compact" }
                    label { input { r#type: "checkbox", checked: dark(), onchange: move |event| dark.set(event.checked()) } "Dark theme" }
                }
            }
            div { class: "review-workspace",
                nav { class: "review-navigation", "aria-label": "Example categories",
                    label { class: "review-mobile-category", "Example category"
                        select { value: page().label(), onchange: move |event| {
                            if let Some(category) = GalleryPage::ALL.iter().find(|category| category.label() == event.value()) { page.set(*category); }
                        },
                            for category in GalleryPage::ALL.iter().copied() { option { value: category.label(), selected: page() == category, "{category.label()}" } }
                        }
                    }
                    for category in GalleryPage::ALL.iter().copied() {
                        button { r#type: "button", class: if page() == category { "review-nav-item selected" } else { "review-nav-item" },
                            "aria-current": if page() == category { Some("page") } else { None },
                            onclick: move |_| page.set(category),
                            Icon { name: category.icon(), size: 20 }
                            "{category.label()}"
                        }
                    }
                    div { class: "review-nav-footer",
                        "Rust components" br {} "Dioxus 0.8.0-alpha.2" br {}
                        a { href: "https://github.com/siemens/ix", target: "_blank", rel: "noopener noreferrer", "Upstream reference" }
                    }
                }
                main { id: "review-content", class: "review-content", tabindex: "-1",
                    h1 { "{page().label()}" }
                    p { class: "review-intro", "Interactive Rust ports of iX example scenarios. This is a work in progress; exact Brand tokens and complete upstream parity are still being verified." }
                    match page() {
                        GalleryPage::Actions => rsx! { toggle_button_review::ToggleButtonReview {} icon_button_review::IconButtonReview {} button_review::ButtonReview {} actions::ActionExamples {} },
                        GalleryPage::Sources => rsx! { source_examples::SourceExamples {} },
                        GalleryPage::Forms => rsx! { text_commit_review::TextCommitReview {} native_field_review::NativeFieldReview {} number_input_review::NumberInputReview {} forms::FormExamples {} },
                        GalleryPage::Layout => rsx! { workflow_review::WorkflowReview {}  layout::LayoutExamples {} },
                        GalleryPage::DateTime => rsx! { date_dropdown_review::DateDropdownReview {} datetime::DateTimeExamples {} },
                        GalleryPage::DataGrid => rsx! { grid_scenarios::GridExamples {} },
                        GalleryPage::Tree => rsx! { tree::TreeExamples {} },
                        GalleryPage::Application => rsx! { header_nested_review::HeaderNestedReview {} header_layout_review::HeaderLayoutReview {} header_switch_review::HeaderSwitchReview {} menu_hover_review::MenuHoverReview {} menu_category_review::MenuCategoryReview {} menu_navigation_review::MenuNavigationReview {} menu_overlay_review::MenuOverlayReview {} application::ApplicationExamples {} },
                        GalleryPage::Feedback => rsx! { dropdown_review::DropdownOrderReview {} FeedbackExamples {} },
                        GalleryPage::Composition => rsx! { tabs_review::TabsReviewExamples {} composition::CompositionExamples {} },
                        GalleryPage::Chat => rsx! { chat::ChatExamples {} },
                        GalleryPage::Patterns => rsx! { filter_keyboard_review::FilterKeyboardReview {} filter_order_review::FilterOrderReview {} filter_state_review::FilterStateReview {} filter_mode_review::FilterModeReview {} filter_reset_review::FilterResetReview {}  patterns::PatternExamples {} },
                        GalleryPage::Dialogs => rsx! { dialogs::DialogExamples {} },
                        GalleryPage::Icons => rsx! { IconExamples {} },
                        GalleryPage::Theme => rsx! { theme_review::ThemeReview {} },
                        GalleryPage::Coverage => rsx! { CoverageExamples {} },
                    }
                }
            }
        }
    }
}

#[component]
fn CoverageExamples() -> Element {
    let mut query = use_signal(String::new);
    let declared_tags = PORT_CATALOG
        .iter()
        .filter(|entry| entry.upstream_tag().is_some())
        .count();
    let mapped_tags = PORT_CATALOG
        .iter()
        .filter(|entry| entry.upstream_tag().is_some() && entry.rust_api.is_some())
        .count();
    let needle = query().to_lowercase();
    rsx! {
        section { class: "demo-section",
            h2 { "Upstream coverage" }
            p { "This inventory covers {declared_tags} current core component tags; deprecated components and legacy examples are excluded. {mapped_tags} tags have an initial Rust component, model, composition or internal integration. These are different mapping types; none is claimed to have complete verified parity. Source examples contains {UPSTREAM_EXAMPLE_NAMES.len()} Rust adaptations for reviewing current upstream HTML scenarios." }
            label { class: "review-icon-search", "Filter components"
                input { r#type: "search", value: query(), oninput: move |event| query.set(event.value()) }
            }
            table { class: "review-coverage",
                thead { tr { th { "Upstream tag" } th { "Rust mapping" } th { "Review status" } } }
                tbody {
                    for entry in PORT_CATALOG.iter().filter(|entry| entry.upstream.contains(&needle) || entry.upstream_tag().is_some_and(|tag| tag.contains(&needle))) {
                        tr { td { code { {entry.upstream_tag().unwrap_or_else(|| entry.upstream.to_string())} } }
                            td { {entry.rust_api.unwrap_or("—")} }
                            td { "{entry.mapping_kind().label()}" if entry.rust_api.is_some() { "; parity review pending" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FeedbackExamples() -> Element {
    let mut selected = use_signal(|| "Nothing selected".to_string());
    let mut show_toast = use_signal(|| false);
    let mut reset = use_signal(|| 0);
    rsx! {
        section { class: "demo-section", h2 { "Message bar" }
            p { "Persistent information, warnings, and an alarm with an action, as in message-bar.html." }
            div { class: "demo-stack", key: "messages-{reset}",
                MessageBar { persistent: true, "Message text" }
                MessageBar { persistent: true, message_type: MessageType::Warning, "Message text" }
                MessageBar { persistent: true, message_type: MessageType::Alarm,
                    div { class: "demo-row", "Message text" Button { onclick: move |_| selected.set("Alarm acknowledged".into()), "Action" } }
                }
                MessageBar { message_type: MessageType::Success, onclose: move |_| selected.set("Message closed".into()), "Closable success message" }
            }
            div { class: "demo-row", Button { variant: ButtonVariant::Secondary, onclick: move |_| reset += 1, "Restore messages" } }
        }
        section { class: "demo-section", h2 { "Dropdown" }
            p { "Category labels, checked and disabled items from dropdown.html and dropdown-item examples." }
            Dropdown { label: "Open",
                DropdownHeader { label: "Category" }
                DropdownItem { label: "Item 2", onclick: move |_| selected.set("Item 2".into()) }
                DropdownItem { label: "Item 3", checkable: true, checked: selected() == "Item 3", onclick: move |_| selected.set("Item 3".into()) }
                DropdownItem { label: "Item 4", disabled: true }
                Divider {}
                DropdownItem { label: "Item 5", icon: "cogwheel", onclick: move |_| selected.set("Item 5".into()) }
            }
            p { role: "status", "{selected}" }
        }
        section { class: "demo-section", h2 { "Tooltip" }
            Tooltip { id: "review-tooltip", label: "Additional information", Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip", "Hover or focus" } }
            TooltipReviewExamples {}
        }
        section { class: "demo-section", h2 { "Toast" }
            Button { onclick: move |_| show_toast.set(true), "Show toast" }
            if show_toast() {
                div { class: "review-toast",
                    Toast { title: "Configuration saved", message_type: MessageType::Success, onclose: move |_| show_toast.set(false),
                        "This demonstration notification stays visible until dismissed."
                    }
                }
            }
        }
    }
}

#[component]
fn TooltipReviewExamples() -> Element {
    let mut actions = use_signal(|| 0usize);
    rsx! {
        h3 { "Clipping, shared triggers and interactive content" }
        p { "Focus or hover a trigger to show its tooltip. The first tooltip escapes a clipped host. The next two triggers share one tooltip. Escape dismisses the open tooltip." }
        div { class: "review-tooltip-clip",
            span { id: "review-tooltip-clipped-anchor", class: "review-tooltip-trigger",
                Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-clipped", "Clipped tooltip trigger" }
            }
            TooltipFor { id: "review-tooltip-clipped", anchor_id: "review-tooltip-clipped-anchor", placement: TooltipPlacement::Bottom,
                "This tooltip remains visible outside the clipped container."
            }
        }
        div { class: "demo-row",
            span { "data-review-tooltip": "shared", class: "review-tooltip-trigger",
                Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-shared", "Shared trigger one" }
            }
            span { "data-review-tooltip": "shared", class: "review-tooltip-trigger",
                Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-shared", "Shared trigger two" }
            }
        }
        TooltipFor { id: "review-tooltip-shared", anchor_selectors: vec!["[data-review-tooltip=shared]".into()],
            title: "Shared contextual help", title_icon: "info", "One tooltip follows the active trigger."
        }
        span { id: "review-tooltip-interactive-anchor", class: "review-tooltip-trigger",
            Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-interactive", "Interactive tooltip trigger" }
        }
        TooltipFor { id: "review-tooltip-interactive", anchor_id: "review-tooltip-interactive-anchor", interactive: true,
            placement: TooltipPlacement::Right, title: "Review action",
            p { "The tooltip stays open while you interact with its content." }
            Button { variant: ButtonVariant::Secondary, onclick: move |_| actions += 1, "Apply tooltip action" }
        }
        p { role: "status", "Tooltip actions applied: {actions}" }
    }
}

#[component]
fn IconExamples() -> Element {
    let mut query = use_signal(String::new);
    let names: Vec<_> = icon_names()
        .filter(|name| name.contains(&query().to_lowercase()))
        .collect();
    rsx! {
        section { class: "demo-section",
            h2 { "Industrial pictograms" }
            p { "{names.len()} matching icons. Logos and brand marks are excluded." }
            label { class: "review-icon-search", "Filter icons"
                input { r#type: "search", value: query(), placeholder: "Search by upstream name", oninput: move |event| query.set(event.value()) }
            }
            div { class: "review-icons",
                for name in names.into_iter().take(180) {
                    div { class: "review-icon-item", Icon { name, size: 32, label: name } code { "{name}" } }
                }
            }
            p { "Showing up to 180 at once. Filter to find more." }
        }
    }
}

const GALLERY_STYLES: &str = r#"
html,body { margin:0;min-height:100%; }
.review-app { min-height:100vh; }
.review-header { display:flex;align-items:center;gap:24px;min-height:56px;padding:8px 24px;border-bottom:1px solid var(--ix-color-border);background:var(--ix-color-surface-1); }
.review-name { font-size:20px;white-space:nowrap; }
.review-subtitle { color:var(--ix-color-text-secondary);font-size:14px; }
.review-settings { display:flex;gap:20px;margin-left:auto;font-size:14px; }
.review-settings label { display:flex;align-items:center;gap:6px; }
.review-workspace { display:grid;grid-template-columns:232px minmax(0,1fr);min-height:calc(100vh - 57px); }
.review-navigation { padding:16px 8px;display:flex;flex-direction:column;border-right:1px solid var(--ix-color-border);background:var(--ix-color-surface-1); }
.review-mobile-category { display:none; }
.review-nav-item { display:flex;align-items:center;gap:12px;text-align:left;min-height:44px;border:0;padding:8px 16px;color:var(--ix-color-text);background:transparent;border-radius:var(--ix-radius);font-size:14px; }
.review-nav-item:hover { background:var(--ix-color-surface-2); }
.review-nav-item.selected { color:var(--ix-color-primary);background:color-mix(in srgb,var(--ix-color-primary) 12%,transparent); }
.review-nav-footer { margin-top:auto;padding:32px 16px 12px;font-size:12px;line-height:1.8;color:var(--ix-color-text-secondary); }
.review-content { width:100%;max-width:1440px;padding:32px 40px 80px;min-width:0; }
.review-content h1 { margin:0 0 8px;font-size:28px;line-height:1.3;font-weight:600; }
.review-intro { max-width:85ch;margin:0 0 32px;color:var(--ix-color-text-secondary);font-size:14px; }
.demo-section { padding:0 0 32px;margin:0 0 32px;border-bottom:1px solid var(--ix-color-border);min-width:0; }
.demo-section h2 { margin:0 0 16px;font-size:22px;font-weight:600; }
.demo-section h3 { margin:24px 0 12px;font-size:18px; }
.demo-section>p { color:var(--ix-color-text-secondary);font-size:14px;max-width:80ch; }
.demo-row { display:flex;align-items:center;flex-wrap:wrap;gap:12px;margin:12px 0; }
.demo-stack { display:flex;flex-direction:column;gap:16px;max-width:760px; }
.demo-label { min-width:130px;font-size:14px;color:var(--ix-color-text-secondary); }
.demo-grid { display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,260px),1fr));gap:24px; }
.review-toast { position:fixed;right:24px;bottom:24px;z-index:50;max-width:calc(100vw - 48px); }
.review-tooltip-clip{height:64px;max-width:320px;overflow:hidden;padding:8px;margin-block:16px;border:1px solid var(--ix-color-border)}.review-tooltip-trigger{display:inline-flex;max-width:100%}
.review-icons { display:grid;grid-template-columns:repeat(auto-fill,minmax(120px,1fr));gap:1px;background:var(--ix-color-border);border:1px solid var(--ix-color-border);margin-top:24px; }
.review-icon-item { display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;min-height:116px;background:var(--ix-color-surface-1);padding:12px;min-width:0;text-align:center; }
.review-icon-item code { font-size:11px;overflow-wrap:anywhere; }
.review-icon-search { display:flex;flex-direction:column;gap:8px;max-width:380px; }
.review-icon-search input { padding:8px 12px;color:var(--ix-color-text);background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius); }
.review-coverage { width:100%;border-collapse:collapse;margin-top:24px;font-size:14px; }
.review-coverage th,.review-coverage td { text-align:left;border-bottom:1px solid var(--ix-color-border);padding:10px 12px;overflow-wrap:anywhere; }
.skip-link { position:fixed;top:0;left:16px;transform:translateY(-150%);z-index:100;padding:8px 16px;background:var(--ix-color-primary);color:var(--ix-color-on-primary)!important; }
.skip-link:focus { transform:translateY(0); }
@media(max-width:900px) { .review-content { padding:24px; }.review-workspace {grid-template-columns:192px minmax(0,1fr);}.review-header {gap:12px}.review-subtitle {display:none} }
@media(max-width:640px) { .review-header {padding:12px 16px;flex-wrap:wrap}.review-settings {gap:12px}.review-workspace {display:block}.review-navigation {border-right:0;border-bottom:1px solid var(--ix-color-border);padding:12px 16px}.review-nav-item {display:none}.review-mobile-category {display:flex;flex-direction:column;gap:8px;font-size:12px;width:100%}.review-mobile-category select{width:100%;min-width:0;height:40px;padding:8px 12px;font:inherit;font-size:14px;color:var(--ix-color-text);background:var(--ix-color-surface);border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.review-nav-footer {display:none}.review-content {padding:24px 16px}.demo-label {width:100%}.review-content h1 {font-size:24px} }
"#;
