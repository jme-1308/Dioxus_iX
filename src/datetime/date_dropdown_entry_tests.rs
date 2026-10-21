// Included under tests::scheduling_tests to reuse the native event fixture.
use crate::datetime::clock_composition_tests::{collect, target};

#[derive(Clone)]
struct EntryFixture {
    from: IxDate,
    min: Option<IxDate>,
    controller: DateDropdownController,
    changes: Rc<RefCell<Vec<DateRangeChange>>>,
}
fn entry_fixture(data: EntryFixture) -> Element {
    rsx! {DateDropdown {
        today: date(2026,10,16), from: data.from, min_date: data.min,
        max_date: date(2026,10,25), date_range_id: "selected",
        controller: data.controller.clone(),
        options: vec![
            DateDropdownOption {id:"selected".into(),label:"Selected day".into(),from:date(2026,10,16),to:date(2026,10,16)},
            DateDropdownOption {id:"week".into(),label:"Week".into(),from:date(2026,10,14),to:date(2026,10,20)},
        ],
        onrangechange: move |change| data.changes.borrow_mut().push(change),
    }}
}
struct EntryConverter;
macro_rules! entry_converter {
    (enum Event { $(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for EntryConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {entry_converter!(@data $group,$convert,event)})*
        }
    };
    (@data Visible,$convert:ident,$event:ident) => {dioxus::html::VisibleData::new(crate::test_native_surface::Visibility(*$event.downcast::<bool>().unwrap()))};
    (@data $other:ident,$convert:ident,$event:ident) => {<ScheduleConverter as dioxus::html::HtmlEventConverter>::$convert(&ScheduleConverter,$event)};
}
dioxus::html::with_html_event_groups!(entry_converter);

#[derive(Default)]
struct EntryTargets {
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    mounts: Vec<dioxus::core::ElementId>,
    visible: Vec<dioxus::core::ElementId>,
    focuses: Vec<(String, Rc<Cell<usize>>)>,
}
impl EntryTargets {
    fn collect(&mut self, edits: dioxus::core::Mutations) {
        let mut current = None;
        for edit in &edits.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = Some(*id)
                }
                dioxus::core::Mutation::NewEventListener { name } => {
                    if let Some(id) = current {
                        if name == "mounted" {
                            self.mounts.push(id);
                        }
                        if name == "visible" {
                            self.visible.push(id);
                        }
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
    fn settle(&mut self, dom: &mut VirtualDom) {
        for _ in 0..8 {
            self.collect(dom.render_immediate_to_vec());
            for id in std::mem::take(&mut self.mounts) {
                let attrs = &self.attrs[&id.raw()];
                let label = attrs
                    .get("aria-label")
                    .or_else(|| attrs.get("id"))
                    .cloned()
                    .unwrap_or_default();
                let calls = Rc::new(Cell::new(0));
                self.focuses.push((label, calls.clone()));
                schedule_dispatch(dom, "mounted", id, Box::new(ScheduleMount(calls)));
            }
            for id in std::mem::take(&mut self.visible) {
                schedule_dispatch(dom, "visible", id, Box::new(true));
            }
            dom.process_events();
        }
    }
    fn trigger(&self) -> dioxus::core::ElementId {
        let id = self
            .attrs
            .values()
            .find_map(|attrs| {
                attrs
                    .get("id")
                    .filter(|id| id.starts_with("ix-date-dropdown-trigger-"))
            })
            .unwrap();
        target(&self.attrs, "id", id)
    }
    fn calendar_focuses(&self) -> usize {
        self.focuses
            .iter()
            .filter(|(label, _)| label.ends_with("October 2026"))
            .map(|(_, calls)| calls.get())
            .sum()
    }
    fn day_focuses(&self, label: &str) -> usize {
        self.focuses
            .iter()
            .filter(|(current, _)| current == label)
            .map(|(_, calls)| calls.get())
            .sum()
    }
    fn key(&mut self, dom: &mut VirtualDom, key: Key) {
        schedule_dispatch(dom, "keydown", self.trigger(), Box::new(ScheduleKey(key)));
        self.settle(dom);
    }
    fn click(&mut self, dom: &mut VirtualDom, id: dioxus::core::ElementId) {
        schedule_dispatch(dom, "click", id, Box::new(()));
        self.settle(dom);
    }
}
fn entry_setup(from: IxDate, min: Option<IxDate>) -> (VirtualDom, EntryTargets, EntryFixture) {
    let data = EntryFixture {
        from,
        min,
        controller: DateDropdownController::default(),
        changes: Default::default(),
    };
    let mut dom = VirtualDom::new_with_props(entry_fixture, data.clone());
    let mut targets = EntryTargets::default();
    targets.collect(dom.rebuild_to_vec());
    targets.settle(&mut dom);
    (dom, targets, data)
}

#[test]
fn keyboard_entry_focuses_selected_day_while_pointer_open_and_presets_keep_native_focus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(EntryConverter));
    let (mut dom, mut targets, data) = entry_setup(date(2026, 10, 16), None);
    let trigger = targets.trigger();
    targets.click(&mut dom, trigger);
    assert_eq!(
        targets.calendar_focuses(),
        0,
        "pointer opening must not focus a calendar day"
    );
    assert!(
        data.changes.borrow().is_empty(),
        "opening is not a selection notification"
    );
    targets.key(&mut dom, Key::Escape);
    let before = targets.calendar_focuses();
    targets.key(&mut dom, Key::Enter);
    assert!(
        targets.day_focuses("16 October 2026") > 0,
        "keyboard opening focuses the current selected day through its registered mounted handle"
    );
    assert!(targets.calendar_focuses() > before);
    assert_eq!(
        data.controller.get_date_range().unwrap().dates.from,
        Some(date(2026, 10, 16))
    );
    let before = targets.calendar_focuses();
    let preset = target(
        &targets.attrs,
        "aria-label",
        "Week: 2026/10/14 to 2026/10/20",
    );
    targets.click(&mut dom, preset);
    assert_eq!(
        targets.calendar_focuses(),
        before,
        "a preset remount after keyboard opening must not steal focus from its native button"
    );
    let range = data.controller.get_date_range().unwrap();
    assert_eq!(range.id, "week");
    assert_eq!(range.dates.from, Some(date(2026, 10, 14)));
    assert_eq!(range.dates.to, Some(date(2026, 10, 20)));
    assert_eq!(data.changes.borrow().last(), Some(&range));
}

#[test]
fn keyboard_entry_focuses_the_current_eligible_day_when_controlled_selection_is_outside_bounds() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(EntryConverter));
    let (mut dom, mut targets, data) = entry_setup(date(2026, 10, 12), Some(date(2026, 10, 14)));
    targets.key(&mut dom, Key::Enter);
    assert!(
        targets.day_focuses("14 October 2026") > 0,
        "entry resolves to the first allowed day, not the disabled controlled date"
    );
    assert_eq!(targets.day_focuses("12 October 2026"), 0);
    assert!(
        data.changes.borrow().is_empty(),
        "focus alone must not change the owner's draft or publish selection"
    );
    assert_eq!(
        data.controller.get_date_range().unwrap().dates.from,
        Some(date(2026, 10, 12))
    );
}
