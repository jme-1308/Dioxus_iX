//! Custom maintenance-window composition using the existing date/time card.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

pub const STYLES: &str = include_str!("styles/date_time_card.css");

#[derive(Clone, Debug, PartialEq, Eq)]
struct WindowPreparation {
    asset: String,
    date: String,
    time: String,
    note: String,
}

fn preparation_from_values(values: &BTreeMap<String, String>) -> Option<WindowPreparation> {
    let asset = values.get("asset")?;
    let date = values.get("date")?;
    let time = values.get("time")?;
    if asset != "Pump P-101" {
        return None;
    }
    IxDate::parse_iso(date)?;
    TimeFormat::Hm.parse(time)?;
    Some(WindowPreparation {
        asset: asset.clone(),
        date: date.clone(),
        time: time.clone(),
        note: values.get("note")?.clone(),
    })
}

#[component]
fn CardPreparation(generation: usize) -> Element {
    let mut hide_header = use_signal(|| false);
    let mut show_footer = use_signal(|| true);
    let mut time_appearance = use_signal(|| false);
    let mut no_padding = use_signal(|| false);
    let mut embedded = use_signal(|| false);
    let mut corners = use_signal(DateTimeCorners::default);
    let mut date = use_signal(|| String::from("2026-11-03"));
    let mut time = use_signal(|| String::from("09:00"));
    let mut root_node = use_signal(|| None::<Rc<MountedData>>);
    let mut note_node = use_signal(|| None::<Rc<MountedData>>);
    let mut snapshot = use_signal(|| None::<WindowPreparation>);
    let mut payload = use_signal(|| String::from("{}"));
    let mut message = use_signal(|| {
        String::from("Prepare a window, then save a local snapshot. Nothing is scheduled.")
    });
    let mounted = root_node.read().is_some();
    let can_focus = mounted && note_node.read().is_some();
    let corner_value = match corners() {
        DateTimeCorners::Rounded => "rounded",
        DateTimeCorners::Left => "left",
        DateTimeCorners::Right => "right",
        DateTimeCorners::Straight => "straight",
    };
    let header = rsx! {
        div { class: "date-card-heading",
            h3 { "Pump P-101 maintenance window" }
            p { "Prepare access and inspection notes for the cooling-water pump." }
        }
    };
    let footer = rsx! {
        div { class: "date-card-footer-actions",
            span { "Local preparation only" }
            Button { id: "date-card-save-footer", button_type: ButtonType::Submit, form: "date-card-form", "Save preparation" }
        }
    };
    let saved = snapshot();
    let has_saved = saved.is_some();
    rsx! {
        div { id: "date-card-owner", "data-generation": generation,
            div { class: "date-card-options", role: "group", "aria-label": "Card presentation",
                label { input { id: "date-card-hide-header", r#type: "checkbox", checked: hide_header(), onchange: move |event: FormEvent| hide_header.set(event.checked()) } "Hide header" }
                label { input { id: "date-card-show-footer", r#type: "checkbox", checked: show_footer(), onchange: move |event: FormEvent| show_footer.set(event.checked()) } "Show footer" }
                label { input { id: "date-card-time-appearance", r#type: "checkbox", checked: time_appearance(), onchange: move |event: FormEvent| time_appearance.set(event.checked()) } "Time picker spacing" }
                label { input { id: "date-card-no-padding", r#type: "checkbox", checked: no_padding(), onchange: move |event: FormEvent| no_padding.set(event.checked()) } "Remove body padding" }
                label { input { id: "date-card-embedded", r#type: "checkbox", checked: embedded(), onchange: move |event: FormEvent| embedded.set(event.checked()) } "Embedded appearance" }
                label { r#for: "date-card-corners", "Corners"
                    select { id: "date-card-corners", class: "ix-input", value: corner_value,
                        onchange: move |event: FormEvent| corners.set(match event.value().as_str() {
                            "left" => DateTimeCorners::Left, "right" => DateTimeCorners::Right,
                            "straight" => DateTimeCorners::Straight, _ => DateTimeCorners::Rounded,
                        }),
                        option { value: "rounded", "Rounded" }
                        option { value: "left", "Left" }
                        option { value: "right", "Right" }
                        option { value: "straight", "Straight" }
                    }
                }
            }
            DateTimeCard { id: "date-time-card-demo", class: "date-card-composition", header, footer,
                hide_header: hide_header(), has_footer: show_footer(), time_picker_appearance: time_appearance(),
                no_padding: no_padding(), embedded: embedded(), corners: corners(),
                onmounted: move |event: MountedEvent| root_node.set(Some(event.data())),
                form { id: "date-card-form",
                    onsubmit: move |event: FormEvent| {
                        event.prevent_default();
                        let values = event.values().into_iter().filter_map(|(name, value)| match value {
                            dioxus::html::FormValue::Text(value) => Some((name, value)), _ => None,
                        }).collect::<BTreeMap<_, _>>();
                        let Some(prepared) = preparation_from_values(&values) else {
                            message.set("Enter a valid date and start time before saving.".into()); return;
                        };
                        payload.set(serde_json::to_string(&values).unwrap_or_default());
                        snapshot.set(Some(prepared));
                        message.set("Maintenance window saved locally. Nothing was scheduled.".into());
                    },
                    input { r#type: "hidden", name: "asset", value: "Pump P-101" }
                    div { class: "date-card-fields",
                        Input { id: "date-card-date", name: "date", input_type: "date", label: "Maintenance date",
                            required: true, value: date(), onvaluechange: move |value| date.set(value),
                        }
                        Input { id: "date-card-time", name: "time", input_type: "time", label: "Start time",
                            required: true, value: time(), onvaluechange: move |value| time.set(value),
                        }
                    }
                    label { class: "date-card-note-label", r#for: "date-card-note", "Preparation note" }
                    textarea { id: "date-card-note", name: "note", class: "ix-input ix-textarea date-card-note", rows: "3", maxlength: "400",
                        placeholder: "Access instructions or tools to bring",
                        onmounted: move |event: MountedEvent| note_node.set(Some(event.data())),
                    }
                    p { class: "date-card-hint", "The note stays in place while you change the card presentation. Save captures the current date, time and note." }
                }
            }
            div { class: "date-card-actions",
                Button { id: "date-card-focus-note", variant: ButtonVariant::Secondary, button_type: ButtonType::Button, disabled: !can_focus,
                    onclick: move |_| {
                        let root = root_node.peek().clone();
                        let note = note_node.peek().clone();
                        if let (Some(root), Some(note)) = (root, note) {
                            spawn(async move {
                                if !root_node.peek().as_ref().is_some_and(|current| Rc::ptr_eq(current, &root)) { return; }
                                if !note_node.peek().as_ref().is_some_and(|current| Rc::ptr_eq(current, &note)) { return; }
                                let _ = note.set_focus(true).await;
                            });
                        }
                    }, "Edit preparation note"
                }
                if !show_footer() {
                    Button { id: "date-card-save-external", button_type: ButtonType::Submit, form: "date-card-form", "Save preparation" }
                }
            }
            p { id: "date-card-status", class: "date-card-status", role: "status", "aria-live": "polite",
                "data-saved": has_saved, "data-payload": payload(), "data-mounted": mounted,
                "{message}"
            }
            if let Some(prepared) = saved {
                dl { id: "date-card-snapshot", class: "date-card-snapshot", "data-asset": prepared.asset.clone(),
                    "data-date": prepared.date.clone(), "data-time": prepared.time.clone(), "data-note": prepared.note.clone(),
                    div { dt { "Equipment" } dd { "{prepared.asset}" } }
                    div { dt { "Window start" } dd { "{prepared.date} at {prepared.time}" } }
                    div { dt { "Preparation note" } dd { if prepared.note.is_empty() { "No note" } else { "{prepared.note}" } } }
                }
            }
        }
    }
}

#[component]
pub fn DateTimeCardExample() -> Element {
    let mut generation = use_signal(|| 0_usize);
    rsx! {
        style { {STYLES} }
        section { id: "date-time-card-example", class: "demo-section date-time-card-example",
            h2 { "Prepare a maintenance window" }
            p { class: "date-card-intro", "Compose a date and time card around an inspection form. Save a local preparation snapshot; this example does not reserve a window or send a request." }
            for owner in [generation()] { CardPreparation { key: "{owner}", generation: owner } }
            Button { id: "date-card-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                onclick: move |_| generation += 1, "Reset preparation"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_preparation_payload_requires_valid_date_time_and_keeps_saved_note() {
        let mut values = BTreeMap::from([
            ("asset".into(), "Pump P-101".into()),
            ("date".into(), "2026-11-03".into()),
            ("time".into(), "09:00".into()),
            ("note".into(), "Bring inlet seals".into()),
        ]);
        let prepared = preparation_from_values(&values).unwrap();
        values.insert("note".into(), "Next draft".into());
        assert_eq!(prepared.note, "Bring inlet seals");
        values.insert("date".into(), "2026-02-30".into());
        assert!(preparation_from_values(&values).is_none());
        values.insert("date".into(), "2026-11-03".into());
        values.insert("time".into(), "25:00".into());
        assert!(preparation_from_values(&values).is_none());
    }
}
