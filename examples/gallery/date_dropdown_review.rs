//! Behavioral review in addition to the isolated upstream preview adaptations.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn DateDropdownReview() -> Element {
    let today = IxDate::new(2026, 10, 16).unwrap();
    let mut notifications = use_signal(Vec::<String>::new);
    let options = vec![
        DateDropdownOption {
            id: "today".into(),
            label: "Review today".into(),
            from: today,
            to: today,
        },
        DateDropdownOption {
            id: "week".into(),
            label: "Review last 7 days".into(),
            from: today.add_days(-6).unwrap(),
            to: today,
        },
    ];
    let count = notifications.read().len();
    let latest = notifications
        .read()
        .last()
        .cloned()
        .unwrap_or_else(|| "No range events yet.".into());
    rsx! {
        section { class: "demo-section", id: "date-dropdown-events",
            h2 { "Date range notifications" }
            p { "Choose a preset to publish its range while the calendar stays open. Escape, outside dismissal and the trigger publish the current range again. Done publishes both its selection and dismissal events, matching the source component." }
            div { id: "date-dropdown-event-review",
                DateDropdown { today, options, date_range_id: "today",
                    onrangechange: move |change: DateRangeChange| {
                        let from = change.dates.from.map_or_else(|| "none".into(), |date| date.to_string());
                        let to = change.dates.to.map_or_else(|| "none".into(), |date| date.to_string());
                        notifications.write().push(format!("{}: {from} to {to}", change.id));
                    },
                }
            }
            p { role: "status", id: "date-dropdown-event-status", "Range events: {count}. {latest}" }
            Button { variant: ButtonVariant::Secondary, onclick: move |_| notifications.write().clear(), "Clear range event log" }
        }
    }
}
