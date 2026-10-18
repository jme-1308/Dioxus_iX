//! Public rendering regressions; actual paint and controls are checked in the preview.
use super::*;

fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn progress_exposes_normalized_value_and_helper_in_both_presentations() {
    let html = render(|| {
        rsx! {
            ProgressIndicator {
                class: "download", label: "Download", helper_text: "Receiving records",
                min: 100.0, max: 200.0, value: 150.0,
                "50%"
            }
            ProgressIndicator {
                progress_type: ProgressType::Circular, size: ProgressSize::Xl,
                label: "Complete", value: 200.0, "100%"
            }
        }
    });
    assert_eq!(html.matches("role=\"progressbar\"").count(), 2);
    for expected in [
        "aria-label=\"Download\"",
        "aria-description=\"Receiving records\"",
        "aria-valuenow=50",
        "width:50%",
        "aria-valuenow=100",
        "stroke-dashoffset=\"0\"",
        "ix-progress-circle-value",
        "100%",
    ] {
        assert!(html.contains(expected), "missing {expected}: {html}");
    }
}

#[test]
fn progress_tooltip_preserves_description_and_invalid_range_is_empty() {
    let html = render(|| {
        rsx! { ProgressIndicator {
            label: "Import", helper_text: "Check the connection and retry",
            show_text_as_tooltip: true, status: ProgressStatus::Error,
            min: 10.0, max: 10.0, value: 12.0,
        } }
    });
    assert!(html.contains("title=\"Check the connection and retry\""));
    assert!(html.contains("aria-description=\"Check the connection and retry\""));
    assert!(html.contains("aria-valuenow=0"));
    assert!(html.contains("width:0%"));
    assert!(!html.contains("ix-progress-helper\""));
}
