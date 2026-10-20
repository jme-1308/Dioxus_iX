// Shared helpers inspect the actual input-to-dialog relationship rather than a
// private popup ID. Browser hit testing and native popovers remain probe checks.
fn temporal_opening<'a>(html: &'a str, tag: &str, id: &str) -> &'a str {
    html.split('<')
        .filter_map(|part| part.split_once('>').map(|(opening, _)| opening))
        .find(|opening| {
            opening.starts_with(&format!("{tag} "))
                && temporal_form_attribute(opening, "id") == Some(id)
        })
        .unwrap_or_else(|| panic!("missing {tag} {id}: {html}"))
}
fn temporal_surface<'a>(html: &'a str, input_id: &str) -> &'a str {
    let input = temporal_opening(html, "input", input_id);
    let surface_id = temporal_form_attribute(input, "aria-controls").unwrap();
    temporal_opening(html, "dialog", surface_id)
}
fn temporal_surface_open(html: &str, input_id: &str) -> bool {
    !temporal_surface(html, input_id)
        .split_ascii_whitespace()
        .any(|attribute| matches!(attribute, "hidden" | "hidden=true" | "hidden=\"true\""))
}
#[derive(Clone, Default)]
struct TemporalSurfaceSettings {
    disabled: bool,
    readonly: bool,
    top_layer: bool,
}
#[derive(Clone, Default)]
struct TemporalSurfaceFixture {
    settings: Rc<RefCell<TemporalSurfaceSettings>>,
    values: Rc<RefCell<Vec<String>>>,
}
fn temporal_surface_fixture(fixture: TemporalSurfaceFixture) -> Element {
    let settings = fixture.settings.borrow().clone();
    let date_values = fixture.values.clone();
    let time_values = fixture.values.clone();
    rsx! {
        DateInput {id:"surface-date",today:date(2026,10,20),value:"2026/10/20",disabled:settings.disabled,readonly:settings.readonly,enable_top_layer:settings.top_layer,
            onvaluechange:move|value|date_values.borrow_mut().push(value)}
        TimeInput {id:"surface-time",value:"13:30:00",disabled:settings.disabled,readonly:settings.readonly,enable_top_layer:settings.top_layer,
            onvaluechange:move|value|time_values.borrow_mut().push(value)}
        DateTimeInput {id:"surface-combined",today:date(2026,10,20),value:"2026/10/20 13:30:00",disabled:settings.disabled,readonly:settings.readonly,enable_top_layer:settings.top_layer,
            onvaluechange:move|value|fixture.values.borrow_mut().push(value)}
    }
}
fn temporal_surface_setup() -> (VirtualDom, ScheduleTargets, TemporalSurfaceFixture) {
    let fixture = TemporalSurfaceFixture::default();
    let mut dom = VirtualDom::new_with_props(temporal_surface_fixture, fixture.clone());
    let mut targets = ScheduleTargets::default();
    targets.collect(dom.rebuild_to_vec());
    schedule_render(&mut dom, &mut targets);
    for id in [
        "surface-date-toggle",
        "surface-time-toggle",
        "surface-combined-toggle",
    ] {
        targets.mount_toggle(&dom, targets.id(id));
    }
    schedule_render(&mut dom, &mut targets);
    (dom, targets, fixture)
}
fn temporal_key(dom: &mut VirtualDom, targets: &mut ScheduleTargets, input: &str, key: Key) {
    schedule_dispatch(
        dom,
        "keydown",
        targets.id(input),
        Box::new(ScheduleKey(key)),
    );
    schedule_render(dom, targets);
}
#[derive(Clone)]
struct TemporalSurfaceVisibility(bool);
impl dioxus::html::HasVisibleData for TemporalSurfaceVisibility {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn is_intersecting(&self) -> dioxus::html::VisibleResult<bool> {
        Ok(self.0)
    }
}
struct TemporalSurfaceConverter;
macro_rules! temporal_surface_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for TemporalSurfaceConverter {
            $(fn $convert(&self, event:&dioxus::html::PlatformEventData) -> dioxus::html::$data {
                temporal_surface_converter!(@data $group, $convert, event)
            })*
        }
    };
    (@data Visible, $convert:ident, $event:ident) => {
        dioxus::html::VisibleData::new($event.downcast::<TemporalSurfaceVisibility>().unwrap().clone())
    };
    (@data $other:ident, $convert:ident, $event:ident) => {
        <ScheduleConverter as dioxus::html::HtmlEventConverter>::$convert(&ScheduleConverter, $event)
    };
}
dioxus::html::with_html_event_groups!(temporal_surface_converter);
fn temporal_calendar_visible(dom: &mut VirtualDom, targets: &mut ScheduleTargets, visible: bool) {
    let root = dom
        .get_scope(ScopeId::APP)
        .unwrap()
        .try_mounted_root_node()
        .unwrap();
    let listener = dropdown_listener(dom, root, "October 2026", "onvisible")
        .expect("calendar visibility listener");
    dom.in_runtime(|| {
        listener.call(
            Event::new(
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(
                    TemporalSurfaceVisibility(visible),
                ))),
                false,
            )
            .into_any(),
        )
    });
    schedule_render(dom, targets);
}
#[test]
fn temporal_inputs_link_persistent_dialogs_and_close_when_disabled_or_readonly() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let (mut dom, mut targets, fixture) = temporal_surface_setup();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("role=\"dialog\"").count(), 3);
    assert!(
        !html.contains("popover=\"manual\""),
        "inline is the default"
    );
    for id in ["surface-date", "surface-time", "surface-combined"] {
        let input = temporal_opening(&html, "input", id);
        assert_eq!(
            temporal_form_attribute(input, "aria-haspopup"),
            Some("dialog")
        );
        assert!(!temporal_surface_open(&html, id));
    }
    fixture.settings.borrow_mut().top_layer = true;
    dom.mark_dirty(ScopeId::APP);
    schedule_render(&mut dom, &mut targets);
    assert_eq!(
        dioxus_ssr::render(&dom)
            .matches("popover=\"manual\"")
            .count(),
        3
    );
    for readonly in [false, true] {
        for id in ["surface-date", "surface-time", "surface-combined"] {
            temporal_key(&mut dom, &mut targets, id, Key::ArrowDown);
            assert!(temporal_surface_open(&dioxus_ssr::render(&dom), id));
            {
                let mut settings = fixture.settings.borrow_mut();
                settings.disabled = !readonly;
                settings.readonly = readonly;
            }
            dom.mark_dirty(ScopeId::APP);
            schedule_render(&mut dom, &mut targets);
            assert!(!temporal_surface_open(&dioxus_ssr::render(&dom), id));
            temporal_key(&mut dom, &mut targets, id, Key::ArrowDown);
            assert!(
                !temporal_surface_open(&dioxus_ssr::render(&dom), id),
                "guarded input cannot reopen"
            );
            fixture.settings.borrow_mut().disabled = false;
            fixture.settings.borrow_mut().readonly = false;
            dom.mark_dirty(ScopeId::APP);
            schedule_render(&mut dom, &mut targets);
        }
    }
    // A native popover may still be hidden when its selected day mounts.
    // Visibility supplies the initial focus once; later viewport changes must
    // not take focus away from the user's current control.
    dioxus::html::set_event_converter(Box::new(TemporalSurfaceConverter));
    temporal_key(&mut dom, &mut targets, "surface-combined", Key::ArrowDown);
    let focus = Rc::new(Cell::new(0));
    // Resolve the live day listener: earlier guard cases intentionally remove
    // and recreate calendars, so collected renderer IDs can name old nodes.
    let root = dom
        .get_scope(ScopeId::APP)
        .unwrap()
        .try_mounted_root_node()
        .unwrap();
    let mounted = dropdown_listener(&dom, root, "20 October 2026", "onmounted")
        .expect("current selected calendar day mount handler");
    dom.in_runtime(|| {
        mounted.call(
            Event::new(
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(
                    ScheduleMount(focus.clone()),
                ))),
                false,
            )
            .into_any(),
        )
    });
    schedule_render(&mut dom, &mut targets);
    let mounted_focus = focus.get();
    assert!(
        mounted_focus > 0,
        "the live selected day receives its mounted handle"
    );
    temporal_calendar_visible(&mut dom, &mut targets, false);
    assert_eq!(
        focus.get(),
        mounted_focus,
        "hidden observer does not request focus"
    );
    temporal_calendar_visible(&mut dom, &mut targets, true);
    assert!(
        focus.get() > mounted_focus,
        "showing the popover completes initial calendar focus"
    );
    let visible_focus = focus.get();
    temporal_calendar_visible(&mut dom, &mut targets, false);
    temporal_calendar_visible(&mut dom, &mut targets, true);
    assert_eq!(
        focus.get(),
        visible_focus,
        "later visibility changes never steal focus"
    );
    assert!(
        fixture.values.borrow().is_empty(),
        "opening and guards never commit values"
    );
}
fn temporal_roving_listener(
    dom: &VirtualDom,
    node: dioxus::core::MountedVNode<'_>,
    selected: bool,
    event: &str,
) -> Option<dioxus::dioxus_core::ListenerCallback> {
    for anchor in node.template().anchors() {
        let attributes: Vec<_> = node.dynamic_attr_values()[anchor.attributes()]
            .iter()
            .flat_map(|slot| slot.iter())
            .collect();
        if attributes.iter().any(|attribute| attribute.name == "tabindex" && matches!(attribute.value, dioxus::core::AttributeValue::Int(0)))
            && attributes.iter().any(|attribute| attribute.name == "aria-selected" && matches!(attribute.value, dioxus::core::AttributeValue::Bool(value) if value == selected))
        {
            for attribute in attributes {
                if attribute.name == event {
                    if let dioxus::core::AttributeValue::Listener(listener) = &attribute.value {
                        return Some(listener.clone());
                    }
                }
            }
        }
    }
    node.dynamic_anchors()
        .flat_map(|anchor| anchor.nodes())
        .find_map(|slot| match &*slot {
            dioxus::core::DynamicNode::Component(component) => component
                .mounted_scope(slot, node, dom)
                .and_then(|scope| scope.try_mounted_root_node())
                .and_then(|child| temporal_roving_listener(dom, child, selected, event)),
            dioxus::core::DynamicNode::Fragment(_) => node
                .mounted_fragment_children(slot, dom)
                .into_iter()
                .find_map(|child| temporal_roving_listener(dom, child, selected, event)),
            _ => None,
        })
}
fn temporal_change_clock_hour(dom: &mut VirtualDom, targets: &mut ScheduleTargets) {
    let root = dom
        .get_scope(ScopeId::APP)
        .unwrap()
        .try_mounted_root_node()
        .unwrap();
    let listener = temporal_roving_listener(dom, root, true, "onkeydown")
        .expect("selected clock hour key handler");
    dom.in_runtime(|| {
        listener.call(
            Event::new(
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(ScheduleKey(
                    Key::ArrowDown,
                )))),
                true,
            )
            .into_any(),
        )
    });
    schedule_render(dom, targets);
    let root = dom
        .get_scope(ScopeId::APP)
        .unwrap()
        .try_mounted_root_node()
        .unwrap();
    let listener =
        temporal_roving_listener(dom, root, false, "onclick").expect("new provisional clock hour");
    dom.in_runtime(|| {
        listener.call(
            Event::new(
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))),
                true,
            )
            .into_any(),
        )
    });
    schedule_render(dom, targets);
}
#[test]
fn temporal_clock_and_combined_drafts_cancel_reopen_and_publish_only_on_confirmation() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let (mut dom, mut targets, fixture) = temporal_surface_setup();
    temporal_key(&mut dom, &mut targets, "surface-time", Key::ArrowDown);
    temporal_change_clock_hour(&mut dom, &mut targets);
    assert!(fixture.values.borrow().is_empty());
    temporal_key(&mut dom, &mut targets, "surface-time", Key::Escape);
    assert!(!temporal_surface_open(
        &dioxus_ssr::render(&dom),
        "surface-time"
    ));
    temporal_key(&mut dom, &mut targets, "surface-time", Key::ArrowDown);
    let html = dioxus_ssr::render(&dom);
    schedule_dispatch(&dom, "click", targets.confirmation(&html), Box::new(()));
    schedule_render(&mut dom, &mut targets);
    assert_eq!(
        *fixture.values.borrow(),
        ["13:30:00"],
        "cancelled hour draft is discarded"
    );
    temporal_key(&mut dom, &mut targets, "surface-combined", Key::ArrowDown);
    schedule_dispatch(
        &dom,
        "click",
        targets.label("21 October 2026"),
        Box::new(()),
    );
    schedule_render(&mut dom, &mut targets);
    assert_eq!(
        fixture.values.borrow().len(),
        1,
        "calendar edits remain unpublished"
    );
    temporal_key(&mut dom, &mut targets, "surface-combined", Key::Escape);
    temporal_key(&mut dom, &mut targets, "surface-combined", Key::ArrowDown);
    let html = dioxus_ssr::render(&dom);
    schedule_dispatch(&dom, "click", targets.confirmation(&html), Box::new(()));
    schedule_render(&mut dom, &mut targets);
    assert_eq!(
        *fixture.values.borrow(),
        ["13:30:00", "2026/10/20 13:30:00"]
    );
    temporal_key(&mut dom, &mut targets, "surface-time", Key::ArrowDown);
    temporal_change_clock_hour(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    schedule_dispatch(&dom, "click", targets.confirmation(&html), Box::new(()));
    schedule_render(&mut dom, &mut targets);
    assert_eq!(
        fixture.values.borrow().last().map(String::as_str),
        Some("14:30:00")
    );
    assert!(!temporal_surface_open(
        &dioxus_ssr::render(&dom),
        "surface-time"
    ));
}
#[test]
fn temporal_keyboard_selection_closes_current_picker_without_pointer_range_handoff() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ScheduleConverter));
    let mut h = TemporalMountHarness::new();
    temporal_key(&mut h.dom, &mut h.targets, "mounted-date", Key::ArrowDown);
    schedule_dispatch(
        &h.dom,
        "click",
        h.targets.label("21 October 2026"),
        Box::new(()),
    );
    schedule_render(&mut h.dom, &mut h.targets);
    let html = dioxus_ssr::render(&h.dom);
    for id in ["mounted-date", "mounted-time", "mounted-combined"] {
        assert!(
            !temporal_surface_open(&html, id),
            "keyboard selection must not advance range fields"
        );
    }
    assert_eq!(h.focuses.each_ref().map(|calls| calls.get()), [0, 0, 0]);
}
