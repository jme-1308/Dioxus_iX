//! A composed listbox filters an illustrative, application-owned inspection queue.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Copy)]
struct Inspection {
    id: &'static str,
    title: &'static str,
    status: &'static str,
}

const INSPECTIONS: [Inspection; 3] = [
    Inspection {
        id: "pump",
        title: "Pump 001 seal inspection",
        status: "open",
    },
    Inspection {
        id: "motor",
        title: "Motor 002 bearing inspection",
        status: "open",
    },
    Inspection {
        id: "valve",
        title: "Valve 003 travel inspection",
        status: "completed",
    },
];

fn filtered_inspections(filter: &str) -> Vec<Inspection> {
    INSPECTIONS
        .iter()
        .copied()
        .filter(|inspection| filter == "all" || inspection.status == filter)
        .collect()
}

#[component]
pub fn SelectItemReviewExample() -> Element {
    let mut selected = use_signal(|| "open".to_string());
    let mut changes = use_signal(|| 0_usize);
    let current = selected();
    let label = match current.as_str() {
        "all" => "All inspections",
        "completed" => "Completed inspections",
        _ => "Open inspections",
    };
    let visible = filtered_inspections(&current);
    let count = visible.len();
    let select_filter = EventHandler::new(move |next: String| {
        if matches!(next.as_str(), "all" | "open" | "completed") {
            selected.set(next);
            let next_count = changes.peek().saturating_add(1);
            changes.set(next_count);
        }
    });

    rsx! {
        section { id: "select-item-review", class: "demo-section",
            h2 { "Inspection queue filter" }
            p { "Filter an illustrative local queue by status. Unavailable inspections cannot be selected. This example does not connect to equipment." }
            div { class: "demo-row",
                Dropdown {
                    class: "select-item-filter",
                    label: "Filter inspections by status",
                    aria_label: "Filter inspections by status",
                    button_label: rsx! { span { "{label}" } },
                    surface_role: DropdownSurfaceRole::Listbox,
                    navigation_mode: DropdownNavigationMode::RovingTabindex,
                    focus_checked_item: true,
                    enable_top_layer: true,
                    for (value, option_label, disabled) in [
                        ("all", "All inspections", false),
                        ("open", "Open inspections", false),
                        ("completed", "Completed inspections", false),
                        ("unavailable", "Unavailable inspections", true),
                    ] {
                        SelectItem {
                            key: "{value}",
                            id: "select-item-option-{value}",
                            value,
                            label: option_label,
                            selected: selected.read().as_str() == value,
                            disabled,
                            onitemclick: select_filter,
                        }
                    }
                }
                Button {
                    id: "select-item-reset",
                    button_type: ButtonType::Button,
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| { selected.set("open".into()); changes.set(0); },
                    "Reset filter"
                }
            }
            output {
                id: "select-item-result",
                role: "status",
                "data-filter": current,
                "data-count": count.to_string(),
                "data-changes": changes().to_string(),
                "{count} of 3 inspections shown. {label}."
            }
            ul { id: "select-item-queue", "aria-label": "Filtered inspection queue",
                for inspection in visible {
                    li {
                        key: "{inspection.id}",
                        id: "select-item-row-{inspection.id}",
                        "data-status": inspection.status,
                        "{inspection.title} — "
                        span { if inspection.status == "open" { "Open" } else { "Completed" } }
                    }
                }
            }
        }
    }
}
