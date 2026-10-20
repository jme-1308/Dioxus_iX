//! Local equipment readings with explicit recovery and independent preparation.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq)]
struct EquipmentReading {
    id: &'static str,
    label: &'static str,
    value: f64,
    unit: &'static str,
    state: KpiState,
}

fn seed_readings() -> Vec<EquipmentReading> {
    vec![
        EquipmentReading {
            id: "motor-speed",
            label: "Motor 001 speed",
            value: 122.6,
            unit: "rpm",
            state: KpiState::Neutral,
        },
        EquipmentReading {
            id: "pump-temperature",
            label: "Pump 002 temperature",
            value: 78.2,
            unit: "°C",
            state: KpiState::Warning,
        },
        EquipmentReading {
            id: "fan-vibration",
            label: "Fan 003 vibration",
            value: 1.4,
            unit: "mm/s",
            state: KpiState::Neutral,
        },
    ]
}

fn severity(state: KpiState) -> &'static str {
    match state {
        KpiState::Neutral => "neutral",
        KpiState::Warning => "warning",
        KpiState::Alarm => "alarm",
    }
}

fn next_severity(state: KpiState) -> KpiState {
    match state {
        KpiState::Neutral => KpiState::Warning,
        KpiState::Warning => KpiState::Alarm,
        KpiState::Alarm => KpiState::Neutral,
    }
}

fn layout_value(layout: EmptyStateLayout) -> &'static str {
    match layout {
        EmptyStateLayout::Large => "large",
        EmptyStateLayout::Compact => "compact",
        EmptyStateLayout::CompactBreak => "compact-break",
    }
}

#[component]
fn ReadingPreparation() -> Element {
    let mut note = use_signal(|| "Prepare line inspection".to_string());
    rsx! {
        label { class: "readings-preparation", r#for: "readings-note",
            span { "Inspection preparation" }
            input { id: "readings-note", r#type: "text", maxlength: "160", value: note(),
                oninput: move |event: FormEvent| note.set(event.value()),
            }
        }
    }
}

/// Load and sample illustrative records using explicit local workflow controls.
#[component]
pub fn EquipmentReadingsExample() -> Element {
    let mut readings = use_signal(Vec::<EquipmentReading>::new);
    let mut pending = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut filter = use_signal(|| "all".to_string());
    let mut vertical = use_signal(|| false);
    let mut layout = use_signal(|| EmptyStateLayout::Large);
    let mut long_label = use_signal(|| false);
    let mut german = use_signal(|| false);
    let mut load_requests = use_signal(|| 0_usize);
    let mut completions = use_signal(|| 0_usize);
    let mut cancellations = use_signal(|| 0_usize);
    let mut samples = use_signal(|| 0_usize);
    let mut filter_clears = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);

    let all_readings = readings();
    let current_filter = filter();
    let visible_readings: Vec<_> = all_readings
        .iter()
        .filter(|reading| current_filter == "all" || current_filter == severity(reading.state))
        .cloned()
        .collect();
    let count = all_readings.len();
    let visible_count = visible_readings.len();
    let filter_empty = count > 0 && visible_count == 0;
    let recovery = use_callback(move |_| {
        if *disabled.peek() || *pending.peek() {
            return;
        }
        if readings.peek().is_empty() {
            pending.set(true);
            let count = load_requests.peek().saturating_add(1);
            load_requests.set(count);
        } else {
            filter.set("all".into());
            let count = filter_clears.peek().saturating_add(1);
            filter_clears.set(count);
        }
    });

    rsx! {
        section { id: "equipment-readings", class: "equipment-readings-example",
            h2 { "Equipment readings" }
            p { class: "readings-intro", "Load illustrative equipment readings, then choose Complete load or Cancel load. Sample readings advances the local values and example severity states. No equipment connection is used." }
            div { class: "readings-toolbar",
                Button { id: "readings-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        readings.set(Vec::new()); pending.set(false); disabled.set(false); filter.set("all".into()); vertical.set(false);
                        layout.set(EmptyStateLayout::Large); long_label.set(false); german.set(false); load_requests.set(0);
                        completions.set(0); cancellations.set(0); samples.set(0); filter_clears.set(0);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore equipment readings"
                }
                Button { id: "readings-complete", disabled: !pending(),
                    onclick: move |_| {
                        if *pending.peek() {
                            readings.set(seed_readings()); pending.set(false);
                            let count = completions.peek().saturating_add(1); completions.set(count);
                        }
                    }, "Complete load"
                }
                Button { id: "readings-cancel", variant: ButtonVariant::Secondary, disabled: !pending(),
                    onclick: move |_| {
                        if *pending.peek() { pending.set(false); let count = cancellations.peek().saturating_add(1); cancellations.set(count); }
                    }, "Cancel load"
                }
                Button { id: "readings-sample", variant: ButtonVariant::Secondary, disabled: pending() || count == 0,
                    onclick: move |_| {
                        if *pending.peek() || readings.peek().is_empty() { return; }
                        let mut next = readings.peek().clone();
                        for reading in &mut next {
                            reading.value += if reading.id == "fan-vibration" { 0.2 } else { 1.0 };
                            reading.state = next_severity(reading.state);
                        }
                        readings.set(next); let count = samples.peek().saturating_add(1); samples.set(count);
                    }, "Sample readings"
                }
                Button { id: "readings-clear", variant: ButtonVariant::Secondary, disabled: count == 0 && !pending(),
                    onclick: move |_| { readings.set(Vec::new()); pending.set(false); filter.set("all".into()); }, "Clear readings"
                }
                label { class: "readings-select-field", r#for: "readings-filter", span { "Severity" }
                    select { id: "readings-filter", value: filter(), onchange: move |event: FormEvent| filter.set(event.value()),
                        option { value: "all", selected: filter() == "all", "All readings" }
                        option { value: "neutral", selected: filter() == "neutral", "Neutral" }
                        option { value: "warning", selected: filter() == "warning", "Warning" }
                        option { value: "alarm", selected: filter() == "alarm", "Alarm" }
                    }
                }
            }
            details { id: "readings-options", class: "readings-options",
                summary { "Reading options" }
                div { class: "readings-option-fields",
                    label { input { id: "readings-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable recovery action" }
                    label { input { id: "readings-vertical", r#type: "checkbox", checked: vertical(), onchange: move |event: FormEvent| vertical.set(event.checked()) } "Vertical measurements" }
                    label { input { id: "readings-long-label", r#type: "checkbox", checked: long_label(), onchange: move |event: FormEvent| long_label.set(event.checked()) } "Long reading labels" }
                    label { input { id: "readings-german", r#type: "checkbox", checked: german(), onchange: move |event: FormEvent| german.set(event.checked()) } "German severity icon labels" }
                    label { class: "readings-select-field", r#for: "readings-layout", span { "Empty state layout" }
                        select { id: "readings-layout", value: layout_value(layout()), onchange: move |event: FormEvent| layout.set(match event.value().as_str() {
                            "compact" => EmptyStateLayout::Compact, "compact-break" => EmptyStateLayout::CompactBreak, _ => EmptyStateLayout::Large,
                        }),
                            option { value: "large", selected: layout() == EmptyStateLayout::Large, "Large" }
                            option { value: "compact", selected: layout() == EmptyStateLayout::Compact, "Compact" }
                            option { value: "compact-break", selected: layout() == EmptyStateLayout::CompactBreak, "Compact break" }
                        }
                    }
                }
            }
            for generation in [epoch()] { ReadingPreparation { key: "reading-preparation-{generation}" } }
            if visible_count == 0 {
                EmptyState { id: "readings-empty", icon: "bulb", aria_label_icon: "Equipment readings",
                    header: if pending() { "Preparing local readings" } else if filter_empty { "No matching readings" } else { "No equipment readings" },
                    subheader: if pending() { "Use Complete load to add the illustrative records, or Cancel load to return." } else if filter_empty { "Choose another severity or clear the filter to see the loaded readings." } else { "Load the local example records to begin an equipment review." },
                    action: if pending() { "Loading readings" } else if filter_empty { "Clear filter" } else { "Load readings" },
                    aria_label_action: if filter_empty { "Clear reading filter" } else { "Load illustrative equipment readings" },
                    action_disabled: disabled(), action_loading: pending(), layout: layout(), onaction: recovery,
                }
            } else {
                div { id: "readings-values", class: "readings-values",
                    for reading in visible_readings {
                        div { key: "{reading.id}", id: format!("readings-record-{}", reading.id), class: "readings-record",
                            "data-state": severity(reading.state), "data-value": format!("{:.1}", reading.value), "data-unit": reading.unit,
                            Kpi { id: format!("readings-kpi-{}", reading.id),
                                label: if long_label() { format!("{} measured at the local equipment inspection point", reading.label) } else { reading.label.to_string() },
                                value: format!("{:.1}", reading.value), unit: reading.unit, state: reading.state, vertical: vertical(),
                                aria_label_alarm_icon: if german() { "Alarmmeldung" } else { "Alarm" },
                                aria_label_warning_icon: if german() { "Warnmeldung" } else { "Warning" },
                            }
                        }
                    }
                }
            }
            output { id: "readings-result", class: "readings-result", role: "status",
                "data-record-count": count.to_string(), "data-visible-count": visible_count.to_string(), "data-pending": pending().to_string(),
                "data-load-requests": load_requests().to_string(), "data-completions": completions().to_string(), "data-cancellations": cancellations().to_string(),
                "data-samples": samples().to_string(), "data-filter-clears": filter_clears().to_string(), "data-filter": filter(),
                "data-vertical": vertical().to_string(), "data-empty-layout": layout_value(layout()), "data-disabled": disabled().to_string(),
                "data-german": german().to_string(), "data-long-label": long_label().to_string(),
                "{visible_count} of {count} readings · Load requests: {load_requests} · Completed: {completions} · Cancelled: {cancellations} · Samples: {samples}"
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/equipment_readings.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipment_readings_starts_empty_with_explicit_recovery_and_retained_preparation() {
        let mut dom = VirtualDom::new(EquipmentReadingsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "equipment-readings",
            "readings-empty",
            "readings-complete",
            "readings-cancel",
            "readings-sample",
            "readings-clear",
            "readings-reset",
            "readings-filter",
            "readings-options",
            "readings-note",
            "readings-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("No equipment readings")
                && html.contains("Load illustrative equipment readings")
        );
        assert!(html.contains("Prepare line inspection") && html.contains("for=\"readings-note\""));
        assert!(
            html.contains("data-record-count=\"0\"")
                && html.contains("data-visible-count=\"0\"")
                && html.contains("data-pending=\"false\"")
        );
        assert!(
            !html.contains("id=\"readings-kpi-"),
            "records are seeded only by completed recovery"
        );
        assert!(
            html.split("<svg")
                .skip(1)
                .filter_map(|tail| tail.split('>').next())
                .any(|attrs| attrs.contains("role=\"img\"")
                    && attrs.contains("aria-label=\"Equipment readings\""))
        );
    }
}
