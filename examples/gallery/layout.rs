//! Rust equivalents of upstream iX preview scenarios.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn LayoutExamples() -> Element {
    let mut card_selected = use_signal(|| false);
    let mut scan_count = use_signal(|| 0usize);
    let mut tab = use_signal(|| 0usize);
    let mut tabs = use_signal(|| {
        vec![
            TabItem::new("tab-1", "Tab 1"),
            TabItem::new("tab-2", "Tab 2"),
            TabItem::new("tab-3", "Tab 3"),
        ]
    });
    let mut rounded_tab = use_signal(|| 0usize);
    let mut state_tab = use_signal(|| 0usize);
    let mut state_items = use_signal(|| {
        vec![
            TabItem {
                icon: Some("info".into()),
                counter: Some(12),
                ..TabItem::new("status", "Status")
            },
            TabItem {
                disabled: true,
                ..TabItem::new("disabled", "Disabled")
            },
            TabItem {
                closable: true,
                ..TabItem::new("close", "Close example")
            },
        ]
    });
    let mut page = use_signal(|| 0usize);
    let mut advanced_page = use_signal(|| 0usize);
    let mut item_count = use_signal(|| 15usize);
    let mut group_selected = use_signal(|| 0usize);
    let mut event_selected = use_signal(|| None::<usize>);
    let mut workflow = use_signal(|| 2usize);
    let mut created = use_signal(|| false);
    let mut location = use_signal(|| "Level 3".to_string());
    let mut disclosure = use_signal(|| false);
    rsx! {
        section { class: "gallery-section", id: "cards",
            h2 { "Card" }
            p { class: "gallery-description", "The upstream card example: selectable content, plus all semantic variants." }
            div { class: "gallery-row",
                Card { selected: card_selected(), onselect: move |_| card_selected.toggle(),
                    CardContent {
                        Icon { name: "capacity" }
                        strong { "Number of components" }
                        p { "Vanilla JavaScript" br {} "Angular" br {} "Blazor" br {} "React" br {} "Vue.js" }
                        strong { style: "display:block;font-size:40px;line-height:48px;margin-top:8px", "123" }
                    }
                }
                div { style: "display:flex;flex-wrap:wrap;gap:8px;max-width:480px",
                    for (variant, name) in [(CardVariant::Outline,"Outline"),(CardVariant::Filled,"Filled"),(CardVariant::Primary,"Primary"),(CardVariant::Alarm,"Alarm"),(CardVariant::Critical,"Critical"),(CardVariant::Warning,"Warning"),(CardVariant::Success,"Success"),(CardVariant::Info,"Info"),(CardVariant::Neutral,"Neutral")] {
                        Card { variant, passive: true, class: "gallery-small-card", CardContent { "{name}" } }
                    }
                }
            }
            p { role: "status", "Card selected: {card_selected}" }
        }
        section { class: "gallery-section", id: "action-card",
            h2 { "Action card" }
            div { class: "gallery-row",
                ActionCard { icon: "refresh", heading: "Scan for new devices", subheading: "Secondary text", onclick: move |_| scan_count += 1 }
                ActionCard { icon: "refresh", heading: "Passive action", subheading: "Interaction disabled", passive: true }
            }
            p { role: "status", "Device scans requested: {scan_count}" }
        }
        section { class: "gallery-section", id: "push-card",
            h2 { "Push card" }
            PushCard { icon: "bulb", notification: "99", heading: "Heading content", subheading: "Subheading", expanded: disclosure(), onexpand: move |expanded| disclosure.set(expanded),
                table { class: "gallery-table",
                    thead { tr { th { scope: "col", "#" } th { scope: "col", "First" } th { scope: "col", "Last" } th { scope: "col", "Handle" } } }
                    tbody { tr { th { scope: "row", "1" } td { "Mark" } td { "Otto" } td { "@mdo" } } tr { th { scope: "row", "2" } td { "Jacob" } td { "Thornton" } td { "@fat" } } tr { th { scope: "row", "3" } td { colspan: "2", "Larry the Bird" } td { "@twitter" } } }
                }
            }
            p { role: "status", if disclosure() { "Details expanded" } else { "Details collapsed" } }
        }
        section { class: "gallery-section", id: "card-list",
            h2 { "Card list" }
            CardList { label: "Stack Layout",
                PushCard { icon: "rocket", heading: "Heading content", subheading: "Subheading", notification: "3", variant: CardVariant::Filled, "First notification details" }
                PushCard { icon: "bulb", heading: "Heading content", subheading: "Subheading", notification: "1", variant: CardVariant::Warning, "Warning notification details" }
                PushCard { icon: "rocket", heading: "Heading content", subheading: "Subheading", notification: "3", variant: CardVariant::Success, "Success notification details" }
            }
        }
        section { class: "gallery-section", id: "tile",
            h2 { "Tile" }
            div { class: "gallery-row",
                Tile { size: TileSize::Small, "92.8 °C" }
                Tile { size: TileSize::Medium, header: "Tile header", "92.8 °C" }
                Tile { size: TileSize::Big, header: "Tile header", subheader: "Temperature", footer: rsx! { a { href: "#key-value", "Details" } }, "92.8 °C" }
            }
        }
        section { class: "gallery-section", id: "tabs",
            h2 { "Tabs" }
            Tabs { items: tabs(), selected: tab(), onchange: move |index| tab.set(index), onclose: move |index| { tabs.write().remove(index); tab.set(tab().min(tabs().len().saturating_sub(1))); } }
            div { role: "tabpanel", "aria-label": "Example content", style: "padding:16px 0", "Content Tab {tab() + 1}" }
            h3 { "Rounded tabs" }
            Tabs { items: vec![TabItem::new("rounded-1", "Tab 1"), TabItem::new("rounded-2", "Tab 2"), TabItem::new("rounded-3", "Tab 3")], selected: rounded_tab(), rounded: true, onchange: move |index| rounded_tab.set(index) }
            h3 { "Counters, disabled and closable tabs" }
            Tabs { label: "Tab states", items: state_items(), selected: state_tab(), onchange: move |index| state_tab.set(index), onclose: move |index| { state_items.write().remove(index); state_tab.set(0); } }
            div { role: "tabpanel", "aria-label": "Tab state example", style: "padding:16px 0", if state_tab() == 0 { "Status: 12 active items" } else { "This tab can be closed with its close action." } }
        }
        section { class: "gallery-section", id: "breadcrumb",
            h2 { "Breadcrumb" }
            Breadcrumb {
                BreadcrumbItem { label: "Home", icon: "home", onclick: move |_| location.set("Home".into()) }
                BreadcrumbItem { label: "Level 1", onclick: move |_| location.set("Level 1".into()) }
                BreadcrumbItem { label: "Level 2", onclick: move |_| location.set("Level 2".into()) }
                BreadcrumbItem { label: "Level 3", current: true }
            }
            p { role: "status", "Navigation target: {location}" }
        }
        section { class: "gallery-section", id: "pagination",
            h2 { "Pagination" }
            Pagination { count: 100, selected_page: page(), onpage: move |selected| page.set(selected) }
            p { role: "status", "Selected page: {page() + 1}" }
            h3 { "Pagination advanced" }
            Pagination { count: 100, selected_page: advanced_page(), advanced: true, item_count: item_count(), onpage: move |selected| advanced_page.set(selected), onitemcount: move |count| { item_count.set(count); advanced_page.set(0); } }
            p { role: "status", "Page {advanced_page() + 1} · {item_count} items per page" }
        }
        section { class: "gallery-section", id: "blind",
            h2 { "Blind" }
            Blind { label: "Example", sublabel: "Expandable information", "This information is revealed by the disclosure control. The header works with a pointer, Enter, or Space and reports its expanded state to assistive technology." }
        }
        section { class: "gallery-section", id: "group",
            h2 { "Group" }
            Group { header: "Header text", subheader: "Subheader text",
                for index in 0..3 { GroupItem { text: format!("Example text {}", index + 1), selected: group_selected() == index, onselect: move |_| group_selected.set(index) } }
                GroupItem { text: "Disabled item", disabled: true }
            }
            p { role: "status", "Selected item: {group_selected() + 1}" }
        }
        section { class: "gallery-section", id: "key-value",
            h2 { "Key value" }
            KeyValueList {
                KeyValue { label: "Label", value: "Value" }
                KeyValue { label: "Asset", value: "Motor 001", icon: "settings" }
                KeyValue { label: "Status", value: "Operational", label_position: KeyValueLabelPosition::Left }
            }
        }
        section { class: "gallery-section", id: "kpi",
            h2 { "KPI" }
            div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:16px",
                Kpi { label: "Motor speed", value: "Nominal" }
                Kpi { label: "Motor speed", value: "122.6", unit: "rpm" }
                Kpi { label: "Motor speed", value: "122.6", state: KpiState::Alarm }
                Kpi { label: "Motor speed", value: "122.6", state: KpiState::Warning }
                Kpi { label: "Motor speed", value: "Nominal", vertical: true }
                Kpi { label: "Motor speed", value: "122.6", unit: "rpm", state: KpiState::Alarm, vertical: true }
            }
        }
        section { class: "gallery-section", id: "event-list",
            h2 { "Event list" }
            EventList {
                for (index, (name, status)) in [("Motor temperature above threshold", CardVariant::Alarm), ("Maintenance required", CardVariant::Warning), ("System ready", CardVariant::Success), ("New event received", CardVariant::Info)].into_iter().enumerate() {
                    EventListItem { status, selected: event_selected() == Some(index), chevron: true, onselect: move |_| event_selected.set(Some(index)),
                        span { style: "font-variant-numeric:tabular-nums;color:var(--ix-color-text-secondary)", "10:4{index}:00" } strong { "{name}" }
                    }
                }
                EventListItem { disabled: true, "Disabled event" }
            }
            p { role: "status", if let Some(index) = event_selected() { "Event {index + 1} selected" } else { "Select an event to review it" } }
        }
        section { class: "gallery-section", id: "workflow",
            h2 { "Workflow" }
            WorkflowSteps { clickable: true, selected_index: workflow() as isize, onstepselected: move |event: WorkflowStepSelectedEvent| workflow.set(event.index),
                for (index, status) in [WorkflowStatus::Done, WorkflowStatus::Success, WorkflowStatus::Open, WorkflowStatus::Warning, WorkflowStatus::Error].into_iter().enumerate() {
                    WorkflowStep { label: format!("Step {}", index + 1), status }
                }
                WorkflowStep { label: "Step 6", disabled: true }
            }
            p { role: "status", "Selected step: {workflow() + 1}" }
            h3 { "Workflow vertical" }
            WorkflowSteps { vertical: true, clickable: true, selected_index: workflow() as isize, onstepselected: move |event: WorkflowStepSelectedEvent| workflow.set(event.index),
                for (index, status) in [WorkflowStatus::Done, WorkflowStatus::Success, WorkflowStatus::Open, WorkflowStatus::Warning, WorkflowStatus::Error].into_iter().enumerate() {
                    WorkflowStep { label: format!("Step {}", index + 1), status }
                }
                WorkflowStep { label: "Step 6", disabled: true }
            }
        }
        section { class: "gallery-section", id: "empty-state",
            h2 { "Empty state" }
            if created() { EventList { EventListItem { status: CardVariant::Success, "Element created. The empty state now contains a real item." } } }
            else { EmptyState { header: "No elements available", subheader: "Create an element first", icon: "add", action: "Create element", onaction: move |_| created.set(true) } }
            h3 { "Compact layout" }
            EmptyState { header: "No results", subheader: "Adjust the filter to find elements", icon: "search", layout: EmptyStateLayout::Compact }
        }
    }
}

pub const STYLES: &str = r#"
.gallery-row{display:flex;flex-wrap:wrap;align-items:flex-start;gap:16px}.gallery-small-card{width:132px}.gallery-table{border-collapse:collapse;width:100%;font-size:14px;text-align:left}.gallery-table th,.gallery-table td{padding:8px;border-bottom:1px solid var(--ix-color-border)}.gallery-section>h3{font-size:16px;line-height:24px;margin:24px 0 12px}.gallery-description{color:var(--ix-color-text-secondary);font-size:14px;line-height:20px;max-width:72ch}.gallery-section>p[role=status]{font-size:14px;color:var(--ix-color-text-secondary);margin:12px 0 0}.gallery-row>.ix-tile{flex-shrink:0}
"#;
