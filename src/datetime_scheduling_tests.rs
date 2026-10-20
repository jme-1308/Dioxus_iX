// Controlled scheduling regressions; native focus and hit-testing are verified by the gallery probe.
#[derive(Clone)]
struct ScheduleConfig {
    from: Option<IxDate>,
    to: Option<IxDate>,
    min: Option<IxDate>,
    max: Option<IxDate>,
    labels: CalendarLabels,
}
impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            from: Some(date(2026, 10, 12)),
            to: None,
            min: None,
            max: None,
            labels: CalendarLabels::default(),
        }
    }
}
#[derive(Clone, Default)]
struct ScheduleFixture {
    config: Rc<RefCell<ScheduleConfig>>,
    changes: Rc<std::cell::Cell<usize>>,
}
fn schedule_calendar(fixture: ScheduleFixture) -> Element {
    let config = fixture.config.borrow().clone();
    rsx! {DatePicker {
        today:date(2026,10,20),from:config.from,to:config.to,min_date:config.min,max_date:config.max,labels:config.labels,
        ondatechange:move |_|fixture.changes.set(fixture.changes.get()+1),
    }}
}
fn schedule_settle(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn schedule_refresh(dom: &mut VirtualDom) {
    dom.mark_dirty(ScopeId::APP);
    schedule_settle(dom);
}
fn schedule_month(dom: &VirtualDom, month: &str) {
    assert!(
        dioxus_ssr::render(dom).contains(&format!("aria-label=\"{month} 2026\"")),
        "expected visible {month} calendar"
    );
}
#[test]
fn scheduling_owner_end_date_change_updates_visible_month_without_selection_callback() {
    let fixture = ScheduleFixture::default();
    let mut dom = VirtualDom::new_with_props(schedule_calendar, fixture.clone());
    dom.rebuild_in_place();
    schedule_settle(&mut dom);
    schedule_month(&dom, "October");
    fixture.config.borrow_mut().from = Some(date(2026, 11, 3));
    schedule_refresh(&mut dom);
    schedule_month(&dom, "November");
    fixture.config.borrow_mut().to = Some(date(2026, 12, 5));
    schedule_refresh(&mut dom);
    schedule_month(&dom, "December");
    assert_eq!(
        fixture.changes.get(),
        0,
        "controlled synchronization is not user selection"
    );
}

#[derive(Clone)]
struct ScheduleKey(Key);
impl dioxus::html::ModifiersInteraction for ScheduleKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for ScheduleKey {
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
#[derive(Clone)]
struct ScheduleMount(Rc<Cell<usize>>);
impl dioxus::html::RenderedElementBacking for ScheduleMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let calls = self.0.clone();
        Box::pin(async move {
            calls.set(calls.get() + 1);
            Ok(())
        })
    }
}
struct ScheduleConverter;
macro_rules! schedule_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for ScheduleConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;schedule_converter!(@data $group,event)})*
        }
    };
    (@data Mouse,$event:ident)=>{MouseData::new(DateDropdownMouse)};
    (@data Keyboard,$event:ident)=>{KeyboardData::new($event.downcast::<ScheduleKey>().unwrap().clone())};
    (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<ScheduleMount>().unwrap().clone())};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected scheduling test event")};
}
dioxus::html::with_html_event_groups!(schedule_converter);
#[derive(Default)]
struct ScheduleTargets {
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
}
impl ScheduleTargets {
    fn collect(&mut self, mutations: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    self.attrs
                        .entry(current.raw())
                        .or_default()
                        .insert(name, value);
                }
                _ => {}
            }
        }
    }
    fn label(&self, label: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            self.attrs
                .iter()
                .find_map(|(id, map)| {
                    (map.get("aria-label").map(String::as_str) == Some(label)).then_some(*id)
                })
                .unwrap_or_else(|| panic!("missing {label}")),
        )
    }
    fn confirmation(&self, html: &str) -> dioxus::core::ElementId {
        // Static child attributes can precede their SetId in the stack protocol.
        // Resolve the actual native button's dynamic ID instead of attributing
        // nested .ix-button-content classes to the last registered node.
        let tag = html
            .split('<')
            .find(|tag| {
                tag.starts_with("button ")
                    && tag
                        .split('>')
                        .next()
                        .unwrap()
                        .contains("ix-button-standard")
                    && tag.split('>').next().unwrap().contains("ix-button-primary")
            })
            .expect("combined confirmation button");
        let native_id = tag
            .split('>')
            .next()
            .unwrap()
            .split(" id=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        dioxus::core::ElementId::from_raw(
            self.attrs
                .iter()
                .find_map(|(id, attrs)| {
                    (attrs.get("id").map(String::as_str) == Some(native_id)).then_some(*id)
                })
                .expect("registered confirmation button ID"),
        )
    }
}
fn schedule_render(dom: &mut VirtualDom, targets: &mut ScheduleTargets) {
    for _ in 0..4 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn schedule_dispatch(
    dom: &VirtualDom,
    event: &str,
    id: dioxus::core::ElementId,
    payload: Box<dyn std::any::Any>,
) {
    let event_data = Event::new(
        Rc::new(dioxus::html::PlatformEventData::new(payload)) as Rc<dyn std::any::Any>,
        event != "mounted",
    );
    dom.runtime().handle_event(event, event_data, id);
}
fn schedule_calendar_setup(fixture: ScheduleFixture) -> (VirtualDom, ScheduleTargets) {
    let mut dom = VirtualDom::new_with_props(schedule_calendar, fixture);
    let mut targets = ScheduleTargets::default();
    targets.collect(dom.rebuild_to_vec());
    schedule_render(&mut dom, &mut targets);
    (dom, targets)
}

#[test]
fn scheduling_browsing_survives_unrelated_render_and_clears_while_bounds_stay_authoritative() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let fixture = ScheduleFixture::default();
    let (mut dom, mut targets) = schedule_calendar_setup(fixture.clone());
    schedule_dispatch(&dom, "click", targets.label("Next month"), Box::new(()));
    schedule_render(&mut dom, &mut targets);
    schedule_month(&dom, "November");
    fixture.config.borrow_mut().labels.done = "Finish review".into();
    schedule_refresh(&mut dom);
    schedule_month(&dom, "November");
    fixture.config.borrow_mut().from = None;
    schedule_refresh(&mut dom);
    schedule_month(&dom, "November");
    fixture.config.borrow_mut().to = Some(date(2026, 2, 2));
    schedule_refresh(&mut dom);
    schedule_month(&dom, "February");
    {
        let mut config = fixture.config.borrow_mut();
        config.min = Some(date(2026, 4, 5));
        config.max = Some(date(2026, 9, 20));
    }
    schedule_refresh(&mut dom);
    schedule_month(&dom, "September");
    fixture.config.borrow_mut().to = Some(date(2026, 12, 31));
    schedule_refresh(&mut dom);
    schedule_month(&dom, "September");
    let html = dioxus_ssr::render(&dom);
    let focused = html
        .split('<')
        .find(|tag| tag.contains("aria-label=\"20 September 2026\""))
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(
        focused.contains("tabindex=0") || focused.contains("tabindex=\"0\""),
        "new out-of-bounds endpoint uses clamped roving date"
    );
    fixture.config.borrow_mut().to = None;
    schedule_refresh(&mut dom);
    schedule_month(&dom, "September");
    assert_eq!(fixture.changes.get(), 0);
}

#[test]
fn scheduling_keyboard_focus_intent_survives_browsing_and_owner_sync_never_requests_native_focus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let fixture = ScheduleFixture::default();
    let (mut dom, mut targets) = schedule_calendar_setup(fixture.clone());
    schedule_dispatch(
        &dom,
        "keydown",
        targets.label("12 October 2026"),
        Box::new(ScheduleKey(Key::PageDown)),
    );
    schedule_render(&mut dom, &mut targets);
    schedule_month(&dom, "November");
    let calls = Rc::new(Cell::new(0));
    schedule_dispatch(
        &dom,
        "mounted",
        targets.label("12 November 2026"),
        Box::new(ScheduleMount(calls.clone())),
    );
    schedule_render(&mut dom, &mut targets);
    assert_eq!(
        calls.get(),
        1,
        "ordinary keyboard movement owns the newly mounted day focus"
    );
    fixture.config.borrow_mut().to = Some(date(2026, 12, 5));
    dom.mark_dirty(ScopeId::APP);
    schedule_render(&mut dom, &mut targets);
    schedule_month(&dom, "December");
    schedule_dispatch(
        &dom,
        "mounted",
        targets.label("5 December 2026"),
        Box::new(ScheduleMount(calls.clone())),
    );
    schedule_render(&mut dom, &mut targets);
    assert_eq!(
        calls.get(),
        1,
        "external endpoint update must not take outside/native focus"
    );
    assert_eq!(fixture.changes.get(), 0);
}

#[derive(Clone, Default)]
struct ScheduleInputFixture {
    label: Rc<RefCell<String>>,
    values: Rc<RefCell<Vec<String>>>,
}
fn schedule_datetime_input(fixture: ScheduleInputFixture) -> Element {
    rsx! {DateTimeInput {
        today:date(2026,10,20),value:"2026/10/20 13:30:00",done_label:fixture.label.borrow().clone(),
        onvaluechange:move|value|fixture.values.borrow_mut().push(value),
    }}
}
#[test]
fn scheduling_datetime_input_forwards_live_confirmation_label_and_existing_value_commit() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let fixture = ScheduleInputFixture::default();
    *fixture.label.borrow_mut() = "Bestätigen".into();
    let mut dom = VirtualDom::new_with_props(schedule_datetime_input, fixture.clone());
    let mut targets = ScheduleTargets::default();
    targets.collect(dom.rebuild_to_vec());
    schedule_render(&mut dom, &mut targets);
    schedule_dispatch(
        &dom,
        "click",
        targets.label("Toggle calendar"),
        Box::new(()),
    );
    schedule_render(&mut dom, &mut targets);
    assert!(dioxus_ssr::render(&dom).contains("Bestätigen"));
    assert!(fixture.values.borrow().is_empty());
    *fixture.label.borrow_mut() = "Lagre tidspunkt".into();
    dom.mark_dirty(ScopeId::APP);
    schedule_render(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Lagre tidspunkt"));
    assert!(!html.contains("Bestätigen"));
    schedule_dispatch(&dom, "click", targets.confirmation(&html), Box::new(()));
    schedule_render(&mut dom, &mut targets);
    assert_eq!(*fixture.values.borrow(), ["2026/10/20 13:30:00"]);
    assert!(
        !dioxus_ssr::render(&dom).contains("ix-datetime-picker"),
        "existing confirmation closes the picker"
    );

    let mut default_dom = VirtualDom::new(
        || rsx! {DateTimeInput {today:date(2026,10,20),value:"2026/10/20 13:30:00",onvaluechange:|_|{}}},
    );
    let mut defaults = ScheduleTargets::default();
    defaults.collect(default_dom.rebuild_to_vec());
    schedule_render(&mut default_dom, &mut defaults);
    schedule_dispatch(
        &default_dom,
        "click",
        defaults.label("Toggle calendar"),
        Box::new(()),
    );
    schedule_render(&mut default_dom, &mut defaults);
    assert!(
        dioxus_ssr::render(&default_dom).contains(">Done<"),
        "omitting the new label keeps the existing Rust default"
    );
}

#[test]
fn scheduling_month_option_matches_initial_calendar_and_external_owner_update() {
    fn selected_month(html: &str, expected: u8) {
        let select = html
            .split("class=\"ix-calendar-month\"")
            .nth(1)
            .unwrap()
            .split("</select>")
            .next()
            .unwrap();
        let selected: Vec<_> = select
            .split("<option")
            .skip(1)
            .filter(|option| {
                option
                    .split('>')
                    .next()
                    .unwrap()
                    .split_ascii_whitespace()
                    .any(|attribute| matches!(attribute, "selected=true" | "selected=\"true\""))
            })
            .collect();
        assert_eq!(
            selected.len(),
            1,
            "exactly one native month option must be selected"
        );
        assert!(
            selected[0]
                .split('>')
                .next()
                .unwrap()
                .contains(&format!("value=\"{expected}\""))
        );
    }
    let fixture = ScheduleFixture::default();
    let mut dom = VirtualDom::new_with_props(schedule_calendar, fixture.clone());
    dom.rebuild_in_place();
    selected_month(&dioxus_ssr::render(&dom), 10);
    schedule_month(&dom, "October");
    fixture.config.borrow_mut().from = Some(date(2026, 11, 3));
    schedule_refresh(&mut dom);
    selected_month(&dioxus_ssr::render(&dom), 11);
    schedule_month(&dom, "November");
    assert_eq!(fixture.changes.get(), 0);
}
