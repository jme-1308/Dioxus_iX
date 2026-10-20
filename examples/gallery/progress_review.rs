//! Shared, development-only progress example. All interaction state stays in Rust.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn ProgressReview() -> Element {
    let mut progress = use_signal(|| 35_u32);
    let mut paused = use_signal(|| false);
    let mut failed = use_signal(|| false);
    let complete = progress() == 100;
    let status = if failed() {
        ProgressStatus::Error
    } else if complete {
        ProgressStatus::Success
    } else if paused() {
        ProgressStatus::Paused
    } else {
        ProgressStatus::Default
    };
    let message = if failed() {
        "Connection lost. Retry to continue from the saved progress."
    } else if complete {
        "Transfer complete. Reset to try again."
    } else if paused() {
        "Transfer paused. Resume to continue."
    } else {
        "Ready for the next batch. Advance to transfer another 15%."
    };
    let states = [
        (
            ProgressStatus::Default,
            "Transferring",
            "Batch 3 of 5 is being transferred.",
            60.0,
        ),
        (
            ProgressStatus::Info,
            "Preparing",
            "Checking the destination before transfer.",
            20.0,
        ),
        (
            ProgressStatus::Warning,
            "Low storage",
            "Free destination space before continuing.",
            75.0,
        ),
        (
            ProgressStatus::Error,
            "Transfer interrupted",
            "Connection lost. Reconnect and retry.",
            45.0,
        ),
        (
            ProgressStatus::Paused,
            "Paused",
            "Resume when the destination is ready.",
            60.0,
        ),
        (
            ProgressStatus::Success,
            "Complete",
            "All records have been transferred.",
            100.0,
        ),
    ];
    rsx! {
        style { {STYLES} }
        section { id: "progress-review", class: "progress-review",
            header {
                h2 { "Progress indicators" }
                p { "Show how far a task has progressed, with a clear status and next action. This illustrative transfer advances only when you choose Advance." }
            }
            section { class: "progress-demo", aria_labelledby: "progress-demo-heading",
                h3 { id: "progress-demo-heading", "Transfer production records" }
                div { class: "progress-demo-indicators",
                    div { class: "progress-demo-linear",
                        ProgressIndicator {
                            value: f64::from(progress()), status,
                            label: "Records transferred", helper_text: message,
                            "{progress()}%"
                        }
                    }
                    ProgressIndicator {
                        value: f64::from(progress()), status,
                        progress_type: ProgressType::Circular, size: ProgressSize::Xl,
                        label: "Overall progress", "{progress()}%"
                    }
                }
                div { class: "progress-demo-actions",
                    Button {
                        class: "progress-advance", disabled: paused() || failed() || complete,
                        onclick: move |_| progress.set((progress() + 15).min(100)),
                        "Advance 15%"
                    }
                    Button {
                        class: "progress-pause", variant: ButtonVariant::Secondary,
                        disabled: failed() || complete,
                        onclick: move |_| paused.set(!paused()),
                        if paused() { "Resume" } else { "Pause" }
                    }
                    if failed() {
                        Button {
                            class: "progress-retry", variant: ButtonVariant::Secondary,
                            onclick: move |_| { failed.set(false); paused.set(false); },
                            "Retry transfer"
                        }
                    } else {
                        Button {
                            class: "progress-interrupt", variant: ButtonVariant::Tertiary,
                            disabled: complete || paused(), onclick: move |_| failed.set(true),
                            "Simulate connection loss"
                        }
                    }
                    Button {
                        class: "progress-reset", variant: ButtonVariant::Tertiary,
                        onclick: move |_| { progress.set(0); paused.set(false); failed.set(false); },
                        "Reset"
                    }
                }
                output { id: "progress-demo-status", role: "status", "{progress()}% · {message}" }
            }
            section { class: "progress-states", aria_labelledby: "progress-states-heading",
                h3 { id: "progress-states-heading", "Task states" }
                div { class: "progress-state-list",
                    for (state, label, helper, value) in states {
                        div { class: "progress-state-row", key: "{state.as_str()}",
                            ProgressIndicator {
                                value, status: state, label, helper_text: helper,
                                "{value:.0}%"
                            }
                            ProgressIndicator {
                                value, status: state, progress_type: ProgressType::Circular,
                                size: ProgressSize::Md, label: format!("{label}, circular"),
                            }
                        }
                    }
                }
            }
            section { class: "progress-sizes", aria_labelledby: "progress-sizes-heading",
                h3 { id: "progress-sizes-heading", "Circular sizes" }
                p { "Use compact indicators beside a task; larger indicators can show a value inside the ring." }
                div { class: "progress-size-list",
                    for size in [ProgressSize::Xs, ProgressSize::Sm, ProgressSize::Md, ProgressSize::Lg, ProgressSize::Xl] {
                        div { class: "progress-size-example", key: "{size.as_str()}",
                            ProgressIndicator {
                                value: 65.0, progress_type: ProgressType::Circular, size,
                                label: format!("{} size", size.as_str()), "65%"
                            }
                        }
                    }
                }
            }
        }
    }
}

const STYLES: &str = include_str!("styles/progress_review.css");
