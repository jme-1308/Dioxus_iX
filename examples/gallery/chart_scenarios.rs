//! The upstream chart examples, expressed as native Rust chart data and geometry.
//! Source data/options: ix/examples/html-examples/src/preview-examples/echarts*.html.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

pub const SCENARIOS: &[&str] = &[
    "echarts",
    "echarts-bar-horizontal-stacked",
    "echarts-bar-simple",
    "echarts-circle",
    "echarts-empty-state",
    "echarts-gauge",
    "echarts-line-advanced",
    "echarts-line-multiple-y-axis",
    "echarts-line-simple",
    "echarts-pie",
    "echarts-progress-arc",
    "echarts-progress-circle",
    "echarts-special-toolbox",
];

fn labels(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}
fn values(values: &[i32]) -> Vec<f64> {
    values.iter().map(|value| *value as f64).collect()
}
fn weekdays() -> Vec<String> {
    labels(&["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"])
}
fn simple_line() -> Vec<ChartSeries> {
    vec![ChartSeries::new(
        "",
        values(&[150, 230, 224, 218, 135, 147, 260]),
        ChartKind::Line,
    )]
}

fn main_series() -> Vec<ChartSeries> {
    [
        ("Direct", "", [320, 332, 301, 334, 390, 330, 320]),
        ("Email", "Ad", [120, 132, 101, 134, 90, 230, 210]),
        ("Union Ads", "Ad", [220, 182, 191, 234, 290, 330, 310]),
        ("Video Ads", "Ad", [150, 232, 201, 154, 190, 330, 410]),
        (
            "Search Engine",
            "",
            [862, 1018, 964, 1026, 1679, 1600, 1570],
        ),
        (
            "Baidu",
            "Search Engine",
            [620, 732, 701, 734, 1090, 1130, 1120],
        ),
        (
            "Google",
            "Search Engine",
            [120, 132, 101, 134, 290, 230, 220],
        ),
        ("Bing", "Search Engine", [60, 72, 71, 74, 190, 130, 110]),
        ("Others", "Search Engine", [62, 82, 91, 84, 109, 110, 120]),
    ]
    .into_iter()
    .map(|(name, stack, data)| ChartSeries {
        stack: stack.into(),
        min_max_line: name == "Search Engine",
        focus_series: true,
        bar_width: (name == "Baidu").then_some(5.0),
        ..ChartSeries::new(name, values(&data), ChartKind::Bar)
    })
    .collect()
}

fn slices(values: &[(&str, f64)]) -> Vec<ChartSlice> {
    values
        .iter()
        .map(|(name, value)| ChartSlice {
            name: (*name).into(),
            value: *value,
        })
        .collect()
}

const STOCK_DATA: [f64; 12] = [
    77.67, 82.81, 84.09, 91.75, 118.15, 107.48, 99.36, 93.07, 137.18, 104.38, 156.48, 168.52,
];

#[component]
pub fn ChartScenario(name: String) -> Element {
    let content = match name.as_str() {
        "echarts" => {
            rsx! { CartesianChart { categories: weekdays(), series: main_series(), axis_tooltip: true, pointer: ChartPointer::Shadow, label: "Sales channels" } }
        }
        "echarts-bar-simple" => {
            rsx! { CartesianChart { categories: labels(&["Product A", "Product B", "Product C", "Product D", "Product E", "Product F"]), series: vec![ChartSeries::new("", vec![10.3,9.2,7.3,6.4,6.2,4.4], ChartKind::Bar)], x_title: "Product", y_title: "Number of sold products\n (in Mio)", label: "Product sales" } }
        }
        "echarts-bar-horizontal-stacked" => rsx! { CartesianChart {
            categories: labels(&["2023", "2022", "2021", "2020", "2019"]), horizontal: true,
            x_title: "Revenue (in Millions of USD)", y_title: "Years", label: "Revenue by year and region",
            series: vec![
                ChartSeries { stack: "x".into(), ..ChartSeries::new("Europe", values(&[87,22,28,43,79]), ChartKind::Bar) },
                ChartSeries { stack: "x".into(), ..ChartSeries::new("U.S", values(&[35,24,33,5,40]), ChartKind::Bar) },
                ChartSeries { stack: "x".into(), ..ChartSeries::new("China", values(&[19,44,23,5,10]), ChartKind::Bar) },
            ],
        } },
        "echarts-line-simple" => {
            rsx! { CartesianChart { categories: weekdays(), series: simple_line(), label: "Weekday values", legend: false } }
        }
        "echarts-line-advanced" => rsx! { CartesianChart {
            categories: (2013..2025).map(|year| year.to_string()).collect::<Vec<_>>(),
            series: vec![ChartSeries { smooth: true, area: true, ..ChartSeries::new("", STOCK_DATA.to_vec(), ChartKind::Line) }],
            dashed_grid: true, mark_extremes: true, average_line: true, legend: false, axis_tooltip: true, label: "Stock data 2013–2024",
        } },
        "echarts-line-multiple-y-axis" => rsx! { MultipleAxesScenario {} },
        "echarts-circle" => {
            rsx! { PieChart { donut: true, label: "OS Share", slices: slices(&[("Windows",72.17),("macOS",15.42),("Linux",4.03),("Chrome OS",2.27),("Other",6.11)]) } }
        }
        "echarts-pie" => {
            rsx! { PieChart { label: "CO2 emissions from<", slices: slices(&[("China",29.4),("U.S",14.3),("EEA",9.8),("India",6.8),("Russia",4.9),("Japan",3.5),("Other",31.5)]) } }
        }
        "echarts-gauge" => rsx! { GaugeChart { value: 45.3 } },
        "echarts-progress-arc" => rsx! { GaugeChart { value: 68.0, variant: GaugeVariant::Arc } },
        "echarts-progress-circle" => {
            rsx! { GaugeChart { value: 68.0, variant: GaugeVariant::Circle } }
        }
        "echarts-special-toolbox" => {
            let mut series = simple_line();
            series[0].step_end = true;
            rsx! { ChartToolbox { categories: weekdays(), series, label: "Weekday values", file_name: "echarts-special-toolbox.png" } }
        }
        "echarts-empty-state" => rsx! { EmptyChartScenario {} },
        _ => rsx! { p { "Chart source not reproduced yet." } },
    };
    rsx! { div { class: "source-chart-example", {content} } }
}

// The upstream example generates three independent uniform random sequences on
// creation. Keep the same ranges/precision and keep samples stable across theme
// switches. A per-mount timestamp seeds a small platform-neutral Rust generator.
fn weather_series(seed: u64) -> Vec<ChartSeries> {
    let mut state = seed;
    let mut sample = |maximum: f64| {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (((state >> 11) as f64 / ((1_u64 << 53) as f64)) * maximum * 100.0).round() / 100.0
    };
    [
        ("Evaporation", 100.0, "var(--ix-chart-weather-evaporation)"),
        (
            "Precipitation",
            200.0,
            "var(--ix-chart-weather-precipitation)",
        ),
        ("Temperature", 30.0, "var(--ix-chart-weather-temperature)"),
    ]
    .into_iter()
    .enumerate()
    .map(|(axis, (name, maximum, color))| ChartSeries {
        axis,
        color: color.into(),
        ..ChartSeries::new(
            name,
            (0..12).map(|_| sample(maximum)).collect(),
            ChartKind::Line,
        )
    })
    .collect()
}

#[component]
fn MultipleAxesScenario() -> Element {
    let series = use_hook(|| weather_series(chrono::Utc::now().timestamp_millis() as u64));
    let axes = vec![
        ChartAxis {
            name: "Evaporation".into(),
            unit: " ml".into(),
            side: ChartAxisSide::Right,
            color: "var(--ix-chart-weather-evaporation)".into(),
            ..ChartAxis::default()
        },
        ChartAxis {
            name: "Precipitation".into(),
            unit: " ml".into(),
            side: ChartAxisSide::Right,
            offset: 80.0,
            color: "var(--ix-chart-weather-precipitation)".into(),
            ..ChartAxis::default()
        },
        ChartAxis {
            name: "Temperature".into(),
            unit: " °C".into(),
            color: "var(--ix-chart-weather-temperature)".into(),
            ..ChartAxis::default()
        },
    ];
    rsx! {
        p { class: "source-chart-data-notice", "Synthetic example data" }
        CartesianChart { categories: labels(&["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"]), series, axes, axis_tooltip: true, pointer: ChartPointer::Cross, minimum_width: 760.0, label: "Monthly evaporation, precipitation and temperature" }
    }
}

#[component]
fn EmptyChartScenario() -> Element {
    let mut loading = use_signal(|| false);
    rsx! { div { class: "source-empty-chart",
        CartesianChart { categories: weekdays(), series: vec![ChartSeries::new("", vec![], ChartKind::Line)], legend: false, label: "Weekday values, no data available" }
        div { class: "source-empty-chart-overlay", "aria-busy": loading(),
            if loading() { div { class: "source-chart-loading", Spinner { label: "Loading..." } span { "Loading..." } } }
            else { EmptyState { header: "No elements available", subheader: "Failed to retrieve data", icon: "info", action: "Try again", onaction: move |_| { loading.set(true); spawn(async move { futures_timer::Delay::new(Duration::from_secs(2)).await; loading.set(false); }); } } }
        }
    } }
}

pub const STYLES: &str = include_str!("styles/chart_scenarios.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_data_and_stack_membership_are_preserved() {
        let series = main_series();
        assert_eq!(series.len(), 9);
        assert_eq!(
            series[4].values,
            values(&[862, 1018, 964, 1026, 1679, 1600, 1570])
        );
        for index in 0..7 {
            assert_eq!(
                series[1..4].iter().map(|s| s.values[index]).sum::<f64>(),
                [490.0, 546.0, 493.0, 522.0, 570.0, 890.0, 930.0][index]
            );
        }
        assert_eq!(series[5].bar_width, Some(5.0));
        let weather = weather_series(12);
        for (axis, maximum) in [100.0, 200.0, 30.0].into_iter().enumerate() {
            assert_eq!(weather[axis].values.len(), 12);
            assert!(
                weather[axis]
                    .values
                    .iter()
                    .all(|v| *v >= 0.0 && *v <= maximum)
            );
        }
        assert_eq!(weather, weather_series(12));
    }

    #[test]
    fn all_chart_sources_render_without_external_chart_scripts_or_nonfinite_geometry() {
        for name in SCENARIOS {
            let mut dom = VirtualDom::new_with_props(
                ChartScenario,
                ChartScenarioProps {
                    name: (*name).into(),
                },
            );
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(html.contains("<svg"), "{name}");
            assert!(!html.contains("<script"), "{name}");
            assert!(
                !html.contains("NaN") && !html.contains("Infinity"),
                "{name}"
            );
            if *name == "echarts-line-multiple-y-axis" {
                assert!(html.contains("Synthetic example data"));
                assert!(html.contains("0 0 760 600"));
                assert!(html.contains("Scroll horizontally or use arrow keys"));
                let svg = html
                    .split_once("<svg")
                    .unwrap()
                    .1
                    .split_once("</svg>")
                    .unwrap()
                    .0;
                for month in [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ] {
                    assert!(
                        svg.contains(&format!(">{month}<")),
                        "missing full-density month {month}"
                    );
                }
            }
        }
    }
}
