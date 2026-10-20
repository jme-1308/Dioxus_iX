use super::*;
use std::collections::BTreeMap;

#[test]
fn flip_indices_handle_empty_single_and_out_of_range_pages() {
    assert_eq!(next_flip_index(0, 0), 0);
    assert_eq!(next_flip_index(9, 1), 0);
    assert_eq!(next_flip_index(1, 2), 0);
    assert_eq!(flip_index(99, 3), 2);
    assert_eq!(FlipTileDimension::Rem(f64::NAN).css(16.0), "16rem");
}

#[derive(Clone)]
struct Config {
    ids: Vec<&'static str>,
    index: Option<usize>,
    default_index: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            ids: vec!["overview", "service", "events"],
            index: Some(0),
            default_index: 0,
        }
    }
}
#[derive(Default)]
struct Fixture {
    config: RefCell<Config>,
    veto: Cell<bool>,
    before: RefCell<Vec<(usize, usize)>>,
    accepted: RefCell<Vec<usize>>,
    mounts: Rc<Cell<usize>>,
}
fn fixture(data: Rc<Fixture>) -> Element {
    let config = data.config.borrow().clone();
    let requests = data.clone();
    let items = config
        .ids
        .iter()
        .map(|id| {
            FlipTileItem::new(
                *id,
                *id,
                rsx! { LocalPage { id: *id, mounts: data.mounts.clone() } },
            )
        })
        .collect();
    rsx! { FlipTile {
        items, index: config.index, default_index: config.default_index,
        label: "Next view", aria_label: "Fixture views", header: rsx! { "Motor" }, footer: rsx! { "Service" },
        onbeforetoggle: move |request: FlipTileChangeEvent| {
            requests.before.borrow_mut().push((request.previous,request.next));
            if requests.veto.get() { request.clone().prevent_default(); assert!(request.default_prevented()); }
        },
        ontoggle: move |next| data.accepted.borrow_mut().push(next),
    } }
}
#[component]
fn LocalPage(id: &'static str, mounts: Rc<Cell<usize>>) -> Element {
    use_hook(|| {
        mounts.set(mounts.get() + 1);
    });
    let mut edited = use_signal(|| 0usize);
    rsx! { FlipTileContent {
        button { "data-local-page": id, "data-local-state": format!("{id}:{}",edited()),
            onclick: move |_| edited.set(edited()+1), "Edit {id}"
        }
    } }
}
struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for Mouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Mouse {
    fn modifiers(&self) -> dioxus::html::Modifiers {
        dioxus::html::Modifiers::empty()
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
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
struct Mount(Rc<Cell<usize>>);
impl dioxus::html::RenderedElementBacking for Mount {
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
struct Animation(String);
impl dioxus::html::HasAnimationData for Animation {
    fn animation_name(&self) -> String {
        self.0.clone()
    }
    fn pseudo_element(&self) -> String {
        String::new()
    }
    fn elapsed_time(&self) -> f32 {
        0.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                    let _=event; converter!(@data $group,event)
                })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Focus,$event:ident) => { dioxus::html::FocusData::new(Focus) };
        (@data Mounted,$event:ident) => { MountedData::new($event.downcast::<Mount>().unwrap().clone()) };
        (@data Animation,$event:ident) => { dioxus::html::AnimationData::new(Animation($event.downcast::<String>().unwrap().clone())) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected flip tile test event") };
    }
dioxus::html::with_html_event_groups!(converter);
#[derive(Default)]
struct Targets {
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    listeners: BTreeMap<String, Vec<dioxus::core::ElementId>>,
}
impl Targets {
    fn collect(&mut self, changes: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in changes.edits {
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
                dioxus::core::Mutation::NewEventListener { name } => {
                    self.listeners.entry(name).or_default().push(current)
                }
                _ => {}
            }
        }
    }
    fn attr(&self, name: &str, value: &str) -> dioxus::core::ElementId {
        dioxus::core::ElementId::from_raw(
            self.attrs
                .iter()
                .find_map(|(id, attrs)| {
                    (attrs.get(name).map(String::as_str) == Some(value)).then_some(*id)
                })
                .unwrap_or_else(|| panic!("Missing {name}={value}")),
        )
    }
    fn listener(&self, name: &str) -> dioxus::core::ElementId {
        self.listeners[name][0]
    }
}
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    data: Box<dyn std::any::Any>,
) {
    dom.runtime().handle_event(
        name,
        Event::new(
            Rc::new(dioxus::html::PlatformEventData::new(data)),
            name != "mounted",
        ),
        id,
    );
}
fn settle(dom: &mut VirtualDom, targets: &mut Targets) {
    for _ in 0..4 {
        targets.collect(dom.render_immediate_to_vec());
        dom.process_events();
    }
}
fn refresh(dom: &mut VirtualDom, targets: &mut Targets) {
    dom.mark_dirty(ScopeId::APP);
    settle(dom, targets);
}
fn end(dom: &mut VirtualDom, targets: &mut Targets, name: &str) {
    dispatch(
        dom,
        "animationend",
        targets.listener("animationend"),
        Box::new(name.to_string()),
    );
    settle(dom, targets);
}
fn complete(dom: &mut VirtualDom, targets: &mut Targets) {
    end(dom, targets, "ix-pattern-flip-out");
    end(dom, targets, "ix-pattern-flip-in");
}
fn opening_with<'a>(html: &'a str, attr: &str, value: &str) -> &'a str {
    html.split('<')
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| {
            tag.contains(&format!("{attr}=\"{value}\""))
                || tag.contains(&format!("{attr}={value} "))
        })
        .unwrap_or_else(|| panic!("Missing {attr}={value}: {html}"))
}
fn visible(html: &str, id: &str) -> bool {
    !opening_with(html, "data-flip-tile-page", id).contains(" hidden")
}
fn phase_is(dom: &VirtualDom, phase: &str) -> bool {
    let html = dioxus_ssr::render(dom);
    html.contains(&format!("data-phase=\"{phase}\""))
        || html.contains(&format!("data-phase={phase}"))
}

#[test]
fn flip_tile_actual_toggle_veto_held_default_and_latest_animation_target() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets);
    let eye = targets.attr("aria-label", "Next view");
    data.veto.set(true);
    dispatch(&dom, "click", eye, Box::new(()));
    settle(&mut dom, &mut targets);
    assert!(data.accepted.borrow().is_empty());
    assert!(phase_is(&dom, "idle"));
    data.veto.set(false);
    dispatch(&dom, "click", eye, Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(*data.accepted.borrow(), [1]);
    assert!(phase_is(&dom, "idle"));
    assert!(
        visible(&dioxus_ssr::render(&dom), "overview"),
        "held index remains authoritative"
    );
    data.config.borrow_mut().index = Some(1);
    refresh(&mut dom, &mut targets);
    assert!(phase_is(&dom, "outgoing"));
    let busy_html = dioxus_ssr::render(&dom);
    let busy_eye = opening_with(&busy_html, "aria-label", "Next view");
    assert!(busy_eye.contains("aria-disabled=\"true\"") || busy_eye.contains("aria-disabled=true"));
    assert!(
        !busy_eye.contains(" disabled"),
        "busy eye preserves physical keyboard focus"
    );
    dispatch(&dom, "click", eye, Box::new(()));
    settle(&mut dom, &mut targets);
    assert_eq!(data.before.borrow().len(), 2, "busy eye never requests");
    data.config.borrow_mut().index = Some(2);
    refresh(&mut dom, &mut targets);
    complete(&mut dom, &mut targets);
    assert!(visible(&dioxus_ssr::render(&dom), "events"));
    assert!(phase_is(&dom, "idle"));
    data.config.borrow_mut().index = None;
    refresh(&mut dom, &mut targets);
    complete(&mut dom, &mut targets);
    assert!(visible(&dioxus_ssr::render(&dom), "overview"));
    dispatch(&dom, "click", eye, Box::new(()));
    settle(&mut dom, &mut targets);
    complete(&mut dom, &mut targets);
    assert!(visible(&dioxus_ssr::render(&dom), "service"));
    assert_eq!(*data.before.borrow(), [(0, 1), (0, 1), (0, 1)]);
    assert_eq!(*data.accepted.borrow(), [1, 1]);
}

#[test]
fn flip_tile_retained_composed_state_survives_turns_and_keyed_reordering() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    data.config.borrow_mut().index = None;
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    settle(&mut dom, &mut targets);
    assert_eq!(data.mounts.get(), 3);
    assert_eq!(
        dioxus_ssr::render(&dom)
            .matches("data-flip-tile-page=")
            .count(),
        3
    );
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Next view"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    complete(&mut dom, &mut targets);
    dispatch(
        &dom,
        "click",
        targets.attr("data-local-page", "service"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    assert!(dioxus_ssr::render(&dom).contains("data-local-state=\"service:1\""));
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Next view"),
        Box::new(()),
    );
    settle(&mut dom, &mut targets);
    complete(&mut dom, &mut targets);
    assert!(!visible(&dioxus_ssr::render(&dom), "service"));
    data.config.borrow_mut().ids = vec!["events", "service", "overview"];
    refresh(&mut dom, &mut targets);
    assert_eq!(data.mounts.get(), 3, "reorder never remounts keyed content");
    assert!(dioxus_ssr::render(&dom).contains("data-local-state=\"service:1\""));
    data.config.borrow_mut().ids = vec!["service"];
    refresh(&mut dom, &mut targets);
    assert!(phase_is(&dom, "idle"));
    assert!(visible(&dioxus_ssr::render(&dom), "service"));
    assert!(dioxus_ssr::render(&dom).contains("data-local-state=\"service:1\""));
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Next view"),
        Box::new(()),
    );
    assert_eq!(data.accepted.borrow().len(), 2, "single page eye is inert");
    data.config.borrow_mut().ids.clear();
    refresh(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("data-flip-tile-page="));
    assert!(html.contains("Content 0 of 0"));
    assert!(phase_is(&dom, "idle"));
    dispatch(
        &dom,
        "click",
        targets.attr("aria-label", "Next view"),
        Box::new(()),
    );
    assert_eq!(data.accepted.borrow().len(), 2);
}

#[test]
fn flip_tile_actual_page_change_returns_inside_focus_to_live_eye_or_article_only() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Fixture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = Targets::default();
    targets.collect(dom.rebuild_to_vec());
    let article_calls = Rc::new(Cell::new(0));
    let eye_calls = Rc::new(Cell::new(0));
    dispatch(
        &dom,
        "mounted",
        targets.attr("aria-label", "Fixture views"),
        Box::new(Mount(article_calls.clone())),
    );
    dispatch(
        &dom,
        "mounted",
        targets.attr("aria-label", "Next view"),
        Box::new(Mount(eye_calls.clone())),
    );
    settle(&mut dom, &mut targets);
    dispatch(&dom, "focusin", targets.listener("focusin"), Box::new(()));
    data.config.borrow_mut().index = Some(1);
    refresh(&mut dom, &mut targets);
    end(&mut dom, &mut targets, "ix-pattern-flip-out");
    assert_eq!(
        eye_calls.get(),
        1,
        "busy eye remains physically available for focus return"
    );
    end(&mut dom, &mut targets, "ix-pattern-flip-in");
    dispatch(&dom, "focusout", targets.listener("focusout"), Box::new(()));
    data.config.borrow_mut().index = Some(2);
    refresh(&mut dom, &mut targets);
    complete(&mut dom, &mut targets);
    assert_eq!(article_calls.get(), 0);
    assert_eq!(eye_calls.get(), 1, "outside focus stays outside");
    dispatch(&dom, "focusin", targets.listener("focusin"), Box::new(()));
    data.config.borrow_mut().ids = vec!["events", "service", "overview"];
    refresh(&mut dom, &mut targets);
    assert_eq!(
        eye_calls.get(),
        2,
        "idle reorder returns focus to enabled stable eye"
    );
    dispatch(&dom, "focusin", targets.listener("focusin"), Box::new(()));
    data.config.borrow_mut().ids.clear();
    refresh(&mut dom, &mut targets);
    assert_eq!(
        article_calls.get(),
        1,
        "empty collection returns focus before removed content loses ownership"
    );
}
