use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn ActionExamples() -> Element {
    let mut last_action = use_signal(|| "No action yet".to_string());
    let mut toggle = use_signal(|| false);
    let mut favorite = use_signal(|| true);
    let mut loading = use_signal(|| false);
    let mut progress = use_signal(|| 75.0);
    let variants = [
        ButtonVariant::Primary,
        ButtonVariant::Secondary,
        ButtonVariant::Tertiary,
        ButtonVariant::SubtlePrimary,
        ButtonVariant::SubtleSecondary,
        ButtonVariant::SubtleTertiary,
        ButtonVariant::DangerPrimary,
        ButtonVariant::DangerSecondary,
        ButtonVariant::DangerTertiary,
    ];
    let statuses = [
        StatusVariant::Primary,
        StatusVariant::Alarm,
        StatusVariant::Critical,
        StatusVariant::Warning,
        StatusVariant::Info,
        StatusVariant::Neutral,
        StatusVariant::Success,
    ];
    let items = vec![
        SplitButtonItem::new("item-1", "Item 1"),
        SplitButtonItem::new("item-2", "Item 2"),
    ];
    rsx! {
        section { class: "demo-section", id: "buttons",
            h2 { "Buttons" }
            p { "Primary, secondary, tertiary, subtle, and danger variants from the upstream button examples." }
            for variant in variants {
                div { class: "demo-row",
                    span { class: "demo-label", "{variant.as_str()}" }
                    Button { variant, onclick: move |_| last_action.set(format!("{} clicked", variant.as_str())), "Button" }
                    Button { variant, icon: "add", onclick: move |_| last_action.set("Icon button clicked".into()), "With icon" }
                    Button { variant, disabled: true, "Disabled" }
                    Button { variant, loading: true, "Loading" }
                }
            }
            div { class: "demo-row",
                Button { loading: loading(), onclick: move |_| loading.set(true), "Start action" }
                Button { variant: ButtonVariant::SubtleSecondary, onclick: move |_| { loading.set(false); last_action.set("Action reset".into()); }, "Reset loading" }
                LinkButton { url: "https://github.com/siemens/ix", target: "_blank", "Upstream project" }
                LinkButton { url: "#buttons", disabled: true, "Disabled link" }
            }
            p { role: "status", "{last_action}" }
            InertActionsExample {}
        }
        section { class: "demo-section", id: "icon-buttons",
            h2 { "Icon buttons" }
            div { class: "demo-row",
                for size in [ButtonIconSize::Twelve, ButtonIconSize::Sixteen, ButtonIconSize::Twenty, ButtonIconSize::TwentyFour] {
                    IconButton { icon: "add", label: format!("Add, {} pixel icon", size.pixels()), size, onclick: move |_| last_action.set(format!("{} pixel button clicked", size.pixels())) }
                }
                IconButton { icon: "add", label: "Add item", variant: ButtonVariant::Primary, oval: true }
                IconButton { icon: "trashcan", label: "Delete item", variant: ButtonVariant::DangerSecondary }
                IconButton { icon: "add", label: "Disabled action", disabled: true }
                IconButton { icon: "add", label: "Loading action", loading: true }
            }
        }
        section { class: "demo-section", id: "toggle-buttons",
            h2 { "Toggle buttons" }
            div { class: "demo-row",
                ToggleButton { pressed: toggle(), icon: "check", onpressedchange: move |value| toggle.set(value), "Toggle button" }
                ToggleButton { pressed: true, disabled: true, onpressedchange: |_| {}, "Disabled pressed" }
                IconToggleButton { icon: "star", label: "Favorite", pressed: favorite(), onpressedchange: move |value| favorite.set(value) }
                IconToggleButton { icon: "star", label: "Disabled favorite", pressed: true, disabled: true, onpressedchange: |_| {} }
            }
            p { role: "status", "Toggle pressed: {toggle}. Favorite: {favorite}." }
        }
        section { class: "demo-section", id: "split-buttons",
            h2 { "Split button" }
            div { class: "demo-row",
                SplitButton { label: "Action text", items: items.clone(), onclick: move |_| last_action.set("Main action clicked".into()), onselect: move |value| last_action.set(value) }
                SplitButton { label: "Action text", items: items.clone(), disabled: true, onselect: |_| {} }
                SplitButton { label: "Action text", items: items.clone(), disable_button: true, onselect: move |value| last_action.set(value) }
                SplitButton { label: "Action text", items, disable_dropdown_button: true, onselect: |_| {} }
            }
        }
        section { class: "demo-section", id: "status-labels",
            h2 { "Pills" }
            div { class: "demo-row",
                for variant in statuses { Pill { variant, "{variant.as_str()}" } }
            }
            div { class: "demo-row",
                for variant in statuses { Pill { variant, outline: true, icon: "info", "{variant.as_str()}" } }
            }
        }
        section { class: "demo-section", id: "badges",
            h2 { "Badges" }
            div { class: "demo-row",
                Badge { label: "7" }
                Badge { label: "120", variant: StatusVariant::Alarm }
                Badge { badge_type: BadgeType::Label, label: "New", variant: StatusVariant::Info }
                Badge { badge_type: BadgeType::Dot, variant: StatusVariant::Success, aria_label: "Online" }
                Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Warning, aria_label: "Warning" }
                Badge { label: "5", border: true,
                    IconButton { icon: "notification", label: "Notifications, 5 unread", variant: ButtonVariant::SubtlePrimary }
                }
            }
        }
        section { class: "demo-section", id: "avatars-spinners",
            h2 { "Avatars and spinners" }
            div { class: "demo-row",
                Avatar { username: "Alex Taylor", initials: "AT" }
                Avatar { username: "Operator" }
                Avatar { username: "Large profile", initials: "LP", size: 48 }
                for size in [SpinnerSize::Xxs, SpinnerSize::Xs, SpinnerSize::Sm, SpinnerSize::Md, SpinnerSize::Xxl] {
                    Spinner { size, variant: SpinnerVariant::Primary }
                }
            }
        }
        section { class: "demo-section", id: "progress",
            h2 { "Progress indicator" }
            p { "The download example, circular sizes, and all upstream status variants." }
            div { class: "demo-progress-stack",
                ProgressIndicator { label: "Download", value: progress(), helper_text: "This is a help text for the progress indicator", "{progress}%" }
                for status in [ProgressStatus::Default, ProgressStatus::Error, ProgressStatus::Info, ProgressStatus::Warning, ProgressStatus::Success, ProgressStatus::Paused] {
                    ProgressIndicator { value: progress(), status, label: status.as_str(), "{progress}%" }
                }
            }
            div { class: "demo-row",
                for size in [ProgressSize::Xs, ProgressSize::Sm, ProgressSize::Md, ProgressSize::Lg, ProgressSize::Xl] {
                    ProgressIndicator { value: progress(), progress_type: ProgressType::Circular, size, "{progress}%" }
                }
            }
            div { class: "demo-row",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| progress.set((progress() - 10.0_f64).max(0.0)), "Decrease progress" }
                Button { onclick: move |_| progress.set((progress() + 10.0_f64).min(100.0)), "Increase progress" }
            }
        }
        section { class: "demo-section", id: "typography",
            h2 { "Typography" }
            Typography { format: TypographyFormat::Display, "Display" }
            Typography { format: TypographyFormat::H1, "Heading 1" }
            Typography { format: TypographyFormat::H2, "Heading 2" }
            Typography { format: TypographyFormat::H3, "Heading 3" }
            Typography { format: TypographyFormat::Body, "Body: Industrial interfaces need clear, consistent text." }
            Typography { format: TypographyFormat::BodySm, text_color: TypographyColor::Soft, "Small secondary text" }
            Typography { format: TypographyFormat::Code, "let status = MachineStatus::Running;" }
            Divider {}
        }
    }
}

/// An inspection note stays editable while its native action controls are suspended.
#[component]
pub fn InertActionsExample() -> Element {
    let mut suspended = use_signal(|| false);
    let mut note = use_signal(|| "Check cooling system".to_string());
    let mut runs = use_signal(|| 0usize);
    let mut reviews = use_signal(|| 0usize);
    let mut refreshes = use_signal(|| 0usize);
    let mut submissions = use_signal(|| 0usize);
    let mut submitted_note = use_signal(String::new);
    rsx! {
        div { id: "inert-actions",
            h3 { "Inspection actions" }
            p { "Suspend the actions while preparing the inspection note. The note remains editable." }
            form { id: "inert-actions-form",
                onsubmit: move |event: FormEvent| {
                    event.prevent_default();
                    let submitted = event.values().into_iter().find_map(|(name, value)| {
                        match value {
                            dioxus::html::FormValue::Text(value) if name == "note" => Some(value),
                            _ => None,
                        }
                    }).unwrap_or_default();
                    submitted_note.set(submitted);
                    submissions += 1;
                },
                div { class: "demo-row",
                    label {
                        input { id: "inert-suspend", r#type: "checkbox", checked: suspended(),
                            onchange: move |event: FormEvent| suspended.set(event.checked()),
                        }
                        "Suspend inspection actions"
                    }
                }
                div { class: "demo-row",
                    Input { id: "inert-note", name: "note", label: "Inspection note", value: note(),
                        onvaluechange: move |value| note.set(value),
                    }
                }
                div { class: "demo-row",
                    Button { id: "inert-before", variant: ButtonVariant::Secondary, "Previous action" }
                    Button { id: "inert-run", button_type: ButtonType::Submit, inert: suspended(),
                        onclick: move |_| runs += 1, "Run inspection" }
                    Button { id: "inert-link", variant: ButtonVariant::Secondary, href: "#inert-actions", inert: suspended(),
                        onclick: move |event: MouseEvent| { event.prevent_default(); reviews += 1; }, "Review inspection" }
                    IconButton { id: "inert-icon", icon: "refresh", label: "Refresh inspection", inert: suspended(),
                        onclick: move |_| refreshes += 1 }
                    Button { id: "inert-after", variant: ButtonVariant::Secondary, "Next action" }
                }
                div { class: "demo-row",
                    Button { id: "inert-reset", variant: ButtonVariant::SubtleSecondary,
                        onclick: move |_| {
                            suspended.set(false); runs.set(0); reviews.set(0); refreshes.set(0);
                            submissions.set(0); submitted_note.set(String::new());
                        },
                        "Reset counts and resume actions"
                    }
                }
            }
            output { id: "inert-result", role: "status", "aria-live": "polite",
                "data-run": runs(), "data-review": reviews(), "data-refresh": refreshes(),
                "data-submissions": submissions(), "data-submitted-note": submitted_note(),
                "data-suspended": suspended(),
                "Actions suspended: {suspended}. Runs: {runs}. Reviews: {reviews}. Refreshes: {refreshes}. Submissions: {submissions}. Submitted note: {submitted_note}"
            }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/actions.css");
