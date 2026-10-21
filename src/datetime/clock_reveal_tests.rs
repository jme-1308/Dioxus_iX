use super::clock_composition_tests::{collect, settle, target};
use super::*;
use dioxus::html::geometry::{
    PixelsRect, PixelsVector2D,
    euclid::{Point2D, Size2D},
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

type Attributes = BTreeMap<usize, BTreeMap<String, String>>;
#[derive(Clone)]
struct Settings {
    time: IxTime,
    format: TimeFormat,
    interval: u8,
    min: Option<IxTime>,
    max: Option<IxTime>,
    autofocus: bool,
    show: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            time: time(1),
            format: TimeFormat::Hms,
            interval: 1,
            min: None,
            max: None,
            autofocus: false,
            show: true,
        }
    }
}
#[derive(Default)]
struct Capture {
    settings: RefCell<Settings>,
    changes: RefCell<Vec<IxTime>>,
}
fn time(hour: u8) -> IxTime {
    IxTime::new(hour, 10, 20, 0).unwrap()
}
fn fixture(data: Rc<Capture>) -> Element {
    let value = data.settings.borrow().clone();
    rsx! {if value.show {TimePicker {time:value.time,format:value.format,hour_interval:value.interval,min_time:value.min,max_time:value.max,auto_focus:value.autofocus,embedded:true,ontimechange:move|next|data.changes.borrow_mut().push(next)}}}
}
#[derive(Clone, Default)]
struct Geometry {
    offset: Rc<Cell<f64>>,
    scrolls: Rc<RefCell<Vec<f64>>>,
    focus: Rc<Cell<usize>>,
}
#[derive(Clone)]
struct Node {
    geometry: Geometry,
    row: Option<usize>,
    blocked: Option<Rc<Cell<usize>>>,
}
struct BlockedRect(Rc<Cell<usize>>);
impl Future for BlockedRect {
    type Output = dioxus::html::MountedResult<PixelsRect>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}
impl Drop for BlockedRect {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
impl dioxus::html::RenderedElementBacking for Node {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_client_rect(
        &self,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<PixelsRect>>>> {
        if let Some(drops) = &self.blocked {
            return Box::pin(BlockedRect(drops.clone()));
        }
        let geometry = self.geometry.clone();
        let row = self.row;
        Box::pin(async move {
            Ok(PixelsRect::new(
                Point2D::new(
                    10.0,
                    100.0 + row.map_or(0.0, |row| row as f64 * 32.0 - geometry.offset.get()),
                ),
                Size2D::new(80.0, if row.is_some() { 32.0 } else { 200.0 }),
            ))
        })
    }
    fn get_scroll_offset(
        &self,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<PixelsVector2D>>>> {
        let offset = self.geometry.offset.clone();
        Box::pin(async move { Ok(PixelsVector2D::new(7.0, offset.get())) })
    }
    fn scroll(
        &self,
        offset: PixelsVector2D,
        behavior: ScrollBehavior,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        assert_eq!(behavior, ScrollBehavior::Instant);
        assert_eq!(offset.x, 7.0);
        assert!(self.row.is_none(), "scroll only the owned list");
        let geometry = self.geometry.clone();
        Box::pin(async move {
            geometry.offset.set(offset.y);
            geometry.scrolls.borrow_mut().push(offset.y);
            Ok(())
        })
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        assert!(self.row.is_some());
        let count = self.geometry.focus.clone();
        Box::pin(async move {
            count.set(count.get() + 1);
            Ok(())
        })
    }
}
struct Visible(bool);
impl dioxus::html::HasVisibleData for Visible {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn is_intersecting(&self) -> dioxus::html::VisibleResult<bool> {
        Ok(self.0)
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
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
}
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter{
    (enum Event { $(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{impl dioxus::html::HtmlEventConverter for Converter{$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{let _=event;converter!(@data $group,event)})*}};
    (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<Node>().unwrap().clone())};
    (@data Visible,$event:ident)=>{VisibleData::new(Visible(*$event.downcast::<bool>().unwrap()))};
    (@data Keyboard,$event:ident)=>{KeyboardData::new($event.downcast::<Keyboard>().unwrap().clone())};
    (@data Focus,$event:ident)=>{FocusData::new(Focus)};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected clock reveal event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: impl std::any::Any,
) {
    let value =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event(name, Event::new(value, name != "mounted"), id);
}
// Capture registered option listeners in their actual rendered unit order. This
// fixture never reconstructs a DOM tree or guesses membership from removed IDs.
fn listeners(edits: &dioxus::core::Mutations, name: &str) -> Vec<dioxus::core::ElementId> {
    let mut current = None;
    let mut result = Vec::new();
    for edit in &edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = Some(*id)
            }
            dioxus::core::Mutation::NewEventListener { name: event } if event == name => {
                result.push(current.unwrap())
            }
            _ => {}
        }
    }
    result
}
fn setup(
    data: Rc<Capture>,
) -> (
    VirtualDom,
    Attributes,
    BTreeMap<u16, dioxus::core::ElementId>,
) {
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let edits = dom.rebuild_to_vec();
    let values = clock_unit_values(
        ClockUnit::Hour,
        u16::from(data.settings.borrow().interval),
        data.settings.borrow().format,
        data.settings.borrow().time,
    );
    let ids = listeners(&edits, "keydown");
    let hours = values.into_iter().zip(ids).collect();
    let mut attrs = BTreeMap::new();
    collect(edits, &mut attrs);
    settle(&mut dom, &mut attrs);
    (dom, attrs, hours)
}
fn refresh(dom: &mut VirtualDom, attrs: &mut Attributes) -> Vec<dioxus::core::ElementId> {
    dom.mark_dirty(ScopeId::APP);
    let edits = dom.render_immediate_to_vec();
    let options = listeners(&edits, "keydown");
    collect(edits, attrs);
    settle(dom, attrs);
    options
}
fn mount_hours(
    dom: &VirtualDom,
    list: dioxus::core::ElementId,
    hours: &BTreeMap<u16, dioxus::core::ElementId>,
    geometry: &Geometry,
) {
    dispatch(
        dom,
        "mounted",
        list,
        Node {
            geometry: geometry.clone(),
            row: None,
            blocked: None,
        },
    );
    for (value, id) in hours {
        dispatch(
            dom,
            "mounted",
            *id,
            Node {
                geometry: geometry.clone(),
                row: Some(usize::from(*value)),
                blocked: None,
            },
        );
    }
}
#[test]
fn accepted_time_and_constraints_reveal_current_values_and_hidden_lists_wait_for_latest() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    let (mut dom, mut attrs, hours) = setup(data.clone());
    let list = target(&attrs, "aria-label", "hr");
    let geometry = Geometry::default();
    mount_hours(&dom, list, &hours, &geometry);
    dispatch(&dom, "visible", list, true);
    settle(&mut dom, &mut attrs);
    assert!(geometry.scrolls.borrow().is_empty());
    data.settings.borrow_mut().time = time(13);
    refresh(&mut dom, &mut attrs);
    assert_eq!(geometry.offset.get(), 248.0);
    assert_eq!(
        geometry.focus.get(),
        0,
        "ordinary controlled synchronization never focuses"
    );
    dispatch(&dom, "visible", list, false);
    settle(&mut dom, &mut attrs);
    let count = geometry.scrolls.borrow().len();
    data.settings.borrow_mut().time = time(20);
    refresh(&mut dom, &mut attrs);
    data.settings.borrow_mut().time = time(23);
    refresh(&mut dom, &mut attrs);
    assert_eq!(geometry.scrolls.borrow().len(), count);
    dispatch(&dom, "visible", list, true);
    settle(&mut dom, &mut attrs);
    assert_eq!(
        geometry.offset.get(),
        568.0,
        "reappearance uses the latest accepted time"
    );
    {
        let mut settings = data.settings.borrow_mut();
        settings.time = time(13);
        settings.interval = 2;
        settings.min = Some(time(14));
        settings.max = Some(time(22));
    }
    refresh(&mut dom, &mut attrs);
    assert_eq!(
        geometry.offset.get(),
        448.0,
        "nearest allowed interval option14 is revealed"
    );
    assert_eq!(geometry.focus.get(), 0);
    assert!(data.changes.borrow().is_empty());
}
#[test]
fn provisional_navigation_keeps_its_viewport_and_autofocus_is_initial_only() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    data.settings.borrow_mut().autofocus = true;
    let (mut dom, mut attrs, hours) = setup(data.clone());
    let list = target(&attrs, "aria-label", "hr");
    let geometry = Geometry::default();
    mount_hours(&dom, list, &hours, &geometry);
    dispatch(&dom, "visible", list, true);
    settle(&mut dom, &mut attrs);
    assert_eq!(geometry.focus.get(), 1);
    dispatch(&dom, "focus", hours[&1], ());
    dispatch(&dom, "keydown", hours[&1], Keyboard(Key::ArrowDown));
    settle(&mut dom, &mut attrs);
    assert_eq!(geometry.focus.get(), 2);
    assert!(data.changes.borrow().is_empty());
    data.settings.borrow_mut().time = time(13);
    refresh(&mut dom, &mut attrs);
    assert!(
        geometry.scrolls.borrow().is_empty(),
        "selected reveal cannot displace active provisional navigation"
    );
    assert_eq!(geometry.focus.get(), 2);
    dispatch(&dom, "keydown", hours[&2], Keyboard(Key::Enter));
    settle(&mut dom, &mut attrs);
    assert_eq!(&*data.changes.borrow(), &[time(2)]);
    dispatch(&dom, "blur", hours[&2], ());
    settle(&mut dom, &mut attrs);
    assert_eq!(geometry.offset.get(), 248.0);
    assert_eq!(geometry.focus.get(), 2);
    data.settings.borrow_mut().time = time(20);
    refresh(&mut dom, &mut attrs);
    assert_eq!(geometry.offset.get(), 472.0);
    assert_eq!(geometry.focus.get(), 2);
}
#[test]
fn replacement_mounted_options_retry_and_framework_cancels_retired_geometry_work() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    data.settings.borrow_mut().format = TimeFormat::Hms12;
    let (mut dom, mut attrs, hours) = setup(data.clone());
    let list = target(&attrs, "aria-label", "hr");
    let geometry = Geometry::default();
    mount_hours(&dom, list, &hours, &geometry);
    dispatch(&dom, "visible", list, true);
    settle(&mut dom, &mut attrs);
    data.settings.borrow_mut().time = time(23);
    let new_hours = refresh(&mut dom, &mut attrs);
    assert_eq!(new_hours.len(), 12);
    let drops = Rc::new(Cell::new(0usize));
    dispatch(
        &dom,
        "mounted",
        list,
        Node {
            geometry: geometry.clone(),
            row: None,
            blocked: Some(drops.clone()),
        },
    );
    for (index, id) in new_hours.into_iter().enumerate() {
        dispatch(
            &dom,
            "mounted",
            id,
            Node {
                geometry: geometry.clone(),
                row: Some(index),
                blocked: None,
            },
        );
    }
    settle(&mut dom, &mut attrs);
    assert!(
        geometry.scrolls.borrow().is_empty(),
        "new half-day waits for current mounted geometry"
    );
    dispatch(
        &dom,
        "mounted",
        list,
        Node {
            geometry: geometry.clone(),
            row: None,
            blocked: None,
        },
    );
    settle(&mut dom, &mut attrs);
    assert!(
        drops.get() > 0,
        "replaced resource cancels pending geometry"
    );
    assert_eq!(geometry.offset.get(), 152.0);
    assert_eq!(geometry.focus.get(), 0);
    dispatch(
        &dom,
        "mounted",
        list,
        Node {
            geometry: geometry.clone(),
            row: None,
            blocked: Some(drops.clone()),
        },
    );
    settle(&mut dom, &mut attrs);
    let before = drops.get();
    data.settings.borrow_mut().show = false;
    refresh(&mut dom, &mut attrs);
    assert!(
        drops.get() > before,
        "scope retirement cancels remaining mounted work"
    );
}

#[test]
fn native_focus_before_visibility_supersedes_optional_initial_autofocus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    data.settings.borrow_mut().autofocus = true;
    let (mut dom, mut attrs, hours) = setup(data.clone());
    let list = target(&attrs, "aria-label", "hr");
    let geometry = Geometry::default();
    mount_hours(&dom, list, &hours, &geometry);
    dispatch(&dom, "focus", hours[&1], ());
    dispatch(&dom, "visible", list, true);
    settle(&mut dom, &mut attrs);
    assert_eq!(geometry.focus.get(), 0);
    dispatch(&dom, "blur", hours[&1], ());
    settle(&mut dom, &mut attrs);
    data.settings.borrow_mut().time = time(13);
    refresh(&mut dom, &mut attrs);
    assert_eq!(geometry.offset.get(), 248.0);
    assert_eq!(
        geometry.focus.get(),
        0,
        "ordinary reveal cannot reclaim user focus"
    );
}

#[test]
fn focus_transitions_preserve_manual_neighbor_scroll_and_pending_external_values_reveal_on_blur() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let edits = dom.rebuild_to_vec();
    let options = listeners(&edits, "keydown");
    assert_eq!(options.len(), 24 + 60 + 60);
    let minutes: BTreeMap<_, _> = (0..60u16).zip(options[24..84].iter().copied()).collect();
    let seconds: BTreeMap<_, _> = (0..60u16).zip(options[84..144].iter().copied()).collect();
    let mut attrs = BTreeMap::new();
    collect(edits, &mut attrs);
    settle(&mut dom, &mut attrs);
    let minute_list = target(&attrs, "aria-label", "min");
    let second_list = target(&attrs, "aria-label", "sec");
    let minute_geometry = Geometry::default();
    let second_geometry = Geometry::default();
    mount_hours(&dom, minute_list, &minutes, &minute_geometry);
    mount_hours(&dom, second_list, &seconds, &second_geometry);
    dispatch(&dom, "visible", minute_list, true);
    dispatch(&dom, "visible", second_list, true);
    settle(&mut dom, &mut attrs);
    assert!(!second_geometry.scrolls.borrow().is_empty());
    second_geometry.scrolls.borrow_mut().clear();
    second_geometry.offset.set(1600.0); // The user scrolls toward Seconds58.
    dispatch(&dom, "focus", minutes[&1], ());
    settle(&mut dom, &mut attrs);
    dispatch(&dom, "blur", minutes[&1], ());
    settle(&mut dom, &mut attrs);
    assert_eq!(second_geometry.offset.get(), 1600.0);
    assert!(
        second_geometry.scrolls.borrow().is_empty(),
        "unchanged neighboring selection was already revealed"
    );
    dispatch(&dom, "focus", seconds[&58], ());
    settle(&mut dom, &mut attrs);
    data.settings.borrow_mut().time = IxTime::new(1, 1, 20, 0).unwrap();
    refresh(&mut dom, &mut attrs);
    assert_eq!(
        second_geometry.offset.get(),
        1600.0,
        "active unit defers the accepted external value"
    );
    // Seconds20 was already completed before manual scrolling/focus. Returning
    // to that accepted value must still reveal it, never provisional Seconds58.
    dispatch(&dom, "blur", seconds[&58], ());
    settle(&mut dom, &mut attrs);
    assert_eq!(second_geometry.offset.get(), 640.0);
    assert_eq!(&*second_geometry.scrolls.borrow(), &[640.0]);
    assert_eq!(second_geometry.focus.get(), 0);
}
