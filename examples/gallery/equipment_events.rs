//! Illustrative equipment events with independent selection, acknowledgement and retained notes.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EquipmentEvent {
    id: u16,
    asset: &'static str,
    title: &'static str,
    severity: CardVariant,
    severity_label: &'static str,
    time: &'static str,
    acknowledged: bool,
    disabled: bool,
    chevron: bool,
    initial_note: &'static str,
}

fn initial_events() -> Vec<EquipmentEvent> {
    vec![
        EquipmentEvent {
            id: 101,
            asset: "Motor 001",
            title: "vibration high",
            severity: CardVariant::Alarm,
            severity_label: "Alarm",
            time: "09:42",
            acknowledged: false,
            disabled: false,
            chevron: true,
            initial_note: "Check bearing housing",
        },
        EquipmentEvent {
            id: 102,
            asset: "Pump 002",
            title: "service due",
            severity: CardVariant::Warning,
            severity_label: "Warning",
            time: "09:28",
            acknowledged: false,
            disabled: false,
            chevron: false,
            initial_note: "Schedule service",
        },
        EquipmentEvent {
            id: 103,
            asset: "Valve 003",
            title: "calibration complete",
            severity: CardVariant::Success,
            severity_label: "Success",
            time: "09:16",
            acknowledged: true,
            disabled: false,
            chevron: true,
            initial_note: "Calibration verified",
        },
        EquipmentEvent {
            id: 104,
            asset: "Sensor 004",
            title: "archived event",
            severity: CardVariant::Info,
            severity_label: "Info",
            time: "08:55",
            acknowledged: false,
            disabled: true,
            chevron: false,
            initial_note: "Archived record",
        },
    ]
}

fn test_event() -> EquipmentEvent {
    EquipmentEvent {
        id: 105,
        asset: "Drive 005",
        title: "temperature check",
        severity: CardVariant::Info,
        severity_label: "Info",
        time: "10:05",
        acknowledged: false,
        disabled: false,
        chevron: true,
        initial_note: "Review cooling fan",
    }
}

fn event_matches(event: EquipmentEvent, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || format!(
            "EVT-{} {} {} {}",
            event.id, event.asset, event.title, event.severity_label
        )
        .to_lowercase()
        .contains(&query)
}

fn event_state(event: EquipmentEvent) -> &'static str {
    if event.disabled {
        "Archived"
    } else if event.acknowledged {
        "Acknowledged"
    } else {
        "Awaiting acknowledgement"
    }
}

#[component]
fn EquipmentEventRow(
    event: EquipmentEvent,
    visible: bool,
    selected: bool,
    small: bool,
    filled: bool,
    item_color: Option<String>,
    onselect: EventHandler<u16>,
    onacknowledge: EventHandler<u16>,
) -> Element {
    let mut note = use_signal(|| event.initial_note.to_string());
    let note_id = format!("events-note-{}", event.id);
    let action_label = format!(
        "{} EVT-{}",
        if event.acknowledged {
            "Reopen"
        } else {
            "Acknowledge"
        },
        event.id
    );
    rsx! {
        // Filtering hides this wrapper while the row and its local note remain mounted.
        div { class: "equipment-event-visibility", hidden: !visible,
            EventListItem { id: format!("events-row-{}", event.id),
                aria_label: format!("Select EVT-{}", event.id), selected, disabled: event.disabled,
                status: event.severity, chevron: event.chevron, filled, item_color,
                onselect: move |_| onselect.call(event.id),
                div { class: "equipment-event-content", "data-small": small,
                    div { class: "equipment-event-summary",
                        div { class: "equipment-event-title", title: format!("{}: {}", event.asset, event.title),
                            strong { "{event.asset}" }
                            span { "{event.title}" }
                        }
                        div { class: "equipment-event-meta",
                            "EVT-{event.id} · {event.severity_label} · {event.time} · {event_state(event)}"
                        }
                    }
                    div { class: "equipment-event-controls",
                        label { class: "equipment-event-note-field", r#for: note_id.clone(),
                            span { "Note" }
                            input { id: note_id, class: "equipment-event-note", r#type: "text", value: note(), disabled: event.disabled,
                                "aria-label": format!("Operator note for EVT-{}", event.id),
                                oninput: move |input: FormEvent| { if !event.disabled { note.set(input.value()); } },
                            }
                        }
                        button { id: format!("events-ack-{}", event.id), class: "equipment-event-ack", r#type: "button",
                            disabled: event.disabled, "aria-label": action_label,
                            onclick: move |_| { if !event.disabled { onacknowledge.call(event.id); } },
                            if event.acknowledged { "Reopen" } else { "Acknowledge" }
                        }
                    }
                }
            }
        }
    }
}

/// A local equipment queue with retained operator notes and independent row actions.
#[component]
pub fn EquipmentEventsExample() -> Element {
    let mut events = use_signal(initial_events);
    let mut selected = use_signal(|| 101_u16);
    let mut selections = use_signal(|| 0_usize);
    let mut acknowledgements = use_signal(|| 0_usize);
    let mut query = use_signal(String::new);
    let mut compact = use_signal(|| false);
    let mut height = use_signal(|| EventItemHeight::Large);
    let mut chevrons = use_signal(|| Some(true));
    let mut animated = use_signal(|| false);
    let mut filled = use_signal(|| false);
    let mut indicator = use_signal(|| "severity".to_string());
    let mut epoch = use_signal(|| 0_u64);

    let current_events = events();
    let current_query = query();
    let visible_count = current_events
        .iter()
        .filter(|event| event_matches(**event, &current_query))
        .count();
    let total = current_events.len();
    let has_test_event = current_events.iter().any(|event| event.id == 105);
    let current_selected = current_events
        .iter()
        .find(|event| event.id == selected())
        .copied();
    let height_value = match height() {
        EventItemHeight::Small => "small",
        EventItemHeight::Large => "large",
        EventItemHeight::Pixels(_) => "custom",
    };
    let chevron_value = match chevrons() {
        Some(true) => "all",
        Some(false) => "none",
        None => "per-row",
    };
    let select_event = use_callback(move |id: u16| {
        if events
            .peek()
            .iter()
            .any(|event| event.id == id && !event.disabled)
        {
            selected.set(id);
            let next = selections.peek().saturating_add(1);
            selections.set(next);
        }
    });
    let acknowledge = use_callback(move |id: u16| {
        let mut next = events.peek().clone();
        if let Some(event) = next
            .iter_mut()
            .find(|event| event.id == id && !event.disabled)
        {
            event.acknowledged = !event.acknowledged;
            events.set(next);
            let count = acknowledgements.peek().saturating_add(1);
            acknowledgements.set(count);
        }
    });

    rsx! {
        section { id: "equipment-events", class: "equipment-events-example",
            h2 { "Equipment event queue" }
            p { class: "equipment-events-intro", "Review illustrative plant events, keep a local operator note and acknowledge work independently of row selection. Filtering and reordering keep existing notes; Restore event queue clears this example." }
            div { class: "equipment-events-toolbar",
                Button { id: "events-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        events.set(initial_events()); selected.set(101); selections.set(0); acknowledgements.set(0);
                        query.set(String::new()); compact.set(false); height.set(EventItemHeight::Large);
                        chevrons.set(Some(true)); animated.set(false); filled.set(false); indicator.set("severity".into());
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    },
                    "Restore event queue"
                }
                Button { id: "events-add", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let mut next = events.peek().clone();
                        if next.iter().any(|event| event.id == 105) {
                            next.retain(|event| event.id != 105);
                            if *selected.peek() == 105 { selected.set(101); }
                        } else { next.push(test_event()); }
                        events.set(next);
                    },
                    if has_test_event { "Remove test event" } else { "Add test event" }
                }
                Button { id: "events-reverse", variant: ButtonVariant::Secondary,
                    onclick: move |_| { let mut next = events.peek().clone(); next.reverse(); events.set(next); },
                    "Reverse event order"
                }
                label { class: "equipment-events-filter", r#for: "events-search",
                    span { "Filter events" }
                    input { id: "events-search", class: "equipment-events-search", r#type: "search", placeholder: "Asset or event", value: current_query.clone(), oninput: move |event: FormEvent| query.set(event.value()) }
                }
                if !current_query.is_empty() {
                    Button { id: "events-clear", variant: ButtonVariant::Secondary, onclick: move |_| query.set(String::new()), "Clear filter" }
                }
            }
            details { id: "events-options", class: "equipment-events-options",
                summary { "Queue appearance" }
                div { class: "equipment-events-option-fields",
                    label { input { id: "events-compact", r#type: "checkbox", checked: compact(), onchange: move |event: FormEvent| compact.set(event.checked()) } "Compact rows" }
                    label { r#for: "events-height", "Row height"
                        select { id: "events-height", value: height_value,
                            onchange: move |event: FormEvent| height.set(match event.value().as_str() { "small" => EventItemHeight::Small, "custom" => EventItemHeight::Pixels(128), _ => EventItemHeight::Large }),
                            option { value: "large", selected: height_value == "large", "Large" }
                            option { value: "small", selected: height_value == "small", "Small" }
                            option { value: "custom", selected: height_value == "custom", "128 pixels" }
                        }
                    }
                    label { r#for: "events-chevrons", "Row chevrons"
                        select { id: "events-chevrons", value: chevron_value,
                            onchange: move |event: FormEvent| chevrons.set(match event.value().as_str() { "none" => Some(false), "per-row" => None, _ => Some(true) }),
                            option { value: "all", selected: chevron_value == "all", "All rows" }
                            option { value: "none", selected: chevron_value == "none", "No rows" }
                            option { value: "per-row", selected: chevron_value == "per-row", "Per row" }
                        }
                    }
                    label { input { id: "events-animated", r#type: "checkbox", checked: animated(), onchange: move |event: FormEvent| animated.set(event.checked()) } "Animate new events" }
                    label { input { id: "events-filled", r#type: "checkbox", checked: filled(), onchange: move |event: FormEvent| filled.set(event.checked()) } "Filled rows" }
                    label { r#for: "events-color", "EVT-102 indicator"
                        select { id: "events-color", value: indicator(), onchange: move |event: FormEvent| indicator.set(event.value()),
                            option { value: "severity", selected: indicator() == "severity", "Severity" }
                            option { value: "primary", selected: indicator() == "primary", "Primary" }
                            option { value: "info", selected: indicator() == "info", "Info" }
                        }
                    }
                }
            }
            for generation in [epoch()] {
                EventList { key: "equipment-events-{generation}", id: "events-list", class: "equipment-events-list", aria_label: "Equipment events",
                    compact: compact(), item_height: height(), chevron: chevrons, animated,
                    for event in current_events.iter().copied() {
                        EquipmentEventRow { key: "{event.id}", event, visible: event_matches(event, &current_query), selected: selected() == event.id,
                            small: height() == EventItemHeight::Small, filled: filled(),
                            item_color: if event.id == 102 { match indicator().as_str() { "primary" => Some("--ix-color-primary".into()), "info" => Some("--ix-color-info".into()), _ => None } } else { None },
                            onselect: select_event, onacknowledge: acknowledge,
                        }
                    }
                }
            }
            if visible_count == 0 {
                EmptyState { header: "No matching events", subheader: "Change the asset or event filter to view the queue.", icon: "search", layout: EmptyStateLayout::Compact }
            }
            div { id: "events-detail", class: "equipment-events-detail", role: "region", "aria-label": "Selected event",
                h3 { "Selected event" }
                if let Some(event) = current_selected {
                    p { class: "equipment-events-detail-title", strong { "EVT-{event.id} · {event.asset}" } " — {event.title}" }
                    p { "{event.severity_label} · {event.time} · {event_state(event)}" }
                }
            }
            output { id: "events-result", class: "equipment-events-result", role: "status",
                "data-selections": selections().to_string(), "data-acknowledgements": acknowledgements().to_string(), "data-visible": visible_count.to_string(), "data-total": total.to_string(),
                "{visible_count} of {total} events shown · Selection changes: {selections} · Acknowledgements: {acknowledgements}"
            }
        }
    }
}

pub const STYLES: &str = r#"
.equipment-events-example{min-width:0;max-width:100%;margin-bottom:48px}.equipment-events-intro{max-width:72ch;line-height:1.55}.equipment-events-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;min-width:0;max-width:100%;margin:20px 0 16px}.equipment-events-toolbar .ix-button{max-width:100%}.equipment-events-filter{display:flex;align-items:center;flex-wrap:wrap;gap:8px;max-width:100%;min-width:0;font-size:14px}.equipment-events-search{box-sizing:border-box;flex:1;min-width:120px;max-width:100%;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.equipment-events-search::placeholder{color:var(--ix-color-text-secondary);opacity:1}.equipment-events-options{margin:0 0 24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.equipment-events-options>summary{padding:10px 12px;cursor:pointer;font-size:14px;font-weight:600}.equipment-events-option-fields{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;padding:12px;min-width:0}.equipment-events-option-fields>label{display:inline-flex;align-items:center;gap:8px;max-width:100%;font-size:14px}.equipment-events-option-fields input[type=checkbox]{margin:0;width:16px;height:16px;accent-color:var(--ix-color-primary)}.equipment-events-option-fields select{box-sizing:border-box;max-width:100%;min-height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}.equipment-events-list,.equipment-event-visibility{min-width:0;max-width:100%}.equipment-event-visibility[hidden]{display:none}.equipment-event-content{display:flex;flex-direction:column;gap:8px;width:100%;min-width:0;color:inherit}.equipment-event-summary{display:flex;flex-direction:column;justify-content:center;gap:2px;height:38px;min-width:0;overflow:hidden}.equipment-event-title{display:flex;align-items:baseline;gap:8px;min-width:0;line-height:18px;font-size:14px;color:inherit}.equipment-event-title strong{flex:none;font-weight:600;white-space:nowrap}.equipment-event-title>span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.equipment-event-meta{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;line-height:16px;font-size:12px;color:inherit}.equipment-event-controls{display:flex;align-items:center;gap:8px;min-width:0;height:32px}.equipment-event-note-field{display:flex;align-items:center;gap:6px;flex:1;min-width:0;max-width:100%;font-size:12px;line-height:18px;color:inherit}.equipment-event-note-field>span{flex:none}.equipment-event-note{box-sizing:border-box;flex:1;width:100%;min-width:0;height:32px;padding:4px 6px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.equipment-event-ack{box-sizing:border-box;flex:none;min-width:90px;height:32px;padding:4px 6px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:12px/20px var(--ix-font);cursor:pointer}.equipment-event-ack:hover:not(:disabled){background:var(--ix-color-surface-2)}.equipment-event-note:disabled,.equipment-event-ack:disabled{cursor:default}.equipment-event-content[data-small=true]{flex-direction:row;align-items:center;gap:8px}.equipment-event-content[data-small=true] .equipment-event-summary{height:auto;flex:1}.equipment-event-content[data-small=true] .equipment-event-meta,.equipment-event-content[data-small=true] .equipment-event-note-field{display:none}.equipment-event-content[data-small=true] .equipment-event-controls{flex:none}.equipment-events-search:focus-visible,.equipment-event-note:focus-visible,.equipment-event-ack:focus-visible,.equipment-events-option-fields select:focus-visible,.equipment-events-options>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.equipment-events-option-fields input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.equipment-events-detail{margin-top:24px;padding:16px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);min-width:0;max-width:100%}.equipment-events-detail h3{margin:0 0 12px;font-size:18px;line-height:24px}.equipment-events-detail p{margin:8px 0 0;line-height:1.5;overflow-wrap:anywhere}.equipment-events-result{display:block;margin-top:16px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}@media(max-width:420px){.equipment-events-toolbar{align-items:flex-start;gap:10px 12px}.equipment-events-filter{width:100%}.equipment-events-option-fields{align-items:flex-start}}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipment_events_renders_real_records_independent_controls_and_initial_selection() {
        let mut dom = VirtualDom::new(EquipmentEventsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "equipment-events",
            "events-list",
            "events-reset",
            "events-add",
            "events-reverse",
            "events-search",
            "events-options",
            "events-detail",
            "events-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        for id in 101..=104 {
            for prefix in ["events-row", "events-note", "events-ack"] {
                assert!(
                    html.contains(&format!("id=\"{prefix}-{id}\"")),
                    "{prefix}-{id}"
                );
            }
            assert!(html.contains(&format!("aria-label=\"Select EVT-{id}\"")));
        }
        for text in [
            "Motor 001",
            "vibration high",
            "Pump 002",
            "service due",
            "Valve 003",
            "calibration complete",
            "Sensor 004",
            "archived event",
            "Check bearing housing",
            "Schedule service",
            "Calibration verified",
            "Archived record",
        ] {
            assert!(html.contains(text), "{text}");
        }
        assert!(html.contains("aria-label=\"Reopen EVT-103\""));
        assert!(html.contains("aria-label=\"Acknowledge EVT-101\""));
        assert!(html.contains("aria-label=\"Selected event\""));
        assert!(html.contains("data-selections=\"0\""));
        assert!(html.contains("data-acknowledgements=\"0\""));
        assert!(html.contains("data-visible=\"4\""));
        assert!(html.contains("data-total=\"4\""));
        assert!(!html.contains("id=\"events-row-105\""));
    }
}
