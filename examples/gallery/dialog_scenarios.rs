//! Isolated dialog previews adapted from the upstream HTML examples (MIT).
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

pub const SCENARIOS: &[&str] = &[
    "modal",
    "modal-close",
    "modal-form-ix-button-submit",
    "modal-non-blocking",
    "modal-sizes",
    "popover",
    "popover-news",
];

#[component]
pub fn DialogScenario(name: String) -> Element {
    let preview = match name.as_str() {
        "modal" => rsx! { MessageModal {} },
        "modal-close" => rsx! { MessageModal { auto_close: true } },
        "modal-form-ix-button-submit" => rsx! { ResourceFormModal {} },
        "modal-non-blocking" => rsx! { NonBlockingModal {} },
        "modal-sizes" => rsx! { ModalSizes {} },
        "popover" => rsx! { ReleasePopover {} },
        "popover-news" => rsx! { AboutNews {} },
        _ => rsx! { p { "This dialog scenario is not registered." } },
    };
    rsx! { div { class: "ix-dialog-scenario", "data-dialog-scenario": name, {preview} } }
}

fn result_text(result: ModalResult) -> String {
    match result {
        ModalResult::Closed(value) if value.is_empty() => "Modal closed".into(),
        ModalResult::Closed(value) => value,
        ModalResult::Dismissed(ModalDismissReason::Programmatic(payload)) => payload,
        ModalResult::Dismissed(_) => "Modal dismissed".into(),
        ModalResult::NativeClosed => "Modal closed".into(),
    }
}

#[component]
fn MessageModal(#[props(default)] auto_close: bool) -> Element {
    let modal = use_modal();
    // An upstream opening creates a fresh instance. Keep the same guarantee for
    // delayed work even though this Rust controller is intentionally reusable.
    let mut opening = use_signal(|| 0_u64);
    let mut feedback = use_signal(String::new);
    rsx! {
        ModalTrigger { modal, if auto_close { "Show modal (auto-dismiss)" } else { "Show modal" } }
        Modal { modal, label: "Message headline",
            onresult: move |result| feedback.set(result_text(result)),
            onopenchange: move |open| if open {
                let this_opening = opening.peek().wrapping_add(1);
                opening.set(this_opening);
                feedback.set(String::new());
                if auto_close { spawn(async move {
                    futures_timer::Delay::new(Duration::from_secs(5)).await;
                    if *opening.peek() == this_opening && modal.is_open() {
                        modal.close("closed after 5 seconds");
                    }
                }); }
            },
            ModalHeader { "Message headline" }
            ModalContent { "Message text lorem ipsum" }
            ModalFooter {
                ModalClose { dismiss: true, variant: if auto_close { ButtonVariant::Secondary } else { ButtonVariant::SubtlePrimary }, "Cancel" }
                ModalClose { variant: ButtonVariant::Primary, "OK" }
            }
        }
        p { class: "dialog-scenario-feedback", role: "status", "{feedback}" }
    }
}

#[component]
fn ResourceFormModal() -> Element {
    let modal = use_modal();
    let mut name = use_signal(String::new);
    let mut feedback = use_signal(String::new);
    rsx! {
        ModalTrigger { modal, "Show modal" }
        Modal { modal, label: "Create Resource",
            onopenchange: move |open| if open { name.set(String::new()); feedback.set(String::new()); },
            ondialogdismiss: move |_| feedback.set("Modal dismissed".into()),
            ModalHeader { "Create Resource" }
            ModalContent {
                form { id: "create-resource-form",
                    onsubmit: move |event| {
                        event.prevent_default();
                        feedback.set(format!("Submitted name: {}", name()));
                        modal.close("");
                    },
                    Input { id: "name", name: "name", label: "Name", value: name(), onvaluechange: move |value| name.set(value) }
                }
            }
            ModalFooter {
                ModalClose { dismiss: true, variant: ButtonVariant::SubtlePrimary, "Cancel" }
                Button { button_type: ButtonType::Submit, form: "create-resource-form", "Submit" }
            }
        }
        p { class: "dialog-scenario-feedback", role: "status", "{feedback}" }
    }
}

#[component]
fn NonBlockingModal() -> Element {
    let modal = use_modal();
    let mut behind = use_signal(|| 0);
    let mut feedback = use_signal(String::new);
    rsx! {
        div { div { class: "dialog-scenario-behind",
            Typography { "Content behind the dialog" }
            Button { onclick: move |_| behind += 1, "Behind control" }
        }
            ModalTrigger { modal, "Show non-blocking modal" }
        }
        Modal { modal, label: "Message headline", is_non_blocking: true,
            onresult: move |result| feedback.set(result_text(result)),
            ModalHeader { oncloseclick: move |event: MouseEvent| { event.prevent_default(); modal.dismiss(ModalDismissReason::Programmatic("dismiss payload".into())); }, "Message headline" }
            ModalContent { "Message text lorem ipsum" }
            ModalFooter {
                Button { variant: ButtonVariant::SubtlePrimary, onclick: move |_| modal.dismiss(ModalDismissReason::Programmatic("dismiss payload".into())), "Cancel" }
                ModalClose { variant: ButtonVariant::Primary, value: "close payload!", autofocus: true, "OK" }
            }
        }
        p { class: "dialog-scenario-feedback", role: "status", "Behind control clicks: {behind}" }
        p { class: "dialog-scenario-feedback", role: "status", "{feedback}" }
    }
}

#[component]
fn ModalSizes() -> Element {
    rsx! { div { class: "dialog-scenario-sizes",
        for size in [ModalSize::Xs, ModalSize::Sm, ModalSize::Md, ModalSize::Lg, ModalSize::Xl, ModalSize::FullWidth, ModalSize::FullScreen] {
            SizeModal { key: "{size.as_str()}", size }
        }
    } }
}

#[component]
fn SizeModal(size: ModalSize) -> Element {
    let modal = use_modal();
    rsx! {
        ModalTrigger { modal, "Show modal size {size.as_str()}" }
        Modal { modal, size, label: "Modal with size {size.as_str()}",
            div { class: "dialog-scenario-size-content", ModalClose { dismiss: true, variant: ButtonVariant::Primary, "Modal with size {size.as_str()}" } }
        }
    }
}

const RELEASE_IMAGE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='160'%3E%3Crect fill='%232a2a4a' width='100%25' height='100%25'/%3E%3Ctext x='50%25' y='50%25' fill='%23e0e0e0' text-anchor='middle' dy='.3em' font-size='18'%3ERelease%20preview%3C/text%3E%3C/svg%3E";

#[component]
fn ReleasePopover() -> Element {
    let popover = use_popover();
    let mut saves = use_signal(|| 0);
    rsx! {
        PopoverTrigger { popover, "Open" }
        Popover { popover, label: "Release highlights", placement: PopoverPlacement::Bottom, has_spike: true, close_on_click_outside: true,
            PopoverHeader { icon: "info", icon_color: "--ix-color-info-text", close_label: "Close popover",
                additional_items: rsx! { Pill { variant: StatusVariant::Info, "New" } },
                "Release highlights"
            }
            PopoverImage { image: RELEASE_IMAGE, image_alt: "Release preview" }
            PopoverContent { "Check out the new dashboard and improved performance metrics." }
            PopoverFooter { alignment: PopoverFooterAlignment::Vertical, start: rsx! { span { "v4.0.0" } },
                Button { variant: ButtonVariant::Secondary, onclick: move |_| popover.hide(), "Cancel" }
                Button { onclick: move |_| saves += 1, "Save" }
            }
        }
        if saves() > 0 { p { class: "dialog-scenario-feedback", role: "status", "Save clicks: {saves}" } }
    }
}

#[component]
fn AboutNews() -> Element {
    let mut show = use_signal(|| true);
    let expanded = use_signal(|| false);
    let mut active = use_signal(|| None::<MenuOverlayKind>);
    let mut tab = use_signal(|| "tab-1".to_string());
    rsx! {
        Application { class: "dialog-scenario-news-app", menu_expanded: expanded,
            header: rsx! { ApplicationHeader { name: "" } },
            menu: rsx! { Menu { active_overlay: active,
                about: rsx! { MenuAbout { TabSet { items: vec![TabItem::new("tab-1", "Example")], active_key: tab(), onselectedchange: move |key| tab.set(key),
                    TabPanel { tab_key: "tab-1", {VNode::empty()} }
                } } },
                {VNode::empty()}
            } },
            MenuAboutNews { label: "Test", show: show(), expanded: expanded(), active_about_tab_key: "tab-1",
                onclose: move |_| show.set(false),
                onshowmore: move |key: Option<String>| { if let Some(key) = key { tab.set(key); active.set(Some(MenuOverlayKind::About)); } show.set(false); },
                "Test"
            }
        }
    }
}

/// Review-only controller interactions, kept separate from the upstream fixtures.
#[component]
pub fn DialogControllerProbe() -> Element {
    let modal = use_modal();
    let popover = use_popover();
    let hover = use_popover();
    let mut status = use_signal(|| "No controller operation yet".to_string());
    let mut protect_popover = use_signal(|| false);
    let mut actions = use_signal(|| 0_u32);
    rsx! {
        section { class: "demo-section", id: "native-dialog-controllers",
            h2 { "Rust dialog controllers" }
            p { "Open native top-layer surfaces from Rust, test contained keyboard focus, and compare hover activation with the normal click trigger." }
            div { class: "demo-row",
                Button { onclick: move |_| async move {
                    status.set(match modal.open().await { Ok(()) => "Modal opened from Rust".into(), Err(error) => error.to_string() });
                }, "Open modal from Rust" }
                Button { onclick: move |_| async move {
                    status.set(match popover.show().await { Ok(()) => "Popover shown from Rust".into(), Err(error) => error.to_string() });
                }, "Show popover from Rust" }
                PopoverTrigger { popover: hover, trigger_mode: PopoverTriggerMode::Hover, "Hover or focus for details" }
            }
            Modal { modal, label: "Controller modal", close_on_backdrop_click: true,
                onresult: move |result| status.set(format!("Controller result: {result:?}")),
                ModalHeader { "Controller modal" }
                ModalContent { "This is a native modal: background controls are inert while it is open." }
                ModalFooter { ModalClose { value: "controller-confirmed", "Confirm controller modal" } }
            }
            Popover { popover, label: "Controller popover", close_on_click_outside: true,
                onshowchange: move |event: ToggleEvent| if popover.is_open() && protect_popover() { event.prevent_default(); },
                onshowchanged: move |open| status.set(format!("Controller popover visible: {open}")),
                PopoverHeader { close_label: "Close controller popover", "Controller popover" }
                PopoverContent {
                    Checkbox { label: "Keep popover open", checked: protect_popover(), oncheckedchange: move |checked| protect_popover.set(checked) }
                    p { "When protected, Escape, its trigger, header close and outside click request dismissal without hiding this panel." }
                }
                PopoverFooter {
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| actions += 1, "First controller action" }
                    Button { onclick: move |_| actions += 1, "Last controller action" }
                }
            }
            Popover { popover: hover, label: "Hover details", has_spike: true,
                PopoverHeader { close_label: "Close hover details", "Hover details" }
                PopoverContent { "Move between this panel and its trigger. Leaving both dismisses after 150 milliseconds; Escape returns focus without reopening it." }
                PopoverFooter { Button { onclick: move |_| hover.hide(), "Dismiss hover details" } }
            }
            p { role: "status", "{status}" }
            p { role: "status", "Controller actions: {actions}" }
        }
    }
}

pub const STYLES: &str = include_str!("styles/dialog_scenarios.css");

#[cfg(test)]
mod tests {
    use super::*;

    fn render(name: &str) -> String {
        let mut dom =
            VirtualDom::new_with_props(DialogScenario, DialogScenarioProps { name: name.into() });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn registered_previews_are_distinct_known_source_cases_and_render() {
        let names: std::collections::BTreeSet<_> = SCENARIOS.iter().copied().collect();
        assert_eq!(names.len(), 7);
        assert_eq!(names.len(), SCENARIOS.len());
        for name in SCENARIOS {
            assert!(UPSTREAM_EXAMPLE_NAMES.contains(name));
            let html = render(name);
            assert!(
                html.contains(&format!("data-dialog-scenario=\"{name}\"")),
                "{name}"
            );
            assert!(!html.contains("This dialog scenario is not registered"));
        }
    }

    #[test]
    fn external_submit_button_targets_a_real_named_native_input() {
        let html = render("modal-form-ix-button-submit");
        assert!(html.contains("id=\"create-resource-form\""));
        assert!(html.contains("form=\"create-resource-form\""));
        assert!(html.contains("type=\"submit\""));
        assert!(html.contains("name=\"name\""));
        assert!(html.contains("for=\"name\""));
        assert!(
            !html.contains("required=\"true\""),
            "source permits an empty Name"
        );
    }

    #[test]
    fn source_size_matrix_and_non_blocking_semantics_are_preserved() {
        let html = render("modal-sizes");
        assert_eq!(html.matches("<dialog").count(), 7);
        for size in [
            "360",
            "480",
            "600",
            "720",
            "840",
            "full-width",
            "full-screen",
        ] {
            assert!(html.contains(&format!("Show modal size {size}")));
            assert!(html.contains(&format!("ix-modal-size-{size}")));
        }
        let html = render("modal-non-blocking");
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains("popover=\"manual\""));
        assert!(html.contains("Behind control"));
        assert!(html.contains("autofocus"));
    }

    #[test]
    fn modal_results_keep_application_payloads() {
        assert_eq!(
            result_text(ModalResult::Closed("closed after 5 seconds".into())),
            "closed after 5 seconds"
        );
        assert_eq!(
            result_text(ModalResult::Dismissed(ModalDismissReason::Programmatic(
                "dismiss payload".into()
            ))),
            "dismiss payload"
        );
        assert_eq!(
            result_text(ModalResult::Closed("close payload!".into())),
            "close payload!"
        );
    }

    #[test]
    fn controller_probe_is_separate_from_upstream_cases() {
        let mut dom = VirtualDom::new(DialogControllerProbe);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Open modal from Rust"));
        assert!(html.contains("Show popover from Rust"));
        assert!(html.contains("Hover or focus for details"));
        assert!(html.contains("Keep popover open"));
        assert!(!html.contains("<script"));
        assert_eq!(SCENARIOS.len(), 7);
    }
}
