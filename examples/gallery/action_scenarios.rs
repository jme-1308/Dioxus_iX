//! Individual action scenarios adapted from the upstream HTML previews (MIT).
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeSet, time::Duration};

pub const SCENARIOS: &[&str] = &[
    "avatar",
    "avatar-image",
    "avatar-initials",
    "badge",
    "badge-counter",
    "badge-dot",
    "badge-label",
    "badge-status-icon",
    "button-danger-primary",
    "button-danger-secondary",
    "button-danger-tertiary",
    "button-group",
    "button-loading",
    "button-secondary",
    "button-subtle-primary",
    "button-subtle-secondary",
    "button-subtle-tertiary",
    "button-tertiary",
    "button-text-icon",
    "button-with-icon",
    "button-with-link",
    "buttons",
    "chip",
    "divider",
    "icon-toggle-button-secondary",
    "icon-toggle-button-subtle-primary",
    "icon-toggle-button-subtle-secondary",
    "icon-toggle-button-subtle-tertiary",
    "icon-toggle-button-tertiary",
    "link-button",
    "link-button-disabled",
    "loading",
    "pill",
    "pill-variants",
    "progress-indicator",
    "progress-indicator-circular",
    "progress-indicator-circular-sizes",
    "progress-indicator-circular-status",
    "progress-indicator-linear-sizes",
    "progress-indicator-linear-status",
    "spinner",
    "spinner-large",
    "split-button",
    "split-button-icons",
    "toggle-button-primary",
    "toggle-button-secondary",
    "toggle-button-subtle-primary",
    "toggle-button-subtle-secondary",
    "toggle-button-subtle-tertiary",
    "toggle-button-tertiary",
];

fn variant(name: &str) -> ButtonVariant {
    match name {
        "secondary" => ButtonVariant::Secondary,
        "tertiary" => ButtonVariant::Tertiary,
        "subtle-primary" => ButtonVariant::SubtlePrimary,
        "subtle-secondary" => ButtonVariant::SubtleSecondary,
        "subtle-tertiary" => ButtonVariant::SubtleTertiary,
        "danger-primary" => ButtonVariant::DangerPrimary,
        "danger-secondary" => ButtonVariant::DangerSecondary,
        "danger-tertiary" => ButtonVariant::DangerTertiary,
        _ => ButtonVariant::Primary,
    }
}
fn statuses() -> [(StatusVariant, &'static str); 8] {
    [
        (StatusVariant::Primary, "Primary"),
        (StatusVariant::Alarm, "Alarm"),
        (StatusVariant::Critical, "Critical"),
        (StatusVariant::Warning, "Warning"),
        (StatusVariant::Info, "Info"),
        (StatusVariant::Neutral, "Neutral"),
        (StatusVariant::Success, "Success"),
        (StatusVariant::Custom, "Custom"),
    ]
}
const CUSTOM_BG: &str = "var(--si-sys-color-data-categorical-9,var(--ix-color-secondary))";
const CUSTOM_FG: &str =
    "var(--si-sys-color-text-inverse,var(--ix-color-on-secondary,var(--ix-color-on-primary)))";
const ACCENT_BG: &str = "var(--si-sys-color-background-accent-hover,var(--ix-color-primary))";

#[component]
pub fn ActionScenario(name: String) -> Element {
    let content = match name.as_str() {
        "avatar" => rsx! { Avatar {} },
        "avatar-initials" => rsx! { Avatar { initials: "JD" } },
        "avatar-image" => {
            rsx! { Avatar { image: "https://www.gravatar.com/avatar/00000000000000000000000000000000" } }
        }
        "buttons" => rsx! { ButtonPair { variant: ButtonVariant::Primary } },
        "button-secondary"
        | "button-tertiary"
        | "button-subtle-primary"
        | "button-subtle-secondary"
        | "button-subtle-tertiary"
        | "button-danger-primary"
        | "button-danger-secondary"
        | "button-danger-tertiary" => {
            rsx! { ButtonPair { variant: variant(name.trim_start_matches("button-")) } }
        }
        "button-group" => rsx! { ButtonGroupScenario {} },
        "button-text-icon" => rsx! { TextIconScenario {} },
        "button-with-icon" => rsx! { IconButtonScenario {} },
        "button-with-link" => rsx! { ButtonLinkScenario {} },
        "button-loading" => rsx! { ButtonLoadingScenario {} },
        "toggle-button-primary"
        | "toggle-button-secondary"
        | "toggle-button-tertiary"
        | "toggle-button-subtle-primary"
        | "toggle-button-subtle-secondary"
        | "toggle-button-subtle-tertiary" => {
            rsx! { ToggleScenario { variant: variant(name.trim_start_matches("toggle-button-")), icon_only: false } }
        }
        "icon-toggle-button-secondary"
        | "icon-toggle-button-tertiary"
        | "icon-toggle-button-subtle-primary"
        | "icon-toggle-button-subtle-secondary"
        | "icon-toggle-button-subtle-tertiary" => {
            rsx! { ToggleScenario { variant: variant(name.trim_start_matches("icon-toggle-button-")), icon_only: true } }
        }
        "link-button" | "link-button-disabled" => {
            rsx! { LinkButton { url: "https://ix.siemens.io/", disabled: name == "link-button-disabled", "Link text" } }
        }
        "divider" => rsx! { Divider {} },
        "spinner" => rsx! { Spinner {} },
        "spinner-large" => rsx! { Spinner { size: SpinnerSize::Xxl } },
        "split-button" => rsx! { SplitScenario { icons: false } },
        "split-button-icons" => rsx! { SplitScenario { icons: true } },
        "pill" => rsx! { PillScenario {} },
        "pill-variants" => rsx! { PillVariants {} },
        "chip" => rsx! { ChipScenario {} },
        "badge" => rsx! { BadgeScenario {} },
        "badge-counter" => rsx! { BadgeMatrix { kind: BadgeType::Counter } },
        "badge-dot" => rsx! { BadgeMatrix { kind: BadgeType::Dot } },
        "badge-label" => rsx! { BadgeMatrix { kind: BadgeType::Label } },
        "badge-status-icon" => rsx! { BadgeMatrix { kind: BadgeType::StatusIcon } },
        "progress-indicator"
        | "progress-indicator-circular"
        | "progress-indicator-linear-sizes"
        | "progress-indicator-circular-sizes"
        | "progress-indicator-linear-status"
        | "progress-indicator-circular-status" => rsx! { ProgressScenario { name: name.clone() } },
        "loading" => rsx! { LoadingScenario {} },
        _ => rsx! { p { "This action scenario is not registered." } },
    };
    rsx! { div { class: "ix-action-scenario", "data-action-scenario": name, {content} } }
}

#[component]
fn ButtonPair(variant: ButtonVariant) -> Element {
    let mut count = use_signal(|| 0);
    rsx! { div { class: "action-scenario-row",
        Button { variant, onclick: move |_| count += 1, "Button" }
        Button { variant, disabled: true, "Button" }
    } p { class: "action-scenario-feedback", role: "status", "Button clicks: {count}" } }
}

#[component]
fn ButtonGroupScenario() -> Element {
    let mut selected = use_signal(|| 2);
    rsx! { div { class: "action-scenario-button-group", role: "group", aria_label: "Button group",
        for index in 1..=4 {
            Button { variant: if index == 2 { ButtonVariant::Primary } else { ButtonVariant::SubtleSecondary },
                onclick: move |_| selected.set(index), "Button {index}" }
        }
    } p { class: "action-scenario-feedback", role: "status", "Last button clicked: {selected}" } }
}

#[component]
fn TextIconScenario() -> Element {
    let mut count = use_signal(|| 0);
    rsx! { div { class: "action-scenario-row",
        Button { icon: "star", onclick: move |_| count += 1, "Button" }
        Button { icon: "star", icon_right: "bulb", onclick: move |_| count += 1, "Button" }
        Button { icon_right: "bulb", onclick: move |_| count += 1, "Button" }
        Button { variant: ButtonVariant::SubtlePrimary, icon: "star", onclick: move |_| count += 1, "Button" }
        Button { variant: ButtonVariant::SubtlePrimary, icon: "star", onclick: move |_| count += 1, "Button" }
        Button { variant: ButtonVariant::Tertiary, icon: "star", onclick: move |_| count += 1, "Button" }
    } p { class: "action-scenario-feedback", role: "status", "Button clicks: {count}" } }
}

#[component]
fn IconButtonScenario() -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        for oval in [false, true] { div { class: "action-scenario-row",
            for variant in [ButtonVariant::Primary, ButtonVariant::SubtlePrimary, ButtonVariant::DangerPrimary, ButtonVariant::SubtlePrimary, ButtonVariant::Tertiary] {
                IconButton { icon: "info", label: "Info", variant, oval, onclick: move |_| count += 1 }
            }
        } }
        div { class: "action-scenario-row",
            for size in [ButtonIconSize::Twelve, ButtonIconSize::Sixteen, ButtonIconSize::TwentyFour] { IconButton { icon: "info", label: "Info", size, onclick: move |_| count += 1 } }
        }
        p { class: "action-scenario-feedback", role: "status", "Info clicks: {count}" }
    }
}

#[component]
fn ToggleScenario(variant: ButtonVariant, icon_only: bool) -> Element {
    let mut bold = use_signal(|| false);
    let mut underline = use_signal(|| true);
    let text_style = format!(
        "font-weight:{};text-decoration:{}",
        if bold() { "bold" } else { "normal" },
        if underline() { "underline" } else { "none" }
    );
    rsx! { div { class: "action-scenario-row action-scenario-toggle-row",
        if icon_only {
            IconToggleButton { variant, icon: "text-bold", label: "Bold", pressed: bold(), onpressedchange: move |value| bold.set(value) }
            IconToggleButton { variant, icon: "text-italic", label: "Italic", disabled: true, onpressedchange: |_| {} }
            IconToggleButton { variant, icon: "text-underline", label: "Underline", pressed: underline(), onpressedchange: move |value| underline.set(value) }
            IconToggleButton { variant, icon: "", label: "Strikethrough", disabled: true, loading: true, onpressedchange: |_| {} }
        } else {
            ToggleButton { variant, icon: "text-bold", pressed: bold(), onpressedchange: move |value| bold.set(value), "Bold" }
            ToggleButton { variant, icon: "text-italic", disabled: true, onpressedchange: |_| {}, "Italic" }
            ToggleButton { variant, icon: "text-underline", pressed: underline(), onpressedchange: move |value| underline.set(value), "Underline" }
            ToggleButton { variant, disabled: true, loading: true, onpressedchange: |_| {}, "Strikethrough" }
        }
    }
    p { style: text_style, "Lorem ipsum text" }
    }
}

#[component]
fn ButtonLoadingScenario() -> Element {
    let mut loading = use_signal(|| [false; 4]);
    let begin = use_callback(move |index: usize| {
        loading.write()[index] = true;
        spawn(async move {
            futures_timer::Delay::new(Duration::from_millis(2500)).await;
            loading.write()[index] = false;
        });
    });
    rsx! { div { class: "action-scenario-row",
        Button { variant: ButtonVariant::SubtlePrimary, loading: loading()[0], onclick: move |_| begin.call(0), "Button" }
        Button { variant: ButtonVariant::SubtlePrimary, icon: "star", loading: loading()[1], onclick: move |_| begin.call(1), "Button" }
        Button { variant: ButtonVariant::SubtlePrimary, icon: "star", aria_label: "Star", loading: loading()[2], onclick: move |_| begin.call(2), "" }
        IconButton { variant: ButtonVariant::SubtlePrimary, icon: "star", label: "Toggle loading", loading: loading()[3], onclick: move |_| begin.call(3) }
        Button { variant: ButtonVariant::SubtlePrimary, loading: true, "Button" }
        IconButton { variant: ButtonVariant::SubtlePrimary, loading: true, icon: "star", label: "Loading action" }
    } }
}

#[component]
fn ButtonLinkScenario() -> Element {
    let mut count = use_signal(|| 0);
    let url = "https://siemens.com";
    rsx! { div { class: "action-scenario-links",
        h2 { "Button Link Examples" }
        div { h3 { "Basic Link Buttons" } div { class: "action-scenario-row",
            Button { href: url, "Visit Siemens" }
            Button { href: url, target: "_blank", rel: "noopener noreferrer", "Open in New Tab" }
            Button { href: "/dashboard", "Dashboard" }
        } }
        div { h3 { "Styled Link Buttons" } div { class: "action-scenario-row",
            Button { href: url, "Primary Link" } Button { href: url, variant: ButtonVariant::Secondary, "Secondary Link" } Button { href: url, variant: ButtonVariant::DangerPrimary, "Danger Link" }
        } }
        div { h3 { "Secondary and Tertiary Links" } div { class: "action-scenario-row", Button { href: url, variant: ButtonVariant::Secondary, "Secondary Link" } Button { href: url, variant: ButtonVariant::Tertiary, "Tertiary Link" } } }
        div { h3 { "Icon Links" } div { class: "action-scenario-row", Button { href: url, icon: "home", "Home" } Button { href: url, target: "_blank", icon_right: "open-external", "External" } } }
        div { h3 { "Disabled Link" } Button { href: url, disabled: true, "Disabled Link" } }
        div { h3 { "Comparison" } div { class: "action-scenario-row", Button { onclick: move |_| count += 1, "Regular Button" } Button { href: "#section", "Link Button" } } }
        p { class: "action-scenario-feedback", role: "status", "Regular button clicks: {count}" }
        span { id: "section" }
    } }
}

#[component]
fn SplitScenario(icons: bool) -> Element {
    let mut feedback = use_signal(String::new);
    let items = if icons {
        vec![
            SplitButtonItem {
                value: "cut".into(),
                label: String::new(),
                icon: Some("cut".into()),
                disabled: false,
            },
            SplitButtonItem {
                value: "bulb".into(),
                label: String::new(),
                icon: Some("bulb".into()),
                disabled: false,
            },
        ]
    } else {
        vec![
            SplitButtonItem::new("item-1", "Item 1"),
            SplitButtonItem::new("item-2", "Item 2"),
        ]
    };
    rsx! { div { class: "action-scenario-row",
        if icons { SplitButton { label: "", icon: "document", items, onclick: move |_| feedback.set("Document action".into()), onselect: move |value| feedback.set(value) } }
        else { for mode in 0..4 { SplitButton { label: "Action text", items: items.clone(), disabled: mode == 1, disable_button: mode == 2, disable_dropdown_button: mode == 3,
            onclick: move |_| feedback.set("Action text".into()), onselect: move |value| feedback.set(value) } } }
    } p { class: "action-scenario-feedback", role: "status", "{feedback}" } }
}

#[component]
fn PillVariants() -> Element {
    rsx! { div { class: "action-scenario-pill-variants",
        for (variant, label) in statuses() { div { class: "action-scenario-pill-row",
            for outline in [false, true] { Pill { variant, icon: "info", outline, class: "action-scenario-pill-min", background: ACCENT_BG, color: if outline { "var(--ix-color-primary)" } else { "var(--ix-color-on-primary)" }, "{label}" } }
        } }
    } }
}

#[component]
fn PillScenario() -> Element {
    rsx! { div { class: "action-scenario-pill-wrap",
        Pill { variant: StatusVariant::Custom, background: ACCENT_BG, color: "var(--ix-color-on-primary)", "Label" }
        Pill { tooltip: "Custom tooltip text", "Label" }
        Pill { class: "action-scenario-width-8", "Label" }
        Pill { icon: "star", "Label" }
        Pill { icon: "star", aria_label: "Featured", "" }
        Pill { icon: "star", class: "action-scenario-width-8", "Label" }
        Pill { icon: "star", outline: true, align_left: true, class: "action-scenario-width-8", "Label" }
        Pill { variant: StatusVariant::Alarm, "Label" }
        Pill { variant: StatusVariant::Alarm, class: "action-scenario-width-8", "Label" }
        Pill { variant: StatusVariant::Alarm, icon: "star", "Label" }
        Pill { variant: StatusVariant::Alarm, icon: "star", class: "action-scenario-width-8", "Label" }
        Pill { variant: StatusVariant::Alarm, icon: "star", align_left: true, class: "action-scenario-width-8", "Label" }
        Pill { icon: "star", class: "action-scenario-width-4", "Label" }
        Pill { icon: "star", outline: true, class: "action-scenario-width-4", "Label" }
        Pill { class: "action-scenario-width-3", "Label" }
        Pill { outline: true, class: "action-scenario-width-3", "Label" }
    } }
}

#[component]
fn ChipScenario() -> Element {
    let mut closed = use_signal(BTreeSet::<usize>::new);
    let mut feedback = use_signal(String::new);
    rsx! { div { class: "action-scenario-chips",
        for outline in [false, true] { div { class: "action-scenario-chip-column",
            for (index, (variant, label)) in statuses().into_iter().enumerate() {
                if !closed.read().contains(&(index + usize::from(outline) * 9)) {
                    Chip { variant, icon: "print", outline, closable: matches!(variant, StatusVariant::Primary | StatusVariant::Alarm | StatusVariant::Custom),
                        background: CUSTOM_BG, color: if outline { "var(--ix-color-text)" } else { CUSTOM_FG },
                        tooltip: if variant == StatusVariant::Info { Some("Open details for this status".to_string()) } else { None },
                        aria_label: if variant == StatusVariant::Info { Some("In review".to_string()) } else { None },
                        onclick: move |_| feedback.set(format!("{label} clicked")),
                        onclose: move |_| { closed.write().insert(index + usize::from(outline) * 9); feedback.set(format!("{label} closed")); },
                        "{label}"
                    }
                }
            }
            if !closed.read().contains(&(8 + usize::from(outline) * 9)) {
                Chip { icon: if outline { None } else { Some("print".to_string()) }, outline, closable: true,
                    onclick: move |_| feedback.set("Chip clicked".into()), onclose: move |_| { closed.write().insert(8 + usize::from(outline) * 9); },
                    if outline { "Chip without icon" } else { "Chip with icon" }
                }
            }
        } }
    } p { class: "action-scenario-feedback", role: "status", "{feedback}" } }
}

#[component]
fn BadgeMatrix(kind: BadgeType) -> Element {
    let variants = if kind == BadgeType::StatusIcon {
        vec![
            (StatusVariant::Alarm, "Alarm"),
            (StatusVariant::Error, "Error"),
            (StatusVariant::Critical, "Critical"),
            (StatusVariant::Warning, "Warning"),
            (StatusVariant::Success, "Success"),
            (StatusVariant::Info, "Info"),
        ]
    } else {
        statuses().to_vec()
    };
    rsx! { div { class: "action-scenario-badge-grid",
        for (variant, label) in variants {
            for (outline, pulse) in [(false,false),(true,false),(false,true),(true,true)] {
                Badge { badge_type: kind, variant, outline, enable_animation: pulse,
                    label: match kind { BadgeType::Counter => Some("8".to_string()), BadgeType::Label => Some(label.to_string()), _ => None },
                    icon: if kind == BadgeType::Label { Some("info".to_string()) } else { None },
                    role: if matches!(kind, BadgeType::Dot | BadgeType::StatusIcon) { Some("img".to_string()) } else { None },
                    aria_label: if matches!(kind, BadgeType::Dot | BadgeType::StatusIcon) { Some(format!("{label}{}{}", if pulse { " pulse" } else { "" }, if outline { " outline" } else { "" })) } else { None },
                    background: CUSTOM_BG, color: if outline { "var(--ix-color-text)" } else { CUSTOM_FG },
                }
            }
        }
    } }
}

#[component]
fn BadgeScenario() -> Element {
    let mut clicked = use_signal(String::new);
    rsx! {
        div { class: "action-scenario-badge-row",
            Badge { label: "3", variant: StatusVariant::Alarm, Button { aria_label: "Review, 3 unread", onclick: move |_| clicked.set("Review".into()), "Review" } }
            Badge { label: "1", variant: StatusVariant::Alarm, enable_animation: true, IconButton { icon: "info", label: "What's new, 1 notification", onclick: move |_| clicked.set("What's new".into()) } }
            Badge { label: "9", variant: StatusVariant::Success, position: BadgePosition::BottomAfter, Button { aria_label: "Bottom after, 9 items", onclick: move |_| clicked.set("Bottom after".into()), "Bottom after" } }
            Badge { badge_type: BadgeType::Dot, variant: StatusVariant::Alarm, Button { aria_label: "Messages, unread", onclick: move |_| clicked.set("Messages".into()), "Messages" } }
            Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Warning, Button { aria_label: "Device, warning", onclick: move |_| clicked.set("Device".into()), "Device" } }
        }
        div { class: "action-scenario-badge-row",
            Badge { label: "12", variant: StatusVariant::Info } Badge { label: "99", variant: StatusVariant::Warning }
            Badge { label: "142" } Badge { label: "100", variant: StatusVariant::Alarm }
            Badge { label: "8", outline: true } Badge { label: "8", border: true }
            Badge { badge_type: BadgeType::Dot, variant: StatusVariant::Alarm, role: "img", aria_label: "Unread" }
            Badge { badge_type: BadgeType::Dot, variant: StatusVariant::Success, outline: true, role: "img", aria_label: "Online" }
            Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Alarm, role: "img", aria_label: "Alarm" }
            Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Success, outline: true, role: "img", aria_label: "Success" }
        }
        div { class: "action-scenario-badge-row",
            Badge { badge_type: BadgeType::Label, label: "Label", variant: StatusVariant::Alarm, align_left: true, icon: "star", class: "action-scenario-badge-width-7", tooltip: "Label" }
            Badge { badge_type: BadgeType::Label, label: "Label", icon: "star", class: "action-scenario-badge-width-4", tooltip: "Label" }
            Badge { label: "12", variant: StatusVariant::Info, tooltip: "12" } Badge { label: "12", variant: StatusVariant::Info, tooltip: "Twelve notifications" }
            Badge { badge_type: BadgeType::Dot, variant: StatusVariant::Alarm, role: "img", aria_label: "Unread", tooltip: "Unread" }
            Badge { badge_type: BadgeType::Dot, variant: StatusVariant::Alarm, role: "img", aria_label: "Unread", tooltip: "Custom tooltip text" }
            Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Warning, role: "img", aria_label: "Warning", tooltip: "Warning" }
            Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Warning, role: "img", aria_label: "Warning", tooltip: "Custom tooltip text" }
        }
        p { class: "action-scenario-feedback", role: "status", "{clicked}" }
    }
}

#[component]
fn ProgressScenario(name: String) -> Element {
    let kind = if name.contains("circular") {
        ProgressType::Circular
    } else {
        ProgressType::Linear
    };
    let examples = if name.ends_with("-sizes") {
        [
            (ProgressSize::Xs, "XS"),
            (ProgressSize::Sm, "SM"),
            (ProgressSize::Md, "MD"),
            (ProgressSize::Lg, "LG"),
            (ProgressSize::Xl, "XL"),
        ]
        .into_iter()
        .map(|(size, label)| {
            (
                size,
                ProgressStatus::Default,
                Some(format!("Size - {label}")),
            )
        })
        .collect::<Vec<_>>()
    } else if name.ends_with("-status") {
        [
            (ProgressStatus::Default, "Default"),
            (ProgressStatus::Success, "Success"),
            (ProgressStatus::Error, "Error"),
            (ProgressStatus::Info, "Info"),
            (ProgressStatus::Warning, "Warning"),
            (ProgressStatus::Paused, "Paused"),
        ]
        .into_iter()
        .map(|(status, label)| (ProgressSize::Md, status, Some(format!("Status - {label}"))))
        .collect()
    } else {
        vec![(ProgressSize::Md, ProgressStatus::Default, None)]
    };
    rsx! { div { class: "action-scenario-progress",
        for (size, status, heading) in examples {
            div { if let Some(heading) = heading { h2 { class: "action-scenario-progress-header", "{heading}" } }
                ProgressIndicator { progress_type: kind, label: "Download", value: 75.0, helper_text: "This is a help text for the progress indicator", size, status, "75%" }
            }
        }
    } }
}

#[component]
fn LoadingScenario() -> Element {
    let modal = use_modal();
    let mut count = use_signal(|| 0);
    let mut running = use_signal(|| false);
    let mut done = use_signal(|| false);
    rsx! {
        div { onclick: move |_| { if !running() {
            running.set(true); done.set(false); count.set(0);
            spawn(async move {
                futures_timer::Delay::new(Duration::from_secs(1)).await; count.set(1);
                futures_timer::Delay::new(Duration::from_secs(1)).await; count.set(2);
                done.set(true); futures_timer::Delay::new(Duration::from_millis(250)).await; modal.close("Done"); running.set(false);
            });
        } }, ModalTrigger { modal, disabled: running(), "Start loading" } }
        Modal { modal, label: "Loading", size: ModalSize::Xs,
            ondismissrequest: move |request: ModalDismissRequest| { request.deny(); },
            ModalLoading { if done() { "Done" } else { "Loading {count}/2" } }
        }
        if done() { p { class: "action-scenario-feedback", role: "status", "Done" } }
    }
}

pub(crate) const STYLES: &str = r#"
.ix-action-scenario{min-width:0;color:var(--ix-color-text)}.action-scenario-row{display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin:4px 0}.action-scenario-row>.ix-button{margin:0}.action-scenario-feedback{font-size:13px;color:var(--ix-color-text-secondary);margin:16px 0 0;min-height:20px}.action-scenario-button-group{display:inline-flex}.action-scenario-button-group>.ix-button{border-radius:0;margin-left:-1px}.action-scenario-button-group>.ix-button:first-child{border-radius:var(--ix-radius) 0 0 var(--ix-radius);margin-left:0}.action-scenario-button-group>.ix-button:last-child{border-radius:0 var(--ix-radius) var(--ix-radius) 0}.action-scenario-toggle-row{margin-bottom:16px}.action-scenario-links{padding:32px;max-width:800px}.action-scenario-links>div{margin-bottom:16px}.action-scenario-links h2{font-size:24px}.action-scenario-links h3{font-size:18px;margin-bottom:12px}.action-scenario-pill-variants{display:flex;flex-direction:column;gap:16px}.action-scenario-pill-row{display:grid;grid-template-columns:repeat(2,minmax(112px,1fr));gap:16px;max-width:720px}.action-scenario-pill-min{min-width:112px}.action-scenario-pill-wrap{display:flex;flex-wrap:wrap;align-items:center;gap:8px}.action-scenario-width-8{width:8rem}.action-scenario-width-4{width:4rem}.action-scenario-width-3{width:3rem}.action-scenario-chips{display:flex;gap:32px;flex-wrap:wrap}.action-scenario-chip-column{display:flex;flex-direction:column;align-items:flex-start;gap:8px}.action-scenario-badge-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));column-gap:32px;row-gap:24px;align-items:center;padding-top:16px}.action-scenario-badge-row{display:flex;flex-wrap:wrap;align-items:center;gap:32px;margin-bottom:24px;padding-top:24px}.action-scenario-badge-width-7{width:7rem;--ix-badge-max-width:7rem}.action-scenario-badge-width-4{width:4rem;--ix-badge-max-width:4rem}.action-scenario-progress{display:flex;flex-direction:column;gap:16px;max-width:560px}.action-scenario-progress-header{font-size:24px;line-height:32px;margin:.7rem 0}@media(max-width:480px){.action-scenario-links{padding:8px}.action-scenario-badge-grid{column-gap:16px}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_lists_distinct_scenarios() {
        let names: BTreeSet<_> = SCENARIOS.iter().copied().collect();
        assert_eq!(names.len(), 50);
        assert_eq!(names.len(), SCENARIOS.len());
        for name in SCENARIOS {
            assert!(
                UPSTREAM_EXAMPLE_NAMES.contains(name),
                "Unknown source scenario: {name}"
            );
        }
    }
    #[test]
    fn every_registered_action_scenario_renders_its_selected_preview() {
        for name in SCENARIOS {
            let mut dom = VirtualDom::new_with_props(
                ActionScenario,
                ActionScenarioProps {
                    name: (*name).to_string(),
                },
            );
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(
                html.contains(&format!("data-action-scenario=\"{name}\"")),
                "{name}"
            );
            assert!(
                !html.contains("This action scenario is not registered"),
                "{name}"
            );
        }
    }
}
