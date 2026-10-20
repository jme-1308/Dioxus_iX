//! A local appointment draft with bounded dates and quarter-hour scheduling.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScheduleDraft {
    task: String,
    start: String,
    end: String,
    time: String,
}

fn initial_draft() -> ScheduleDraft {
    ScheduleDraft {
        task: "Motor 001 bearing inspection".into(),
        start: "2026-10-20".into(),
        end: "2026-10-22".into(),
        time: "09:00".into(),
    }
}

fn first_date() -> IxDate {
    IxDate::new(2026, 10, 1).unwrap()
}
fn last_date() -> IxDate {
    IxDate::new(2026, 11, 30).unwrap()
}
fn first_time() -> IxTime {
    IxTime::new(8, 0, 0, 0).unwrap()
}
fn last_time() -> IxTime {
    IxTime::new(17, 0, 0, 0).unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Appointment {
    task: String,
    start: IxDate,
    end: IxDate,
    time: IxTime,
}

impl Appointment {
    fn draft(&self) -> ScheduleDraft {
        ScheduleDraft {
            task: self.task.clone(),
            start: self.start.to_string(),
            end: self.end.to_string(),
            time: TimeFormat::Hm.format(self.time),
        }
    }
}

fn validate(draft: &ScheduleDraft) -> Result<Appointment, &'static str> {
    let task = draft.task.trim();
    if task.is_empty() {
        return Err("Enter a maintenance task containing more than spaces.");
    }
    let start = DateFormat::Iso
        .parse(&draft.start)
        .ok_or("Enter a valid start date as YYYY-MM-DD.")?;
    let end = DateFormat::Iso
        .parse(&draft.end)
        .ok_or("Enter a valid end date as YYYY-MM-DD.")?;
    if start < first_date() || end > last_date() || start > last_date() || end < first_date() {
        return Err("Choose dates from 1 October through 30 November 2026.");
    }
    if start > end {
        return Err(
            "The end date must be on or after the start date. Correct the dates or choose a preset.",
        );
    }
    let time = TimeFormat::Hm
        .parse(&draft.time)
        .ok_or("Enter a valid start time as HH:MM.")?;
    if time < first_time() || time > last_time() {
        return Err("Choose a start time from 08:00 through 17:00.");
    }
    if time.minute() % 15 != 0 {
        return Err("This appointment uses quarter-hour slots. Choose minutes 00, 15, 30 or 45.");
    }
    Ok(Appointment {
        task: task.into(),
        start,
        end,
        time,
    })
}

fn calendar_labels(locale: CalendarLocale) -> CalendarLabels {
    let mut labels = locale.labels();
    if locale == CalendarLocale::De {
        labels.previous_month = "Vorheriger Monat".into();
        labels.next_month = "Nächster Monat".into();
        labels.select_month = "Monat auswählen".into();
        labels.select_year = "Jahr auswählen".into();
        labels.done = "Fertig".into();
    }
    labels
}

#[component]
fn SchedulePreparation() -> Element {
    let mut note = use_signal(|| "Prepare Motor 001 maintenance".to_string());
    rsx! {
        label { class: "schedule-preparation", r#for: "schedule-preparation",
            span { "Inspection preparation" }
            input { id: "schedule-preparation", r#type: "text", maxlength: "160", value: note(), oninput: move |event: FormEvent| note.set(event.value()) }
        }
    }
}

#[component]
fn ScheduleEditor(
    draft: Signal<ScheduleDraft>,
    disabled: bool,
    read_only: bool,
    locale: CalendarLocale,
    oncalendarstatus: EventHandler<bool>,
) -> Element {
    let mut draft = draft;
    let current = draft();
    let blocked = disabled || read_only;
    let labels = calendar_labels(locale);
    let mut time_labels = locale.time_labels();
    if locale == CalendarLocale::De {
        time_labels.header = "Uhrzeit".into();
        time_labels.confirm = "Bestätigen".into();
    }
    let update_calendar = use_callback(move |selection: DateSelection| {
        if blocked {
            return;
        }
        let mut next = draft.peek().clone();
        next.start = selection
            .from
            .map_or_else(String::new, |date| date.to_string());
        next.end = selection
            .to
            .map_or_else(String::new, |date| date.to_string());
        draft.set(next);
    });
    rsx! {
        div { class: "schedule-editor",
            div { id: "schedule-calendar", class: "schedule-calendar", role: "group", "aria-labelledby": "schedule-calendar-heading",
                h3 { id: "schedule-calendar-heading", "Choose the draft date range" }
                DatePicker { today: IxDate::new(2026, 10, 20).unwrap(), from: DateFormat::Iso.parse(&current.start), to: DateFormat::Iso.parse(&current.end),
                    min_date: Some(first_date()), max_date: Some(last_date()), disabled: blocked, labels: labels.clone(),
                    ondatechange: move |selection| { if !blocked { update_calendar.call(selection); oncalendarstatus.call(false); } },
                    onselect: move |selection| { if !blocked { update_calendar.call(selection); oncalendarstatus.call(true); } },
                }
            }
            div { class: "schedule-fields",
                label { class: "schedule-task-field", r#for: "schedule-task", span { "Maintenance task" }
                    input { id: "schedule-task", r#type: "text", required: true, maxlength: "120", value: current.task, disabled, readonly: read_only,
                        oninput: move |event: FormEvent| { if !blocked { draft.write().task = event.value(); } },
                    }
                }
                DateInput { id: "schedule-start", today: IxDate::new(2026, 10, 20).unwrap(), label: "Start date", format: DateFormat::Iso,
                    value: current.start, min_date: Some(first_date()), max_date: Some(last_date()), required: true, disabled, readonly: read_only, labels: labels.clone(),
                    calendar_button_label: if locale == CalendarLocale::De { "Startkalender öffnen" } else { "Open start calendar" },
                    helper_text: "YYYY-MM-DD · 2026-10-01 to 2026-11-30",
                    onvaluechange: move |value| { if !blocked { draft.write().start = value; oncalendarstatus.call(false); } },
                }
                DateInput { id: "schedule-end", today: IxDate::new(2026, 10, 20).unwrap(), label: "End date", format: DateFormat::Iso,
                    value: current.end, min_date: Some(first_date()), max_date: Some(last_date()), required: true, disabled, readonly: read_only, labels,
                    calendar_button_label: if locale == CalendarLocale::De { "Endkalender öffnen" } else { "Open end calendar" },
                    helper_text: "On or after the start date",
                    onvaluechange: move |value| { if !blocked { draft.write().end = value; oncalendarstatus.call(false); } },
                }
                TimeInput { id: "schedule-time", label: "Start time", value: current.time, format: TimeFormat::Hm, required: true,
                    min_time: Some(first_time()), max_time: Some(last_time()), minute_interval: 15, disabled, readonly: read_only, labels: time_labels,
                    time_button_label: if locale == CalendarLocale::De { "Uhrzeit auswählen" } else { "Choose start time" },
                    helper_text: "Appointments start at 15-minute intervals from 08:00 to 17:00.",
                    onvaluechange: move |value| { if !blocked { draft.write().time = value; } },
                }
            }
        }
    }
}

/// Schedule one illustrative local maintenance appointment with an explicit save boundary.
#[component]
pub fn MaintenanceScheduleExample() -> Element {
    let mut draft = use_signal(initial_draft);
    let mut saved = use_signal(|| None::<Appointment>);
    let mut disabled = use_signal(|| false);
    let mut read_only = use_signal(|| false);
    let mut locale = use_signal(|| CalendarLocale::En);
    let mut validation = use_signal(|| None::<String>);
    let mut calendar_confirmed = use_signal(|| false);
    let mut saves = use_signal(|| 0_usize);
    let mut cancellations = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let current = draft();
    let appointment = saved();
    let blocked = disabled() || read_only();
    let has_saved = appointment.is_some();
    let preset = use_callback(move |november: bool| {
        if *disabled.peek() || *read_only.peek() {
            return;
        }
        let mut next = draft.peek().clone();
        next.start = if november { "2026-11-03" } else { "2026-10-20" }.into();
        next.end = if november { "2026-11-05" } else { "2026-10-22" }.into();
        draft.set(next);
        calendar_confirmed.set(false);
        validation.set(None);
    });
    let recover = use_callback(move |_| {
        if !*disabled.peek() && !*read_only.peek() {
            draft.set(initial_draft());
            validation.set(None);
            calendar_confirmed.set(false);
        }
    });
    rsx! {
        section { id: "maintenance-schedule", class: "maintenance-schedule-example",
            h2 { "Maintenance scheduling" }
            p { class: "schedule-intro", "Plan an illustrative local Motor 001 appointment from 1 October through 30 November 2026. Appointments use quarter-hour start slots between 08:00 and 17:00. Calendar confirmation changes the draft; Save appointment commits it. No maintenance service is contacted." }
            div { class: "schedule-toolbar",
                Button { id: "schedule-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        draft.set(initial_draft()); saved.set(None); disabled.set(false); read_only.set(false); locale.set(CalendarLocale::En);
                        validation.set(None); calendar_confirmed.set(false); saves.set(0); cancellations.set(0); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore schedule"
                }
                Button { id: "schedule-october", variant: ButtonVariant::Secondary, disabled: blocked, onclick: move |_| preset.call(false), "Oct 20–22" }
                Button { id: "schedule-november", variant: ButtonVariant::Secondary, disabled: blocked, onclick: move |_| preset.call(true), "Nov 3–5" }
            }
            details { id: "schedule-options", class: "schedule-options",
                summary { "Scheduling options" }
                div { class: "schedule-option-fields",
                    label { input { id: "schedule-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable scheduling" }
                    label { input { id: "schedule-readonly", r#type: "checkbox", checked: read_only(), onchange: move |event: FormEvent| read_only.set(event.checked()) } "Read-only draft" }
                    label { r#for: "schedule-locale", span { "Calendar language" }
                        select { id: "schedule-locale", value: if locale() == CalendarLocale::De { "de" } else { "en" },
                            onchange: move |event: FormEvent| locale.set(if event.value() == "de" { CalendarLocale::De } else { CalendarLocale::En }),
                            option { value: "en", selected: locale() == CalendarLocale::En, "English" }
                            option { value: "de", selected: locale() == CalendarLocale::De, "Deutsch" }
                        }
                    }
                }
            }
            for generation in [epoch()] { SchedulePreparation { key: "schedule-preparation-{generation}" } }
            for generation in [epoch()] {
                ScheduleEditor { key: "schedule-editor-{generation}", draft, disabled: disabled(), read_only: read_only(), locale: locale(),
                    oncalendarstatus: move |confirmed| calendar_confirmed.set(confirmed),
                }
            }
            div { class: "schedule-actions",
                Button { id: "schedule-save", disabled: blocked,
                    onclick: move |_| {
                        if *disabled.peek() || *read_only.peek() { return; }
                        match validate(&draft.peek()) {
                            Ok(appointment) => { saved.set(Some(appointment)); validation.set(None); let count = saves.peek().saturating_add(1); saves.set(count); }
                            Err(error) => validation.set(Some(error.into())),
                        }
                    }, "Save appointment"
                }
                Button { id: "schedule-cancel", variant: ButtonVariant::Secondary, disabled: blocked,
                    onclick: move |_| {
                        if *disabled.peek() || *read_only.peek() { return; }
                        let restored = saved.peek().as_ref().map_or_else(initial_draft, Appointment::draft);
                        draft.set(restored); validation.set(None); calendar_confirmed.set(false);
                    }, "Cancel draft"
                }
                button { id: "schedule-delete", class: "ix-button ix-button-secondary", r#type: "button", "aria-disabled": (blocked || !has_saved).to_string(),
                    onclick: move |_| {
                        if !*disabled.peek() && !*read_only.peek() && saved.peek().is_some() {
                            saved.set(None); let count = cancellations.peek().saturating_add(1); cancellations.set(count);
                        }
                    }, "Cancel saved appointment"
                }
            }
            if let Some(error) = validation() { p { id: "schedule-validation", class: "schedule-validation", role: "alert", "{error}" } }
            article { id: "schedule-saved", class: "schedule-saved", "aria-labelledby": "schedule-saved-heading",
                h3 { id: "schedule-saved-heading", "Saved local appointment" }
                if let Some(appointment) = appointment {
                    dl {
                        div { dt { "Task" } dd { "data-field": "task", "{appointment.task}" } }
                        div { dt { "Start date" } dd { "data-field": "start", "{appointment.start}" } }
                        div { dt { "End date" } dd { "data-field": "end", "{appointment.end}" } }
                        div { dt { "Start time" } dd { "data-field": "time", {TimeFormat::Hm.format(appointment.time)} } }
                    }
                } else {
                    div { id: "schedule-empty", EmptyState { header: "No saved appointment", subheader: "Edit the draft above, then save the local appointment.", action: "Start again", action_disabled: blocked, layout: EmptyStateLayout::CompactBreak, onaction: recover } }
                }
            }
            output { id: "schedule-result", class: "schedule-result", role: "status",
                "data-saved": saved().is_some().to_string(), "data-start": current.start, "data-end": current.end, "data-time": current.time, "data-task": current.task,
                "data-saves": saves().to_string(), "data-cancellations": cancellations().to_string(), "data-disabled": disabled().to_string(), "data-readonly": read_only().to_string(),
                "data-locale": if locale() == CalendarLocale::De { "de" } else { "en" }, "data-calendar-confirmed": calendar_confirmed().to_string(),
                "Saved appointments: {saves} · Cancelled appointments: {cancellations}"
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/maintenance_schedule.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schedule_validates_real_appointment_bounds_order_and_working_slots() {
        let mut draft = initial_draft();
        let appointment = validate(&draft).unwrap();
        assert_eq!(appointment.start.to_string(), "2026-10-20");
        draft.end = "2026-10-19".into();
        assert!(validate(&draft).unwrap_err().contains("end date"));
        draft = initial_draft();
        draft.time = "09:07".into();
        assert!(validate(&draft).unwrap_err().contains("quarter-hour"));
        draft = initial_draft();
        draft.task = "   ".into();
        assert!(validate(&draft).unwrap_err().contains("maintenance task"));
    }
    #[test]
    fn maintenance_schedule_renders_controlled_draft_and_empty_saved_record() {
        let mut dom = VirtualDom::new(MaintenanceScheduleExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "maintenance-schedule",
            "schedule-task",
            "schedule-start",
            "schedule-end",
            "schedule-time",
            "schedule-calendar",
            "schedule-october",
            "schedule-november",
            "schedule-save",
            "schedule-cancel",
            "schedule-delete",
            "schedule-preparation",
            "schedule-empty",
            "schedule-saved",
            "schedule-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(
            html.contains("Motor 001 bearing inspection")
                && html.contains("Prepare Motor 001 maintenance")
        );
        assert!(
            html.contains("value=\"2026-10-20\"")
                && html.contains("value=\"2026-10-22\"")
                && html.contains("value=\"09:00\"")
        );
        assert!(
            html.contains("No saved appointment")
                && html.contains("data-saved=\"false\"")
                && html.contains("data-saves=\"0\"")
        );
    }
}
