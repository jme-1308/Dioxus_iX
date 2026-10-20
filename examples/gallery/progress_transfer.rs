//! A local transfer queue with composed progress help and immutable batch records.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const STYLES: &str = include_str!("styles/progress_transfer.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TransferBatch {
    id: &'static str,
    label: &'static str,
    records: usize,
}
const BATCHES: [TransferBatch; 3] = [
    TransferBatch {
        id: "equipment",
        label: "Equipment records",
        records: 4,
    },
    TransferBatch {
        id: "measurements",
        label: "Measurements",
        records: 5,
    },
    TransferBatch {
        id: "service-notes",
        label: "Service notes",
        records: 3,
    },
];
#[derive(Clone, Debug, PartialEq, Eq)]
struct CompletedBatch {
    batch: TransferBatch,
    note: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TransferQueue {
    completed: Vec<CompletedBatch>,
    paused: bool,
    failed: bool,
    note: String,
}
impl TransferQueue {
    fn processed(&self) -> usize {
        self.completed
            .iter()
            .map(|record| record.batch.records)
            .sum()
    }
    fn complete(&self) -> bool {
        self.completed.len() == BATCHES.len()
    }
    fn advance(&mut self) {
        if self.paused || self.failed {
            return;
        }
        if let Some(batch) = BATCHES.get(self.completed.len()) {
            self.completed.push(CompletedBatch {
                batch: *batch,
                note: self.note.clone(),
            });
        }
    }
    fn toggle_pause(&mut self) {
        if !self.failed && !self.complete() {
            self.paused = !self.paused;
        }
    }
    fn fail(&mut self) {
        if !self.paused && !self.complete() {
            self.failed = true;
        }
    }
    fn retry(&mut self) {
        if self.failed && !self.complete() {
            self.failed = false;
        }
    }
}

#[component]
fn TransferPreparation() -> Element {
    let mut preparation = use_signal(|| String::from("Check the destination folder"));
    rsx! {
        div { class: "transfer-preparation",
            Input { id: "transfer-preparation", label: "Transfer preparation", value: preparation(), max_length: 160,
                onvaluechange: move |value| preparation.set(value),
            }
        }
    }
}

#[component]
fn TransferControls(mut queue: Signal<TransferQueue>) -> Element {
    let current = queue();
    let complete = current.complete();
    rsx! {
        div { class: "transfer-controls",
            div { class: "transfer-note",
                Input { id: "transfer-note", label: "Batch note", placeholder: "Optional note for the next batch", value: current.note,
                    max_length: 160, readonly: complete,
                    onvaluechange: move |value| { let mut current = queue.write(); if !current.complete() { current.note = value; } },
                }
            }
            div { class: "transfer-actions",
                Button { id: "transfer-advance", button_type: ButtonType::Button, disabled: complete || current.paused || current.failed,
                    onclick: move |_| queue.write().advance(), "Process next batch"
                }
                Button { id: "transfer-pause", variant: ButtonVariant::Secondary, button_type: ButtonType::Button, disabled: complete || current.failed,
                    onclick: move |_| queue.write().toggle_pause(), if current.paused { "Resume" } else { "Pause" }
                }
                Button { id: "transfer-retry", variant: ButtonVariant::Secondary, button_type: ButtonType::Button, disabled: !current.failed || complete,
                    onclick: move |_| queue.write().retry(), "Retry transfer"
                }
                Button { id: "transfer-fail", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button, disabled: complete || current.paused || current.failed,
                    onclick: move |_| queue.write().fail(), "Simulate connection loss"
                }
                Button { id: "transfer-reset", variant: ButtonVariant::Tertiary, button_type: ButtonType::Button,
                    onclick: move |_| queue.set(TransferQueue::default()), "Reset queue"
                }
            }
        }
    }
}

#[component]
pub fn ProgressTransferExample() -> Element {
    let queue = use_signal(TransferQueue::default);
    let mut tooltip = use_signal(|| false);
    let progress_mount = use_focus_mount("transfer-progress");
    let current = queue();
    let processed = current.processed();
    let total: usize = BATCHES.iter().map(|batch| batch.records).sum();
    let remaining = total - processed;
    let remaining_batches = BATCHES.len() - current.completed.len();
    let complete = current.complete();
    let status = if current.failed {
        ProgressStatus::Error
    } else if complete {
        ProgressStatus::Success
    } else if current.paused {
        ProgressStatus::Paused
    } else {
        ProgressStatus::Default
    };
    let summary = if current.failed {
        "Connection lost. Retry to continue from the completed batches.".to_string()
    } else if complete {
        format!(
            "All {total} records transferred in {} batches.",
            BATCHES.len()
        )
    } else if current.paused {
        "Paused. Resume before processing the next batch.".to_string()
    } else {
        format!(
            "Ready to process {}.",
            BATCHES[current.completed.len()].label
        )
    };
    let helper = if tooltip() {
        rsx! {
            div { id: "transfer-details", class: "transfer-details",
                strong { "{remaining} records remaining" }
                span { "{remaining_batches} batches left in this local queue." }
            }
        }
    } else {
        rsx! { TransferControls { queue } }
    };
    rsx! {
        style { {STYLES} }
        section { id: "progress-transfer", class: "demo-section progress-transfer-example",
            h2 { "Transfer inspection records" }
            p { class: "transfer-intro", "Process a local queue one batch at a time. Completed batches keep their record count and the note used for that batch. Nothing is sent to a device or server." }
            TransferPreparation {}
            label { class: "transfer-option",
                input { id: "transfer-tooltip-mode", r#type: "checkbox", checked: tooltip(), onchange: move |event: FormEvent| tooltip.set(event.checked()) }
                "Show progress details in a tooltip"
            }
            div { class: "transfer-progress-section",
                ProgressIndicator { id: "transfer-progress", value: processed as f64, max: total as f64, status,
                    label: "Inspection records transferred", helper_text: summary.clone(), helper_content: helper,
                    show_text_as_tooltip: tooltip(), tabindex: if tooltip() { 0 } else { -1 }, onmounted: progress_mount,
                    "{processed} / {total} records"
                }
                if tooltip() {
                    p { class: "transfer-tooltip-hint", "Focus or hover over the progress indicator for remaining work. Queue controls stay below." }
                    div { class: "transfer-controls-outside", TransferControls { queue } }
                }
            }
            p { id: "transfer-status", class: "transfer-status", role: "status", "aria-live": "polite",
                "data-processed": processed.to_string(), "data-total": total.to_string(), "data-completed": current.completed.len().to_string(),
                "data-paused": current.paused.to_string(), "data-failed": current.failed.to_string(), "data-mode": if tooltip() { "tooltip" } else { "inline" },
                "{processed} of {total} records processed. {summary}"
            }
            h3 { "Batch queue" }
            ol { class: "transfer-queue",
                for (index, batch) in BATCHES.iter().enumerate() {
                    li { key: "{batch.id}", id: "transfer-task-{batch.id}", "data-state": if index < current.completed.len() { "complete" } else if index == current.completed.len() { "next" } else { "pending" },
                        div { strong { "{batch.label}" } span { "{batch.records} records" } }
                        span { class: "transfer-task-state", if index < current.completed.len() { "Transferred" } else if index == current.completed.len() { "Next batch" } else { "Waiting" } }
                    }
                }
            }
            table { id: "transfer-records", class: "transfer-records",
                caption { "Completed batches" }
                thead { tr { th { scope: "col", "Batch" } th { scope: "col", "Records" } th { scope: "col", "Batch note" } } }
                tbody {
                    if current.completed.is_empty() {
                        tr { td { colspan: "3", "No batch transferred yet." } }
                    }
                    for record in current.completed.iter() {
                        tr { key: "{record.batch.id}", id: "transfer-record-{record.batch.id}", "data-batch": record.batch.id, "data-units": record.batch.records.to_string(), "data-note": record.note.clone(),
                            th { scope: "row", "{record.batch.label}" }
                            td { "{record.batch.records}" }
                            td { if record.note.is_empty() { "No note" } else { "{record.note}" } }
                        }
                    }
                }
            }
        }
    }
}
