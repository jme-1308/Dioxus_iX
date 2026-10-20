//! Interactive, logo-free iX port review examples. All authored assets are Rust.
use dioxus::prelude::*;
use dioxus_ix::*;

mod action_scenarios;
mod actions;
mod advanced_chart_scenarios;
mod application;
mod application_scenarios;
mod asset_flip_tile;
mod asset_hierarchy;
mod asset_workspace;
mod button_review;
mod card_collections;
mod chart_scenarios;
mod chart_workflows;
mod chat;
mod chat_composer_review;
mod chat_scenarios;
mod checkbox_review;
mod chip_workflows;
mod composition;
mod date_dropdown_review;
mod datetime;
mod datetime_scenarios;
mod dialog_scenarios;
mod dialogs;
mod dropdown_review;
mod equipment_events;
mod expanding_search_review;
mod feedback_scenarios;
mod filter_keyboard_review;
mod filter_mode_review;
mod filter_order_review;
mod filter_reset_review;
mod filter_state_review;
mod form_scenarios;
mod forms;
mod grid_cell_renderers;
mod grid_custom_editors;
mod grid_numeric_values;
mod grid_scenarios;
mod grid_typed_editors;
mod grid_worklist;
mod group_review;
mod header_layout_review;
mod header_nested_review;
mod header_switch_review;
mod icon_button_review;
mod icon_catalog;
mod icon_scenarios;
mod icon_toggle_button_review;
mod input_review;
mod inspection_groups;
mod layout;
mod layout_scenarios;
mod localized_dates;
mod maintenance_launch;
mod menu_category_review;
mod menu_hover_review;
mod menu_navigation_review;
mod menu_overlay_review;
mod native_field_review;
mod navigation;
mod number_input_review;
mod number_input_token_review;
mod operator_profile;
mod paged_equipment;
mod pattern_scenarios;
mod patterns;
mod progress_review;
mod radio_review;
mod select_visual_review;
mod service_package;
mod service_request;
mod slider_review;
mod source_examples;
mod spinner_review;
mod split_work_orders;
mod tabs_review;
mod text_commit_review;
mod theme_review;
mod toast_review;
mod toggle_button_review;
mod toggle_review;
mod tree;
mod upload_review;
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
    Charts,
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
        Self::Charts,
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
    fn slug(self) -> &'static str {
        match self {
            Self::Actions => "actions",
            Self::Sources => "sources",
            Self::Forms => "forms",
            Self::Layout => "layout",
            Self::DateTime => "datetime",
            Self::DataGrid => "grid",
            Self::Charts => "charts",
            Self::Tree => "tree",
            Self::Application => "application",
            Self::Feedback => "feedback",
            Self::Composition => "composition",
            Self::Chat => "chat",
            Self::Patterns => "patterns",
            Self::Dialogs => "dialogs",
            Self::Icons => "icons",
            Self::Theme => "theme",
            Self::Coverage => "coverage",
        }
    }
    fn from_slug(slug: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|page| page.slug() == slug)
            .unwrap_or(Self::Actions)
    }
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
            Self::Charts => "Charts",
            Self::Tree => "Tree",
            Self::Application => "Application shell",
            Self::Icons => "Icons",
            Self::Theme => "Theme & typography",
            Self::Coverage => "Components",
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
            Self::Charts => "chart-diagram",
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
    let mut location = navigation::use_gallery_location();
    let page = GalleryPage::from_slug(&location.read().page);
    let dark = location.read().dark;
    let compact = location.read().compact;
    rsx! {
        document::Title { "Dioxus iX | Component examples" }
        style { {GALLERY_STYLES} }
        style { {actions::STYLES} }
        style { {application::STYLES} }
        style { {chat::STYLES} }
        style { {chat_composer_review::STYLES} }
        style { {chip_workflows::STYLES} }
        style { {icon_catalog::STYLES} }
        style { {composition::STYLES} }
        style { {patterns::STYLES} }
        style { {asset_flip_tile::STYLES} }
        style { {asset_hierarchy::STYLES} }
        style { {forms::STYLES} }
        style { {layout::STYLES} }
        style { {card_collections::STYLES} }
        style { {paged_equipment::STYLES} }
        style { {datetime::STYLES} }
        style { {localized_dates::STYLES} }
        style { {dialogs::STYLES} }
        style { {tree::STYLES} }
        style { {toast_review::STYLES} }
        style { {grid_scenarios::STYLES} }
        style { {grid_worklist::STYLES} }
        style { {grid_typed_editors::STYLES} }
        style { {grid_cell_renderers::STYLES} }
        style { {grid_numeric_values::STYLES} }
        style { {grid_custom_editors::STYLES} }
        style { {split_work_orders::STYLES} }
        style { {chart_workflows::STYLES} }
        style { {service_request::STYLES} }
        style { {service_package::STYLES} }
        style { {equipment_events::STYLES} }
        style { {maintenance_launch::STYLES} }
        style { {asset_workspace::STYLES} }
        style { {inspection_groups::STYLES} }
        style { {operator_profile::STYLES} }
        IxTheme {
            scheme: if dark { ColorScheme::Dark } else { ColorScheme::Light },
            density: if compact { Density::Compact } else { Density::Normal },
            class: "review-app",
            a { class: "skip-link", href: "#review-content", "Skip to examples" }
            header { class: "review-header",
                strong { class: "review-name", "Dioxus iX" }
                span { class: "review-subtitle", "Component examples" }
                div { class: "review-settings",
                    label { input { r#type: "checkbox", checked: compact, onchange: move |event| location.write().compact = event.checked() } "Compact" }
                    label { input { r#type: "checkbox", checked: dark, onchange: move |event| location.write().dark = event.checked() } "Dark theme" }
                }
            }
            div { class: "review-workspace",
                nav { class: "review-navigation", "aria-label": "Example categories",
                    label { class: "review-mobile-category", "Example category"
                        select { value: page.slug(), onchange: move |event| location.write().page = event.value(),
                            for category in GalleryPage::ALL.iter().copied() { option { value: category.slug(), selected: page == category, "{category.label()}" } }
                        }
                    }
                    for category in GalleryPage::ALL.iter().copied() {
                        button { r#type: "button", class: if page == category { "review-nav-item selected" } else { "review-nav-item" },
                            "aria-current": if page == category { Some("page") } else { None },
                            onclick: move |_| location.write().page = category.slug().into(),
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
                    h1 { "{page.label()}" }
                    div { class: "review-view-links",
                        a { href: location.read().to_hash(), onclick: move |event| follow_gallery_link(event, location, location.peek().clone()), "Link to this view" }
                        a { href: "https://github.com/jme-1308/Dioxus_iX/blob/main/IMPLEMENTATION.md#work-queue", target: "_blank", rel: "noopener noreferrer", "Implementation status" }
                    }
                    p { class: "review-intro", "Interactive Rust ports of iX examples, with recognizable styling and useful core interactions. This independent port uses public theme fallbacks." }
                    match page {
                        GalleryPage::Actions => rsx! { operator_profile::OperatorProfileExample {} split_work_orders::SplitWorkOrdersExample {} chip_workflows::ChipWorkflowsExample {} progress_review::ProgressReview {} spinner_review::SpinnerReview {} icon_toggle_button_review::IconToggleButtonReview {} toggle_button_review::ToggleButtonReview {} icon_button_review::IconButtonReview {} button_review::ButtonReview {} actions::ActionExamples {} },
                        GalleryPage::Sources => rsx! { source_examples::SourceExamples { location } },
                        GalleryPage::Forms => rsx! { service_request::ServiceRequestExample {} upload_review::UploadReview {} select_visual_review::SelectVisualReview {} slider_review::SliderReview {}
        expanding_search_review::ExpandingSearchReview {} number_input_token_review::NumberInputTokenReview {} input_review::InputReview {} group_review::GroupReview {} toggle_review::ToggleReview {} radio_review::RadioReview {} checkbox_review::CheckboxReview {} text_commit_review::TextCommitReview {} native_field_review::NativeFieldReview {} number_input_review::NumberInputReview {} forms::FormExamples {} },
                        GalleryPage::Layout => rsx! { inspection_groups::InspectionGroupsExample {} asset_workspace::AssetWorkspaceExample {} maintenance_launch::MaintenanceLaunchExample {} equipment_events::EquipmentEventsExample {} service_package::ServicePackageExample {} asset_hierarchy::AssetHierarchyExample {} paged_equipment::PagedEquipmentExample {} card_collections::CardCollectionsExample {} workflow_review::WorkflowReview {}  layout::LayoutExamples {} },
                        GalleryPage::DateTime => rsx! { localized_dates::LocalizedDatesExample {} date_dropdown_review::DateDropdownReview {} datetime::DateTimeExamples {} },
                        GalleryPage::DataGrid => rsx! { grid_custom_editors::GridCustomEditorsExample {} grid_numeric_values::GridNumericValuesExample {} grid_cell_renderers::GridCellRenderersExample {} grid_typed_editors::GridTypedEditorsExample {} grid_scenarios::GridExamples {} },
                        GalleryPage::Charts => rsx! { chart_workflows::ChartWorkflows {} },
                        GalleryPage::Tree => rsx! { tree::TreeExamples {} },
                        GalleryPage::Application => rsx! { header_nested_review::HeaderNestedReview {} header_layout_review::HeaderLayoutReview {} header_switch_review::HeaderSwitchReview {} menu_hover_review::MenuHoverReview {} menu_category_review::MenuCategoryReview {} menu_navigation_review::MenuNavigationReview {} menu_overlay_review::MenuOverlayReview {} application::ApplicationExamples {} },
                        GalleryPage::Feedback => rsx! { dropdown_review::DropdownOrderReview {} FeedbackExamples {} },
                        GalleryPage::Composition => rsx! { tabs_review::TabsReviewExamples {} composition::CompositionExamples {} },
                        GalleryPage::Chat => rsx! { chat::ChatExamples {} },
                        GalleryPage::Patterns => rsx! { asset_flip_tile::AssetFlipTileExample {} filter_keyboard_review::FilterKeyboardReview {} filter_order_review::FilterOrderReview {} filter_state_review::FilterStateReview {} filter_mode_review::FilterModeReview {} filter_reset_review::FilterResetReview {}  patterns::PatternExamples {} },
                        GalleryPage::Dialogs => rsx! { dialogs::DialogExamples {} },
                        GalleryPage::Icons => rsx! { IconExamples {} },
                        GalleryPage::Theme => rsx! { theme_review::ThemeReview {} },
                        GalleryPage::Coverage => rsx! { CoverageExamples { location } },
                    }
                }
            }
        }
    }
}

#[component]
fn CoverageExamples(location: Signal<navigation::GalleryLocation>) -> Element {
    let mut query = use_signal(String::new);
    let declared_tags = PORT_CATALOG
        .iter()
        .filter(|entry| entry.upstream_tag().is_some())
        .count();
    let mapped_tags = PORT_CATALOG
        .iter()
        .filter(|entry| entry.upstream_tag().is_some() && entry.rust_api.is_some())
        .count();
    let needle = query();
    let entries: Vec<_> = PORT_CATALOG
        .iter()
        .filter(|entry| catalog_matches(entry, &needle))
        .collect();
    rsx! {
        section { class: "demo-section",
            h2 { "Component directory" }
            p { "Find Rust APIs and their iX names. {mapped_tags} of {declared_tags} current core tags have a Rust component, model, composition or internal integration. Check Implementation status for delivered capabilities and limitations." }
            label { class: "review-icon-search", "Filter components"
                input { r#type: "search", value: query(), placeholder: "ix-progress-indicator or ProgressIndicator", oninput: move |event| query.set(event.value()) }
            }
            p { role: "status", "{entries.len()} matching components" }
            if entries.is_empty() {
                p { "No components match this search. Try an iX tag or Rust API name." }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| query.set(String::new()), "Clear component search" }
            }
            div { class: "review-table-scroll",
            table { class: "review-coverage",
                thead { tr { th { "Upstream tag" } th { "Rust mapping" } th { "Mapping" } th { "Explore" } } }
                tbody {
                    for entry in entries {
                        tr { td { code { {entry.upstream_tag().unwrap_or_else(|| entry.upstream.to_string())} } }
                            td { {entry.rust_api.unwrap_or("—")} }
                            td { "{entry.mapping_kind().label()}" }
                            td { if entry.rust_api.is_some() { a {
                                href: component_examples_link(entry, &location.read()),
                                onclick: move |event| {
                                    let next = navigation::GalleryLocation::from_hash(&component_examples_link(entry, &location.peek()));
                                    follow_gallery_link(event, location, next);
                                },
                                "Examples"
                            } } }
                        }
                    }
                }
            }
            }
        }
    }
}

fn catalog_matches(entry: &PortEntry, query: &str) -> bool {
    fn normalized(value: &str) -> String {
        value
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .flat_map(char::to_lowercase)
            .collect()
    }
    let needle = normalized(query);
    normalized(&entry.upstream_tag().unwrap_or_default()).contains(&needle)
        || entry
            .rust_api
            .is_some_and(|api| normalized(api).contains(&needle))
}
fn component_examples_link(entry: &PortEntry, current: &navigation::GalleryLocation) -> String {
    let mut next = current.with_page("sources");
    next.query = entry.upstream.into();
    next.to_hash()
}

fn follow_gallery_link(
    event: MouseEvent,
    mut location: Signal<navigation::GalleryLocation>,
    next: navigation::GalleryLocation,
) {
    // Keep real hrefs for copy/new-tab use; ordinary activation follows the
    // Dioxus-owned state path, which then writes through the History provider.
    if event.modifiers().is_empty()
        && event.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary)
    {
        event.prevent_default();
        location.set(next);
    }
}

#[component]
fn FeedbackExamples() -> Element {
    let mut selected = use_signal(|| "Nothing selected".to_string());
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
        toast_review::ToastLifetimeExample {}
    }
}

#[component]
fn TooltipReviewExamples() -> Element {
    let mut actions = use_signal(|| 0usize);
    let clipped = use_tooltip("review-tooltip-clipped");
    let clipped_anchor = use_tooltip_anchor(clipped.clone(), "review-tooltip-clipped-anchor");
    let shared = use_tooltip("review-tooltip-shared");
    let shared_one = use_tooltip_anchor(shared.clone(), "review-tooltip-shared-one");
    let shared_two = use_tooltip_anchor(shared.clone(), "review-tooltip-shared-two");
    let interactive = use_tooltip("review-tooltip-interactive");
    let interactive_anchor =
        use_tooltip_anchor(interactive.clone(), "review-tooltip-interactive-anchor");
    rsx! {
        h3 { "Clipping, shared triggers and interactive content" }
        p { "Focus or hover a trigger to show its tooltip. The first tooltip escapes a clipped host. The next two triggers share one tooltip. Escape dismisses the open tooltip." }
        div { class: "review-tooltip-clip",
            span { id: "review-tooltip-clipped-anchor", class: "review-tooltip-trigger",
                onmounted: clipped_anchor.onmounted, onresize: clipped_anchor.onresize, onvisible: clipped_anchor.onvisible,
                onmouseenter: clipped_anchor.onmouseenter, onmouseleave: clipped_anchor.onmouseleave,
                onfocusin: clipped_anchor.onfocusin, onfocusout: clipped_anchor.onfocusout, onkeydown: clipped_anchor.onkeydown,
                "aria-describedby": clipped_anchor.describedby(""),
                Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-clipped", "Clipped tooltip trigger" }
            }
            TooltipFor { id: "review-tooltip-clipped", controller: clipped, placement: TooltipPlacement::Bottom,
                "This tooltip remains visible outside the clipped container."
            }
        }
        div { class: "demo-row",
            span { id: "review-tooltip-shared-one", "data-review-tooltip": "shared", class: "review-tooltip-trigger",
                onmounted: shared_one.onmounted, onresize: shared_one.onresize, onvisible: shared_one.onvisible,
                onmouseenter: shared_one.onmouseenter, onmouseleave: shared_one.onmouseleave,
                onfocusin: shared_one.onfocusin, onfocusout: shared_one.onfocusout, onkeydown: shared_one.onkeydown,
                "aria-describedby": shared_one.describedby(""),
                Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-shared", "Shared trigger one" }
            }
            span { id: "review-tooltip-shared-two", "data-review-tooltip": "shared", class: "review-tooltip-trigger",
                onmounted: shared_two.onmounted, onresize: shared_two.onresize, onvisible: shared_two.onvisible,
                onmouseenter: shared_two.onmouseenter, onmouseleave: shared_two.onmouseleave,
                onfocusin: shared_two.onfocusin, onfocusout: shared_two.onfocusout, onkeydown: shared_two.onkeydown,
                "aria-describedby": shared_two.describedby(""),
                Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-shared", "Shared trigger two" }
            }
        }
        TooltipFor { id: "review-tooltip-shared", controller: shared,
            title: "Shared contextual help", title_icon: "info", "One tooltip follows the active trigger."
        }
        span { id: "review-tooltip-interactive-anchor", class: "review-tooltip-trigger",
                onmounted: interactive_anchor.onmounted, onresize: interactive_anchor.onresize, onvisible: interactive_anchor.onvisible,
                onmouseenter: interactive_anchor.onmouseenter, onmouseleave: interactive_anchor.onmouseleave,
                onfocusin: interactive_anchor.onfocusin, onfocusout: interactive_anchor.onfocusout, onkeydown: interactive_anchor.onkeydown,
                "aria-describedby": interactive_anchor.describedby(""),
            Button { variant: ButtonVariant::Secondary, aria_describedby: "review-tooltip-interactive", "Interactive tooltip trigger" }
        }
        TooltipFor { id: "review-tooltip-interactive", controller: interactive, interactive: true,
            placement: TooltipPlacement::Right, title: "Review action",
            p { "The tooltip stays open while you interact with its content." }
            Button { variant: ButtonVariant::Secondary, onclick: move |_| actions += 1, "Apply tooltip action" }
        }
        p { role: "status", "Tooltip actions applied: {actions}" }
    }
}

#[component]
fn IconExamples() -> Element {
    rsx! { icon_catalog::IconCatalogExample {} }
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
.review-view-links{display:flex;flex-wrap:wrap;gap:16px;margin:12px 0 16px;font-size:14px}.review-view-links a,.review-content a{text-underline-offset:3px}.review-navigation button:focus-visible,.review-content input:focus-visible,.review-content select:focus-visible,.review-content a:focus-visible,.review-header input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.review-table-scroll{max-width:100%;overflow:auto}.review-coverage{min-width:560px}.review-icon-search input{font:inherit}
.review-intro { max-width:85ch;margin:0 0 32px;color:var(--ix-color-text-secondary);font-size:14px; }
.demo-section { padding:0 0 32px;margin:0 0 32px;border-bottom:1px solid var(--ix-color-border);min-width:0; }
.demo-section h2 { margin:0 0 16px;font-size:22px;font-weight:600; }
.demo-section h3 { margin:24px 0 12px;font-size:18px; }
.demo-section>p { color:var(--ix-color-text-secondary);font-size:14px;max-width:80ch; }
.demo-row { display:flex;align-items:center;flex-wrap:wrap;gap:12px;margin:12px 0; }
.demo-stack { display:flex;flex-direction:column;gap:16px;max-width:760px; }
.demo-label { min-width:130px;font-size:14px;color:var(--ix-color-text-secondary); }
.demo-grid { display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,260px),1fr));gap:24px; }
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

#[cfg(test)]
mod discovery_tests {
    use super::*;
    #[test]
    fn component_directory_accepts_ix_tags_and_rust_api_names() {
        let entry = PORT_CATALOG
            .iter()
            .find(|entry| entry.upstream == "progress-indicator")
            .unwrap();
        assert!(catalog_matches(entry, "ProgressIndicator"));
        assert!(catalog_matches(entry, "ix-progress-indicator"));
        assert!(!catalog_matches(entry, "unknown-component"));
        let current = navigation::GalleryLocation::default();
        let linked =
            navigation::GalleryLocation::from_hash(&component_examples_link(entry, &current));
        assert_eq!(linked.page, "sources");
        assert_eq!(linked.query, "progress-indicator");
        for page in GalleryPage::ALL {
            assert_eq!(
                navigation::GalleryLocation::from_hash(&format!("#page={}", page.slug())).page,
                page.slug()
            );
        }
    }
}
