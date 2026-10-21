//! Incoming-data regressions through actual chart legend listeners and rendering.
use super::*;
use std::cell::Cell;
use std::collections::BTreeSet;

#[derive(Clone, Default)]
struct Fixture(Rc<Inputs>);
#[derive(Default)]
struct Inputs {
    series: Cell<Option<Signal<Vec<ChartSeries>>>>,
    slices: Cell<Option<Signal<Vec<ChartSlice>>>>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
fn series(values: &[(&str, f64)]) -> Vec<ChartSeries> {
    values
        .iter()
        .map(|(name, value)| ChartSeries::new(*name, vec![*value], ChartKind::Bar))
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
fn cartesian_fixture(fixture: Fixture) -> Element {
    let input = use_signal(|| series(&[("A", 10.0), ("B", 20.0), ("", 5.0)]));
    fixture.0.series.set(Some(input));
    rsx! { CartesianChart {categories:vec!["One".into()],series:input(),label:"Live Cartesian legend"} }
}
fn pie_fixture(fixture: Fixture) -> Element {
    let input = use_signal(|| slices(&[("A", 10.0), ("B", 20.0)]));
    fixture.0.slices.set(Some(input));
    rsx! { PieChart {slices:input(),label:"Live pie legend"} }
}

struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for Mouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Mouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for Mouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for Mouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! legend_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self, _: &dioxus::html::PlatformEventData) -> dioxus::html::$data { legend_converter!(@data $group) })*
        }
    };
    (@data Mouse) => { MouseData::new(Mouse) };
    (@data $other:ident) => { panic!("Unexpected chart legend test event") };
}
dioxus::html::with_html_event_groups!(legend_converter);

fn first_legend_button(edits: dioxus::core::Mutations) -> dioxus::core::ElementId {
    // Only legend buttons use aria-pressed. Record their actual registered IDs
    // and click listeners; no chart geometry or renderer tree is reproduced.
    let mut current = None;
    let mut pressed = Vec::new();
    let mut clicks = BTreeSet::new();
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = Some(id)
            }
            dioxus::core::Mutation::SetAttribute { name, .. } if name == "aria-pressed" => {
                pressed.push(current.expect("dynamic pressed attribute has a registered target"));
            }
            dioxus::core::Mutation::NewEventListener { name } if name == "click" => {
                clicks.insert(current.expect("click listener has a registered target"));
            }
            _ => {}
        }
    }
    pressed
        .into_iter()
        .find(|id| clicks.contains(id))
        .expect("actual first legend toggle")
}
fn settle(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}
fn click(dom: &VirtualDom, id: dioxus::core::ElementId) {
    let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
    dom.runtime()
        .handle_event("click", Event::new(data, true), id);
}
fn assert_legend(dom: &VirtualDom, expected: &[(&str, bool)]) -> String {
    let html = dioxus_ssr::render(dom);
    let legend = html
        .split_once("class=\"ix-chart-legend\"")
        .unwrap()
        .1
        .split("</div>")
        .next()
        .unwrap();
    let actual: Vec<_> = legend
        .split("<button ")
        .skip(1)
        .map(|part| {
            let name = part
                .split_once("</span>")
                .unwrap()
                .1
                .split("</button>")
                .next()
                .unwrap()
                .trim();
            let attributes = part.split('>').next().unwrap();
            (
                name,
                attributes.contains("aria-pressed=\"true\"")
                    || attributes.contains("aria-pressed=true"),
            )
        })
        .collect();
    assert_eq!(actual, expected);
    html
}
fn cartesian_mark(html: &str, name: &str, value: u8) -> bool {
    let label = if name.is_empty() {
        "One".into()
    } else {
        format!("One, {name}")
    };
    html.contains(&format!("aria-label=\"{label}: {value}\""))
}
fn pie_mark(html: &str, name: &str, value: u8) -> bool {
    html.contains(&format!("aria-label=\"{name}: {value} ("))
}
fn update_series(dom: &mut VirtualDom, fixture: &Fixture, values: &[(&str, f64)]) {
    dom.in_scope(ScopeId::APP, || {
        fixture.0.series.get().unwrap().set(series(values))
    });
    settle(dom);
}
fn update_slices(dom: &mut VirtualDom, fixture: &Fixture, values: &[(&str, f64)]) {
    dom.in_scope(ScopeId::APP, || {
        fixture.0.slices.get().unwrap().set(slices(values))
    });
    settle(dom);
}

#[test]
fn cartesian_legend_choice_follows_names_across_reorder_replacement_and_current_callbacks() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = Fixture::default();
    let mut dom = VirtualDom::new_with_props(cartesian_fixture, fixture.clone());
    let first = first_legend_button(dom.rebuild_to_vec());
    settle(&mut dom);
    let initial = assert_legend(&dom, &[("A", true), ("B", true)]);
    assert!(
        cartesian_mark(&initial, "", 5),
        "unnamed series stays drawn without a legend toggle"
    );
    click(&dom, first);
    settle(&mut dom);
    let hidden = assert_legend(&dom, &[("A", false), ("B", true)]);
    assert!(!cartesian_mark(&hidden, "A", 10) && cartesian_mark(&hidden, "B", 20));
    update_series(&mut dom, &fixture, &[("B", 21.0), ("A", 11.0), ("", 5.0)]);
    let reordered = assert_legend(&dom, &[("B", true), ("A", false)]);
    assert!(cartesian_mark(&reordered, "B", 21) && !cartesian_mark(&reordered, "A", 11));
    click(&dom, first);
    settle(&mut dom);
    let both_hidden = assert_legend(&dom, &[("B", false), ("A", false)]);
    assert!(!cartesian_mark(&both_hidden, "B", 21));
    click(&dom, first);
    settle(&mut dom);
    assert_legend(&dom, &[("B", true), ("A", false)]);
    update_series(&mut dom, &fixture, &[("C", 30.0), ("B", 22.0)]);
    let replaced = assert_legend(&dom, &[("C", true), ("B", true)]);
    assert!(cartesian_mark(&replaced, "C", 30) && cartesian_mark(&replaced, "B", 22));
    update_series(&mut dom, &fixture, &[("A", 12.0), ("A", 15.0), ("C", 31.0)]);
    let duplicates = assert_legend(&dom, &[("A", false), ("A", false), ("C", true)]);
    assert!(!cartesian_mark(&duplicates, "A", 12) && !cartesian_mark(&duplicates, "A", 15));
    click(&dom, first);
    settle(&mut dom);
    let shown = assert_legend(&dom, &[("A", true), ("A", true), ("C", true)]);
    assert!(cartesian_mark(&shown, "A", 12) && cartesian_mark(&shown, "A", 15));
    click(&dom, first);
    settle(&mut dom);
    assert_legend(&dom, &[("A", false), ("A", false), ("C", true)]);
    drop(dom);
    let mut remounted = VirtualDom::new_with_props(cartesian_fixture, Fixture::default());
    remounted.rebuild_in_place();
    assert_legend(&remounted, &[("A", true), ("B", true)]);
}

#[test]
fn pie_legend_choice_follows_names_across_reorder_replacement_and_current_callbacks() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = Fixture::default();
    let mut dom = VirtualDom::new_with_props(pie_fixture, fixture.clone());
    let first = first_legend_button(dom.rebuild_to_vec());
    settle(&mut dom);
    assert_legend(&dom, &[("A", true), ("B", true)]);
    click(&dom, first);
    settle(&mut dom);
    let hidden = assert_legend(&dom, &[("A", false), ("B", true)]);
    assert!(!pie_mark(&hidden, "A", 10) && pie_mark(&hidden, "B", 20));
    update_slices(&mut dom, &fixture, &[("B", 21.0), ("A", 11.0)]);
    let reordered = assert_legend(&dom, &[("B", true), ("A", false)]);
    assert!(pie_mark(&reordered, "B", 21) && !pie_mark(&reordered, "A", 11));
    click(&dom, first);
    settle(&mut dom);
    let both_hidden = assert_legend(&dom, &[("B", false), ("A", false)]);
    assert!(!pie_mark(&both_hidden, "B", 21));
    click(&dom, first);
    settle(&mut dom);
    assert_legend(&dom, &[("B", true), ("A", false)]);
    update_slices(&mut dom, &fixture, &[("C", 30.0), ("B", 22.0)]);
    let replaced = assert_legend(&dom, &[("C", true), ("B", true)]);
    assert!(pie_mark(&replaced, "C", 30) && pie_mark(&replaced, "B", 22));
    update_slices(&mut dom, &fixture, &[("A", 12.0), ("A", 15.0), ("C", 31.0)]);
    let duplicates = assert_legend(&dom, &[("A", false), ("A", false), ("C", true)]);
    assert!(!pie_mark(&duplicates, "A", 12) && !pie_mark(&duplicates, "A", 15));
    click(&dom, first);
    settle(&mut dom);
    let shown = assert_legend(&dom, &[("A", true), ("A", true), ("C", true)]);
    assert!(pie_mark(&shown, "A", 12) && pie_mark(&shown, "A", 15));
    click(&dom, first);
    settle(&mut dom);
    assert_legend(&dom, &[("A", false), ("A", false), ("C", true)]);
    drop(dom);
    let mut remounted = VirtualDom::new_with_props(pie_fixture, Fixture::default());
    remounted.rebuild_in_place();
    assert_legend(&remounted, &[("A", true), ("B", true)]);
}
