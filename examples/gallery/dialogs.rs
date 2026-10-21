//! Review scenarios based on the upstream modal and popover examples.
use dioxus::prelude::*;
use dioxus_ix::*;

use crate::svg_data_uri;

#[component]
pub fn DialogExamples() -> Element {
    rsx! {
        section { class: "demo-section", id: "modal",
            h2 { "Modal" }
            p { "Native modal dialogs opened through HTML invoker commands. Background controls become inert and browser focus stays inside the dialog." }
            BasicModalExample {}
        }
        section { class: "demo-section", id: "modal-sizes",
            h2 { "Modal sizes" }
            div { class: "demo-row", for size in [ModalSize::Xs, ModalSize::Sm, ModalSize::Md, ModalSize::Lg, ModalSize::Xl, ModalSize::FullWidth, ModalSize::FullScreen] { ModalSizeExample { size } } }
        }
        crate::dialog_scenarios::DialogControllerProbe {}
        section { class: "demo-section", id: "modal-close",
            h2 { "Modal close / auto-dismiss" }
            AutoDismissExample {}
        }
        section { class: "demo-section", id: "modal-form-ix-button-submit",
            h2 { "Modal form / external submit button" }
            ResourceModalExample {}
        }
        section { class: "demo-section", id: "modal-non-blocking",
            h2 { "Non-blocking modal" }
            NonBlockingExample {}
        }
        section { class: "demo-section", id: "modal-dismiss-guard",
            h2 { "Deferred dismissal" }
            p { "Request dismissal with Cancel, the close icon, Escape or the backdrop, then allow or reject the pending decision. A retained request can also be resolved after an asynchronous application operation." }
            GuardedModalExample {}
        }
        section { class: "demo-section", id: "modal-loading",
            h2 { "Loading modal" }
            LoadingModalExample {}
        }
        section { class: "demo-section", id: "popover",
            h2 { "Popover" }
            p { "The release-highlights example with header icon, image, metadata and vertically arranged actions." }
            ReleasePopoverExample {}
        }
        section { class: "demo-section", id: "popover-placements",
            h2 { "Popover placement" }
            p { "CSS anchor positioning follows the trigger and flips placement near viewport edges." }
            div { class: "dialog-placement-review", for placement in [PopoverPlacement::Top, PopoverPlacement::Bottom, PopoverPlacement::Left, PopoverPlacement::Right] { PlacementExample { placement } } }
        }
    }
}

#[component]
fn BasicModalExample() -> Element {
    let modal = use_modal();
    let mut outcome = use_signal(|| "No result yet".to_string());
    rsx! {
        ModalTrigger { modal, "Show modal" }
        Modal { modal, label: "Message headline", onresult: move |result| outcome.set(format!("{result:?}")),
            ModalHeader { "Message headline" }
            ModalContent { "Message text lorem ipsum" }
            ModalFooter {
                ModalClose { dismiss: true, variant: ButtonVariant::SubtlePrimary, "Cancel" }
                ModalClose { value: "close payload!", variant: ButtonVariant::Primary, autofocus: true, "OK" }
            }
        }
        p { role: "status", "{outcome}" }
    }
}

#[component]
fn ModalSizeExample(size: ModalSize) -> Element {
    let modal = use_modal();
    rsx! {
        ModalTrigger { modal, variant: ButtonVariant::Secondary, "Show modal size {size.as_str()}" }
        Modal { modal, label: format!("Modal with size {}", size.as_str()), size,
            ModalHeader { "Modal with size {size.as_str()}" }
            ModalContent { "Message text lorem ipsum" }
            ModalFooter { ModalClose { variant: ButtonVariant::Primary, "Close" } }
        }
    }
}

#[component]
fn AutoDismissExample() -> Element {
    let modal = use_modal();
    let mut outcome =
        use_signal(|| "Open the dialog to start its five-second countdown.".to_string());
    rsx! {
        ModalTrigger { modal, "Show modal (auto-dismiss)" }
        Modal { modal, label: "Message headline", onresult: move |result| outcome.set(format!("{result:?}")),
            ModalHeader { "Message headline" }
            ModalContent {
                p { "Message text lorem ipsum" }
                p { "This example dismisses five seconds after opening." }
                div { class: "dialog-countdown", "aria-hidden": "true",
                    div { class: "dialog-countdown-fill", onanimationend: move |_| modal.dismiss(ModalDismissReason::Programmatic("auto-dismiss after 5 seconds".into())) }
                }
            }
            ModalFooter { ModalClose { dismiss: true, "Cancel" } ModalClose { value: "close payload!", variant: ButtonVariant::Primary, "OK" } }
        }
        p { role: "status", "{outcome}" }
    }
}

#[component]
fn ResourceModalExample() -> Element {
    let modal = use_modal();
    let mut resource_name = use_signal(String::new);
    let mut created = use_signal(String::new);
    let form_id = format!("{}-form", modal.id());
    rsx! {
        ModalTrigger { modal, "Show modal" }
        Modal { modal, label: "Create Resource", ondialogclose: move |name| created.set(name),
            ModalHeader { "Create Resource" }
            ModalContent {
                form { id: form_id.clone(), onsubmit: move |event| { event.prevent_default(); modal.close(resource_name()); },
                    Input { label: "Name", name: "name", value: resource_name(), required: true,
                        helper_text: "Enter a resource name", onvaluechange: move |value| resource_name.set(value) }
                }
            }
            ModalFooter {
                ModalClose { dismiss: true, variant: ButtonVariant::SubtlePrimary, "Cancel" }
                Button { form: form_id, button_type: ButtonType::Submit, "Submit" }
            }
        }
        p { role: "status", "Created resource: {created}" }
    }
}

#[component]
fn NonBlockingExample() -> Element {
    let modal = use_modal();
    let mut clicks = use_signal(|| 0usize);
    rsx! {
        div { class: "demo-row",
            ModalTrigger { modal, "Show non-blocking modal" }
            Button { variant: ButtonVariant::Secondary, onclick: move |_| clicks += 1, "Behind control" }
        }
        p { role: "status", "Behind control clicks: {clicks}" }
        Modal { modal, label: "Message headline", is_non_blocking: true,
            ModalHeader { "Message headline" }
            ModalContent { "Message text lorem ipsum" }
            ModalFooter { ModalClose { dismiss: true, variant: ButtonVariant::SubtlePrimary, "Cancel" } ModalClose { value: "close payload!", variant: ButtonVariant::Primary, autofocus: true, "OK" } }
        }
    }
}

#[component]
fn GuardedModalExample() -> Element {
    let modal = use_modal();
    let mut pending = use_signal(|| None::<ModalDismissRequest>);
    let mut outcome = use_signal(|| "No dismissal decision requested".to_string());
    rsx! {
        ModalTrigger { modal, "Show guarded modal" }
        Modal { modal, label: "Unsaved changes", close_on_backdrop_click: true,
            ondismissrequest: move |request: ModalDismissRequest| pending.set(Some(request)),
            onresult: move |result| { pending.set(None); outcome.set(format!("{result:?}")); },
            ModalHeader { icon: "warning", "Unsaved changes" }
            ModalContent {
                p { "Closing this modal asks for a decision before the native dialog is removed." }
                if let Some(request) = pending() {
                    div { class: "dialog-dismiss-decision", role: "status",
                        p { "Dismissal requested: {request.reason:?}" }
                        div { class: "demo-row",
                            Button { variant: ButtonVariant::Secondary, onclick: { let request = request.clone(); move |_| { request.clone().deny(); pending.set(None); } }, "Keep editing" }
                            Button { variant: ButtonVariant::DangerPrimary, onclick: move |_| { request.clone().allow(); pending.set(None); }, "Discard changes" }
                        }
                    }
                }
            }
            ModalFooter { ModalClose { dismiss: true, "Cancel" } ModalClose { value: "saved", variant: ButtonVariant::Primary, "Save" } }
        }
        p { role: "status", "{outcome}" }
    }
}

#[component]
pub fn LoadingModalExample() -> Element {
    let modal = use_modal();
    let mut reading = use_signal(|| false);
    let mut read_task = use_signal(|| None::<dioxus::core::Task>);
    let mut operation = use_signal(|| 0_u64);
    let mut file_name = use_signal(String::new);
    let mut characters = use_signal(|| 0_usize);
    let mut state = use_signal(|| "idle");
    let mut open = use_signal(|| false);
    let mut outcome = use_signal(String::new);
    let mut preparation = use_signal(|| String::from("Check the text file before reading"));
    let mut result =
        use_signal(|| "Select a local text file to read it asynchronously.".to_string());
    let cancel_read = use_callback(move |(): ()| {
        if !*reading.peek() {
            return;
        }
        let next = operation.peek().wrapping_add(1);
        operation.set(next);
        let pending = read_task.write().take();
        if let Some(task) = pending {
            task.cancel();
        }
        reading.set(false);
        state.set("cancelled");
        result.set(format!(
            "Cancelled reading {}. Choose a file to try again.",
            file_name.peek()
        ));
        modal.close("cancelled");
    });
    rsx! {
        div { id: "loading-import",
            p { "Read a local text file and keep its result here. Nothing is uploaded or sent to a server. You can cancel a pending read from its status dialog." }
            Input { id: "loading-import-preparation", label: "Read preparation", value: preparation(), max_length: 160,
                onvaluechange: move |value| preparation.set(value),
            }
            div { id: "loading-import-upload",
                Upload { accept: "text/*,.txt,.md,.rs", select_file_text: "Select a local text file", button_text: "Choose text file",
                    disabled: reading(),
                    onfileschanged: move |files: Vec<dioxus::html::FileData>| {
                        if *reading.peek() { return; }
                        let Some(file) = files.into_iter().next() else { return; };
                        let sequence = operation.peek().wrapping_add(1);
                        operation.set(sequence);
                        file_name.set(file.name());
                        characters.set(0);
                        reading.set(true);
                        state.set("opening");
                        let task = spawn(async move {
                            if let Err(error) = modal.open().await {
                                if *operation.peek() != sequence { return; }
                                state.set("unavailable");
                                result.set(format!("Cannot open the read status dialog: {error}. The file was not read."));
                                reading.set(false);
                                read_task.set(None);
                                return;
                            }
                            if *operation.peek() != sequence { return; }
                            state.set("reading");
                            let content = file.read_string().await;
                            if *operation.peek() != sequence { return; }
                            match content {
                                Ok(text) => {
                                    let count = text.chars().count();
                                    characters.set(count);
                                    state.set("success");
                                    result.set(format!("Read {}: {count} characters", file.name()));
                                }
                                Err(error) => {
                                    state.set("failed");
                                    result.set(format!("Could not read {}: {error}. Choose a file to try again.", file.name()));
                                }
                            }
                            reading.set(false);
                            read_task.set(None);
                            modal.close(state());
                        });
                        read_task.set(Some(task));
                    }
                }
            }
            p { id: "loading-import-status", role: "status", "aria-live": "polite",
                "data-state": state(), "data-file": file_name(), "data-characters": characters(),
                "data-reading": reading(), "data-open": open(), "data-result": outcome(),
                "{result}"
            }
            Button { id: "loading-import-reset", button_type: ButtonType::Button, variant: ButtonVariant::Tertiary, disabled: reading(),
                onclick: move |_| {
                    if *reading.peek() { return; }
                    let next = operation.peek().wrapping_add(1);
                    operation.set(next);
                    file_name.set(String::new()); characters.set(0); state.set("idle"); outcome.set(String::new());
                    result.set("Select a local text file to read it asynchronously.".into());
                },
                "Clear read result"
            }
            LoadingDialog { modal, label: "Read local text file", size: ModalSize::Sm,
                message: format!("Reading {}…", file_name()),
                onopenchange: move |visible: bool| open.set(visible),
                onresult: move |completed: ModalResult| {
                    outcome.set(format!("{completed:?}"));
                    if *reading.peek() {
                        let next = operation.peek().wrapping_add(1);
                        operation.set(next);
                        let pending = read_task.write().take();
                        if let Some(task) = pending { task.cancel(); }
                        reading.set(false); state.set("cancelled");
                        result.set("The read ended before completion. Choose a file to try again.".into());
                    }
                },
                Button { id: "loading-import-cancel", button_type: ButtonType::Button, variant: ButtonVariant::Secondary,
                    disabled: !reading() || !modal.is_open(), onclick: move |_| cancel_read.call(()), "Cancel read"
                }
            }
        }
    }
}

static RELEASE_IMAGE: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    svg_data_uri::svg_data_uri(include_str!("assets/release-preview.svg"))
});

#[component]
fn ReleasePopoverExample() -> Element {
    let popover = use_popover();
    let mut outcome = use_signal(|| "No action yet".to_string());
    rsx! {
        PopoverTrigger { popover, "Open" }
        Popover { popover, label: "Release highlights", placement: PopoverPlacement::Bottom, has_spike: true, close_on_click_outside: true,
            PopoverHeader { icon: "info", icon_color: "--ix-color-info", close_label: "Close popover",
                additional_items: rsx! { Pill { variant: StatusVariant::Info, "New" } }, "Release highlights" }
            PopoverImage { image: RELEASE_IMAGE.as_str(), image_alt: "Release preview" }
            PopoverContent { "Check out the new dashboard and improved performance metrics." }
            PopoverFooter { alignment: PopoverFooterAlignment::Vertical, start: rsx! { span { "v4.0.0" } },
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { outcome.set("Cancelled".into()); popover.hide(); }, "Cancel" }
                Button { onclick: move |_| { outcome.set("Saved".into()); popover.hide(); }, "Save" }
            }
        }
        p { role: "status", "{outcome}" }
    }
}

#[component]
fn PlacementExample(placement: PopoverPlacement) -> Element {
    let popover = use_popover();
    rsx! {
        PopoverTrigger { popover, variant: ButtonVariant::Secondary, "{placement.as_str()}" }
        Popover { popover, label: format!("{} placement", placement.as_str()), placement, has_spike: true, close_on_click_outside: true,
            PopoverHeader { "{placement.as_str()}" }
            PopoverContent { "This panel follows its trigger. The browser may flip it to remain visible." }
        }
    }
}

pub(crate) const STYLES: &str = include_str!("styles/dialogs.css");
