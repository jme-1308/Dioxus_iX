//! Actual mounted clock ownership through ordinary period, interval and precision changes.
use super::clock_composition_tests::{collect, target};
use super::*;
use dioxus::core::{ElementId, Mutation};
use dioxus::html::geometry::{
    PixelsRect, PixelsVector2D,
    euclid::{Point2D, Size2D},
};
use std::{future::Future, pin::Pin, rc::Weak};

type KeyValue = (ClockUnit, u16);
#[derive(Clone)]
struct Settings {
    time: IxTime,
    format: TimeFormat,
    hours: u8,
    minutes: u8,
    show: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            time: IxTime::new(9, 10, 20, 100).unwrap(),
            format: TimeFormat::HmsMillis12,
            hours: 1,
            minutes: 1,
            show: true,
        }
    }
}
#[derive(Default)]
struct Capture {
    settings: Cell<Option<Signal<Settings>>>,
    changes: RefCell<Vec<IxTime>>,
    confirmations: RefCell<Vec<IxTime>>,
}
fn fixture(data: Rc<Capture>) -> Element {
    let mut settings = use_signal(Settings::default);
    data.settings.set(Some(settings));
    let current = settings();
    let changes = data.clone();
    let confirmations = data.clone();
    rsx! {if current.show {TimePicker {time:current.time,format:current.format,hour_interval:current.hours,minute_interval:current.minutes,
        embedded:true,has_footer:true,ontimechange:move|next|{changes.changes.borrow_mut().push(next);settings.write().time=next;},
        onselect:move|next|confirmations.confirmations.borrow_mut().push(next)
    }}}
}
#[derive(Clone, Default)]
struct Geometry {
    offset: Rc<Cell<f64>>,
    focuses: Rc<Cell<usize>>,
}
struct Probe {
    geometry: Geometry,
    row: Option<usize>,
}
#[derive(Clone)]
struct Mount(Rc<Probe>);
impl dioxus::html::RenderedElementBacking for Mount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_client_rect(
        &self,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<PixelsRect>>>> {
        let geometry = self.0.geometry.clone();
        let row = self.0.row;
        Box::pin(async move {
            Ok(PixelsRect::new(
                Point2D::new(
                    10.,
                    100. + row.map_or(0., |row| row as f64 * 32. - geometry.offset.get()),
                ),
                Size2D::new(80., if row.is_some() { 32. } else { 200. }),
            ))
        })
    }
    fn get_scroll_offset(
        &self,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<PixelsVector2D>>>> {
        let offset = self.0.geometry.offset.clone();
        Box::pin(async move { Ok(PixelsVector2D::new(0., offset.get())) })
    }
    fn scroll(
        &self,
        offset: PixelsVector2D,
        behavior: ScrollBehavior,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(self.0.row.is_none(), "reveal scrolls only its actual list");
        assert_eq!(behavior, ScrollBehavior::Instant);
        let geometry = self.0.geometry.clone();
        Box::pin(async move {
            geometry.offset.set(offset.y);
            Ok(())
        })
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus && self.0.row.is_some());
        let count = self.0.geometry.focuses.clone();
        Box::pin(async move {
            count.set(count.get() + 1);
            Ok(())
        })
    }
}
#[derive(Clone)]
struct Keyboard(Key);
impl dioxus::html::ModifiersInteraction for Keyboard {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for Keyboard {
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
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Visible(bool);
impl dioxus::html::HasVisibleData for Visible {
    fn is_intersecting(&self) -> dioxus::html::VisibleResult<bool> {
        Ok(self.0)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
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
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*}
    };
    (@data Mounted,$event:ident)=>{MountedData::new(Mount($event.downcast::<Rc<Probe>>().unwrap().clone()))};
    (@data Keyboard,$event:ident)=>{KeyboardData::new($event.downcast::<Keyboard>().unwrap().clone())};
    (@data Focus,$event:ident)=>{FocusData::new(Focus)};
    (@data Visible,$event:ident)=>{VisibleData::new(Visible(*$event.downcast::<bool>().unwrap()))};
    (@data Mouse,$event:ident)=>{MouseData::new(Mouse)};
    (@data $other:ident,$event:ident)=>{panic!("unexpected clock mount event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(dom: &VirtualDom, name: &str, id: ElementId, payload: impl std::any::Any) {
    let value =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event(name, Event::new(value, name != "mounted"), id);
}
fn listeners(edits: &dioxus::core::Mutations, name: &str) -> Vec<ElementId> {
    let mut current = None;
    let mut result = Vec::new();
    for edit in &edits.edits {
        match edit {
            Mutation::SetId { id } | Mutation::PushId { id } => current = Some(*id),
            Mutation::NewEventListener { name: actual } if actual == name => {
                result.push(current.expect("registered native listener"))
            }
            Mutation::SetAttribute { .. }
            | Mutation::SetText { .. }
            | Mutation::NewEventListener { .. }
            | Mutation::RemoveEventListener { .. } => {}
            _ => current = None,
        }
    }
    result
}
fn units(settings: &Settings) -> Vec<(ClockUnit, u16)> {
    if !settings.show {
        return Vec::new();
    }
    let mut result = vec![
        (ClockUnit::Hour, u16::from(settings.hours)),
        (ClockUnit::Minute, u16::from(settings.minutes)),
    ];
    if settings.format.has_seconds() {
        result.push((ClockUnit::Second, 1));
    }
    if settings.format.has_milliseconds() {
        result.push((ClockUnit::Millisecond, 100));
    }
    result
}
struct Harness {
    dom: VirtualDom,
    data: Rc<Capture>,
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    ids: BTreeMap<KeyValue, ElementId>,
    options: BTreeMap<KeyValue, Weak<Probe>>,
    lists: BTreeMap<ClockUnit, (ElementId, Weak<Probe>)>,
    rendered: BTreeSet<KeyValue>,
    geometry: BTreeMap<ClockUnit, Geometry>,
    all: Vec<Weak<Probe>>,
}
impl Harness {
    fn new() -> Self {
        let data = Rc::new(Capture::default());
        let mut dom = VirtualDom::new_with_props(fixture, data.clone());
        let edits = dom.rebuild_to_vec();
        let mut harness = Self {
            dom,
            data,
            attrs: BTreeMap::new(),
            ids: BTreeMap::new(),
            options: BTreeMap::new(),
            lists: BTreeMap::new(),
            rendered: BTreeSet::new(),
            geometry: BTreeMap::new(),
            all: Vec::new(),
        };
        harness.mount(edits);
        harness.settle();
        harness
    }
    fn mount(&mut self, edits: dioxus::core::Mutations) {
        let mounted = listeners(&edits, "mounted");
        let keydown = listeners(&edits, "keydown");
        collect(edits, &mut self.attrs);
        let settings = self.data.settings.get().unwrap().peek().clone();
        let units = units(&settings);
        let expected: Vec<_> = units
            .iter()
            .flat_map(|(unit, interval)| {
                clock_unit_values(*unit, *interval, settings.format, settings.time)
                    .into_iter()
                    .map(|value| (*unit, value))
            })
            .collect();
        let added: Vec<_> = expected
            .iter()
            .filter(|key| !self.rendered.contains(key))
            .copied()
            .collect();
        let new_options: Vec<_> = keydown
            .into_iter()
            .filter(|id| {
                mounted.contains(id)
                    && self
                        .attrs
                        .get(&id.raw())
                        .and_then(|attrs| attrs.get("class"))
                        .is_some_and(|class| {
                            class
                                .split_whitespace()
                                .any(|class| class == "ix-clock-option")
                        })
            })
            .collect();
        assert_eq!(
            new_options.len(),
            added.len(),
            "new mounted options follow the existing clock reveal fixture's rendered-unit order"
        );
        for (key, id) in added.into_iter().zip(new_options) {
            let interval = units.iter().find(|(unit, _)| *unit == key.0).unwrap().1;
            let row = clock_unit_values(key.0, interval, settings.format, settings.time)
                .iter()
                .position(|value| *value == key.1)
                .unwrap();
            let geometry = self.geometry.entry(key.0).or_default().clone();
            let probe = Rc::new(Probe {
                geometry,
                row: Some(row),
            });
            self.ids.insert(key, id);
            self.options.insert(key, Rc::downgrade(&probe));
            self.all.push(Rc::downgrade(&probe));
            dispatch(&self.dom, "mounted", id, probe);
        }
        for id in mounted {
            let Some(label) = self
                .attrs
                .get(&id.raw())
                .and_then(|attrs| attrs.get("aria-label"))
            else {
                continue;
            };
            let Some((unit, _)) = units.iter().find(|(unit, _)| unit.name() == label) else {
                continue;
            };
            let geometry = self.geometry.entry(*unit).or_default().clone();
            let probe = Rc::new(Probe {
                geometry,
                row: None,
            });
            self.lists.insert(*unit, (id, Rc::downgrade(&probe)));
            self.all.push(Rc::downgrade(&probe));
            dispatch(&self.dom, "mounted", id, probe);
        }
        self.rendered = expected.into_iter().collect();
    }
    fn settle(&mut self) {
        for _ in 0..6 {
            let edits = self.dom.render_immediate_to_vec();
            self.mount(edits);
            self.dom.process_events();
        }
    }
    fn change(&mut self, change: impl FnOnce(&mut Settings)) {
        self.dom
            .in_runtime(|| self.data.settings.get().unwrap().with_mut(change));
        self.settle();
    }
    fn key(&mut self, key: KeyValue, event: Key) {
        dispatch(&self.dom, "keydown", self.ids[&key], Keyboard(event));
        self.settle();
    }
    fn focus(&mut self, key: KeyValue) {
        dispatch(&self.dom, "focus", self.ids[&key], ());
        self.settle();
    }
    fn click(&mut self, label: &str) {
        dispatch(
            &self.dom,
            "click",
            target(&self.attrs, "aria-label", label),
            (),
        );
        self.settle();
    }
    fn visible(&mut self, unit: ClockUnit) {
        dispatch(&self.dom, "visible", self.lists[&unit].0, true);
        self.settle();
    }
}

#[test]
fn clock_releases_retired_option_and_list_mounts_but_keeps_current_navigation_and_reveal() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut h = Harness::new();
    for unit in [
        ClockUnit::Hour,
        ClockUnit::Minute,
        ClockUnit::Second,
        ClockUnit::Millisecond,
    ] {
        h.visible(unit);
    }
    let am: Vec<_> = h
        .options
        .iter()
        .filter(|((unit, _), _)| *unit == ClockUnit::Hour)
        .map(|(_, weak)| weak.clone())
        .collect();
    assert_eq!(am.len(), 12);
    assert!(am.iter().all(|weak| weak.upgrade().is_some()));
    let minute10 = h.options[&(ClockUnit::Minute, 10)].clone();
    let minute10_id = h.ids[&(ClockUnit::Minute, 10)];
    let seconds = h.lists[&ClockUnit::Second].1.clone();
    let millis = h.lists[&ClockUnit::Millisecond].1.clone();
    let second20 = h.options[&(ClockUnit::Second, 20)].clone();
    let millis100 = h.options[&(ClockUnit::Millisecond, 100)].clone();
    h.focus((ClockUnit::Hour, 9));
    h.key((ClockUnit::Hour, 9), Key::ArrowDown);
    assert!(h.data.changes.borrow().is_empty());
    let hour_focuses = h.geometry[&ClockUnit::Hour].focuses.get();
    assert!(hour_focuses > 0);
    h.geometry[&ClockUnit::Hour].offset.set(0.);
    h.click("PM");
    assert_eq!(h.data.changes.borrow().last().unwrap().hour, 21);
    assert!(
        am.iter().all(|weak| weak.upgrade().is_none()),
        "removed AM option mounts must be released while the clock remains alive"
    );
    assert!(
        h.geometry[&ClockUnit::Hour].offset.get() > 0.,
        "retiring the focused AM key admits the current PM selection reveal"
    );
    assert_eq!(
        h.geometry[&ClockUnit::Hour].focuses.get(),
        hour_focuses,
        "period synchronization never reclaims native focus"
    );
    assert_eq!(h.ids[&(ClockUnit::Minute, 10)], minute10_id);
    assert!(minute10.upgrade().is_some());
    let hour21 = h.options[&(ClockUnit::Hour, 21)].clone();
    let hour21_id = h.ids[&(ClockUnit::Hour, 21)];
    let hour22 = h.options[&(ClockUnit::Hour, 22)].clone();
    let minute11 = h.options[&(ClockUnit::Minute, 11)].clone();
    h.change(|settings| {
        settings.hours = 2;
        settings.minutes = 2;
    });
    assert!(
        hour22.upgrade().is_none() && minute11.upgrade().is_none(),
        "interval changes retire absent option handles"
    );
    assert!(hour21.upgrade().is_some() && minute10.upgrade().is_some());
    assert_eq!(h.ids[&(ClockUnit::Hour, 21)], hour21_id);
    assert_eq!(h.ids[&(ClockUnit::Minute, 10)], minute10_id);
    h.focus((ClockUnit::Minute, 10));
    h.key((ClockUnit::Minute, 10), Key::ArrowDown);
    h.key((ClockUnit::Minute, 12), Key::Enter);
    assert_eq!(h.data.changes.borrow().last().unwrap().minute, 12);
    h.focus((ClockUnit::Second, 20));
    h.change(|settings| settings.format = TimeFormat::Hm12);
    assert!(
        seconds.upgrade().is_none() && millis.upgrade().is_none(),
        "removed unit containers must not survive in mounted maps or completed reveal memo"
    );
    assert!(second20.upgrade().is_none() && millis100.upgrade().is_none());
    assert!(hour21.upgrade().is_some() && minute10.upgrade().is_some());
    let html = dioxus_ssr::render(&h.dom);
    let footer = html.split_once("class=\"ix-date-card-footer\"").unwrap().1;
    let confirm = footer
        .split('<')
        .find(|tag| tag.starts_with("button "))
        .unwrap()
        .split_once("id=\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    dispatch(&h.dom, "click", target(&h.attrs, "id", confirm), ());
    h.settle();
    assert_eq!(
        h.data.confirmations.borrow().last().copied(),
        Some(h.data.settings.get().unwrap().peek().time)
    );
    h.change(|settings| settings.show = false);
    assert!(
        h.all.iter().all(|weak| weak.upgrade().is_none()),
        "picker teardown releases every actual mounted backing"
    );
}
