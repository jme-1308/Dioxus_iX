use super::clock_composition_tests::{collect, target};
use super::*;
use std::{future::Future, pin::Pin, rc::Weak};

#[derive(Default)]
struct Observation {
    handles: RefCell<Vec<(String, Weak<DayProbe>)>>,
    drops: Rc<Cell<usize>>,
}

fn fixture() -> Element {
    rsx! {DatePicker {
        today: IxDate::new(2026, 10, 15).unwrap(),
        from: IxDate::new(2026, 10, 15).unwrap(),
        single_selection: true,
        ondatechange: move |_| {},
    }}
}

struct DayProbe {
    focuses: Cell<usize>,
    drops: Rc<Cell<usize>>,
}
impl Drop for DayProbe {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
struct DayMount(Rc<DayProbe>);
impl dioxus::html::RenderedElementBacking for DayMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focused: bool,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        if focused {
            self.0.focuses.set(self.0.focuses.get() + 1);
        }
        Box::pin(async { Ok(()) })
    }
}
struct Form(String);
impl dioxus::html::HasFileData for Form {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for Form {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
struct RawKey(Key);
impl dioxus::html::ModifiersInteraction for RawKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for RawKey {
    fn key(&self) -> Key {
        self.0.clone()
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Unidentified
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*}) => {
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event; converter!(@data $group,event) })*
        }
    };
    (@data Mounted,$event:ident) => { MountedData::new(DayMount($event.downcast::<Rc<DayProbe>>().unwrap().clone())) };
    (@data Form,$event:ident) => { dioxus::html::FormData::new(Form($event.downcast::<String>().unwrap().clone())) };
    (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RawKey>().unwrap().clone()) };
    (@data $other:ident,$event:ident) => { panic!("Unexpected calendar mount event") };
}
dioxus::html::with_html_event_groups!(converter);

#[derive(Default)]
struct Targets {
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    mounted: Vec<dioxus::core::ElementId>,
}
impl Targets {
    fn collect(&mut self, edits: dioxus::core::Mutations) {
        let mut current = None;
        for edit in &edits.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = Some(*id)
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    if let Some(id) = current {
                        self.mounted.push(id);
                    }
                }
                dioxus::core::Mutation::CreateElement { .. }
                | dioxus::core::Mutation::CreateText { .. }
                | dioxus::core::Mutation::Child { .. }
                | dioxus::core::Mutation::Pop
                | dioxus::core::Mutation::Clone
                | dioxus::core::Mutation::AppendChildren { .. }
                | dioxus::core::Mutation::ReplaceWith { .. }
                | dioxus::core::Mutation::InsertBefore { .. }
                | dioxus::core::Mutation::InsertAfter { .. }
                | dioxus::core::Mutation::Remove => current = None,
                _ => {}
            }
        }
        collect(edits, &mut self.attrs);
    }
}
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: impl std::any::Any,
) {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event(name, Event::new(data, name != "mounted"), id);
}
fn settle(dom: &mut VirtualDom, targets: &mut Targets, observation: &Observation) {
    for _ in 0..4 {
        targets.collect(dom.render_immediate_to_vec());
        for id in std::mem::take(&mut targets.mounted) {
            let attrs = &targets.attrs[&id.raw()];
            if !attrs.get("class").is_some_and(|class| {
                class
                    .split_whitespace()
                    .any(|name| name == "ix-calendar-day")
            }) {
                continue;
            }
            let label = attrs["aria-label"].clone();
            let probe = Rc::new(DayProbe {
                focuses: Cell::new(0),
                drops: observation.drops.clone(),
            });
            observation
                .handles
                .borrow_mut()
                .push((label, Rc::downgrade(&probe)));
            dispatch(dom, "mounted", id, probe);
        }
        dom.process_events();
    }
}
fn change(
    dom: &mut VirtualDom,
    targets: &mut Targets,
    observation: &Observation,
    label: &str,
    value: &str,
) {
    dispatch(
        dom,
        "change",
        target(&targets.attrs, "aria-label", label),
        value.to_string(),
    );
    settle(dom, targets, observation);
}
fn handle(observation: &Observation, label: &str) -> Weak<DayProbe> {
    observation
        .handles
        .borrow()
        .iter()
        .rev()
        .find(|(actual, _)| actual == label)
        .unwrap()
        .1
        .clone()
}

#[test]
fn calendar_retires_removed_day_mounts_and_keeps_current_keyboard_focus_handles() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let observation = Observation::default();
    let mut dom = VirtualDom::new(fixture);
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets, &observation);
    let october: Vec<_> = observation
        .handles
        .borrow()
        .iter()
        .map(|(_, weak)| weak.clone())
        .collect();
    assert_eq!(october.len(), 31);
    assert!(october.iter().all(|weak| weak.upgrade().is_some()));
    change(&mut dom, &mut targets, &observation, "Select month", "11");
    assert!(
        october.iter().all(|weak| weak.upgrade().is_none()),
        "removed October day mounts must be released while the picker remains alive"
    );
    assert_eq!(observation.drops.get(), 31);
    let november = handle(&observation, "15 November 2026");
    assert!(november.upgrade().is_some());
    change(&mut dom, &mut targets, &observation, "Select year", "2027");
    assert!(november.upgrade().is_none());
    let current = handle(&observation, "16 November 2027");
    dispatch(
        &dom,
        "keydown",
        target(&targets.attrs, "aria-label", "15 November 2027"),
        RawKey(Key::ArrowRight),
    );
    settle(&mut dom, &mut targets, &observation);
    assert_eq!(current.upgrade().unwrap().focuses.get(), 1);
    dispatch(
        &dom,
        "keydown",
        target(&targets.attrs, "aria-label", "16 November 2027"),
        RawKey(Key::PageDown),
    );
    settle(&mut dom, &mut targets, &observation);
    assert!(current.upgrade().is_none());
    assert_eq!(
        handle(&observation, "16 December 2027")
            .upgrade()
            .unwrap()
            .focuses
            .get(),
        1
    );
    drop(dom);
    assert!(
        observation
            .handles
            .borrow()
            .iter()
            .all(|(_, weak)| weak.upgrade().is_none())
    );
    assert_eq!(observation.drops.get(), observation.handles.borrow().len());
}
