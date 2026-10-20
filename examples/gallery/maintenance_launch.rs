//! Local maintenance task creation, review and retained preparation fields.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaintenanceKind {
    Inspection,
    Service,
}
impl MaintenanceKind {
    fn label(self) -> &'static str {
        match self {
            Self::Inspection => "Inspection",
            Self::Service => "Service",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MaintenanceTask {
    id: usize,
    kind: MaintenanceKind,
    reviewed: bool,
}

fn initial_tasks() -> Vec<MaintenanceTask> {
    vec![
        MaintenanceTask {
            id: 101,
            kind: MaintenanceKind::Inspection,
            reviewed: false,
        },
        MaintenanceTask {
            id: 102,
            kind: MaintenanceKind::Service,
            reviewed: false,
        },
        MaintenanceTask {
            id: 103,
            kind: MaintenanceKind::Inspection,
            reviewed: false,
        },
    ]
}

fn appearance(value: &str) -> CardVariant {
    match value {
        "filled" => CardVariant::Filled,
        "warning" => CardVariant::Warning,
        "primary" => CardVariant::Primary,
        _ => CardVariant::Outline,
    }
}
fn appearance_value(variant: CardVariant) -> &'static str {
    match variant {
        CardVariant::Filled => "filled",
        CardVariant::Warning => "warning",
        CardVariant::Primary => "primary",
        _ => "outline",
    }
}

#[component]
fn TaskPreparation() -> Element {
    let mut note = use_signal(|| "Prepare tools for Motor 001".to_string());
    let mut toolkit = use_signal(|| false);
    rsx! {
        div { class: "launch-preparation",
            label { class: "launch-note-field", r#for: "launch-note",
                span { "Preparation note" }
                input { id: "launch-note", class: "launch-note", r#type: "text", value: note(), oninput: move |event: FormEvent| note.set(event.value()) }
            }
            label { class: "launch-toolkit-field", r#for: "launch-toolkit",
                input { id: "launch-toolkit", r#type: "checkbox", checked: toolkit(), onchange: move |event: FormEvent| toolkit.set(event.checked()) }
                "Toolkit ready"
            }
        }
    }
}

/// Task cards connected to an actual illustrative queue and retained local notes.
#[component]
pub fn MaintenanceLaunchExample() -> Element {
    let mut tasks = use_signal(initial_tasks);
    let mut next_id = use_signal(|| 104_usize);
    let mut last_kind = use_signal(|| None::<MaintenanceKind>);
    let mut created = use_signal(|| 0_usize);
    let mut card_selected = use_signal(|| false);
    let mut card_selections = use_signal(|| 0_usize);
    let mut details_open = use_signal(|| false);
    let mut detail_requests = use_signal(|| 0_usize);
    let mut reviews = use_signal(|| 0_usize);
    let mut disable_actions = use_signal(|| false);
    let mut passive = use_signal(|| false);
    let mut custom_content = use_signal(|| true);
    let mut hold_details = use_signal(|| false);
    let mut variant = use_signal(|| CardVariant::Outline);
    let mut epoch = use_signal(|| 0_u64);
    let current_tasks = tasks();
    let pending = current_tasks.iter().filter(|task| !task.reviewed).count();
    let create_task = use_callback(move |kind: MaintenanceKind| {
        if !disable_actions() {
            let id = *next_id.peek();
            let mut next = tasks.peek().clone();
            next.push(MaintenanceTask {
                id,
                kind,
                reviewed: false,
            });
            tasks.set(next);
            next_id.set(id + 1);
            last_kind.set(Some(kind));
            let count = created.peek().saturating_add(1);
            created.set(count);
        }
    });

    rsx! {
        section { id: "maintenance-launch", class: "maintenance-launch-example",
            h2 { "Maintenance task launcher" }
            p { class: "launch-intro", "Create illustrative inspection or service tasks for Motor 001, review the local queue and prepare your toolkit. Task creation, queue selection and preparation details are independent. Restore task launcher clears the example." }
            div { class: "launch-toolbar",
                Button { id: "launch-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        tasks.set(initial_tasks()); next_id.set(104); last_kind.set(None); created.set(0);
                        card_selected.set(false); card_selections.set(0); details_open.set(false); detail_requests.set(0); reviews.set(0);
                        disable_actions.set(false); passive.set(false); custom_content.set(true); hold_details.set(false); variant.set(CardVariant::Outline);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    },
                    "Restore task launcher"
                }
                Button { id: "launch-toggle-details", variant: ButtonVariant::Secondary,
                    onclick: move |_| details_open.toggle(), "Toggle task details externally"
                }
            }
            details { id: "launch-options", class: "launch-options",
                summary { "Card options" }
                div { class: "launch-option-fields",
                    label { input { id: "launch-disable-actions", r#type: "checkbox", checked: disable_actions(), onchange: move |event: FormEvent| disable_actions.set(event.checked()) } "Disable task actions" }
                    label { input { id: "launch-passive", r#type: "checkbox", checked: passive(), onchange: move |event: FormEvent| passive.set(event.checked()) } "Passive notification card" }
                    label { input { id: "launch-custom-content", r#type: "checkbox", checked: custom_content(), onchange: move |event: FormEvent| custom_content.set(event.checked()) } "Show action descriptions" }
                    label { input { id: "launch-hold", r#type: "checkbox", checked: hold_details(), onchange: move |event: FormEvent| hold_details.set(event.checked()) } "Hold details changes" }
                    label { r#for: "launch-variant", "Card appearance"
                        select { id: "launch-variant", value: appearance_value(variant()), onchange: move |event: FormEvent| variant.set(appearance(&event.value())),
                            for (value, label) in [("outline", "Outline"), ("filled", "Filled"), ("warning", "Warning"), ("primary", "Primary")] {
                                option { value, selected: appearance_value(variant()) == value, "{label}" }
                            }
                        }
                    }
                }
            }
            div { class: "launch-actions",
                for generation in [epoch()] {
                    ActionCard { key: "inspection-action-{generation}", id: "launch-inspection",
                        heading: "Start inspection", aria_label: "Start inspection for Motor 001",
                        icon: "check", aria_label_icon: "Inspection checklist", variant: variant(),
                        selected: last_kind() == Some(MaintenanceKind::Inspection), passive: disable_actions(),
                        onclick: move |_| create_task.call(MaintenanceKind::Inspection),
                        if custom_content() { span { id: "launch-inspection-description", class: "launch-action-description", "Motor 001 local checklist" } }
                    }
                }
                for generation in [epoch()] {
                    ActionCard { key: "service-action-{generation}", id: "launch-service",
                        heading: "Plan service", aria_label: "Plan service for Motor 001",
                        icon: "cogwheel", aria_label_icon: "Service plan", variant: variant(),
                        selected: last_kind() == Some(MaintenanceKind::Service), passive: disable_actions(),
                        onclick: move |_| create_task.call(MaintenanceKind::Service),
                        if custom_content() { span { id: "launch-service-description", class: "launch-action-description", "Maintenance work order" } }
                    }
                }
                for generation in [epoch()] {
                    ActionCard { key: "locked-action-{generation}", id: "launch-locked", heading: "Remote release",
                        aria_label: "Remote release unavailable", icon: "lock", aria_label_icon: "Release locked",
                        subheading: "Unavailable in this local example", passive: true, variant: variant(),
                    }
                }
            }
            for generation in [epoch()] {
                PushCard { key: "maintenance-backlog-{generation}", id: "launch-backlog", class: "launch-backlog-card",
                    heading: "Pending maintenance", subheading: "Motor 001 local task queue", notification: pending.to_string(),
                    icon: "bulb", aria_label_icon: "Pending maintenance", aria_label: "Select pending maintenance queue",
                    variant: variant(), passive: passive(), selected: card_selected(), expanded: details_open(),
                    details_label: "Task details", aria_label_expand_button: "Show task preparation details",
                    onselect: move |_| { card_selected.toggle(); let count = card_selections.peek().saturating_add(1); card_selections.set(count); },
                    onexpand: move |next| {
                        let count = detail_requests.peek().saturating_add(1); detail_requests.set(count);
                        if !hold_details() { details_open.set(next); }
                    },
                    title_actions: rsx! {
                        button { id: "launch-review", class: "launch-review", r#type: "button",
                            "aria-label": if pending > 0 { "Review all pending tasks" } else { "Reopen all maintenance tasks" },
                            onclick: move |_| {
                                let review = tasks.peek().iter().any(|task| !task.reviewed);
                                let mut next = tasks.peek().clone(); for task in &mut next { task.reviewed = review; } tasks.set(next);
                                let count = reviews.peek().saturating_add(1); reviews.set(count);
                            },
                            if pending > 0 { "Review all" } else { "Reopen all" }
                        }
                    },
                    TaskPreparation {}
                }
            }
            div { class: "launch-records-wrap",
                table { id: "launch-task-records", class: "launch-task-records",
                    caption { "Local maintenance tasks" }
                    thead { tr { th { scope: "col", "Task" } th { scope: "col", "Kind" } th { scope: "col", "Status" } } }
                    tbody {
                        for task in current_tasks {
                            tr { key: "{task.id}", id: format!("launch-task-{}", task.id), "data-reviewed": task.reviewed.to_string(),
                                td { "data-field": "id", "MT-{task.id}" }
                                td { "data-field": "kind", "{task.kind.label()}" }
                                td { "data-field": "status", if task.reviewed { "Reviewed" } else { "Awaiting review" } }
                            }
                        }
                    }
                }
            }
            output { id: "launch-result", class: "launch-result", role: "status",
                "data-created": created().to_string(), "data-card-selections": card_selections().to_string(),
                "data-detail-requests": detail_requests().to_string(), "data-reviews": reviews().to_string(), "data-pending": pending.to_string(),
                "{pending} awaiting review · Created: {created} · Queue selections: {card_selections} · Detail requests: {detail_requests} · Review actions: {reviews}"
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/maintenance_launch.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maintenance_launcher_renders_real_seed_records_cards_and_retained_details() {
        let mut dom = VirtualDom::new(MaintenanceLaunchExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "maintenance-launch",
            "launch-inspection",
            "launch-service",
            "launch-locked",
            "launch-backlog",
            "launch-review",
            "launch-note",
            "launch-toolkit",
            "launch-task-records",
            "launch-reset",
            "launch-options",
            "launch-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        for id in 101..=103 {
            assert!(html.contains(&format!("id=\"launch-task-{id}\"")));
        }
        assert!(
            html.contains("Inspection")
                && html.contains("Service")
                && html.contains("Awaiting review")
        );
        assert!(html.contains("Local maintenance tasks"));
        assert!(html.contains("Start inspection for Motor 001"));
        assert!(html.contains("Plan service for Motor 001"));
        assert!(
            html.split("<svg")
                .skip(1)
                .filter_map(|tail| tail.split('>').next())
                .any(|attributes| {
                    attributes.contains("role=\"img\"")
                        && attributes.contains("aria-label=\"Service plan\"")
                }),
            "Plan service must render its labelled bundled icon"
        );
        assert!(html.contains("Select pending maintenance queue"));
        assert!(html.contains("Review all pending tasks"));
        assert!(html.contains("Show task preparation details"));
        assert!(
            html.contains("Motor 001 local checklist") && html.contains("Maintenance work order")
        );
        assert!(html.contains("Prepare tools for Motor 001"));
        assert!(html.contains("data-created=\"0\"") && html.contains("data-pending=\"3\""));
        assert!(html.contains("aria-expanded=\"false\"") || html.contains("aria-expanded=false"));
        assert!(html.matches("hidden=true").count() + html.matches("hidden=\"true\"").count() >= 1);
        assert!(!html.contains("id=\"launch-task-104\""));
    }
}
