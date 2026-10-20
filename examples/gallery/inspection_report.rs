//! Native report submission over a real, locally filtered inspection dataset.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeMap;

pub const STYLES: &str = include_str!("styles/inspection_report.css");

#[derive(Clone, Copy)]
struct Inspection {
    id: &'static str,
    equipment: &'static str,
    day: u8,
    outcome: &'static str,
}
const INSPECTIONS: [Inspection; 5] = [
    Inspection {
        id: "pump",
        equipment: "Pump 002",
        day: 8,
        outcome: "Follow-up planned",
    },
    Inspection {
        id: "motor",
        equipment: "Motor 001",
        day: 12,
        outcome: "Inspection complete",
    },
    Inspection {
        id: "sensor",
        equipment: "Sensor 004",
        day: 15,
        outcome: "Review calibration",
    },
    Inspection {
        id: "compressor",
        equipment: "Compressor 003",
        day: 18,
        outcome: "Inspection complete",
    },
    Inspection {
        id: "valve",
        equipment: "Valve 005",
        day: 20,
        outcome: "Inspection complete",
    },
];
fn date(day: u8) -> IxDate {
    IxDate::new(2026, 10, day).expect("illustrative October date")
}
fn initial_range() -> DateRangeChange {
    DateRangeChange {
        id: "recent".into(),
        dates: DateSelection {
            from: Some(date(14)),
            to: Some(date(20)),
        },
    }
}
fn in_range(inspection: &Inspection, range: &DateRangeChange) -> bool {
    let Some(from) = range.dates.from else {
        return false;
    };
    let to = range.dates.to.unwrap_or(from);
    let inspected = date(inspection.day);
    inspected >= from.min(to) && inspected <= from.max(to)
}
fn date_text(value: Option<IxDate>) -> String {
    value.map_or_else(|| "none".into(), |value| value.to_string())
}
#[derive(Clone)]
struct ReportSnapshot {
    name: String,
    note: String,
    range_id: String,
    from: String,
    to: String,
    inspection_ids: Vec<String>,
}

#[component]
fn ReportPreparation() -> Element {
    let mut preparation = use_signal(|| String::from("Check access with the shift lead"));
    rsx! { Input { id: "report-preparation", label: "Report preparation", value: preparation(), max_length: 160,
    onvaluechange: move |value| preparation.set(value), } }
}

#[component]
fn ReportDraftForm(onsave: EventHandler<BTreeMap<String, String>>) -> Element {
    let mut name = use_signal(String::new);
    let mut note = use_signal(String::new);
    let mut submissions = use_signal(|| 0_usize);
    rsx! {
        form { id: "report-form", class: "report-form",
            "data-draft-name": name(), "data-draft-note": note(), "data-submissions": submissions(),
            onsubmit: move |event| {
                event.prevent_default(); submissions += 1;
                let values = event.values().into_iter().filter_map(|(name, value)| match value {
                    dioxus::html::FormValue::Text(text) => Some((name, text)), _ => None,
                }).collect::<BTreeMap<_, _>>();
                if values.get("report-name").is_some_and(|name| !name.trim().is_empty()) && values.contains_key("report-note") {
                    onsave.call(values);
                }
            },
            Input { id: "report-name", name: "report-name", label: "Report name", required: true,
                max_length: 100, value: name(), onvaluechange: move |value| name.set(value), }
            CustomField { label: "Report note", html_for: "report-note", helper_id: "report-note-help",
                helper_text: "Optional. The saved report keeps this note and the current inspection list.",
                textarea { id: "report-note", name: "report-note", class: "ix-input ix-textarea", rows: "3", maxlength: "240",
                    value: note(), "aria-describedby": "report-note-help", oninput: move |event| note.set(event.value()), }
            }
            Button { id: "report-save", button_type: ButtonType::Submit, "Save report locally" }
        }
    }
}

#[component]
pub fn InspectionReportExample() -> Element {
    let mut range = use_signal(initial_range);
    let mut locked = use_signal(|| false);
    let mut epoch = use_signal(|| 0_u64);
    let mut snapshots = use_signal(Vec::<ReportSnapshot>::new);
    let mut payload = use_signal(BTreeMap::<String, String>::new);
    let current = range();
    let from = date_text(current.dates.from);
    let to = date_text(current.dates.to);
    let filtered = INSPECTIONS
        .iter()
        .copied()
        .filter(|inspection| in_range(inspection, &current))
        .collect::<Vec<_>>();
    let visible_count = filtered.len();
    let saved = snapshots();
    let saved_count = saved.len();
    let serialized = serde_json::to_string(&payload()).unwrap_or_default();
    let options = [
        ("recent", "Recent inspections", 14),
        ("today", "Today", 20),
        ("all", "All inspections", 1),
    ]
    .into_iter()
    .map(|(id, label, from)| DateDropdownOption {
        id: id.into(),
        label: label.into(),
        from: date(from),
        to: date(20),
    })
    .collect::<Vec<_>>();
    rsx! {
        style { {STYLES} }
        section { id: "report-workflow", class: "inspection-report-example",
            h2 { "Prepare an inspection report" }
            p { class: "report-intro", "Choose a period from five illustrative October inspections, then save a named snapshot with your note. Reports stay in this example; no backend or download is involved." }
            div { class: "report-preparation", ReportPreparation {} }
            label { class: "report-lock-option",
                input { id: "report-lock", r#type: "checkbox", checked: locked(), onchange: move |event| locked.set(event.checked()), }
                "Lock the inspection period"
            }
            div { id: "report-toolbar", class: "report-toolbar",
                span { class: "report-period-label", "Inspection period" }
                DateDropdown { id: "report-period", enable_top_layer: true, today: date(20), options,
                    from: current.dates.from, to: current.dates.to, date_range_id: current.id.clone(),
                    min_date: date(1), max_date: date(20), disabled: locked(),
                    onrangechange: move |next| range.set(next),
                }
            }
            p { id: "report-status", class: "report-status", role: "status", "aria-live": "polite",
                "data-from": from.clone(), "data-to": to.clone(), "data-range-id": current.id,
                "data-visible": visible_count, "data-saved": saved_count, "data-lock": locked(),
                "{visible_count} inspections in the current period. {saved_count} reports saved locally."
            }
            table { id: "report-inspections", class: "report-table",
                caption { "Inspections in this period" }
                thead { tr { th { scope: "col", "Equipment" } th { scope: "col", "Inspected" } th { scope: "col", "Outcome" } } }
                tbody {
                    if filtered.is_empty() { tr { td { colspan: "3", "No inspections in this period. Choose another range." } } }
                    for inspection in filtered {
                        tr { key: "{inspection.id}", id: "report-row-{inspection.id}", "data-inspection": inspection.id, "data-date": date(inspection.day).to_string(),
                            td { "{inspection.equipment}" } td { {date(inspection.day).to_string()} } td { "{inspection.outcome}" }
                        }
                    }
                }
            }
            for generation in [epoch()] {
                ReportDraftForm { key: "report-draft-{generation}", onsave: move |values: BTreeMap<String, String>| {
                    let committed = range.peek().clone();
                    let inspection_ids = INSPECTIONS.iter().filter(|inspection| in_range(inspection, &committed)).map(|inspection| inspection.id.to_string()).collect();
                    let Some(name) = values.get("report-name").cloned() else { return; };
                    let Some(note) = values.get("report-note").cloned() else { return; };
                    snapshots.write().push(ReportSnapshot { name, note, range_id: committed.id, from: date_text(committed.dates.from), to: date_text(committed.dates.to), inspection_ids });
                    payload.set(values);
                } }
            }
            table { id: "report-records", class: "report-table report-records",
                caption { "Saved reports" }
                thead { tr { th { scope: "col", "Report snapshot" } th { scope: "col", "Report note" } } }
                tbody {
                    if saved.is_empty() { tr { td { colspan: "2", "No reports saved yet." } } }
                    for (index, report) in saved.iter().enumerate() {
                        tr { key: "{index}", id: "report-record-{index}", "data-name": report.name.clone(), "data-note": report.note.clone(),
                            "data-from": report.from.clone(), "data-to": report.to.clone(), "data-range-id": report.range_id.clone(), "data-inspection-ids": report.inspection_ids.join(","),
                            td { strong { "{report.name}" } p { "{report.from} to {report.to}" }
                                ul { for id in report.inspection_ids.iter() {
                                    li { {INSPECTIONS.iter().find(|inspection| inspection.id == id.as_str()).map_or("Inspection", |inspection| inspection.equipment)} }
                                } }
                                if report.inspection_ids.is_empty() { p { "No inspections in this snapshot." } }
                            }
                            td { if report.note.is_empty() { "No note" } else { "{report.note}" } }
                        }
                    }
                }
            }
            Button { id: "report-reset", class: "report-reset", variant: ButtonVariant::Secondary, button_type: ButtonType::Button,
                onclick: move |_| { range.set(initial_range()); snapshots.write().clear(); payload.write().clear(); epoch += 1; },
                "Reset report workspace"
            }
            output { id: "report-payload", hidden: true, "{serialized}" }
        }
    }
}
