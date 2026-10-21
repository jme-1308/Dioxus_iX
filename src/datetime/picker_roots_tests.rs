use super::super::super::clock_composition_tests::{collect, settle, target};
use dioxus::core::{ElementId, Mutation};

type RootAttributes = BTreeMap<usize, BTreeMap<String, String>>;
#[derive(Default)]
struct RootCapture {
    revision: Cell<usize>,
    disabled: Cell<bool>,
    dates: RefCell<Vec<DateSelection>>,
    confirmations: RefCell<Vec<DateTimeSelection>>,
    mounts: RefCell<Vec<(usize, usize, Rc<MountedData>)>>,
}
fn picker_roots_fixture(data: Rc<RootCapture>) -> Element {
    let revision = data.revision.get();
    let calendar = data.clone();
    let clock = data.clone();
    let combined = data.clone();
    let dates = data.clone();
    let confirmations = data.clone();
    rsx! {
        section {"data-picker-case":"date",
            DatePicker {id:format!("date-root-{revision}"),class:format!("calendar-{revision}"),aria_label:format!("Calendar {revision}"),
                today:date(2026,10,21),from:date(2026,10,21),min_date:date(2026,10,20),max_date:date(2026,10,25),single_selection:true,
                onmounted:move|event:MountedEvent|calendar.mounts.borrow_mut().push((0,revision,event.data())),
                ondatechange:move|value|dates.dates.borrow_mut().push(value)
            }
        }
        section {"data-picker-case":"time",
            TimePicker {id:format!("time-root-{revision}"),class:format!("clock-{revision}"),aria_label:format!("Clock {revision}"),format:TimeFormat::Hm,
                onmounted:move|event:MountedEvent|clock.mounts.borrow_mut().push((1,revision,event.data())),ontimechange:move |_|{}
            }
        }
        section {"data-picker-case":"combined",
            DateTimePicker {id:format!("combined-root-{revision}"),class:format!("combined-{revision}"),aria_label:format!("Date and time {revision}"),
                today:date(2026,10,21),from:date(2026,10,21),single_selection:true,time:IxTime::new(9,30,0,0).unwrap(),time_format:TimeFormat::Hm,
                disabled:data.disabled.get(),done_label:"Confirm combined",
                onmounted:move|event:MountedEvent|combined.mounts.borrow_mut().push((2,revision,event.data())),
                ondatechange:move |_|{},ontimechange:move |_|{},onselect:move|value|confirmations.confirmations.borrow_mut().push(value)
            }
        }
    }
}
fn native_listeners(edits: &dioxus::core::Mutations, event: &str) -> Vec<ElementId> {
    let mut current = None;
    let mut result = Vec::new();
    for edit in &edits.edits {
        match edit {
            Mutation::SetId { id } | Mutation::PushId { id } => current = Some(*id),
            Mutation::NewEventListener { name } if name == event => result.push(current.unwrap()),
            Mutation::SetAttribute { .. }
            | Mutation::SetText { .. }
            | Mutation::NewEventListener { .. }
            | Mutation::RemoveEventListener { .. } => {}
            _ => current = None,
        }
    }
    result
}
fn root_tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|tag| {
            tag.starts_with("div ")
                && tag
                    .split('>')
                    .next()
                    .unwrap()
                    .contains(&format!("id=\"{id}\""))
        })
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn picker_section<'a>(html: &'a str, name: &str) -> &'a str {
    html.split_once(&format!("data-picker-case=\"{name}\""))
        .unwrap()
        .1
        .split("</section>")
        .next()
        .unwrap()
}
fn confirmation_id(dom: &VirtualDom, attrs: &RootAttributes, section: &str) -> ElementId {
    let html = dioxus_ssr::render(dom);
    let footer = picker_section(&html, section)
        .split_once("ix-date-card-footer")
        .unwrap()
        .1;
    let button = footer
        .split('<')
        .find(|tag| tag.starts_with("button "))
        .unwrap();
    let id = button
        .split_once("id=\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    target(attrs, "id", id)
}
#[test]
fn picker_roots_preserve_defaults_and_forward_current_named_native_mounts_and_actions() {
    let mut defaults = VirtualDom::new(|| {
        rsx! {
            section {"data-picker-case":"card",DateTimeCard {"Card content"}}
            section {"data-picker-case":"date",DatePicker {today:date(2026,10,21),ondatechange:move |_|{}}}
            section {"data-picker-case":"time",TimePicker {format:TimeFormat::Hm,ontimechange:move |_|{}}}
            section {"data-picker-case":"combined",DateTimePicker {today:date(2026,10,21),ondatechange:move |_|{},ontimechange:move |_|{},onselect:move |_|{}}}
        }
    });
    defaults.rebuild_in_place();
    let default_html = dioxus_ssr::render(&defaults);
    assert!(!default_html.contains("role=\"group\""));
    for name in ["card", "date", "time"] {
        let classes = picker_section(&default_html, name)
            .split_once("class=\"")
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>();
        assert_eq!(classes, ["ix-date-time-card", "ix-date-corners-rounded"]);
    }
    assert!(picker_section(&default_html, "combined").contains("class=\"ix-datetime-picker\""));

    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let data = Rc::new(RootCapture::default());
    let mut dom = VirtualDom::new_with_props(picker_roots_fixture, data.clone());
    let mut attrs = RootAttributes::new();
    collect(dom.rebuild_to_vec(), &mut attrs);
    settle(&mut dom, &mut attrs);
    let ids =
        ["date", "time", "combined"].map(|name| target(&attrs, "id", &format!("{name}-root-0")));
    let backings: [Rc<Cell<usize>>; 3] = std::array::from_fn(|_| Rc::new(Cell::new(0)));
    for (id, backing) in ids.iter().zip(&backings) {
        schedule_dispatch(
            &dom,
            "mounted",
            *id,
            Box::new(ScheduleMount(backing.clone())),
        );
    }
    assert_eq!(data.mounts.borrow().len(), 3);
    for (index, revision, mounted) in data.mounts.borrow().iter() {
        assert_eq!(*revision, 0);
        assert!(Rc::ptr_eq(
            &mounted.downcast::<ScheduleMount>().unwrap().0,
            &backings[*index]
        ));
    }
    let day = target(&attrs, "aria-label", "23 October 2026");
    data.revision.set(1);
    dom.mark_dirty(ScopeId::APP);
    let updates = dom.render_immediate_to_vec();
    assert!(
        native_listeners(&updates, "mounted").is_empty(),
        "metadata does not replace mounted roots or controls"
    );
    collect(updates, &mut attrs);
    settle(&mut dom, &mut attrs);
    let html = dioxus_ssr::render(&dom);
    for (index, name) in ["date", "time", "combined"].iter().enumerate() {
        assert_eq!(target(&attrs, "id", &format!("{name}-root-1")), ids[index]);
        let tag = root_tag(&html, &format!("{name}-root-1"));
        assert!(tag.contains("role=\"group\"") && tag.contains("aria-label="));
        schedule_dispatch(
            &dom,
            "mounted",
            ids[index],
            Box::new(ScheduleMount(backings[index].clone())),
        );
    }
    assert!(root_tag(&html, "date-root-1").contains("calendar-1"));
    assert!(root_tag(&html, "time-root-1").contains("clock-1"));
    assert!(root_tag(&html, "combined-root-1").contains("ix-datetime-picker combined-1"));
    for (index, revision, mounted) in data.mounts.borrow().iter().skip(3) {
        assert_eq!(*revision, 1, "the live callback receives native events");
        assert!(Rc::ptr_eq(
            &mounted.downcast::<ScheduleMount>().unwrap().0,
            &backings[*index]
        ));
    }
    assert_eq!(target(&attrs, "aria-label", "23 October 2026"), day);
    schedule_dispatch(&dom, "click", day, Box::new(()));
    assert_eq!(
        data.dates.borrow().as_slice(),
        [DateSelection {
            from: Some(date(2026, 10, 23)),
            to: None
        }]
    );
    let confirm = confirmation_id(&dom, &attrs, "combined");
    schedule_dispatch(&dom, "click", confirm, Box::new(()));
    assert_eq!(
        data.confirmations.borrow().as_slice(),
        [DateTimeSelection {
            dates: DateSelection {
                from: Some(date(2026, 10, 21)),
                to: None
            },
            time: IxTime::new(9, 30, 0, 0).unwrap()
        }]
    );
    data.disabled.set(true);
    dom.mark_dirty(ScopeId::APP);
    collect(dom.render_immediate_to_vec(), &mut attrs);
    settle(&mut dom, &mut attrs);
    schedule_dispatch(&dom, "click", confirm, Box::new(()));
    assert_eq!(
        data.confirmations.borrow().len(),
        1,
        "disabled confirmation remains inactive"
    );
}

#[derive(Default)]
struct ClockRootCapture {
    revision: Cell<usize>,
    current: Cell<Option<Signal<IxTime>>>,
    changes: RefCell<Vec<IxTime>>,
    confirmed: RefCell<Vec<IxTime>>,
}
fn clock_root_fixture(data: Rc<ClockRootCapture>) -> Element {
    let mut time = use_signal(|| IxTime::new(9, 30, 0, 0).unwrap());
    data.current.set(Some(time));
    let revision = data.revision.get();
    let changes = data.clone();
    let confirmed = data.clone();
    rsx! {section {"data-picker-case":"clock",
        TimePicker {id:format!("live-clock-{revision}"),class:format!("consumer-{revision}"),aria_label:(revision!=0).then(||format!("Service clock {revision}")),
            time:time(),format:TimeFormat::Hm,ontimechange:move|next|{changes.changes.borrow_mut().push(next);time.set(next);},
            onselect:move|value|confirmed.confirmed.borrow_mut().push(value)
        }
    }}
}
struct RootFocus;
impl dioxus::html::HasFocusData for RootFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct RootConverter;
macro_rules! root_converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for RootConverter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;root_converter!(@data $group,$convert,event)})*}
    };
    (@data Focus,$convert:ident,$event:ident)=>{FocusData::new(RootFocus)};
    (@data $group:ident,$convert:ident,$event:ident)=>{dioxus::html::HtmlEventConverter::$convert(&ScheduleConverter,$event)};
}
dioxus::html::with_html_event_groups!(root_converter);
#[test]
fn clock_root_metadata_keeps_provisional_keyboard_choice_and_current_confirmation() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(RootConverter));
    let data = Rc::new(ClockRootCapture::default());
    let mut dom = VirtualDom::new_with_props(clock_root_fixture, data.clone());
    let initial = dom.rebuild_to_vec();
    // This fixture contains one Hm clock: its actual native key listeners are
    // registered as the 24 Hours options followed by the 60 Minutes options.
    let options = native_listeners(&initial, "keydown");
    assert_eq!(options.len(), 84);
    let mut attrs = RootAttributes::new();
    collect(initial, &mut attrs);
    settle(&mut dom, &mut attrs);
    let root = target(&attrs, "id", "live-clock-0");
    let focus = Rc::new(Cell::new(0));
    schedule_dispatch(
        &dom,
        "mounted",
        options[10],
        Box::new(ScheduleMount(focus.clone())),
    );
    schedule_dispatch(&dom, "focus", options[9], Box::new(()));
    schedule_dispatch(
        &dom,
        "keydown",
        options[9],
        Box::new(ScheduleKey(Key::ArrowDown)),
    );
    settle(&mut dom, &mut attrs);
    assert_eq!(focus.get(), 1);
    assert!(
        data.changes.borrow().is_empty(),
        "arrows keep an unpublished clock choice"
    );
    data.revision.set(1);
    dom.mark_dirty(ScopeId::APP);
    let updates = dom.render_immediate_to_vec();
    assert!(native_listeners(&updates, "mounted").is_empty());
    assert!(native_listeners(&updates, "keydown").is_empty());
    collect(updates, &mut attrs);
    settle(&mut dom, &mut attrs);
    assert_eq!(target(&attrs, "id", "live-clock-1"), root);
    let html = dioxus_ssr::render(&dom);
    assert!(root_tag(&html, "live-clock-1").contains("aria-label=\"Service clock 1\""));
    let tenth_hour = html
        .split("role=\"option\"")
        .nth(11)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    let tabstop = tenth_hour
        .split_once("tabindex=")
        .unwrap()
        .1
        .split_whitespace()
        .next()
        .unwrap()
        .trim_matches('"');
    assert_eq!(tabstop, "0", "metadata must retain the provisional hour");
    assert!(data.changes.borrow().is_empty());
    schedule_dispatch(
        &dom,
        "keydown",
        options[10],
        Box::new(ScheduleKey(Key::Enter)),
    );
    settle(&mut dom, &mut attrs);
    let expected = IxTime::new(10, 30, 0, 0).unwrap();
    assert_eq!(data.changes.borrow().as_slice(), [expected]);
    schedule_dispatch(
        &dom,
        "click",
        confirmation_id(&dom, &attrs, "clock"),
        Box::new(()),
    );
    assert_eq!(data.confirmed.borrow().as_slice(), [expected]);
    assert_eq!(
        focus.get(),
        1,
        "ordinary metadata and commit do not reclaim native focus"
    );
}
