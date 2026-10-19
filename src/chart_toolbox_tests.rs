//! Actual-listener regression for incoming data while the data view is open.
use super::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

#[derive(Clone, Default)]
struct Fixture(Rc<FixtureState>);
#[derive(Default)]
struct FixtureState {
    input: Cell<Option<Signal<ChartToolboxData>>>,
    changes: Cell<usize>,
}
impl PartialEq for Fixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
fn fixture_root(fixture: Fixture) -> Element {
    let input = use_signal(|| ChartToolboxData {
        categories: vec!["Original category".into()],
        series: vec![ChartSeries::new(
            "Original series",
            vec![17.0],
            ChartKind::Line,
        )],
    });
    fixture.0.input.set(Some(input));
    let data = input();
    rsx! { ChartToolbox { categories: data.categories, series: data.series,
        label: "Lifecycle chart",
        onchange: move |_| fixture.0.changes.set(fixture.0.changes.get() + 1),
    } }
}

struct TestMouse;
impl dioxus::html::InteractionLocation for TestMouse {
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
impl dioxus::html::InteractionElementOffset for TestMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for TestMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for TestMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for TestMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! toolbox_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self, _: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                toolbox_converter!(@data $group)
            })*
        }
    };
    (@data Mouse) => { dioxus::html::MouseData::new(TestMouse) };
    (@data $other:ident) => { panic!("Unexpected chart toolbox lifecycle test event") };
}
dioxus::html::with_html_event_groups!(toolbox_converter);

// Static labels are written onto template prototypes before mounted IDs are
// assigned. Follow only the initial renderer stack to locate the labeled real
// click listener; do not infer its identity from toolbar position.
#[derive(Clone, Default)]
struct Node {
    label: String,
    children: Vec<Rc<RefCell<Node>>>,
    id: Option<dioxus::core::ElementId>,
}
impl Node {
    fn duplicate(&self) -> Self {
        Self {
            label: self.label.clone(),
            children: self
                .children
                .iter()
                .map(|child| Rc::new(RefCell::new(child.borrow().duplicate())))
                .collect(),
            id: None,
        }
    }
}
fn data_view_button(mutations: dioxus::core::Mutations) -> dioxus::core::ElementId {
    use dioxus::core::{AttributeValue, Mutation};
    let mut stack: Vec<Rc<RefCell<Node>>> = Vec::new();
    // The renderer root exists before the first mutation; Dioxus can push it
    // without emitting CreateElement/SetId for that external host node.
    let root = Rc::new(RefCell::new(Node {
        id: Some(dioxus::core::ElementId::ROOT),
        ..Node::default()
    }));
    let mut ids = BTreeMap::from([(dioxus::core::ElementId::ROOT, root)]);
    for edit in mutations.edits {
        match edit {
            Mutation::CreateElement { .. } | Mutation::CreateText { .. } => {
                stack.push(Rc::new(RefCell::new(Node::default())))
            }
            Mutation::AppendChildren { m } => {
                let children = stack.split_off(stack.len() - m);
                stack.last().unwrap().borrow_mut().children.extend(children);
            }
            Mutation::SetId { id } => {
                let node = stack.last().unwrap().clone();
                node.borrow_mut().id = Some(id);
                ids.insert(id, node);
            }
            Mutation::PushId { id } => stack.push(
                ids.get(&id)
                    .unwrap_or_else(|| panic!("unknown renderer node {id:?}"))
                    .clone(),
            ),
            Mutation::Child { index } => {
                let parent = stack.pop().unwrap();
                let child = parent.borrow().children[index].clone();
                stack.push(child);
            }
            Mutation::Pop => {
                stack.pop().unwrap();
            }
            Mutation::Clone => {
                let original = stack.pop().unwrap();
                stack.push(Rc::new(RefCell::new(original.borrow().duplicate())));
            }
            Mutation::SetAttribute {
                name,
                value: AttributeValue::Text(value),
                ..
            } if name == "aria-label" => {
                stack.last().unwrap().borrow_mut().label = value;
            }
            Mutation::NewEventListener { name } if name == "click" => {
                let node = stack.last().unwrap().borrow();
                if node.label == "Data view" {
                    return node.id.expect("mounted data-view button ID");
                }
            }
            _ => {}
        }
    }
    panic!("Data view click listener was not mounted");
}
fn settle(dom: &mut VirtualDom) {
    // Flush scheduled props/effects; no native event chronology is asserted.
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}
fn click(dom: &VirtualDom, id: dioxus::core::ElementId) {
    let payload: Rc<dyn std::any::Any> =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
    dom.runtime()
        .handle_event("click", Event::new(payload, true), id);
}

#[test]
fn incoming_dataset_closes_stale_data_view_and_reopens_with_replacement_draft() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let fixture = Fixture::default();
    let mut dom = VirtualDom::new_with_props(fixture_root, fixture.clone());
    let data_view = data_view_button(dom.rebuild_to_vec());
    settle(&mut dom);
    click(&dom, data_view);
    settle(&mut dom);
    let initial = dioxus_ssr::render(&dom);
    assert!(
        initial.contains("role=\"dialog\""),
        "real Data view click opens the editor"
    );
    let draft = initial
        .split("<textarea")
        .nth(1)
        .expect("open data-view textarea");
    assert!(draft.contains("Original category") && draft.contains("Original series"));

    let mut input = fixture.0.input.get().unwrap();
    dom.in_scope(ScopeId::ROOT, || {
        input.set(ChartToolboxData {
            categories: vec!["Replacement A".into(), "Replacement B".into()],
            series: vec![ChartSeries::new(
                "Replacement series",
                vec![91.0, 43.0],
                ChartKind::Bar,
            )],
        })
    });
    settle(&mut dom);
    let replaced = dioxus_ssr::render(&dom);
    assert!(
        !replaced.contains("role=\"dialog\""),
        "incoming data must close the stale editor before its old draft can overwrite replacement data"
    );
    assert!(replaced.contains("aria-label=\"Replacement A, Replacement series: 91\""));
    assert!(replaced.contains("aria-label=\"Replacement B, Replacement series: 43\""));
    assert!(!replaced.contains("Original category"));
    assert_eq!(
        fixture.0.changes.get(),
        0,
        "external dataset updates are silent"
    );

    click(&dom, data_view);
    settle(&mut dom);
    let reopened = dioxus_ssr::render(&dom);
    let draft = reopened
        .split("<textarea")
        .nth(1)
        .unwrap()
        .split("</textarea>")
        .next()
        .unwrap();
    assert!(
        draft.contains("Replacement series")
            && draft.contains("Replacement A\t91")
            && draft.contains("Replacement B\t43"),
        "reopened editor uses replacement categories and values: {draft}"
    );
    assert!(!draft.contains("Original category") && !draft.contains("Original series"));
    assert_eq!(
        fixture.0.changes.get(),
        0,
        "opening the editor is not a data change"
    );
}
