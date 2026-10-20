//! The two advanced chart sources using Rust surface and sampled-series plots.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const SCENARIOS: &[&str] = &["echarts-special-3d", "echarts-special-zoom"];

fn surface() -> ChartSurface {
    ChartSurface::from_equation(41, |x, y| {
        if x.abs() < 0.1 && y.abs() < 0.1 {
            None
        } else {
            Some((x * std::f64::consts::PI).sin() * (y * std::f64::consts::PI).sin())
        }
    })
}

fn synthetic_series(seed: u64) -> (Vec<f64>, Vec<String>) {
    let mut state = seed.max(1);
    let mut random = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / ((1_u64 << 53) as f64)
    };
    let mut values = Vec::with_capacity(20_000);
    let mut labels = Vec::with_capacity(19_999);
    values.push(random() * 300.0);
    let base = IxDate::new(1968, 10, 3).unwrap();
    for index in 1..20_000 {
        // Math.round's tie direction, including negative values.
        values.push(((random() - 0.5) * 20.0 + values[index - 1] + 0.5).floor());
        let date = base.add_days(index as i64).unwrap();
        labels.push(format!("{}/{}/{}", date.year(), date.month(), date.day()));
    }
    // The source deliberately creates 20,000 values and 19,999 date labels.
    (values, labels)
}

#[component]
pub fn AdvancedChartScenario(name: String) -> Element {
    match name.as_str() {
        "echarts-special-3d" => {
            // Source: a 40rem chart host with 1rem top padding; ECharts measures
            // the remaining content height. This gallery uses the 16px root.
            rsx! { div {class:"source-surface-host",SurfaceChart { surface: surface(), label: "Sine surface with a central hole",height:624.0,show_instructions:false } } }
        }
        "echarts-special-zoom" => rsx! { SyntheticZoom {} },
        _ => rsx! { p { "Unknown advanced chart source." } },
    }
}

#[component]
fn SyntheticZoom() -> Element {
    let series = use_hook(|| synthetic_series(chrono::Local::now().timestamp_micros() as u64));
    rsx! {
        p { class: "source-chart-data-notice", "Synthetic example data" }
        ZoomChart { values: series.0, labels: series.1, label: "Synthetic data" }
    }
}

pub const STYLES: &str = include_str!("styles/advanced_chart_scenarios.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_sampling_interval_hole_and_synthetic_series_counts_are_preserved() {
        let mesh = surface();
        assert_eq!(mesh.columns, 41);
        assert!(mesh.points[20 * 41 + 20].is_none());
        assert!(mesh.points[20 * 41 + 22].is_some());
        let (data, dates) = synthetic_series(42);
        assert_eq!(data.len(), 20_000);
        assert_eq!(dates.len(), 19_999);
        assert_eq!(dates[0], "1968/10/4");
        assert!(data.iter().all(|value| value.is_finite()));
        assert_eq!(synthetic_series(42), (data, dates));
    }
    #[test]
    fn surface_source_keeps_strict_hole_boundary_and_sine_peak_samples() {
        let mesh = surface();
        assert_eq!(
            mesh.points.iter().filter(|point| point.is_none()).count(),
            9
        );
        let edge = mesh.points[20 * 41 + 22].unwrap();
        assert_eq!((edge.x, edge.y), (0.1, 0.0));
        let peak = mesh.points[30 * 41 + 30].unwrap();
        assert_eq!((peak.x, peak.y, peak.z), (0.5, 0.5, 1.0));
        let valley = mesh.points[10 * 41 + 30].unwrap();
        assert_eq!((valley.x, valley.y, valley.z), (0.5, -0.5, -1.0));
        assert_eq!(mesh.points[0].unwrap().x, -1.0);
        assert_eq!(mesh.points.last().unwrap().unwrap().y, 1.0);
    }
}
