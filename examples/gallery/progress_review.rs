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

const STYLES: &str = r#"
.progress-review{color:var(--ix-color-text);display:grid;gap:32px;max-width:960px;margin:24px 0;min-width:0}
.progress-review h2{font-size:28px;line-height:1.25;margin:0 0 12px;font-weight:600}
.progress-review h3{font-size:20px;line-height:1.4;margin:0 0 16px;font-weight:600}
.progress-review p{font-size:14px;line-height:1.6;max-width:72ch;margin:0;color:var(--ix-color-text-secondary)}
.progress-demo{padding:24px;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);border-radius:8px;min-width:0}
.progress-demo-indicators{display:flex;align-items:center;gap:32px;margin-bottom:24px;min-width:0}
.progress-demo-linear{flex:1;min-width:0}
.progress-demo-indicators>.ix-progress-circular{width:112px;flex:0 0 auto}
.progress-demo-actions{display:flex;flex-wrap:wrap;gap:8px;margin-bottom:16px}
.progress-demo output{font-size:14px;line-height:1.5;font-variant-numeric:tabular-nums}
.progress-state-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:0 32px}
.progress-state-row{display:grid;grid-template-columns:minmax(0,1fr) 112px;align-items:center;gap:24px;padding:20px 0;border-bottom:1px solid var(--ix-color-border);min-width:0}
.progress-state-row>.ix-progress-circular{justify-self:start}
.progress-size-list{display:flex;align-items:flex-start;flex-wrap:wrap;gap:32px;margin-top:20px}
.progress-size-example{min-width:100px}
.progress-review .ix-progress-value,.progress-review .ix-progress-circle-value{font-variant-numeric:tabular-nums}
@media(max-width:640px){.progress-review{gap:28px}.progress-demo{padding:16px}.progress-demo-indicators{align-items:flex-start;gap:20px}.progress-state-list{grid-template-columns:minmax(0,1fr)}.progress-state-row{gap:16px}.progress-size-list{gap:24px}.progress-size-example{min-width:72px}}
@media(max-width:380px){.progress-demo-indicators{flex-direction:column}.progress-demo-linear{width:100%}}
"#;
