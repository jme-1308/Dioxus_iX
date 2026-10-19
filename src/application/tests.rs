mod menu_overlay_focus_tests {
    include!("menu_overlay_focus_tests.rs");
}
mod menu_mounted_focus_tests {
    include!("menu_mounted_focus_tests.rs");
}
mod menu_cue_tests {
    include!("menu_cue_tests.rs");
}
mod menu_lifetime_tests {
    include!("menu_lifetime_tests.rs");
}
mod menu_scope_tests {
    include!("menu_scope_tests.rs");
}

use super::*;
struct MenuTestMouse;
impl dioxus::html::InteractionLocation for MenuTestMouse {
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
impl dioxus::html::InteractionElementOffset for MenuTestMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for MenuTestMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for MenuTestMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for MenuTestMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct MenuTestPointer(String);
impl dioxus::html::InteractionLocation for MenuTestPointer {
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
impl dioxus::html::InteractionElementOffset for MenuTestPointer {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for MenuTestPointer {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for MenuTestPointer {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasPointerData for MenuTestPointer {
    fn pointer_id(&self) -> i32 {
        1
    }
    fn width(&self) -> f64 {
        1.0
    }
    fn height(&self) -> f64 {
        1.0
    }
    fn pressure(&self) -> f32 {
        0.0
    }
    fn tangential_pressure(&self) -> f32 {
        0.0
    }
    fn tilt_x(&self) -> i32 {
        0
    }
    fn tilt_y(&self) -> i32 {
        0
    }
    fn twist(&self) -> i32 {
        0
    }
    fn pointer_type(&self) -> String {
        self.0.clone()
    }
    fn is_primary(&self) -> bool {
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct MenuTestForm(String);
impl dioxus::html::HasFileData for MenuTestForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for MenuTestForm {
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
struct MenuTestKeyboard(Key, Modifiers);
impl dioxus::html::ModifiersInteraction for MenuTestKeyboard {
    fn modifiers(&self) -> Modifiers {
        self.1
    }
}
impl dioxus::html::HasKeyboardData for MenuTestKeyboard {
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
struct MenuTestFocus;
impl dioxus::html::HasFocusData for MenuTestFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Default)]
pub(super) struct AppSwitchOpeningCapture(pub(super) Rc<RefCell<Option<AppSwitchOpeningState>>>);
#[derive(Clone, Default)]
pub(super) struct AppSwitchCapture(pub(super) Rc<RefCell<Option<AppSwitchNavigation>>>);
#[derive(Clone, Default)]
pub(super) struct MenuCapture(pub(super) Rc<RefCell<Option<MenuRoving>>>);
#[derive(Clone)]
struct MenuFixtureConfig {
    capture: MenuCapture,
    visible: bool,
    alpha: bool,
    alpha_disabled: bool,
    child_active: bool,
    child_present: bool,
    category_children: bool,
    category_present: bool,
    extra_menu: bool,
    label: String,
    hint: String,
    update: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    activations: usize,
    bubbled: usize,
}
#[component]
fn MenuHoverOtherMenu() -> Element {
    use_context_provider(MenuCapture::default);
    rsx! { Menu {start_expanded:true,pinned:true,application_name:"Other menu",
        MenuCategory {label:"Other category",MenuItem {label:"Other child"}}
    } }
}
fn menu_navigation_fixture(config: Rc<RefCell<MenuFixtureConfig>>) -> Element {
    let settings = config.borrow().clone();
    let capture = settings.capture;
    use_context_provider(|| capture);
    config.borrow_mut().update = Some(dioxus::core::schedule_update());
    let activated = config.clone();
    let child_activated = config.clone();
    let bubbled = config.clone();
    rsx! { div { onkeydown: move |_| bubbled.borrow_mut().bubbled += 1,
        if settings.visible {
            Menu {
                start_expanded:true,pinned:true,label:settings.label,navigation_hint:settings.hint,
                application_name:"Example application",enable_toggle_theme:true,
                settings:rsx!{MenuSettings {"Settings content"}},
                about:rsx!{MenuAbout {"About content"}},
                bottom:rsx!{MenuItem {label:"Footer"}},
                input {id:"menu-search-fixture",aria_label:"Navigation search",onkeydown:move |_|{},onfocus:move |_|{}}
                if settings.alpha { MenuItem {label:"Alpha",disabled:settings.alpha_disabled,onclick:move |_|activated.borrow_mut().activations+=1} }
                MenuItem {label:"Disabled",disabled:true}
                a {href:"#wrapped", MenuItem {label:"Link"}}
                MenuItem {label:"Disabled link",href:"#disabled",disabled:true}
                if settings.category_present {MenuCategory {label:"Category",
                    input {id:"category-search-fixture",aria_label:"Category search",onkeydown:move |_|{},onfocus:move |_|{}}
                    if settings.category_children {
                    if settings.child_present {MenuItem {label:"Child",active:settings.child_active,onclick:move |_|child_activated.borrow_mut().activations+=1}}
                    MenuItem {label:"Disabled child",disabled:true}
                    a {href:"#category-wrapped",MenuItem {label:"Two"}}
                    }
                }}
                MenuItem {label:"Omega"}
                MenuCategory {label:"Second category",MenuItem {label:"Solo"}}
            }
        }
        if settings.extra_menu { MenuHoverOtherMenu {} }
    } }
}
#[derive(Clone, Default)]
struct MenuTestMount {
    id: usize,
    focuses: Rc<RefCell<Vec<usize>>>,
    started: Rc<Cell<bool>>,
    ready: Rc<Cell<bool>>,
    waiter: Rc<RefCell<Option<std::task::Waker>>>,
    read_dropped: Rc<Cell<bool>>,
}
struct MenuReadLifetime(Rc<Cell<bool>>);
impl Drop for MenuReadLifetime {
    fn drop(&mut self) {
        self.0.set(true);
    }
}
impl dioxus::html::RenderedElementBacking for MenuTestMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        let state = self.clone();
        let lifetime = MenuReadLifetime(state.read_dropped.clone());
        Box::pin(async move {
            let _lifetime = lifetime;
            assert!(focus);
            std::future::poll_fn(|context| {
                state.started.set(true);
                if state.ready.get() {
                    std::task::Poll::Ready(())
                } else {
                    *state.waiter.borrow_mut() = Some(context.waker().clone());
                    std::task::Poll::Pending
                }
            })
            .await;
            state.focuses.borrow_mut().push(state.id);
            Ok(())
        })
    }
}
struct AppSwitchTestToggle;
impl dioxus::html::HasToggleData for AppSwitchTestToggle {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct MenuTestConverter;
macro_rules! menu_test_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for MenuTestConverter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;menu_test_converter!(@data $group,event)})*
            }
        };
        (@data Mouse,$event:ident) => {dioxus::html::MouseData::new(MenuTestMouse)};
        (@data Keyboard,$event:ident) => {{
            let (key,modifiers)=match $event.downcast::<(Key,Modifiers)>() {Some((key,modifiers))=>(key.clone(),*modifiers),None=>($event.downcast::<Key>().unwrap().clone(),Modifiers::empty())};
            dioxus::html::KeyboardData::new(MenuTestKeyboard(key,modifiers))
        }};
        (@data Pointer,$event:ident) => {dioxus::html::PointerData::new(MenuTestPointer($event.downcast::<String>().unwrap().clone()))};
        (@data Form,$event:ident) => {dioxus::html::FormData::new(MenuTestForm($event.downcast::<String>().unwrap().clone()))};
        (@data Focus,$event:ident) => {dioxus::html::FocusData::new(MenuTestFocus)};
        (@data Toggle,$event:ident) => {dioxus::html::ToggleData::new(AppSwitchTestToggle)};
        (@data Mounted,$event:ident) => {MountedData::new($event.downcast::<MenuTestMount>().unwrap().clone())};
        (@data $other:ident,$event:ident) => {panic!("Unexpected menu test event")};
    }
dioxus::html::with_html_event_groups!(menu_test_converter);
fn menu_dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: impl std::any::Any,
) -> (bool, bool) {
    let data: Rc<dyn std::any::Any> =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
    // Native focus and mounted notifications do not bubble.
    let event = Event::new(
        data,
        !matches!(
            name,
            "focus" | "blur" | "mounted" | "pointerenter" | "pointerleave"
        ),
    );
    dom.runtime().handle_event(name, event.clone(), id);
    (!event.default_action_enabled(), event.propagates())
}
#[derive(Clone, Default)]
struct GenericMenuItemCapture {
    calls: Rc<Cell<usize>>,
    focus_calls: Rc<Cell<usize>>,
    blur_calls: Rc<Cell<usize>>,
    navigation: Rc<RefCell<Option<NavigationContext>>>,
    category: Rc<RefCell<Option<CategoryNavigation>>>,
}
#[component]
fn GenericMenuItemConsumer() -> Element {
    let capture = use_context::<GenericMenuItemCapture>();
    *capture.navigation.borrow_mut() = try_consume_context::<NavigationContext>();
    let focused = capture.focus_calls.clone();
    let blurred = capture.blur_calls.clone();
    rsx! {MenuItem {id:"mixed-menu-consumer",label:"Mixed consumer",close_on_select:false,
        onfocus:move |_|focused.set(focused.get()+1),
        onblur:move |_|blurred.set(blurred.get()+1),
        onclick:move |_|capture.calls.set(capture.calls.get()+1)
    }}
}
#[component]
fn GenericMenuCategoryChild() -> Element {
    let capture = use_context::<GenericMenuItemCapture>();
    *capture.category.borrow_mut() =
        try_consume_context::<NavigationContext>().and_then(|context| context.nested);
    rsx! {
        MenuItem {label:"Mixed child",close_on_select:false}
        MenuItem {id:"custom-category-child",label:"Custom child",close_on_select:false}
    }
}
fn generic_menu_item_fixture(capture: GenericMenuItemCapture) -> Element {
    use_context_provider(|| capture);
    rsx! {Menu {start_expanded:false,
        MenuItem {id:"ordinary-menu-item",label:"Ordinary item",close_on_select:false}
        crate::Dropdown {label:"Mixed source hosts",
            GenericMenuItemConsumer {}
            crate::SelectItem {id:"mixed-select",value:"Mixed option"}
            MenuItem {id:"property-menu-item",label:"Disabled property",disabled:true,tabindex:"-1",style:"color: red",close_on_select:false}
            MenuCategory {id:"consumer-category",label:"Mixed category",style:"color: blue",GenericMenuCategoryChild {}}
        }
    }}
}
fn item_opening_tag<'a>(html: &'a str, id: &str) -> &'a str {
    let at = html
        .find(&format!("id=\"{id}\""))
        .unwrap_or_else(|| panic!("missing id {id}"));
    let from = html[..at].rfind('<').unwrap();
    let to = at + html[at..].find('>').unwrap() + 1;
    &html[from..to]
}
fn source_item_dom(
    capture: GenericMenuItemCapture,
) -> (VirtualDom, BTreeMap<String, dioxus::core::ElementId>) {
    let mut dom = VirtualDom::new_with_props(generic_menu_item_fixture, capture);
    let mut current = dioxus::core::ElementId::ROOT;
    let mut ids = BTreeMap::new();
    for mutation in dom.rebuild_to_vec().edits {
        match mutation {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" => {
                ids.insert(value, current);
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(_),
                ..
            } if name == "data-ix-overlay-transport" => {
                ids.insert("$overlay-transport".to_string(), current);
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "class" && value.contains("ix-select-item-option") => {
                ids.insert("$select-option".to_string(), current);
            }
            _ => {}
        }
    }
    (dom, ids)
}
#[test]
fn generic_menu_source_hosts_preserve_raw_attributes_and_category_anchor() {
    let (dom, _) = source_item_dom(GenericMenuItemCapture::default());
    let html = dioxus_ssr::render(&dom);
    let host = item_opening_tag(&html, "property-menu-item");
    assert!(host.starts_with("<div "));
    assert!(host.contains("data-ix-item-kind=\"menu\""));
    assert!(host.contains("data-ix-item-focusable=\"false\""));
    assert!(host.contains("tabindex=\"-1\""));
    assert!(host.contains("color: red"));
    assert!(
        !host.contains(" disabled"),
        "property-only disabled is not a source host attribute"
    );
    let control = item_opening_tag(&html, "property-menu-item-control");
    assert!(control.starts_with("<button ") && control.contains("data-ix-item-control"));
    assert!(control.contains("aria-disabled=\"true\""));
    assert!(
        !control.contains(" disabled"),
        "source generated button remains natively focusable"
    );
    let category = item_opening_tag(&html, "consumer-category");
    assert!(category.contains("data-ix-item-kind=\"category\""));
    assert!(category.contains("data-ix-overlay-anchor-id=\"consumer-category\""));
    assert!(category.contains("anchor-name:--ix-menu-category-anchor-"));
    assert!(category.contains("color: blue"));
    assert!(
        !category.contains("data-active"),
        "Category parent projection does not invent an outer marker"
    );
    let ordinary = item_opening_tag(&html, "ordinary-menu-item");
    assert!(ordinary.starts_with("<button ") && !ordinary.contains("data-ix-item-kind"));
}
#[test]
fn rendered_focus_events_update_menu_select_and_category_cues_without_inventory() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(MenuTestConverter));
    let capture = GenericMenuItemCapture::default();
    let (mut dom, ids) = source_item_dom(capture.clone());
    let category = capture.category.borrow().clone().unwrap();
    let runtime = category
        .0
        .overlay
        .borrow()
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap();
    let parent = *ids
        .iter()
        .find(|(id, _)| id.starts_with("ix-menu-category-parent-"))
        .unwrap()
        .1;
    for target in [
        parent,
        ids["mixed-menu-consumer-control"],
        ids["mixed-select"],
    ] {
        menu_dispatch(&dom, "mounted", target, MenuTestMount::default());
    }
    menu_dispatch(
            &dom, "input", ids["$overlay-transport"],
            serde_json::json!({"owner":runtime.owner_id(),"document":format!("rendered-focus-cue-{}",runtime.owner_id()),"sequence":1,"kind":"modality","input_kind":"keydown","key":"Tab","nodes":[]}).to_string(),
        );
    assert!(runtime.keyboard_mode());
    menu_dispatch(&dom, "focus", parent, ());
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    let html = dioxus_ssr::render(&dom);
    let outer = item_opening_tag(&html, "consumer-category");
    assert!(!outer.contains("ix-focused") && !outer.contains("data-active"));
    let parent_tag = html
        .split('>')
        .find(|tag| tag.contains("data-ix-item-part=\"category-parent\""))
        .unwrap();
    assert!(parent_tag.contains("ix-focused") && parent_tag.contains("data-active=\"true\""));
    assert!(
        runtime.item_visual_key().is_none(),
        "Category selects its parent part only"
    );

    menu_dispatch(&dom, "focus", ids["mixed-menu-consumer-control"], ());
    menu_dispatch(&dom, "blur", parent, ());
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    let html = dioxus_ssr::render(&dom);
    assert!(item_opening_tag(&html, "mixed-menu-consumer").contains("ix-focused"));
    let parent_tag = html
        .split('>')
        .find(|tag| tag.contains("data-ix-item-part=\"category-parent\""))
        .unwrap();
    assert!(!parent_tag.contains("ix-focused"));
    assert_eq!(
        capture.focus_calls.get(),
        1,
        "consumer focus fires once on the physical control"
    );
    menu_dispatch(&dom, "blur", ids["mixed-menu-consumer-control"], ());
    assert!(runtime.item_visual_key().is_none());
    assert_eq!(capture.blur_calls.get(), 1);

    menu_dispatch(&dom, "focus", ids["mixed-select"], ());
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert!(item_opening_tag(&dioxus_ssr::render(&dom), "mixed-select").contains("ix-focused"));
    menu_dispatch(&dom, "focus", ids["$select-option"], ());
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    let html = dioxus_ssr::render(&dom);
    let option = html
        .split('>')
        .find(|tag| tag.contains("data-ix-item-part=\"select-option\""))
        .unwrap();
    assert!(option.contains("ix-focused") && option.contains("data-active=\"true\""));
    menu_dispatch(&dom, "blur", ids["$select-option"], ());
    assert!(runtime.item_visual_key().is_none());
    menu_dispatch(&dom, "focus", ids["mixed-select"], ());
    menu_dispatch(&dom, "input", ids["$overlay-transport"],
            serde_json::json!({"owner":runtime.owner_id(),"document":format!("rendered-focus-cue-{}",runtime.owner_id()),"sequence":2,"kind":"modality","input_kind":"mousedown","nodes":[]}).to_string());
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
    assert!(runtime.item_visual_key().is_none());
    menu_dispatch(&dom, "focus", ids["mixed-select"], ());
    assert!(
        runtime.item_visual_key().is_none(),
        "pointer focus does not paint a keyboard cue"
    );
}
#[test]
fn generic_menu_host_click_delivers_real_consumer_once_without_inner_side_effects() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(MenuTestConverter));
    let capture = GenericMenuItemCapture::default();
    let (dom, ids) = source_item_dom(capture.clone());
    let mut navigation = capture.navigation.borrow().clone().unwrap();
    navigation.category.set(Some(usize::MAX));
    menu_dispatch(&dom, "click", ids["mixed-menu-consumer"], ());
    assert_eq!(capture.calls.get(), 1);
    assert_eq!(
        *navigation.category.peek(),
        Some(usize::MAX),
        "host click must not descend to the generated native control"
    );
    menu_dispatch(&dom, "click", ids["mixed-menu-consumer-control"], ());
    assert_eq!(
        capture.calls.get(),
        2,
        "native inner click bubbles one original consumer event"
    );
    assert_eq!(
        *navigation.category.peek(),
        None,
        "actual inner control preserves its existing adapter"
    );
}
#[test]
fn generic_category_children_publish_own_hidden_state_and_keep_native_child_lookup() {
    let capture = GenericMenuItemCapture::default();
    let (mut dom, _) = source_item_dom(capture.clone());
    let category = capture.category.borrow().clone().unwrap();
    let child_id = *category.0.model.borrow().children.keys().next().unwrap();
    let host_id = format!("ix-menu-category-child-{}-{child_id}", category.0.id);
    let html = dioxus_ssr::render(&dom);
    assert!(item_opening_tag(&html, &host_id).contains(" hidden"));
    assert!(
        item_opening_tag(&html, &format!("{host_id}-control")).contains("data-menu-category-child")
    );
    assert!(
        category
            .0
            .model
            .borrow()
            .children
            .values()
            .any(|child| child.host_id.as_deref() == Some("custom-category-child"))
    );
    assert!(item_opening_tag(&html, "custom-category-child").contains(" hidden"));
    category.present_state();
    for _ in 0..5 {
        let _ = dom.render_immediate_to_vec();
    }
    let html = dioxus_ssr::render(&dom);
    assert!(
        !item_opening_tag(&html, &host_id).contains(" hidden"),
        "source supplied child is independently queryable while its category is open"
    );
    assert!(!item_opening_tag(&html, "custom-category-child").contains(" hidden"));
    category.close_flyout();
    for _ in 0..5 {
        let _ = dom.render_immediate_to_vec();
    }
    assert!(item_opening_tag(&dioxus_ssr::render(&dom), &host_id).contains(" hidden"));
}

struct MenuHarness {
    dom: VirtualDom,
    config: Rc<RefCell<MenuFixtureConfig>>,
    context: MenuRoving,
    input: dioxus::core::ElementId,
    bar: dioxus::core::ElementId,
    toggle: dioxus::core::ElementId,
    controls: BTreeMap<String, (dioxus::core::ElementId, usize)>,
    focuses: Rc<RefCell<Vec<usize>>>,
    attributes: BTreeMap<usize, BTreeMap<String, String>>,
}
impl MenuHarness {
    fn new() -> Self {
        Self::with_active(false)
    }
    fn with_active(child_active: bool) -> Self {
        let mut result = Self::unsettled(child_active);
        result.settle();
        result
    }
    fn unsettled(child_active: bool) -> Self {
        Self::unsettled_with_peer(child_active, false)
    }
    fn unsettled_with_peer(child_active: bool, extra_menu: bool) -> Self {
        let config = Rc::new(RefCell::new(MenuFixtureConfig {
            capture: MenuCapture::default(),
            visible: true,
            alpha: true,
            alpha_disabled: false,
            child_active,
            child_present: true,
            category_children: true,
            category_present: true,
            extra_menu,
            label: "Application Navigation".into(),
            hint: "Use Up and Down arrow keys to navigate between menu items".into(),
            update: None,
            activations: 0,
            bubbled: 0,
        }));
        dioxus::html::set_event_converter(Box::new(MenuTestConverter));
        let mut dom = VirtualDom::new_with_props(menu_navigation_fixture, config.clone());
        let mutations = dom.rebuild_to_vec();
        let mut current = dioxus::core::ElementId::ROOT;
        let mut attributes = BTreeMap::<usize, BTreeMap<String, String>>::new();
        let mut inputs = Vec::new();
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
                    attributes
                        .entry(current.raw())
                        .or_default()
                        .insert(name, value);
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "input" => {
                    inputs.push(current)
                }
                _ => {}
            }
        }
        let input = *inputs
            .iter()
            .find(|id| {
                attributes
                    .get(&id.raw())
                    .is_some_and(|attrs| attrs.contains_key("data-menu-observer"))
            })
            .unwrap();
        let target = |label: &str| {
            attributes
                .iter()
                .find(|(_, attrs)| attrs.get("aria-label").is_some_and(|value| value == label))
                .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
                .unwrap_or_else(|| panic!("missing menu target {label}"))
        };
        let bar = target("Application Navigation");
        let toggle = target("Collapse");
        let controls = attributes
            .iter()
            .filter_map(|(id, attrs)| {
                Some((
                    attrs.get("aria-label")?.clone(),
                    (
                        dioxus::core::ElementId::from_raw(*id),
                        attrs
                            .get("data-menu-control")
                            .or_else(|| attrs.get("data-menu-category-child"))?
                            .parse()
                            .unwrap(),
                    ),
                ))
            })
            .collect();
        let context = config.borrow().capture.0.borrow().as_ref().unwrap().clone();
        let focuses = Rc::new(RefCell::new(vec![]));
        let result = Self {
            dom,
            config,
            context,
            input,
            bar,
            toggle,
            controls,
            focuses,
            attributes,
        };
        for (target, id) in result.controls.values() {
            let mount = MenuTestMount {
                id: *id,
                focuses: result.focuses.clone(),
                ..Default::default()
            };
            mount.ready.set(true);
            menu_dispatch(&result.dom, "mounted", *target, mount);
        }
        result
    }
    fn settle(&mut self) {
        for _ in 0..3 {
            let mutations = self.dom.render_immediate_to_vec();
            let mut current = dioxus::core::ElementId::ROOT;
            for edit in mutations.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
                    dioxus::core::Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } => {
                        self.attributes
                            .entry(current.raw())
                            .or_default()
                            .insert(name, value);
                    }
                    _ => {}
                }
            }
        }
    }
    fn update(&mut self) {
        let update = self.config.borrow().update.clone().unwrap();
        update();
        self.settle();
    }
    fn id(&self, label: &str) -> usize {
        self.controls[label].1
    }
    fn target(&self, label: &str) -> dioxus::core::ElementId {
        self.controls[label].0
    }
    fn facts(&self, labels: &[&str]) -> Vec<MenuDomItem> {
        labels
            .iter()
            .map(|label| MenuDomItem {
                id: self.id(label),
                disabled: label.starts_with("Disabled"),
                hidden: false,
                display: String::new(),
                grandparent_display: String::new(),
            })
            .collect()
    }
    fn observe(
        &mut self,
        kind: &str,
        key: &str,
        origin: Option<usize>,
        excluded: bool,
        items: Vec<MenuDomItem>,
    ) -> (bool, bool) {
        let owner = self.context.model.borrow().owner.clone();
        self.raw(serde_json::json!({"owner":owner,"kind":kind,"key":key,"origin":origin,"excluded":excluded,"items":items}))
    }
    // Legacy-shaped fixture inputs describe an interaction plus current
    // read-only facts. Only inventory facts reach the observer; keyboard,
    // click and focus events always enter the real Dioxus listeners.
    fn raw(&mut self, message: serde_json::Value) -> (bool, bool) {
        let kind = message["kind"].as_str().unwrap_or_default();
        let owner = self.context.model.borrow().owner.clone();
        let current_owner = message["owner"].as_str() == Some(owner.as_str());
        let category = message["category"].as_u64().map(|id| id as usize);
        let live_category = category.is_some_and(|id| {
            self.context
                .model
                .borrow()
                .categories
                .get(&id)
                .and_then(|category| category.upgrade())
                .is_some()
        });
        let key = match message["key"].as_str().unwrap_or_default() {
            "ArrowDown" => Key::ArrowDown,
            "ArrowUp" => Key::ArrowUp,
            "ArrowLeft" => Key::ArrowLeft,
            "ArrowRight" => Key::ArrowRight,
            "Home" => Key::Home,
            "End" => Key::End,
            "Enter" => Key::Enter,
            "Escape" => Key::Escape,
            "Tab" => Key::Tab,
            " " => Key::Character(" ".into()),
            _ => Key::Character("Unidentified".into()),
        };
        let interaction = current_owner
            && (kind == "key"
                || kind == "focus"
                || (live_category && matches!(kind, "category-key" | "category-click")));
        if interaction {
            let mut facts = message.clone();
            facts["kind"] = serde_json::json!(if category.is_some() {
                "category-order"
            } else if self.context.model.borrow().native {
                "order"
            } else {
                "ready"
            });
            facts["sequence"] = serde_json::json!(0);
            menu_dispatch(&self.dom, "input", self.input, facts.to_string());
            self.settle();
            let control = |id: usize| {
                self.attributes
                    .iter()
                    .find(|(_, attributes)| {
                        attributes
                            .get("data-menu-control")
                            .or_else(|| attributes.get("data-menu-category-child"))
                            .and_then(|value| value.parse::<usize>().ok())
                            == Some(id)
                    })
                    .map(|(target, _)| dioxus::core::ElementId::from_raw(*target))
            };
            let target = if let Some(category) = category {
                message["child"]
                    .as_u64()
                    .map(|id| id as usize)
                    .or(Some(category))
                    .and_then(control)
            } else if kind == "key" && message["excluded"].as_bool() == Some(true) {
                self.controls.get("Child").map(|(target, _)| *target)
            } else {
                message["origin"]
                    .as_u64()
                    .map(|id| id as usize)
                    .and_then(control)
                    .or(Some(self.bar))
            };
            if let Some(target) = target {
                if kind == "focus" {
                    let flags = menu_dispatch(&self.dom, "focus", target, ());
                    self.settle();
                    return flags;
                }
                if kind == "category-click" {
                    let flags = menu_dispatch(&self.dom, "click", target, ());
                    self.settle();
                    return flags;
                }
                return self.key(target, key);
            }
            return (false, true);
        }
        let flags = menu_dispatch(&self.dom, "input", self.input, message.to_string());
        self.settle();
        flags
    }
    fn key(&mut self, target: dioxus::core::ElementId, key: Key) -> (bool, bool) {
        if target == self.bar {
            menu_dispatch(&self.dom, "focus", target, ());
        }
        let flags = menu_dispatch(&self.dom, "keydown", target, key);
        self.settle();
        flags
    }
    fn toggle_expanded(&mut self) {
        menu_dispatch(&self.dom, "click", self.toggle, ());
        self.settle();
    }
    fn click(&mut self, label: &str) -> (bool, bool) {
        let flags = menu_dispatch(&self.dom, "click", self.target(label), ());
        self.settle();
        flags
    }
    fn modified_key(&mut self, label: &str, key: Key, modifiers: Modifiers) -> (bool, bool) {
        let flags = menu_dispatch(&self.dom, "keydown", self.target(label), (key, modifiers));
        self.settle();
        flags
    }
    fn focused_category_target(&self, label: &str) -> Option<String> {
        let focused = self.focuses.borrow().last().copied()?;
        let category = self.category(label);
        category
            .0
            .model
            .borrow()
            .children
            .get(&focused)?
            .host_id
            .clone()
    }
    fn category(&self, label: &str) -> CategoryNavigation {
        CategoryNavigation(
            self.context.model.borrow().categories[&self.id(label)]
                .upgrade()
                .unwrap(),
        )
    }
    fn category_facts(&self, labels: &[&str]) -> Vec<CategoryDomItem> {
        labels
            .iter()
            .map(|label| CategoryDomItem {
                id: self.id(label),
                disabled: label.starts_with("Disabled"),
                hidden: false,
                tabindex: Some("0".into()),
            })
            .collect()
    }
    fn category_event(
        &mut self,
        kind: &str,
        key: &str,
        category: &str,
        child: Option<&str>,
        sequence: u64,
        items: Vec<CategoryDomItem>,
    ) -> (bool, bool) {
        let owner = self.context.model.borrow().owner.clone();
        self.raw(serde_json::json!({"owner":owner,"kind":kind,"key":key,"category":self.id(category),"child":child.map(|label|self.id(label)),"category_parent":child.is_none(),"sequence":sequence,"shift":false,"category_items":items,"items":[]}))
    }
    fn category_surface(&self, label: &str, surface: &str) -> dioxus::core::ElementId {
        let id = self.id(label).to_string();
        let attribute = if surface == "host" {
            "data-menu-category-root"
        } else {
            "data-menu-category-panel"
        };
        self.attributes
            .iter()
            .find(|(_, attributes)| attributes.get(attribute) == Some(&id))
            .map(|(target, _)| dioxus::core::ElementId::from_raw(*target))
            .unwrap()
    }
    fn pointer(&mut self, label: &str, surface: &str, event: &str, pointer: &str) -> (bool, bool) {
        let target = self.category_surface(label, surface);
        let flags = menu_dispatch(&self.dom, event, target, pointer.to_string());
        self.settle();
        flags
    }
    fn focus_control(&mut self, label: &str) {
        menu_dispatch(&self.dom, "focus", self.target(label), ());
        menu_dispatch(&self.dom, "focusin", self.target(label), ());
        self.settle();
    }
    fn shared_modality(&mut self, label: &str, keyboard: bool) {
        let category = self.category(label);
        let node = category.0.overlay.borrow().as_ref().unwrap().clone();
        let runtime = node.runtime().unwrap();
        let owner = runtime.owner_id();
        let transport = self
            .attributes
            .iter()
            .find(|(_, attributes)| attributes.get("data-ix-overlay-transport") == Some(&owner))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap();
        menu_dispatch(&self.dom, "input", transport, serde_json::json!({
            "owner": owner, "document": format!("category-fixture-{owner}"),
            "sequence": 1, "kind":"modality", "input_kind":if keyboard {"keydown"} else {"mousedown"},
            "key":if keyboard {"Tab"} else {""},
            "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true}]
        }).to_string());
        self.settle();
        assert_eq!(runtime.keyboard_mode(), keyboard);
    }
    fn hover_clock(&self, label: &str) -> CategoryHoverClock {
        let clock = CategoryHoverClock::default();
        self.category(label).0.model.borrow_mut().hover_clock = Some(clock.clone());
        clock
    }
    fn advance_hover(&mut self, clock: &CategoryHoverClock, millis: u64) {
        clock.now.set(clock.now.get() + millis);
        let wake = clock.wake.borrow_mut().take();
        if let Some(wake) = wake {
            wake.wake();
        }
        self.settle();
    }
    fn last(&self) -> Option<usize> {
        self.context.model.borrow().last
    }
    fn tabs(&self) -> BTreeMap<usize, i32> {
        self.context.model.borrow().tabs.clone()
    }
    fn html(&self) -> String {
        dioxus_ssr::render(&self.dom)
    }
    fn release(&mut self, mount: &MenuTestMount) {
        mount.ready.set(true);
        if let Some(waiter) = mount.waiter.borrow_mut().take() {
            waiter.wake();
        }
        self.settle();
    }
}

#[test]
fn menu_navigation_has_separate_native_entry_labels_and_disabled_link_marker() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let html = harness.html();
    let toggle = html.find("ix-menu-toggle").unwrap();
    let bar = html.find("role=\"menubar\"").unwrap();
    assert!(toggle < bar);
    assert!(html.contains("aria-orientation=\"vertical\""));
    assert!(html.contains("aria-label=\"Example application\""));
    assert!(html.contains(
        "aria-description=\"Use Up and Down arrow keys to navigate between menu items\""
    ));
    assert!(harness.tabs().values().all(|value| *value == -1));
    let link = html
        .split("<a ")
        .filter_map(|element| element.split('>').next())
        .find(|element| element.contains("aria-label=\"Disabled link\""))
        .unwrap();
    assert!(
        link.split_ascii_whitespace()
            .any(|attribute| attribute == "disabled" || attribute.starts_with("disabled=")),
        "normal disabled link must carry the source eligibility attribute"
    );
    assert!(!link.contains("href="));
    assert!(link.contains("tabindex=\"-1\"") || link.contains("tabindex=-1"));
    assert_eq!(harness.key(harness.toggle, Key::ArrowDown), (false, true));
    assert_eq!(harness.last(), None);
    assert_eq!(harness.config.borrow().activations, 0);
    harness.config.borrow_mut().label = "Updated navigation".into();
    harness.config.borrow_mut().hint = "Updated hint".into();
    harness.update();
    assert!(harness.html().contains("aria-label=\"Updated navigation\""));
    assert!(harness.html().contains("aria-description=\"Updated hint\""));
}

#[test]
fn menu_navigation_actual_observation_handlers_wrap_restore_and_only_cancel_defaults() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let facts = harness.facts(&[
        "Alpha",
        "Disabled",
        "Link",
        "Disabled link",
        "Category",
        "Omega",
        "Footer",
        "Settings",
        "Toggle theme",
        "About & legal information",
    ]);
    // Ready can follow an equal order report; it must switch membership
    // and rerender metadata even though the DOM facts themselves are equal.
    let initial = harness.facts(&["Alpha"]);
    harness.observe("order", "", None, false, initial.clone());
    harness.observe("ready", "", None, false, initial);
    let alpha = harness.id("Alpha");
    assert_eq!(harness.context.attributes(alpha).2, Some(1));
    let ready_html = harness.html();
    let alpha_tag = ready_html
        .split('<')
        .filter_map(|element| element.split('>').next())
        .find(|tag| tag.contains("aria-label=\"Alpha\""))
        .unwrap();
    assert!(
        alpha_tag.contains("aria-setsize=\"1\""),
        "ready must notify actual rendered metadata when equal facts change the membership policy"
    );
    harness.observe("order", "", None, false, facts.clone());
    let about = harness.id("About & legal information");
    assert_eq!(
        harness.observe("key", "ArrowDown", None, false, facts.clone()),
        (true, false)
    );
    assert_eq!(harness.last(), Some(alpha));
    assert_eq!(
        harness.config.borrow().bubbled,
        0,
        "owned keys stop at the Menu"
    );
    assert_eq!(harness.focuses.borrow().last().copied(), Some(alpha));
    harness.observe("key", "ArrowUp", Some(alpha), false, facts.clone());
    assert_eq!(harness.last(), Some(about));
    menu_dispatch(&harness.dom, "focusout", harness.bar, ());
    harness.settle();
    assert!(harness.tabs().values().all(|tab| *tab == -1));
    harness.observe("key", "ArrowDown", None, false, facts.clone());
    assert_eq!(
        harness.last(),
        Some(about),
        "container recalls last eligible request"
    );
    harness.observe("key", "Home", Some(about), false, facts.clone());
    assert_eq!(harness.last(), Some(alpha));
    harness.observe("key", "End", Some(alpha), false, facts.clone());
    assert_eq!(harness.last(), Some(about));
    assert_eq!(
        harness.observe("key", "Home", Some(alpha), true, facts),
        (false, false)
    );
    assert_eq!(
        harness.last(),
        Some(about),
        "role=menu subtree does not enter root traversal"
    );
    assert_eq!(harness.config.borrow().activations, 0);
}

#[test]
fn menu_navigation_uses_current_native_order_and_actual_origin_after_failed_focus() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let mut facts = harness.facts(&["Alpha", "Link", "Omega", "Footer"]);
    harness.observe("ready", "", None, false, facts.clone());
    let alpha = harness.id("Alpha");
    let link = harness.id("Link");
    let omega = harness.id("Omega");
    menu_dispatch(&harness.dom, "focus", harness.target("Alpha"), ());
    harness.settle();
    harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
    assert_eq!(harness.last(), Some(link));
    assert_eq!(
        harness.context.model.borrow().actual,
        Some(alpha),
        "requesting a CSS-hidden wrapper does not invent focusin"
    );
    harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
    assert_eq!(
        harness.last(),
        Some(link),
        "another real key starts from its actual typed control"
    );
    facts.swap(1, 2);
    harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
    assert_eq!(
        harness.last(),
        Some(omega),
        "the latest observed order is used by the next Dioxus key"
    );
    facts[1].hidden = true;
    harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
    assert_eq!(harness.last(), Some(link));
    facts[2].display = "none".into();
    harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
    assert_eq!(harness.last(), Some(harness.id("Footer")));
    facts[3].grandparent_display = "none".into();
    harness.observe("key", "ArrowDown", Some(alpha), false, facts);
    assert_eq!(harness.last(), Some(alpha));
    assert_eq!(
        harness.observe("key", "Home", None, false, vec![]),
        (false, true)
    );
    assert_eq!(harness.config.borrow().activations, 0);
}

#[test]
fn menu_navigation_focus_observation_refreshes_current_facts_and_unknown_entry_reset() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let facts = harness.facts(&["Alpha", "Category", "Omega"]);
    harness.observe("ready", "", None, false, facts.clone());
    let omega = harness.id("Omega");
    harness.observe("key", "End", None, false, facts);
    let current = harness.facts(&["Omega", "Category", "Alpha"]);
    assert_eq!(
        harness.observe("focus", "", None, false, current.clone()),
        (false, false)
    );
    assert!(harness.tabs().values().all(|tab| *tab == -1));
    assert_eq!(
        harness.last(),
        Some(omega),
        "unknown descendant entry resets roving but retains remembered item"
    );
    assert_eq!(harness.context.model.borrow().actual, None);
    assert_eq!(
        harness.context.attributes(omega).1,
        Some(1),
        "focusin itself supplies fresh current topology"
    );
    let category = harness.id("Category");
    harness.observe("focus", "", Some(category), true, current);
    assert_eq!(harness.context.model.borrow().actual, Some(category));
    assert_eq!(
        harness.tabs()[&category],
        0,
        "category descendants attribute focus to the root category; only keys use the role-menu exclusion"
    );
    let facts = harness.facts(&["Alpha", "Omega"]);
    harness.observe("order", "", None, false, facts);
    assert_eq!(
        harness.key(harness.target("Alpha"), Key::End),
        (true, false)
    );
    assert_eq!(harness.last(), Some(omega));
    assert_eq!(harness.focuses.borrow().last().copied(), Some(omega));
}

#[test]
fn menu_navigation_category_descendants_and_wrong_protocol_owners_are_isolated() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let root_members = harness.context.model.borrow().members.len();
    menu_dispatch(&harness.dom, "click", harness.target("Category"), ());
    let child = harness.target("Child");
    harness.settle();
    assert!(harness.html().contains("role=\"menu\""));
    assert_eq!(
        harness.context.model.borrow().members.len(),
        root_members,
        "expanded descendants do not register as outer items"
    );
    assert_eq!(harness.key(child, Key::Home), (false, false));
    assert_eq!(
        harness.last(),
        None,
        "Category descendants keep Home native without root navigation"
    );
    let ids: Vec<_> = harness
        .context
        .model
        .borrow()
        .members
        .keys()
        .copied()
        .collect();
    let before = harness.last();
    assert_eq!(harness.raw(serde_json::json!({"owner":"other-menu","kind":"order","items":harness.facts(&["Alpha","Omega"])})),(false,true));
    assert_eq!(harness.last(), before);
    assert_eq!(
        harness
            .context
            .model
            .borrow()
            .members
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        ids
    );
    let facts = harness.facts(&["Alpha", "Category", "Omega"]);
    harness.observe("ready", "", None, false, facts.clone());
    assert_eq!(
        harness.observe("key", "Home", Some(harness.id("Category")), true, facts),
        (false, false)
    );
    assert_eq!(harness.last(), before);
}

#[test]
fn native_inventory_keeps_dioxus_navigation_and_custom_field_defaults_in_both_layouts() {
    let _guard = crate::test_event_converter_guard();
    for expanded in [true, false] {
        let mut h = MenuHarness::new();
        if !expanded {
            h.toggle_expanded();
        }
        let facts = h.facts(&["Alpha", "Disabled", "Link", "Category", "Omega"]);
        h.observe("ready", "", None, false, facts);
        assert_eq!(h.key(h.target("Alpha"), Key::ArrowDown), (true, false));
        assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Link")));
        let last = h.last();
        let field = |id: &str| {
            h.attributes
                .iter()
                .find(|(_, attributes)| attributes.get("id").is_some_and(|current| current == id))
                .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
                .unwrap()
        };
        let root_field = field("menu-search-fixture");
        let category_field = field("category-search-fixture");
        for key in [
            Key::Home,
            Key::End,
            Key::ArrowDown,
            Key::ArrowLeft,
            Key::Character("x".into()),
        ] {
            assert_eq!(h.key(root_field, key), (false, true));
            assert_eq!(h.last(), last);
        }
        h.click("Category");
        let focused = h.focuses.borrow().clone();
        for key in [Key::Home, Key::End, Key::ArrowDown] {
            // This caller-authored field is forwarded through the Category
            // children slot, rather than using the typed child key listener.
            // Its native editing default must survive either bubbling path.
            assert!(!h.key(category_field, key).0);
            assert_eq!(
                *h.focuses.borrow(),
                focused,
                "editing fields do not navigate typed items"
            );
            assert_eq!(
                h.last(),
                last,
                "editing fields do not navigate the outer Menu"
            );
        }
        assert_eq!(
            h.key(category_field, Key::Character("x".into())),
            (false, true)
        );
        assert_eq!(h.config.borrow().activations, 0);
    }
}

#[test]
fn menu_navigation_pending_focus_is_invalidated_by_newer_key_removal_and_unmount() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let alpha = harness.id("Alpha");
    let delayed = MenuTestMount {
        id: alpha,
        focuses: harness.focuses.clone(),
        ..Default::default()
    };
    menu_dispatch(
        &harness.dom,
        "mounted",
        harness.target("Alpha"),
        delayed.clone(),
    );
    harness.settle();
    harness.key(harness.bar, Key::Home);
    assert!(
        delayed.started.get(),
        "native delivery boundary must really be pending"
    );
    harness.key(harness.bar, Key::End);
    harness.release(&delayed);
    assert!(
        !harness.focuses.borrow().contains(&alpha),
        "superseded focus cannot deliver after await"
    );
    delayed.ready.set(false);
    delayed.started.set(false);
    harness.key(harness.bar, Key::Home);
    assert!(delayed.started.get());
    harness.config.borrow_mut().alpha_disabled = true;
    harness.update();
    harness.release(&delayed);
    assert!(
        !harness.focuses.borrow().contains(&alpha),
        "changed disabled props invalidate work awaiting native delivery"
    );
    harness.config.borrow_mut().alpha_disabled = false;
    harness.update();
    delayed.ready.set(false);
    delayed.started.set(false);
    harness.key(harness.bar, Key::Home);
    assert!(delayed.started.get());
    harness.config.borrow_mut().alpha = false;
    harness.update();
    harness.release(&delayed);
    assert!(!harness.context.model.borrow().members.contains_key(&alpha));
    assert!(!harness.focuses.borrow().contains(&alpha));
    let omega = harness.id("Omega");
    let unmounted = MenuTestMount {
        id: omega,
        focuses: harness.focuses.clone(),
        ..Default::default()
    };
    menu_dispatch(
        &harness.dom,
        "mounted",
        harness.target("Omega"),
        unmounted.clone(),
    );
    harness.settle();
    harness.key(harness.target("Category"), Key::ArrowDown);
    assert!(unmounted.started.get());
    let context = harness.context.clone();
    harness.config.borrow_mut().visible = false;
    harness.update();
    assert!(!context.model.borrow().alive);
    assert!(
        unmounted.read_dropped.get(),
        "unmount cancels the pending mounted focus"
    );
    harness.release(&unmounted);
    assert!(!harness.focuses.borrow().contains(&omega));
}

#[test]
fn menu_category_overlay_anchor_does_not_install_a_generic_trigger() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    let category = h.category("Category");
    let html = h.html();
    let overlay_id = format!("ix-menu-category-overlay-{}", category.0.id);
    assert!(html.contains(&format!("data-ix-overlay-root=\"{overlay_id}\"")));
    assert!(html.contains(&format!(
        "data-ix-overlay-anchor-id=\"ix-menu-category-overlay-{}\"",
        category.0.id
    )));
    assert!(!html.contains(&format!("data-ix-overlay-trigger=\"{overlay_id}\"")));
    h.toggle_expanded();
    h.pointer("Category", "host", "pointerenter", "mouse");
    assert!(category.is_open());
    assert!(
        h.html()
            .contains(&format!("data-ix-overlay-host=\"{overlay_id}\""))
    );
}

#[test]
fn menu_category_pointer_activation_toggles_panel_and_presents_flyout_after_observation() {
    let _guard = crate::test_event_converter_guard();
    for expanded in [true, false] {
        let mut h = MenuHarness::new();
        if !expanded {
            h.toggle_expanded();
        }
        let category = h.category("Category");
        let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
        h.category_event("category-order", "", "Category", None, 0, facts);
        assert_eq!(h.click("Category"), (false, false));
        assert!(category.is_open());
        assert_eq!(h.config.borrow().activations, 0);
        assert_eq!(h.click("Category"), (false, false));
        assert_eq!(
            category.is_open(),
            !expanded,
            "a second click toggles an expanded panel and keeps a collapsed flyout presented"
        );
        assert_eq!(h.config.borrow().activations, 0);
    }
}

#[test]
fn menu_category_collapsed_escape_closes_and_returns_through_one_dioxus_key() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
    h.category_event("category-order", "", "Category", None, 0, facts);
    h.click("Category");
    assert_eq!(h.key(h.target("Child"), Key::Escape), (false, false));
    assert!(!category.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(category.0.id));
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn menu_category_observation_and_actual_key_initialize_current_active_baseline() {
    let _event_converter_guard = crate::test_event_converter_guard();
    for (active, order_first) in [(false, false), (false, true), (true, false), (true, false)] {
        let mut harness = MenuHarness::unsettled(active);
        let category = harness.category("Category");
        assert!(
            !category.0.model.borrow().initialized,
            "no first effect has been settled"
        );
        let owner = harness.context.model.borrow().owner.clone();
        let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
        if order_first {
            let message = serde_json::json!({"owner":owner,"kind":"category-order","category":harness.id("Category"),"sequence":0,"category_items":facts,"items":[]});
            menu_dispatch(&harness.dom, "input", harness.input, message.to_string());
            assert!(
                category.0.model.borrow().initialized,
                "native readiness establishes the baseline synchronously"
            );
            assert_eq!(*category.0.panel.peek(), active);
        }
        assert_eq!(
            menu_dispatch(
                &harness.dom,
                "keydown",
                harness.target("Category"),
                Key::Enter
            ),
            (true, false)
        );
        assert!(category.0.model.borrow().initialized);
        assert_eq!(
            category.is_open(),
            !active,
            "first Enter toggles from the source active-child baseline"
        );
        harness.settle();
        assert_eq!(
            category.is_open(),
            !active,
            "late first effect must not overwrite accepted input"
        );
        assert_eq!(harness.config.borrow().activations, 0);
    }
}

#[test]
fn menu_category_independent_panels_retain_pointer_selection_and_follow_active_lifecycle() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::with_active(true);
    let first = harness.category("Category");
    let second = harness.category("Second category");
    let panel_tag = |harness: &MenuHarness| {
        let id = format!("id=\"ix-category-{}\"", harness.id("Category"));
        harness
            .html()
            .split('<')
            .filter_map(|element| element.split('>').next())
            .find(|tag| tag.starts_with("div ") && tag.contains(&id))
            .expect("retained category panel")
            .to_string()
    };
    assert!(
        !panel_tag(&harness).contains("popover="),
        "expanded panels must not inherit the native closed-popover display rule"
    );
    assert!(
        first.is_open(),
        "initial active descendant opens its panel after effects settle"
    );
    assert!(!second.is_open());
    harness.click("Second category");
    assert!(
        first.is_open() && second.is_open(),
        "expanded panels are independent"
    );
    harness.click("Child");
    assert_eq!(harness.config.borrow().activations, 1);
    assert!(
        first.is_open() && second.is_open(),
        "pointer child selection retains expanded panels"
    );
    harness.key(harness.target("Child"), Key::ArrowRight);
    assert!(
        first.is_open() && second.is_open(),
        "unsupported generic Right must not change a panel"
    );
    menu_dispatch(&harness.dom, "click", harness.toggle, ());
    harness.settle();
    assert!(
        !*first.0.panel.peek() && !*second.0.panel.peek(),
        "global collapse clears expanded panel memory"
    );
    let collapsed = panel_tag(&harness);
    assert!(collapsed.contains("popover=\"manual\""));
    assert!(collapsed.contains(&format!(
        "position-anchor:--ix-menu-category-anchor-{}",
        harness.id("Category")
    )));
    let html = harness.html();
    assert!(html.contains(&format!(
        "anchor-name:--ix-menu-category-anchor-{}",
        harness.id("Category")
    )));
    assert!(html.contains(&format!(
        "anchor-name:--ix-menu-category-anchor-{}",
        harness.id("Second category")
    )));
    menu_dispatch(&harness.dom, "click", harness.toggle, ());
    harness.settle();
    assert!(
        first.is_open() && !second.is_open(),
        "reexpand opens only active-child categories"
    );
    let expanded = panel_tag(&harness);
    assert!(!expanded.contains("popover="));
    assert!(!expanded.contains("position-anchor:"));
    harness.config.borrow_mut().child_active = false;
    harness.update();
    assert!(
        first.is_open(),
        "removing active state does not close an already open panel"
    );
    harness.click("Category");
    assert!(!first.is_open());
    harness.config.borrow_mut().child_active = true;
    harness.update();
    harness.settle();
    assert!(
        first.is_open(),
        "live active child opens a closed expanded panel"
    );
}

#[test]
fn menu_category_expanded_actual_keys_use_raw_children_local_tab_and_escape_only_closes_panel() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let category = harness.category("Category");
    assert_eq!(
        harness.key(harness.target("Category"), Key::Enter),
        (true, false)
    );
    assert!(category.is_open());
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Child"))
    );
    let html = harness.html();
    let disabled = html
        .split('<')
        .filter_map(|element| element.split('>').next())
        .find(|tag| tag.contains("aria-label=\"Disabled child\""))
        .unwrap();
    assert!(disabled.contains("aria-disabled=\"true\""));
    assert!(
        !disabled
            .split_ascii_whitespace()
            .any(|attribute| attribute == "disabled" || attribute.starts_with("disabled=")),
        "nested disabled button remains programmatically focusable"
    );
    assert_eq!(
        harness.key(harness.target("Child"), Key::ArrowDown),
        (true, false)
    );
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Disabled child"))
    );
    assert_eq!(
        harness.modified_key("Child", Key::ArrowDown, Modifiers::ALT),
        (true, false)
    );
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Disabled child")),
        "expanded raw traversal does not filter Alt arrows"
    );
    assert_eq!(harness.key(harness.target("Two"), Key::Tab), (true, false));
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Child"))
    );
    assert_eq!(
        harness.modified_key("Child", Key::Tab, Modifiers::SHIFT),
        (true, false)
    );
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Two"))
    );
    let before = harness.focuses.borrow().len();
    for key in [Key::Home, Key::End] {
        assert_eq!(harness.key(harness.target("Two"), key), (false, false));
    }
    for key in [Key::ArrowLeft, Key::ArrowRight] {
        assert_eq!(harness.key(harness.target("Two"), key), (false, true));
    }
    assert_eq!(harness.focuses.borrow().len(), before);
    assert_eq!(
        harness.key(harness.target("Two"), Key::Escape),
        (true, false)
    );
    assert!(!category.is_open());
    assert!(
        category.is_expanded(),
        "panel Escape must not collapse the whole Menu"
    );
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Category"))
    );
    assert!(
        harness.html().contains("ix-menu-category-items\" hidden")
            || harness.html().contains("hidden=true")
            || harness.html().contains("hidden=\"true\"")
    );
}

#[test]
fn menu_category_collapsed_arrows_filter_disabled_but_tab_and_repeated_present_stay_native() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    menu_dispatch(&harness.dom, "click", harness.toggle, ());
    harness.settle();
    let first = harness.category("Category");
    let second = harness.category("Second category");
    assert_eq!(
        harness.key(harness.target("Category"), Key::Enter),
        (true, false)
    );
    assert!(first.is_open());
    assert!(!first.is_expanded());
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Child"))
    );
    let before = harness.focuses.borrow().len();
    assert_eq!(
        harness.key(harness.target("Category"), Key::Enter),
        (true, false)
    );
    assert_eq!(
        harness.focuses.borrow().len(),
        before,
        "repeated presentation does not refocus the first child"
    );
    assert_eq!(
        harness.key(harness.target("Child"), Key::ArrowDown),
        (true, false)
    );
    assert_eq!(
        harness.focused_category_target("Category"),
        Some(format!(
            "ix-menu-category-child-{}-{}",
            first.0.id,
            harness.id("Two")
        ))
    );
    assert_eq!(harness.key(harness.target("Two"), Key::Home), (true, false));
    assert_eq!(
        harness.focused_category_target("Category"),
        Some(format!(
            "ix-menu-category-child-{}-{}",
            first.0.id,
            harness.id("Child")
        ))
    );
    assert_eq!(
        harness.key(harness.target("Child"), Key::End),
        (true, false)
    );
    assert_eq!(
        harness.focused_category_target("Category"),
        Some(format!(
            "ix-menu-category-child-{}-{}",
            first.0.id,
            harness.id("Two")
        ))
    );
    let alt_before = harness.focuses.borrow().len();
    for key in [Key::ArrowDown, Key::ArrowUp] {
        assert_eq!(
            harness.modified_key("Child", key, Modifiers::ALT),
            (false, false)
        );
    }
    assert_eq!(
        harness.focuses.borrow().len(),
        alt_before,
        "collapsed Alt arrows neither navigate nor cancel default"
    );
    let before = harness.focuses.borrow().len();
    assert_eq!(
        harness.key(harness.target("Child"), Key::Tab),
        (false, true)
    );
    assert_eq!(
        harness.modified_key("Child", Key::Tab, Modifiers::SHIFT),
        (false, true)
    );
    assert_eq!(
        harness.focuses.borrow().len(),
        before,
        "collapsed Tab uses native order including ARIA-disabled buttons"
    );
    for key in [Key::ArrowLeft, Key::ArrowRight] {
        assert_eq!(harness.key(harness.target("Child"), key), (false, true));
    }
    harness.click("Second category");
    assert!(second.is_open() && !first.is_open());
    let facts = harness.category_facts(&["Solo"]);
    assert_eq!(
        harness.category_event(
            "category-key",
            "Escape",
            "Second category",
            Some("Solo"),
            1,
            facts
        ),
        (false, false)
    );
    assert!(!second.is_open());
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(harness.id("Second category"))
    );
}

#[test]
fn menu_category_return_closes_and_focuses_parent_without_native_child_defaults() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let category = harness.category("Category");
    let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
    harness.category_event("category-order", "", "Category", None, 0, facts.clone());
    for (sequence, key, expanded) in [
        (1, "Enter", true),
        (2, " ", true),
        (3, "Enter", false),
        (4, " ", false),
    ] {
        if category.is_expanded() != expanded {
            harness.toggle_expanded();
        }
        harness.click("Category");
        assert!(category.is_open());
        assert_eq!(
            harness.category_event(
                "category-key",
                key,
                "Category",
                Some("Child"),
                sequence,
                facts.clone()
            ),
            (true, false)
        );
        assert!(!category.is_open());
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(category.0.id)
        );
        assert_eq!(
            harness.config.borrow().activations,
            0,
            "return does not activate the child"
        );
    }
}

#[test]
fn menu_category_observed_order_and_owner_are_current_and_unmount_rejects_late_input() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let category = harness.category("Category");
    harness.click("Category");
    let facts = harness.category_facts(&["Two", "Child", "Disabled child"]);
    harness.category_event(
        "category-key",
        "ArrowUp",
        "Category",
        Some("Child"),
        1,
        facts.clone(),
    );
    assert_eq!(
        category.0.model.borrow().requested,
        Some(Some(harness.id("Two"))),
        "current observed order supersedes mount order"
    );
    let owner = harness.context.model.borrow().owner.clone();
    let before = category.0.model.borrow().generation;
    assert_eq!(harness.raw(serde_json::json!({"owner":"wrong-owner","kind":"category-key","category":harness.id("Category"),"child":harness.id("Child"),"key":"Enter","sequence":2,"category_items":facts,"items":[]})),(false,true));
    assert_eq!(category.0.model.borrow().generation, before);
    let facts = harness.category_facts(&["Two", "Child", "Disabled child"]);
    harness.category_event(
        "category-key",
        "Enter",
        "Category",
        Some("Child"),
        2,
        facts.clone(),
    );
    menu_dispatch(&harness.dom, "click", harness.toggle, ());
    harness.settle();
    harness.config.borrow_mut().category_present = false;
    harness.update();
    assert!(!category.0.model.borrow().alive);
    assert_eq!(harness.raw(serde_json::json!({"owner":owner,"kind":"category-key","category":harness.id("Category"),"child":harness.id("Child"),"key":"Home","sequence":3,"category_items":facts,"items":[]})),(false,true));
    assert!(
        !harness
            .context
            .model
            .borrow()
            .categories
            .contains_key(&harness.id("Category"))
    );
}

#[test]
fn menu_category_keyboard_entry_waits_for_shown_hosts_before_mounted_focus() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    harness.toggle_expanded();
    let category = harness.category("Category");
    let child = harness.id("Child");
    let delayed = MenuTestMount {
        id: child,
        focuses: harness.focuses.clone(),
        ..Default::default()
    };
    menu_dispatch(
        &harness.dom,
        "mounted",
        harness.target("Child"),
        delayed.clone(),
    );
    harness.settle();
    let mut facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
    for item in &mut facts {
        item.hidden = true;
    }
    harness.category_event("category-key", "Enter", "Category", None, 1, facts.clone());
    assert!(category.is_open());
    assert_eq!(category.0.model.borrow().entry_focus, Some(child));
    assert!(
        !delayed.started.get(),
        "closed host tuples cannot authorize entry focus"
    );

    facts[0].hidden = false;
    harness.category_event("category-order", "", "Category", None, 0, facts.clone());
    assert_eq!(category.0.model.borrow().entry_focus, Some(child));
    assert!(
        !delayed.started.get(),
        "partial publication is not the shown boundary"
    );
    for item in &mut facts {
        item.hidden = false;
    }
    harness.category_event("category-order", "", "Category", None, 0, facts.clone());
    assert_eq!(category.0.model.borrow().entry_focus, None);
    assert_eq!(category.0.model.borrow().requested, Some(Some(child)));
    assert!(
        delayed.started.get(),
        "shown hosts admit the real mounted focus future"
    );
    assert!(!delayed.read_dropped.get());
    harness.category_event("category-order", "", "Category", None, 0, facts);
    assert!(
        !delayed.read_dropped.get(),
        "identical observations do not cancel admitted focus"
    );
    harness.config.borrow_mut().child_present = false;
    harness.update();
    assert!(
        delayed.read_dropped.get(),
        "removal still cancels the retained target"
    );
    harness.release(&delayed);
    assert!(!harness.focuses.borrow().contains(&child));
}

#[test]
fn menu_category_waiting_entry_rejects_changed_members_and_later_input() {
    let _event_converter_guard = crate::test_event_converter_guard();
    for invalidate in ["reorder", "eligibility", "key", "close", "mode", "unmount"] {
        let mut harness = MenuHarness::new();
        harness.toggle_expanded();
        let category = harness.category("Category");
        let mut facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
        for item in &mut facts {
            item.hidden = true;
        }
        harness.category_event("category-key", "Enter", "Category", None, 1, facts.clone());
        assert!(
            category.0.model.borrow().entry_focus.is_some(),
            "{invalidate}"
        );
        match invalidate {
            "reorder" => {
                facts.swap(0, 2);
            }
            "eligibility" => {
                facts[0].disabled = true;
            }
            "key" => {
                harness.category_event(
                    "category-key",
                    "Unidentified",
                    "Category",
                    None,
                    2,
                    facts.clone(),
                );
            }
            "close" => {
                harness.category_event(
                    "category-key",
                    "Escape",
                    "Category",
                    None,
                    2,
                    facts.clone(),
                );
                assert!(!category.is_open());
            }
            "mode" => {
                harness.toggle_expanded();
            }
            "unmount" => {
                harness.config.borrow_mut().category_present = false;
                harness.update();
            }
            _ => unreachable!(),
        }
        if matches!(invalidate, "reorder" | "eligibility") {
            harness.category_event("category-order", "", "Category", None, 0, facts.clone());
        }
        assert_eq!(category.0.model.borrow().entry_focus, None, "{invalidate}");
        let before = category.0.model.borrow().requested;
        for item in &mut facts {
            item.hidden = false;
        }
        harness.category_event("category-order", "", "Category", None, 0, facts);
        assert_eq!(
            category.0.model.borrow().requested,
            before,
            "stale reveal: {invalidate}"
        );
    }
}

#[test]
fn menu_category_pending_mounted_focus_is_cancelled_by_child_removal_and_category_unmount() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let category = harness.category("Category");
    harness.click("Category");
    let child = harness.id("Child");
    let delayed = MenuTestMount {
        id: child,
        focuses: harness.focuses.clone(),
        ..Default::default()
    };
    menu_dispatch(
        &harness.dom,
        "mounted",
        harness.target("Child"),
        delayed.clone(),
    );
    harness.settle();
    harness.key(harness.target("Two"), Key::ArrowDown);
    assert!(
        delayed.started.get(),
        "focus must really await its native delivery boundary"
    );
    harness.config.borrow_mut().child_present = false;
    harness.update();
    harness.release(&delayed);
    assert!(!category.0.model.borrow().children.contains_key(&child));
    assert!(!harness.focuses.borrow().contains(&child));
    let two = harness.id("Two");
    let pending = MenuTestMount {
        id: two,
        focuses: harness.focuses.clone(),
        ..Default::default()
    };
    menu_dispatch(
        &harness.dom,
        "mounted",
        harness.target("Two"),
        pending.clone(),
    );
    harness.settle();
    harness.key(harness.target("Disabled child"), Key::ArrowDown);
    assert!(pending.started.get());
    harness.config.borrow_mut().category_present = false;
    harness.update();
    assert!(
        pending.read_dropped.get(),
        "unmount drops the pending focus future"
    );
    assert!(!category.0.model.borrow().alive);
    harness.release(&pending);
    assert!(!harness.focuses.borrow().contains(&two));
}

#[test]
fn menu_category_open_parent_endpoints_and_escape_use_separate_collapsed_scope() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    let category = harness.category("Category");
    menu_dispatch(&harness.dom, "click", harness.toggle, ());
    harness.settle();
    harness.click("Category");
    let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
    for (sequence, key, target) in [
        (1, "ArrowDown", "Child"),
        (2, "ArrowUp", "Two"),
        (3, "Home", "Child"),
        (4, "End", "Two"),
    ] {
        assert_eq!(
            harness.category_event(
                "category-key",
                key,
                "Category",
                None,
                sequence,
                facts.clone()
            ),
            (true, false)
        );
        assert_eq!(
            category.0.model.borrow().requested,
            Some(Some(harness.id(target)))
        );
        assert!(category.is_open());
    }
    assert_eq!(
        harness.category_event("category-key", "Escape", "Category", None, 5, facts.clone()),
        (false, false)
    );
    assert!(!category.is_open());
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(category.0.id)
    );

    assert_eq!(
        harness.category_event(
            "category-key",
            "ArrowDown",
            "Category",
            None,
            7,
            facts.clone()
        ),
        (true, false)
    );
    assert_eq!(
        harness.last(),
        Some(harness.id("Omega")),
        "closed Category leaves the key to ordinary root navigation"
    );
    menu_dispatch(&harness.dom, "click", harness.toggle, ());
    harness.settle();
    harness.click("Category");
    assert_eq!(
        harness.category_event("category-key", "Escape", "Category", None, 8, facts),
        (false, true)
    );
    assert!(category.is_open(), "expanded parent Escape is unhandled");
}

#[test]
fn menu_category_parent_navigation_has_one_mounted_child_destination_in_both_layouts() {
    let _guard = crate::test_event_converter_guard();
    for expanded in [true, false] {
        let mut h = MenuHarness::new();
        if !expanded {
            h.toggle_expanded();
        }
        h.click("Category");
        let root = h.facts(&["Alpha", "Category", "Second category", "Omega"]);
        h.observe("ready", "", None, false, root);
        let child = MenuTestMount {
            id: h.id("Child"),
            focuses: h.focuses.clone(),
            ..Default::default()
        };
        let neighbor = MenuTestMount {
            id: h.id("Second category"),
            focuses: h.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(&h.dom, "mounted", h.target("Child"), child.clone());
        menu_dispatch(
            &h.dom,
            "mounted",
            h.target("Second category"),
            neighbor.clone(),
        );
        h.settle();
        let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
        assert_eq!(
            h.category_event("category-key", "ArrowDown", "Category", None, 1, facts),
            (true, false)
        );
        assert!(child.started.get());
        assert!(
            !neighbor.started.get(),
            "accepted Category navigation never starts competing root focus"
        );
        assert_eq!(
            h.last(),
            None,
            "Category handling does not synthesize an outer traversal"
        );
        h.release(&child);
        assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
        assert!(!neighbor.started.get());
        assert_eq!(h.config.borrow().activations, 0);
    }
}

#[test]
fn menu_category_empty_parent_leaves_navigation_to_the_root() {
    let _guard = crate::test_event_converter_guard();
    for expanded in [true, false] {
        let mut h = MenuHarness::new();
        h.config.borrow_mut().category_children = false;
        h.update();
        if !expanded {
            h.toggle_expanded();
        }
        let facts = h.facts(&["Alpha", "Category", "Second category", "Omega"]);
        h.observe("ready", "", None, false, facts);
        assert_eq!(h.key(h.target("Category"), Key::ArrowDown), (true, false));
        assert_eq!(h.last(), Some(h.id("Second category")));
        assert_eq!(
            h.focuses.borrow().last().copied(),
            Some(h.id("Second category"))
        );
    }
}

#[test]
fn menu_category_mounted_navigation_finishes_without_native_jobs_and_stops_after_hide_or_drop() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = MenuHarness::new();
    menu_dispatch(&harness.dom, "click", harness.toggle, ());
    harness.settle();
    harness.click("Category");
    let category = harness.category("Category");
    let node = category.0.overlay.borrow().clone().unwrap();
    let runtime = node.runtime().unwrap();
    for (sequence, child, expected) in [
        (1, "Two", "Child"),
        (2, "Child", "Two"),
        (3, "Two", "Child"),
    ] {
        let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
        harness.category_event(
            "category-key",
            "ArrowDown",
            "Category",
            Some(child),
            sequence,
            facts,
        );
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id(expected))
        );
        assert!(
            runtime.pending_item_focus_count() == 0,
            "mounted controls never enroll a foreign focus command"
        );
    }
    let target = format!(
        "ix-menu-category-child-{}-{}",
        category.0.id,
        harness.id("Child")
    );
    harness.click("Second category");
    assert!(!category.is_open());
    assert!(!node.queue_item_focus(target.clone(), true, Rc::new(|| panic!("closed owner"))));
    let after_hide = harness.focuses.borrow().clone();
    harness.settle();
    assert_eq!(
        *harness.focuses.borrow(),
        after_hide,
        "dismissed category does not steal focus back"
    );
    harness.config.borrow_mut().category_present = false;
    harness.update();
    assert!(harness.context.model.borrow().alive);
    assert!(!category.0.model.borrow().alive);
    assert!(!node.queue_item_focus(target, true, Rc::new(|| panic!("dropped owner"))));
}
#[test]
fn menu_open_veto_preserves_current_overlay_for_open_switch_and_toggle_close() {
    fn probe() -> Element {
        for (current, requested, opening) in [
            (None, MenuOverlayKind::Settings, true),
            (
                Some(MenuOverlayKind::Settings),
                MenuOverlayKind::About,
                true,
            ),
            (Some(MenuOverlayKind::About), MenuOverlayKind::About, false),
        ] {
            let veto = EventHandler::new(move |event: MenuOpenEvent| {
                assert_eq!(event.overlay, requested);
                assert_eq!(event.opening, opening);
                assert!(!event.default_prevented());
                let retained = event.clone();
                event.prevent_default();
                assert!(retained.default_prevented());
            });
            assert_eq!(request_menu_overlay(current, requested, veto), None);
        }
        rsx! { div { "Menu veto verified" } }
    }
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("Menu veto verified"));
}

#[test]
fn menu_accepted_requests_distinguish_opening_switching_and_closing() {
    fn probe() -> Element {
        let accept = EventHandler::new(|event: MenuOpenEvent| {
            assert!(!event.default_prevented());
        });
        assert_eq!(
            request_menu_overlay(None, MenuOverlayKind::Settings, accept),
            Some(Some(MenuOverlayKind::Settings))
        );
        assert_eq!(
            request_menu_overlay(
                Some(MenuOverlayKind::Settings),
                MenuOverlayKind::About,
                accept
            ),
            Some(Some(MenuOverlayKind::About))
        );
        assert_eq!(
            request_menu_overlay(
                Some(MenuOverlayKind::Settings),
                MenuOverlayKind::Settings,
                accept
            ),
            Some(None)
        );
        rsx! { div { "Menu transitions verified" } }
    }
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("Menu transitions verified"));
}

#[test]
fn pane_size_uses_the_upstream_size_set() {
    assert_eq!(pane_size("480px"), "480px");
    assert_eq!(pane_size("33%"), "33%");
    assert_eq!(pane_size("calc(100% - 1px)"), "320px");
    assert_eq!(pane_size("66%"), "320px");
}

#[test]
fn pane_sizes_track_parent_dimensions_without_consuming_more_than_half() {
    assert_eq!(pane_expanded_size("600px", 720.0), 360.0);
    assert_eq!(pane_expanded_size("240px", 1400.0), 240.0);
    assert_eq!(pane_expanded_size("50%", 719.0), 360.0);
    assert_eq!(pane_expanded_size("33%", 719.0), 240.0);
    assert_eq!(pane_expanded_size("480px", 0.0), 480.0);
    assert_eq!(pane_expanded_size("invalid", 400.0), 200.0);
}

#[test]
fn pane_request_veto_preserves_state_and_reports_layout_slot() {
    fn probe() -> Element {
        let rejected = EventHandler::new(|event: PaneExpandedChangeEvent| {
            assert_eq!(event.slot, "left");
            assert!(!event.expanded);
            let retained = event.clone();
            event.prevent_default();
            assert!(retained.default_prevented());
        });
        assert!(!request_pane_expansion(
            true,
            false,
            "left".into(),
            rejected
        ));
        let accepted = EventHandler::new(|event: PaneExpandedChangeEvent| {
            assert!(event.slot.is_empty());
            assert!(event.expanded);
            assert!(!event.default_prevented());
        });
        assert!(request_pane_expansion(false, true, String::new(), accepted));
        let must_not_run = EventHandler::new(|_: PaneExpandedChangeEvent| -> () {
            panic!("unchanged state is not an expansion request")
        });
        assert!(!request_pane_expansion(
            true,
            true,
            String::new(),
            must_not_run
        ));
        rsx! { div { "Pane requests verified" } }
    }
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("Pane requests verified"));
}

#[test]
fn initial_menu_overlay_and_controlled_key_select_the_expected_content() {
    fn probe() -> Element {
        rsx! { Application {
            menu: rsx! { Menu { initial_overlay: MenuOverlayKind::About,
                about: rsx! { MenuAbout { crate::TabSet { items: vec![crate::TabItem::new("one","One"), crate::TabItem::new("two","Two")], active_key: "two",
                    crate::TabPanel { tab_key: "one", "First panel" }
                    crate::TabPanel { tab_key: "two", "Second panel" }
                } } }
            } }
        } }
    }
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("role=\"dialog\""));
    assert!(html.contains("legal information"));
    assert!(html.contains("Second panel"));
    assert!(html.contains("aria-selected=\"true\"") || html.contains("aria-selected=true"));
    assert!(
        html.contains("hidden"),
        "inactive panel remains mounted but hidden"
    );
}

#[derive(Clone)]
enum AppSwitchRequestMutation {
    Keep,
    Remove,
    Replace(AppSwitchConfig),
}
type HeaderSlotCounts = BTreeMap<String, (usize, usize)>;
#[derive(Clone, Default)]
struct HeaderSlotCapture(Rc<RefCell<HeaderSlotCounts>>);
impl PartialEq for HeaderSlotCapture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn HeaderStatefulSlot(state: HeaderSlotCapture, label: String) -> Element {
    use_hook(|| {
        state.0.borrow_mut().entry(label.clone()).or_default().0 += 1;
    });
    let cleanup = state.clone();
    let cleanup_label = label.clone();
    use_drop(move || cleanup.0.borrow_mut().get_mut(&cleanup_label).unwrap().1 += 1);
    let mut count = use_signal(|| 0_usize);
    rsx! {div {"data-header-slot-state":label.clone(),
        input {value:"initial uncontrolled"}
        button {r#type:"button","data-header-slot-button":label.clone(),onclick:move |_|count+=1,"{label} count {count}"}
    }}
}
#[derive(Clone, Default)]
struct HeaderPeerCapture(Rc<RefCell<HeaderPeerState>>);
impl PartialEq for HeaderPeerCapture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Default)]
struct HeaderPeerState {
    handles: Option<[crate::DropdownHandle; 4]>,
    runtime: Option<crate::overlay_runtime::RuntimeHandle>,
    scope: Option<ScopeId>,
    requests: Vec<(usize, bool)>,
    completions: Vec<(usize, bool)>,
}
#[component]
fn HeaderOverflowPeers(capture: HeaderPeerCapture) -> Element {
    let handles = [
        crate::use_dropdown(),
        crate::use_dropdown(),
        crate::use_dropdown(),
        crate::use_dropdown(),
    ];
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    {
        let mut state = capture.0.borrow_mut();
        state.handles = Some(handles);
        state.runtime = Some(runtime);
        state.scope = Some(dioxus::core::current_scope_id());
    }
    rsx! { div {
        for (index, behavior) in [crate::DropdownCloseBehavior::Both, crate::DropdownCloseBehavior::Outside, crate::DropdownCloseBehavior::Inside, crate::DropdownCloseBehavior::None].into_iter().enumerate() {
            HeaderOverflowPeer { index, behavior, dropdown: handles[index], capture: capture.clone() }
        }
    } }
}
#[component]
fn HeaderOverflowPeer(
    index: usize,
    behavior: crate::DropdownCloseBehavior,
    dropdown: crate::DropdownHandle,
    capture: HeaderPeerCapture,
) -> Element {
    let before = capture.clone();
    rsx! { crate::Dropdown { controller: dropdown, label: "Header peer {index}", class: "header-test-peer-{index}", close_behavior: behavior, disable_focus_trap: true, suppress_trigger_visibility_check: true,
        onbeforechange: move |event: crate::DropdownChangeEvent| before.0.borrow_mut().requests.push((index, event.open)),
        onchange: move |open| capture.0.borrow_mut().completions.push((index, open)),
        button { "Peer content {index}" }
    } }
}
struct AppSwitchFixture {
    capture: AppSwitchCapture,
    opening: AppSwitchOpeningCapture,
    authority: Option<Signal<Option<AppSwitchConfig>>>,
    initial: Option<AppSwitchConfig>,
    snapshot: bool,
    visible: bool,
    update: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    veto: bool,
    mutation: AppSwitchRequestMutation,
    requests: usize,
    ancestor_inputs: usize,
    menu_expanded: Option<Signal<bool>>,
    show_authority: Option<Signal<bool>>,
    menu_events: Vec<(bool, bool)>,
    menu_requests: Vec<(bool, bool)>,
    menu_notifications: Vec<bool>,
    menu_veto: bool,
    menu_present: bool,
    contained: bool,
    slot_state: HeaderSlotCapture,
    slot_top_layer: bool,
    overflow_present: bool,
    overflow_veto: bool,
    overflow_requests: Vec<(bool, bool)>,
    overflow_changes: Vec<bool>,
    overflow_listener_revision: usize,
    overflow_deliveries: Vec<(usize, bool)>,
    overflow_reentrant: Option<Rc<dyn Fn(bool)>>,
    overlay_runtime: Option<crate::overlay_runtime::RuntimeHandle>,
    overflow_peers: Option<HeaderPeerCapture>,
    show_menu: bool,
    sidebar: bool,
    breakpoint: ApplicationBreakpoint,
    name: String,
    suffix: Option<String>,
    image: Option<String>,
    image_alt: Option<String>,
    outline: bool,
    named_icon: Option<String>,
    company: Option<String>,
    company_alt: Option<String>,
    logo: bool,
    switch_label: Option<String>,
    more_label: Option<String>,
}
fn app_switch_config_fixture() -> AppSwitchConfig {
    AppSwitchConfig {
        current_app_id: "same".into(),
        label: String::new(),
        loading_label: String::new(),
        apps: vec![
            AppSwitchEntry {
                id: "same".into(),
                name: "Alpha".into(),
                description: "First destination".into(),
                icon_src: "/alpha.svg".into(),
                url: "/alpha".into(),
                target: "_self".into(),
            },
            AppSwitchEntry {
                id: "same".into(),
                name: "Beta".into(),
                description: "Second destination".into(),
                icon_src: "/beta.svg".into(),
                url: "/beta".into(),
                target: "named-destination".into(),
            },
        ],
    }
}
fn app_switch_fixture(settings: Rc<RefCell<AppSwitchFixture>>) -> Element {
    let capture = settings.borrow().capture.clone();
    use_context_provider(|| capture);
    let opening = settings.borrow().opening.clone();
    use_context_provider(|| opening);
    let mut authority = use_signal(|| settings.borrow().initial.clone());
    let menu_expanded = use_signal(|| false);
    let show_authority = use_signal(|| settings.borrow().show_menu);
    let overlay_runtime = crate::overlay_runtime::use_overlay_runtime();
    {
        let mut state = settings.borrow_mut();
        state.authority = Some(authority);
        state.menu_expanded = Some(menu_expanded);
        state.show_authority = Some(show_authority);
        state.overlay_runtime = Some(overlay_runtime);
        state.update = Some(dioxus::core::schedule_update());
    }
    let requested = settings.clone();
    let input = settings.clone();
    let menu_requested = settings.clone();
    let menu_guard = settings.clone();
    let menu_notified = settings.clone();
    let overflow_requested = settings.clone();
    let overflow_changed = settings.clone();
    let state = settings.borrow();
    let overflow_listener_revision = state.overflow_listener_revision;
    let header = rsx! {ApplicationHeader {
        show_menu:show_authority,enable_top_layer:state.slot_top_layer,
        onbeforeoverflowchange:move |event:HeaderOverflowEvent| {
            let veto={let mut state=overflow_requested.borrow_mut();state.overflow_requests.push((event.previous,event.open));state.overflow_veto};
            if veto {event.prevent_default();}
        },
        onoverflowchange:move |open| {
            let reentrant = {
                let mut state = overflow_changed.borrow_mut();
                state.overflow_changes.push(open);
                state.overflow_deliveries.push((overflow_listener_revision, open));
                state.overflow_reentrant.clone()
            };
            if let Some(reentrant) = reentrant { reentrant(open); }
        },
        onmenutoggle:move |value|menu_requested.borrow_mut().menu_events.push((value,*menu_expanded.peek())),
        name:state.name.clone(),name_suffix:state.suffix.clone(),app_icon:state.named_icon.clone(),
        app_icon_src:state.image.clone(),app_icon_alt:state.image_alt.clone(),app_icon_outline:state.outline,
        company_logo:state.company.clone(),company_logo_alt:state.company_alt.clone(),logo:state.logo.then(||rsx!{span{"Neutral slot"}}),
        aria_label_app_switch_icon_button:state.switch_label.clone(),aria_label_more_menu_icon_button:state.more_label.clone(),
        actions:rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"default"}},
        secondary:rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"secondary"}},
        overflow:state.overflow_present.then(||rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"overflow"}}),
        avatar:rsx!{span{"Avatar slot"}},
        children:rsx!{span{"Leading compatibility"}},
        onopenappswitch:move |event:MouseEvent| {
            let (veto,mutation)={let mut state=requested.borrow_mut();state.requests+=1;(state.veto,state.mutation.clone())};
            if veto{event.prevent_default();}
            match mutation {AppSwitchRequestMutation::Keep=>{},AppSwitchRequestMutation::Remove=>authority.set(None),AppSwitchRequestMutation::Replace(config)=>authority.set(Some(config))}
        }
    }};
    let menu = state.menu_present.then(|| rsx! {Menu {
            onbeforeexpandchange:move |event:MenuExpandEvent| {
                let veto = {let mut state=menu_guard.borrow_mut();state.menu_requests.push((event.previous,event.expanded));state.menu_veto};
                if veto {event.prevent_default();}
            },
            onexpandchange:move |value|menu_notified.borrow_mut().menu_notifications.push(value),
            MenuItem {label:"Only item",close_on_select:false}
        }});
    let peers = state.overflow_peers.clone();
    rsx! {div {oninput:move |event:FormEvent|{input.borrow_mut().ancestor_inputs+=1;event.prevent_default();},
        if state.visible {
            if !state.contained {{header}}
            else if state.snapshot {Application {app_switch_config:authority(),header,menu,menu_expanded,force_breakpoint:state.breakpoint,
                sidebar:state.sidebar.then(||rsx!{div{"Sidebar"}})}}
            else {Application {app_switch_config:authority,header,menu,menu_expanded,force_breakpoint:state.breakpoint,
                sidebar:state.sidebar.then(||rsx!{div{"Sidebar"}})}}
        }
        if let Some(capture) = peers { HeaderOverflowPeers { capture } }
    }}
}
struct AppSwitchHarness {
    dom: VirtualDom,
    settings: Rc<RefCell<AppSwitchFixture>>,
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    input: Option<dioxus::core::ElementId>,
    trigger: Option<dioxus::core::ElementId>,
    dialog: Option<dioxus::core::ElementId>,
    menu_trigger: Option<dioxus::core::ElementId>,
    menu_control: Option<dioxus::core::ElementId>,
    sidebar_input: Option<dioxus::core::ElementId>,
    header_input: Option<dioxus::core::ElementId>,
    overlay_input: Option<dioxus::core::ElementId>,
    slot_buttons: BTreeMap<String, dioxus::core::ElementId>,
    mounts: Vec<dioxus::core::ElementId>,
}
impl AppSwitchHarness {
    fn new(initial: Option<AppSwitchConfig>, snapshot: bool) -> Self {
        let mut harness = Self::unmounted(initial, snapshot);
        harness.settle();
        harness
    }
    fn unmounted(initial: Option<AppSwitchConfig>, snapshot: bool) -> Self {
        Self::configured(initial, snapshot, |_| {})
    }
    fn configured(
        initial: Option<AppSwitchConfig>,
        snapshot: bool,
        configure: impl FnOnce(&mut AppSwitchFixture),
    ) -> Self {
        dioxus::html::set_event_converter(Box::new(MenuTestConverter));
        let settings = Rc::new(RefCell::new(AppSwitchFixture {
            capture: AppSwitchCapture::default(),
            opening: AppSwitchOpeningCapture::default(),
            authority: None,
            initial,
            snapshot,
            visible: true,
            update: None,
            veto: false,
            mutation: AppSwitchRequestMutation::Keep,
            requests: 0,
            ancestor_inputs: 0,
            menu_expanded: None,
            show_authority: None,
            menu_events: vec![],
            menu_requests: vec![],
            menu_notifications: vec![],
            menu_veto: false,
            menu_present: false,
            contained: true,
            slot_state: HeaderSlotCapture::default(),
            slot_top_layer: false,
            overflow_present: true,
            overflow_veto: false,
            overflow_requests: vec![],
            overflow_changes: vec![],
            overflow_listener_revision: 0,
            overflow_deliveries: vec![],
            overflow_reentrant: None,
            overlay_runtime: None,
            overflow_peers: None,
            show_menu: false,
            sidebar: false,
            breakpoint: ApplicationBreakpoint::Large,
            name: String::new(),
            suffix: None,
            image: None,
            image_alt: None,
            outline: false,
            named_icon: None,
            company: None,
            company_alt: None,
            logo: false,
            switch_label: None,
            more_label: None,
        }));
        configure(&mut settings.borrow_mut());
        let mut dom = VirtualDom::new_with_props(app_switch_fixture, settings.clone());
        let mutations = dom.rebuild_to_vec();
        let mut harness = Self {
            dom,
            settings,
            attrs: BTreeMap::new(),
            input: None,
            trigger: None,
            dialog: None,
            menu_trigger: None,
            menu_control: None,
            sidebar_input: None,
            header_input: None,
            overlay_input: None,
            slot_buttons: BTreeMap::new(),
            mounts: vec![],
        };
        harness.collect(mutations);
        harness
    }
    fn collect(&mut self, mutations: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } => {
                    // SetId registers a fresh node; the renderer can recycle
                    // its numeric ID after removing a containing subtree.
                    self.attrs.remove(&id.raw());
                    current = id;
                }
                dioxus::core::Mutation::PushId { id } => current = id,
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    if name == "aria-controls" && value.starts_with("ix-modal-") {
                        self.trigger = Some(current);
                    }
                    if name == "data-header-menu-toggle" {
                        self.menu_trigger = Some(current);
                    }
                    if name == "data-menu-expansion-control" {
                        self.menu_control = Some(current);
                    }
                    if name == "data-header-slot-button" {
                        self.slot_buttons.insert(value.clone(), current);
                    }
                    self.attrs
                        .entry(current.raw())
                        .or_default()
                        .insert(name, value);
                }
                dioxus::core::Mutation::NewEventListener { name } => {
                    match name.as_str() {
                        "input" => {
                            if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                attrs.contains_key("data-application-sidebar-observer")
                            }) {
                                self.sidebar_input = Some(current);
                            } else if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                attrs.contains_key("data-header-layout-observer")
                            }) {
                                self.header_input = Some(current);
                            } else if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                attrs.contains_key("data-ix-overlay-transport")
                            }) {
                                self.overlay_input = Some(current);
                            } else if self
                                .attrs
                                .get(&current.raw())
                                .is_some_and(|attrs| attrs.contains_key("data-app-switch-observer"))
                            {
                                self.input = Some(current);
                            }
                        }
                        "beforetoggle" => {
                            if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                attrs
                                    .get("class")
                                    .is_some_and(|class| class.starts_with("ix-modal "))
                            }) {
                                self.dialog = Some(current);
                            }
                        }
                        "mounted" => self.mounts.push(current),
                        _ => {}
                    }
                }
                dioxus::core::Mutation::Remove => {
                    if self.menu_trigger == Some(current) {
                        self.menu_trigger = None;
                    }
                    if self.menu_control == Some(current) {
                        self.menu_control = None;
                    }
                    if self.sidebar_input == Some(current) {
                        self.sidebar_input = None;
                    }
                }
                _ => {}
            }
        }
    }
    fn render_now(&mut self) {
        for _ in 0..5 {
            let mounts = std::mem::take(&mut self.mounts);
            for target in mounts {
                let mount = MenuTestMount {
                    id: target.raw(),
                    ..Default::default()
                };
                mount.ready.set(true);
                menu_dispatch(&self.dom, "mounted", target, mount);
            }
            let mutations = self.dom.render_immediate_to_vec();
            self.collect(mutations);
        }
    }
    /// Flush the component's actual Dioxus effects; no native ACK exists.
    fn completion_effects(&mut self) -> usize {
        let runtime = self.settings.borrow().overlay_runtime.clone().unwrap();
        let count = runtime.pending_completions();
        self.render_now();
        count
    }
    fn settle(&mut self) {
        for _ in 0..5 {
            self.render_now();
            self.completion_effects();
        }
    }
    fn navigation(&self) -> AppSwitchNavigation {
        self.settings
            .borrow()
            .capture
            .0
            .borrow()
            .as_ref()
            .unwrap()
            .clone()
    }
    fn opening(&self) -> AppSwitchOpeningState {
        self.settings
            .borrow()
            .opening
            .0
            .borrow()
            .as_ref()
            .unwrap()
            .clone()
    }
    fn authority(&self) -> Signal<Option<AppSwitchConfig>> {
        self.settings.borrow().authority.unwrap()
    }
    fn set(&mut self, config: Option<AppSwitchConfig>) {
        let mut authority = self.authority();
        self.dom.in_scope(ScopeId::ROOT, || authority.set(config));
        self.settle();
    }
    fn update(&mut self) {
        let update = self.settings.borrow().update.clone().unwrap();
        update();
        self.settle();
    }
    fn html(&self) -> String {
        dioxus_ssr::render(&self.dom)
    }
    fn write_show_menu(&mut self, value: bool) {
        let mut show = self.settings.borrow().show_authority.unwrap();
        self.dom.in_scope(ScopeId::ROOT, || show.set(value));
        self.settle();
    }
    fn header_menu_click(&self) -> (bool, bool) {
        menu_dispatch(&self.dom, "click", self.menu_trigger.unwrap(), ())
    }
    fn sidebar_report(&self, sequence: u64, present: bool) -> (bool, bool) {
        let input = self.sidebar_input.unwrap();
        let owner = &self.attrs[&input.raw()]["data-application-sidebar-observer"];
        menu_dispatch(&self.dom,"input",input,serde_json::json!({"owner":owner,"sequence":sequence,"connected":true,"present":present}).to_string())
    }
    fn header_root_attrs(&self) -> &BTreeMap<String, String> {
        let input = self.header_input.unwrap();
        let owner = &self.attrs[&input.raw()]["data-header-layout-observer"];
        self.attrs
            .values()
            .find(|attrs| attrs.get("id") == Some(owner) && attrs.contains_key("data-generation"))
            .unwrap()
    }
    fn header_rendered_root_attribute(&self, name: &str) -> String {
        // Static template attributes do not produce SetAttribute edits.
        // Resolve them on the rendered tag of the actual dynamic root ID.
        let html = self.html();
        let root = &self.header_root_attrs()["id"];
        let id = html
            .find(&format!(" id=\"{root}\""))
            .expect("rendered current Header root ID");
        let start = html[..id].rfind('<').expect("Header root opening tag");
        let end = id + html[id..].find('>').expect("Header root tag end");
        let tag = &html[start..end];
        let marker = format!(" {name}=\"");
        let value = tag.find(&marker).expect("rendered Header root attribute") + marker.len();
        tag[value..]
            .split_once('"')
            .expect("Header attribute closing quote")
            .0
            .to_string()
    }
    fn header_message(
        &self,
        sequence: u64,
        kind: &str,
        extra: serde_json::Value,
    ) -> serde_json::Value {
        let input = self.header_input.unwrap();
        let owner = &self.attrs[&input.raw()]["data-header-layout-observer"];
        let generation = self.header_root_attrs()["data-generation"]
            .parse::<u64>()
            .unwrap();
        let mut report = serde_json::json!({"owner":owner,"generation":generation,"sequence":sequence,"connected":true,"kind":kind,"presence":[true,true,true,true],"focus_count":6,"focused_index":0});
        report
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        report
    }
    fn header_raw(&self, report: serde_json::Value, query: bool) -> (bool, bool) {
        menu_dispatch(
            &self.dom,
            if query { "change" } else { "input" },
            self.header_input.unwrap(),
            report.to_string(),
        )
    }
    fn header_peer_nodes(&self) -> Vec<serde_json::Value> {
        (0..4).map(|index| {
                let class = format!("header-test-peer-{index}");
                let attrs = self.attrs.values().find(|attrs| attrs.get("class").is_some_and(|value| value.split_whitespace().any(|value| value == class))).expect("rendered actual peer root");
                serde_json::json!({"id":attrs["data-ix-overlay-root"],"generation":attrs["data-ix-overlay-generation"].parse::<u64>().unwrap(),"connected":true,"discovery_ready":true,"trigger_ancestors":[]})
            }).collect()
    }
    fn header_peer_observe(&self, sequence: u64, child: bool) -> String {
        let peers = self.settings.borrow().overflow_peers.clone().unwrap();
        let runtime = peers.0.borrow().runtime.clone().unwrap();
        let header = self.header_root_attrs()["id"].clone();
        let mut nodes = self.header_peer_nodes();
        let first = nodes[0]["id"].as_str().unwrap().to_string();
        if child {
            nodes[0]["trigger_ancestors"] = serde_json::json!([header]);
        }
        nodes.push(serde_json::json!({"id":header,"generation":self.header_rendered_root_attribute("data-ix-overlay-generation").parse::<u64>().unwrap(),"connected":true,"discovery_ready":true}));
        menu_dispatch(&self.dom,"input",self.overlay_input.unwrap(),serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"observe","nodes":nodes}).to_string());
        first
    }
    fn header_peer_effects(&mut self) -> usize {
        self.completion_effects()
    }
    fn seed_header_peers(&mut self) -> HeaderPeerCapture {
        let peers = self.settings.borrow().overflow_peers.clone().unwrap();
        let (handles, scope) = {
            let state = peers.0.borrow();
            (state.handles.unwrap(), state.scope.unwrap())
        };
        self.dom.in_scope(scope, || {
            for handle in handles {
                handle.open();
            }
        });
        assert_eq!(self.header_peer_effects(), 4);
        assert!(handles.into_iter().all(crate::DropdownHandle::is_open));
        peers.0.borrow_mut().requests.clear();
        peers.0.borrow_mut().completions.clear();
        peers
    }
    fn click(&self) -> (bool, bool) {
        menu_dispatch(&self.dom, "click", self.trigger.unwrap(), ())
    }
    fn beforetoggle(&self) {
        menu_dispatch(&self.dom, "beforetoggle", self.dialog.unwrap(), ());
    }
    fn message(&self, sequence: u64, occurrence: usize) -> serde_json::Value {
        let navigation = self.navigation();
        let model = navigation.model.borrow();
        let entry = &model.display.apps[occurrence];
        serde_json::json!({"owner":model.owner,"mount":model.mount,"generation":navigation.modal.surface_generation(),"dialog":navigation.modal.id(),"connected":true,"native_open":true,"sequence":sequence,"revision":model.revision,"occurrence":occurrence,"url":entry.url,"target":entry.target})
    }
    fn raw(&self, message: serde_json::Value) -> (bool, bool) {
        menu_dispatch(&self.dom, "input", self.input.unwrap(), message.to_string())
    }
}

#[test]
fn header_layout_context_reset_and_direct_same_value_writes_use_actual_handlers() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness =
        AppSwitchHarness::configured(Some(app_switch_config_fixture()), false, |state| {
            state.show_menu = true;
            state.menu_present = true;
        });
    harness.settle();
    assert!(
        !harness.html().contains("ix-header-menu-toggle"),
        "initial contained context resets true"
    );
    harness.write_show_menu(true);
    assert!(
        harness.html().contains("ix-header-menu-toggle"),
        "explicit equal Signal write restores mutable override"
    );
    harness.settings.borrow_mut().name = "Unrelated title".into();
    harness.update();
    assert!(
        harness.html().contains("ix-header-menu-toggle"),
        "unrelated render is not a context emission"
    );
    harness.set(Some(app_switch_config_fixture()));
    assert!(
        !harness.html().contains("ix-header-menu-toggle"),
        "same-value direct config write is an emission"
    );
    harness.write_show_menu(true);
    harness.settings.borrow_mut().sidebar = true;
    harness.update();
    assert!(
        harness.html().contains("ix-header-menu-toggle"),
        "Option presence alone is not assigned-element evidence"
    );
    assert_eq!(harness.sidebar_report(1, true), (false, false));
    harness.settle();
    assert!(
        !harness.html().contains("ix-header-menu-toggle"),
        "actual mounted sidebar element report resets override"
    );
    harness.write_show_menu(true);
    assert_eq!(harness.sidebar_report(2, true), (false, false));
    harness.settle();
    assert!(
        harness.html().contains("ix-header-menu-toggle"),
        "unchanged presence emits no new context"
    );
    assert_eq!(harness.sidebar_report(1, false), (false, false));
    harness.settle();
    assert!(
        harness.html().contains("ix-header-menu-toggle"),
        "stale report is ignored"
    );
    assert_eq!(
        harness.settings.borrow().ancestor_inputs,
        0,
        "private observer does not reach consumer input callbacks"
    );
    assert_eq!(harness.sidebar_report(3, false), (false, false));
    harness.settle();
    assert!(!harness.html().contains("ix-header-menu-toggle"));
}

#[test]
fn header_layout_contained_payload_veto_and_current_menu_lifetime() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::configured(None, false, |state| state.menu_present = true);
    harness.settle();
    harness.write_show_menu(true);
    let expanded = harness.settings.borrow().menu_expanded.unwrap();
    harness.settings.borrow_mut().menu_veto = true;
    assert_eq!(harness.header_menu_click(), (false, true));
    assert_eq!(harness.settings.borrow().menu_events, vec![(false, false)]);
    assert!(!*expanded.peek());
    assert!(
        harness.settings.borrow().menu_requests.is_empty(),
        "Header notification precedes queued Menu request"
    );
    harness.settle();
    assert_eq!(harness.settings.borrow().menu_requests, vec![(false, true)]);
    assert!(!*expanded.peek());
    assert!(harness.settings.borrow().menu_notifications.is_empty());
    harness.settings.borrow_mut().menu_veto = false;
    assert_eq!(harness.header_menu_click(), (false, true));
    assert!(!*expanded.peek());
    harness.settle();
    assert!(*expanded.peek());
    assert_eq!(harness.settings.borrow().menu_notifications, vec![true]);
    assert_eq!(harness.header_menu_click(), (false, true));
    assert_eq!(
        harness.settings.borrow().menu_events.last(),
        Some(&(true, true))
    );
    assert!(*expanded.peek());
    harness.settle();
    assert!(!*expanded.peek());
    assert_eq!(
        harness.settings.borrow().menu_notifications,
        vec![true, false]
    );
    harness.settings.borrow_mut().menu_veto = true;
    menu_dispatch(&harness.dom, "click", harness.menu_control.unwrap(), ());
    harness.settle();
    assert!(
        !*expanded.peek(),
        "Menu own control shares the vetoed request path"
    );
    harness.settings.borrow_mut().menu_veto = false;
    let count = harness.settings.borrow().menu_requests.len();
    assert_eq!(harness.header_menu_click(), (false, true));
    harness.settings.borrow_mut().menu_present = false;
    harness.update();
    assert!(
        !*expanded.peek(),
        "queued Header request sees current unregistered Menu"
    );
    assert_eq!(harness.settings.borrow().menu_requests.len(), count);
    assert_eq!(harness.header_menu_click(), (false, true));
    harness.settle();
    assert!(
        !*expanded.peek(),
        "no registered Menu means no shared state change"
    );
    harness.settings.borrow_mut().menu_present = true;
    harness.update();
    assert_eq!(harness.header_menu_click(), (false, true));
    harness.settle();
    assert!(
        *expanded.peek(),
        "new mounted Menu registers the current request callback"
    );
}

#[test]
fn header_layout_standalone_emits_new_state_and_keeps_leading_children() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::configured(None, false, |state| {
        state.contained = false;
        state.show_menu = true;
    });
    harness.settle();
    assert!(harness.html().contains("Leading compatibility"));
    assert_eq!(harness.header_menu_click(), (false, true));
    assert!(harness.settings.borrow().menu_events[0].0);
    harness.settle();
    assert!(harness.html().contains("aria-label=\"Collapse\""));
    assert_eq!(harness.header_menu_click(), (false, true));
    assert!(!harness.settings.borrow().menu_events[1].0);
    harness.settle();
    assert!(harness.html().contains("aria-label=\"Expand\""));
    assert!(harness.settings.borrow().menu_requests.is_empty());
}

#[test]
fn header_layout_actual_overflow_request_veto_defaults_stop_query_and_once_only_action() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(None, false);
    assert_eq!(
        harness.header_raw(
            harness.header_message(1, "presence", serde_json::json!({})),
            false
        ),
        (false, false)
    );
    harness.settle();
    harness.settings.borrow_mut().overflow_veto = true;
    let veto = harness.header_message(2, "key", serde_json::json!({"more":true,"key":"Enter"}));
    assert_eq!(
        harness.header_raw(veto.clone(), false),
        (true, false),
        "opening key default is prevented even under veto"
    );
    assert_eq!(
        harness.header_raw(veto, true),
        (false, false),
        "opening does not request native propagation stop"
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
    assert_eq!(
        harness.settings.borrow().overflow_requests,
        vec![(false, true)]
    );
    assert!(harness.settings.borrow().overflow_changes.is_empty());
    harness.settings.borrow_mut().overflow_veto = false;
    assert_eq!(
        harness.header_raw(
            harness.header_message(3, "key", serde_json::json!({"more":true,"key":"ArrowUp"})),
            false
        ),
        (true, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "true");
    assert!(
        !harness
            .header_root_attrs()
            .contains_key("data-focus-target"),
        "Header no longer publishes native focus commands; mounted focus is verified by rendered workflows"
    );
    let escape =
        harness.header_message(4, "key", serde_json::json!({"inside":true,"key":"Escape"}));
    assert_eq!(
        harness.header_raw(escape.clone(), false),
        (false, false),
        "Escape original default stays enabled"
    );
    assert_eq!(
        harness.header_raw(escape.clone(), true),
        (true, false),
        "separate current-event stop acknowledgement"
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
    assert_eq!(
        harness.settings.borrow().overflow_changes,
        vec![true, false]
    );
    let stale = harness.header_message(3, "key", serde_json::json!({"more":true,"key":"Enter"}));
    assert_eq!(harness.header_raw(stale, false), (false, false));
    harness.settle();
    assert_eq!(
        harness.settings.borrow().overflow_changes,
        vec![true, false]
    );
    assert_eq!(
        harness.header_raw(
            harness.header_message(5, "pointer", serde_json::json!({})),
            false
        ),
        (false, false)
    );
    assert_eq!(
        harness.header_raw(
            harness.header_message(6, "click", serde_json::json!({"more":true})),
            false
        ),
        (false, false)
    );
    harness.settle();
    assert_eq!(
        harness.header_raw(
            harness.header_message(
                7,
                "click",
                serde_json::json!({"inside":true,"background":true})
            ),
            false
        ),
        (true, false)
    );
    harness.settle();
    assert_eq!(
        harness.header_root_attrs()["data-open"],
        "true",
        "background prevention does not dismiss"
    );
    menu_dispatch(&harness.dom, "click", harness.slot_buttons["overflow"], ());
    assert_eq!(
        harness.header_raw(
            harness.header_message(8, "click", serde_json::json!({"inside":true})),
            false
        ),
        (false, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
    assert_eq!(
        harness.settings.borrow().overflow_changes,
        vec![true, false, true, false]
    );
    assert!(harness.html().contains("overflow count 1"));
    assert_eq!(
        harness.settings.borrow().slot_state.0.borrow()["overflow"],
        (1, 0)
    );
    assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
    // The already-open trigger leaves Enter/Space to native button
    // activation. A separate click report models that actual next event;
    // the key handler must neither fabricate it nor close early.
    assert_eq!(
        harness.header_raw(
            harness.header_message(9, "key", serde_json::json!({"more":true,"key":"Enter"})),
            false
        ),
        (true, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "true");
    let requests_before_keys = harness.settings.borrow().overflow_requests.len();
    let changes_before_keys = harness.settings.borrow().overflow_changes.len();
    for (sequence, key) in [(10, "Enter"), (11, " ")] {
        let report =
            harness.header_message(sequence, "key", serde_json::json!({"more":true,"key":key}));
        assert_eq!(
            harness.header_raw(report.clone(), false),
            (false, false),
            "already-open trigger leaves native default enabled"
        );
        assert_eq!(harness.header_raw(report, true), (false, false));
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "true");
    }
    assert_eq!(
        harness.settings.borrow().overflow_requests.len(),
        requests_before_keys
    );
    assert_eq!(
        harness.settings.borrow().overflow_changes.len(),
        changes_before_keys
    );
    assert_eq!(
        harness.header_raw(
            harness.header_message(12, "click", serde_json::json!({"more":true})),
            false
        ),
        (false, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
    assert_eq!(
        harness.settings.borrow().overflow_requests.len(),
        requests_before_keys + 1
    );
    assert_eq!(
        harness.settings.borrow().overflow_changes.len(),
        changes_before_keys + 1
    );
    let requests = harness.settings.borrow().overflow_requests.len();
    harness.settings.borrow_mut().slot_top_layer = true;
    harness.update();
    assert_eq!(
        harness.header_raw(escape, false),
        (false, false),
        "old generation cannot mutate new presentation"
    );
    harness.settle();
    assert_eq!(harness.settings.borrow().overflow_requests.len(), requests);
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
}

#[test]
fn header_completion_keeps_each_payload_and_resolves_current_committed_listener() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::new(None, false);
    h.header_raw(
        h.header_message(1, "presence", serde_json::json!({})),
        false,
    );
    for sequence in 2..=4 {
        h.header_raw(
            h.header_message(sequence, "click", serde_json::json!({"more":true})),
            false,
        );
    }
    assert_eq!(
        h.settings.borrow().overflow_requests,
        [(false, true), (true, false), (false, true)]
    );
    assert!(
        h.settings.borrow().overflow_changes.is_empty(),
        "request admission never calls the completion listener"
    );
    h.settings.borrow_mut().overflow_listener_revision = 7;
    h.settings.borrow().update.clone().unwrap()();
    assert_eq!(h.completion_effects(), 3);
    assert_eq!(h.settings.borrow().overflow_changes, [true, false, true]);
    assert_eq!(
        h.settings.borrow().overflow_deliveries,
        [(7, true), (7, false), (7, true)]
    );
    assert_eq!(
        h.header_root_attrs()["data-open"],
        "true",
        "captured false payload does not rewrite current state"
    );
    assert_eq!(h.completion_effects(), 0);
}

#[test]
fn header_completion_allows_reentrant_observer_state_changes() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::new(None, false);
    h.header_raw(
        h.header_message(1, "presence", serde_json::json!({})),
        false,
    );
    let actions = [
        h.header_message(3, "click", serde_json::json!({"more":true}))
            .to_string(),
        h.header_message(4, "click", serde_json::json!({"more":true}))
            .to_string(),
    ];
    let owner_runtime = h.dom.in_scope(ScopeId::ROOT, || {
        dioxus::core::Runtime::try_current().unwrap()
    });
    let weak_runtime = Rc::downgrade(&owner_runtime);
    let target = h.header_input.unwrap();
    let step = Rc::new(Cell::new(0_usize));
    let observed_step = step.clone();
    h.settings.borrow_mut().overflow_reentrant = Some(Rc::new(move |open| {
        let index = step.get();
        if index >= 2 || open != (index == 0) {
            return;
        }
        step.set(index + 1);
        let runtime = weak_runtime.upgrade().unwrap();
        let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
            actions[index].clone(),
        )));
        runtime.handle_event("input", Event::new(data, true), target);
    }));
    h.header_raw(
        h.header_message(2, "click", serde_json::json!({"more":true})),
        false,
    );
    assert!(h.settings.borrow().overflow_changes.is_empty());
    h.settle();
    assert_eq!(h.settings.borrow().overflow_changes, [true, false, true]);
    assert_eq!(observed_step.get(), 2);
    assert_eq!(h.header_root_attrs()["data-open"], "true");
    assert_eq!(
        h.settings.borrow().overflow_requests,
        [(false, true), (true, false), (false, true)]
    );
    assert_eq!(h.completion_effects(), 0);
}

#[test]
fn header_completion_survives_mode_changes_and_reports_configuration_close() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::new(None, false);
    h.header_raw(
        h.header_message(1, "presence", serde_json::json!({})),
        false,
    );
    h.header_raw(
        h.header_message(2, "click", serde_json::json!({"more":true})),
        false,
    );
    let registration = h.header_rendered_root_attribute("data-ix-overlay-generation");
    let private = h.header_root_attrs()["data-generation"].clone();
    {
        let mut settings = h.settings.borrow_mut();
        settings.breakpoint = ApplicationBreakpoint::Small;
        settings.overflow_listener_revision = 3;
    }
    h.settings.borrow().update.clone().unwrap()();
    h.render_now();
    assert_ne!(h.header_root_attrs()["data-generation"], private);
    assert_eq!(h.header_root_attrs()["data-open"], "true");
    assert_eq!(
        h.header_rendered_root_attribute("data-ix-overlay-generation"),
        registration
    );
    assert_eq!(h.settings.borrow().overflow_deliveries, [(3, true)]);
    h.settings.borrow_mut().slot_top_layer = true;
    h.settings.borrow().update.clone().unwrap()();
    h.render_now();
    assert_eq!(h.header_root_attrs()["data-open"], "false");
    assert_eq!(
        h.settings.borrow().overflow_requests,
        [(false, true)],
        "configuration does not invent a user request"
    );
    assert_eq!(
        h.settings.borrow().overflow_deliveries,
        [(3, true), (3, false)]
    );
    assert_eq!(h.completion_effects(), 0);
}

#[test]
fn dropped_header_completion_cannot_call_a_remounted_handlers_listener() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::new(None, false);
    h.header_raw(
        h.header_message(1, "presence", serde_json::json!({})),
        false,
    );
    h.header_raw(
        h.header_message(2, "click", serde_json::json!({"more":true})),
        false,
    );
    let runtime = h.settings.borrow().overlay_runtime.clone().unwrap();
    let old_root = h.header_root_attrs()["id"].clone();
    assert_eq!(runtime.pending_completions(), 1);
    h.settings.borrow_mut().visible = false;
    h.settings.borrow().update.clone().unwrap()();
    h.render_now();
    assert_eq!(runtime.pending_completions(), 0);
    assert!(h.settings.borrow().overflow_changes.is_empty());
    {
        let mut settings = h.settings.borrow_mut();
        settings.visible = true;
        settings.overflow_listener_revision = 9;
    }
    h.settings.borrow().update.clone().unwrap()();
    h.render_now();
    let rendered = h.html();
    let rendered_owner = rendered
        .split_once("data-header-layout-observer=\"")
        .map(|(_, value)| value.split('"').next().unwrap());
    assert_ne!(
        h.header_root_attrs()["id"],
        old_root,
        "rendered observer owner: {rendered_owner:?}"
    );
    assert_eq!(rendered_owner, Some(h.header_root_attrs()["id"].as_str()));
    h.header_raw(
        h.header_message(1, "presence", serde_json::json!({})),
        false,
    );
    h.header_raw(
        h.header_message(2, "click", serde_json::json!({"more":true})),
        false,
    );
    assert!(h.settings.borrow().overflow_changes.is_empty());
    assert_eq!(h.completion_effects(), 1);
    assert_eq!(h.settings.borrow().overflow_deliveries, [(9, true)]);
    assert_eq!(h.completion_effects(), 0);
}

#[test]
fn header_more_post_toggle_dismisses_policy_peers_even_when_own_request_is_vetoed() {
    let _event_converter_guard = crate::test_event_converter_guard();
    for (previous, veto) in [(false, false), (false, true), (true, false), (true, true)] {
        let capture = HeaderPeerCapture::default();
        let mut h =
            AppSwitchHarness::configured(None, false, |state| state.overflow_peers = Some(capture));
        h.settle();
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        h.header_peer_observe(1, false);
        if previous {
            h.header_raw(
                h.header_message(2, "click", serde_json::json!({"more":true})),
                false,
            );
            h.settle();
            assert_eq!(h.header_root_attrs()["data-open"], "true");
        }
        let peers = h.seed_header_peers();
        {
            let mut state = h.settings.borrow_mut();
            state.overflow_veto = veto;
            state.overflow_requests.clear();
            state.overflow_changes.clear();
        }
        let message = h.header_message(3, "click", serde_json::json!({"more":true}));
        assert_eq!(h.header_raw(message.clone(), false), (false, false));
        assert_eq!(
            h.settings.borrow().overflow_requests,
            [(previous, !previous)]
        );
        assert_eq!(
            peers.0.borrow().requests,
            [(0, false), (1, false)],
            "independent post-toggle pass: previous={previous}, veto={veto}"
        );
        assert!(
            peers.0.borrow().completions.is_empty(),
            "Dropdown completion is deferred until Dioxus effects run"
        );
        assert_eq!(
            h.header_raw(message, true),
            (false, false),
            "stop query only replays its saved flag"
        );
        assert_eq!(
            peers.0.borrow().requests,
            [(0, false), (1, false)],
            "query cannot run the post-phase twice"
        );
        assert_eq!(h.header_peer_effects(), if veto { 2 } else { 3 });
        assert_eq!(
            h.header_root_attrs()["data-open"],
            (if veto { previous } else { !previous }).to_string()
        );
        let handles = peers.0.borrow().handles.unwrap();
        assert_eq!(
            handles.map(crate::DropdownHandle::is_open),
            [false, false, true, true]
        );
        assert_eq!(
            h.settings.borrow().overflow_changes,
            if veto { vec![] } else { vec![!previous] }
        );
        assert_eq!(peers.0.borrow().completions, [(0, false), (1, false)]);
        assert_eq!(h.header_peer_effects(), 0);
    }
}

#[test]
fn header_closed_more_activation_keys_run_post_phase_with_or_without_alt_and_veto() {
    let _event_converter_guard = crate::test_event_converter_guard();
    for (alt, veto) in [(false, false), (false, true), (true, false), (true, true)] {
        let capture = HeaderPeerCapture::default();
        let mut h =
            AppSwitchHarness::configured(None, false, |state| state.overflow_peers = Some(capture));
        h.settle();
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        h.header_peer_observe(1, false);
        let peers = h.seed_header_peers();
        h.settings.borrow_mut().overflow_veto = veto;
        let message = h.header_message(
            2,
            "key",
            serde_json::json!({"more":true,"key":"ArrowDown","alt":alt}),
        );
        assert_eq!(h.header_raw(message.clone(), false), (!alt, false));
        assert_eq!(h.settings.borrow().overflow_requests, [(false, true)]);
        assert_eq!(
            peers.0.borrow().requests,
            [(0, false), (1, false)],
            "closed activation post-phase: Alt={alt}, veto={veto}"
        );
        assert_eq!(h.header_raw(message, true), (false, false));
        assert_eq!(peers.0.borrow().requests.len(), 2);
        assert_eq!(h.header_peer_effects(), if veto { 2 } else { 3 });
        assert_eq!(h.header_root_attrs()["data-open"], (!veto).to_string());
        assert_eq!(
            h.settings.borrow().overflow_changes,
            if veto { vec![] } else { vec![true] }
        );
        assert_eq!(
            peers
                .0
                .borrow()
                .handles
                .unwrap()
                .map(crate::DropdownHandle::is_open),
            [false, false, true, true]
        );
        assert_eq!(h.header_peer_effects(), 0);
    }
}

#[test]
fn header_more_post_phase_excludes_open_navigation_prevented_and_child_owned_input() {
    let _event_converter_guard = crate::test_event_converter_guard();
    for case in [
        "open-enter",
        "prevented-click",
        "prevented-key",
        "child-trigger",
    ] {
        let capture = HeaderPeerCapture::default();
        let mut h =
            AppSwitchHarness::configured(None, false, |state| state.overflow_peers = Some(capture));
        h.settle();
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        h.header_peer_observe(1, false);
        let previous = matches!(case, "open-enter" | "child-trigger");
        if previous {
            h.header_raw(
                h.header_message(2, "click", serde_json::json!({"more":true})),
                false,
            );
            h.settle();
        }
        let peers = h.seed_header_peers();
        h.settings.borrow_mut().overflow_requests.clear();
        h.settings.borrow_mut().overflow_changes.clear();
        let (kind, extra) = match case {
            "open-enter" => ("key", serde_json::json!({"key":"Enter","more":true})),
            "prevented-click" => (
                "click",
                serde_json::json!({"more":true,"default_prevented":true}),
            ),
            "prevented-key" => (
                "key",
                serde_json::json!({"key":"ArrowDown","more":true,"default_prevented":true}),
            ),
            _ => {
                let child = h.header_peer_observe(2, true);
                (
                    "click",
                    serde_json::json!({"inside":true,"hosts":[h.header_root_attrs()["id"]],"triggers":[child]}),
                )
            }
        };
        let message = h.header_message(3, kind, extra);
        assert_eq!(
            h.header_raw(message.clone(), false),
            (case == "child-trigger", false)
        );
        assert_eq!(h.header_raw(message, true), (false, false));
        h.settle();
        assert_eq!(
            h.header_root_attrs()["data-open"],
            previous.to_string(),
            "{case}"
        );
        assert!(h.settings.borrow().overflow_requests.is_empty(), "{case}");
        assert!(h.settings.borrow().overflow_changes.is_empty(), "{case}");
        assert!(peers.0.borrow().requests.is_empty(), "{case}");
        assert!(
            peers
                .0
                .borrow()
                .handles
                .unwrap()
                .into_iter()
                .all(crate::DropdownHandle::is_open),
            "{case}"
        );
        assert_eq!(h.header_peer_effects(), 0, "{case}");
    }
}

#[test]
fn header_layout_hidden_more_waits_for_vetoable_native_visibility_request() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::configured(None, false, |state| {
        state.breakpoint = ApplicationBreakpoint::Small;
        state.slot_top_layer = true;
        state.overflow_present = false;
    });
    harness.settle();
    let presence = serde_json::json!({"presence":[true,true,false,true]});
    assert_eq!(
        harness.header_raw(
            harness.header_message(1, "presence", presence.clone()),
            false
        ),
        (false, false)
    );
    harness.settle();
    assert!(!harness.slot_buttons.contains_key("overflow"));
    assert!(
        !harness
            .settings
            .borrow()
            .slot_state
            .0
            .borrow()
            .contains_key("overflow")
    );
    let default_button = harness.slot_buttons["default"];
    menu_dispatch(&harness.dom, "click", default_button, ());
    harness.settle();
    assert_eq!(
        harness.header_raw(
            harness.header_message(
                2,
                "click",
                serde_json::json!({"more":true,"presence":[true,true,false,true]})
            ),
            false
        ),
        (false, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "true");
    assert_eq!(harness.settings.borrow().overflow_requests, [(false, true)]);
    assert_eq!(harness.settings.borrow().overflow_changes, [true]);
    let original_private_generation = harness.header_root_attrs()["data-generation"].clone();
    let registration_generation = harness
        .header_rendered_root_attribute("data-ix-overlay-generation")
        .parse::<u64>()
        .unwrap();
    {
        let mut settings = harness.settings.borrow_mut();
        settings.overflow_veto = true;
        settings.breakpoint = ApplicationBreakpoint::Large;
    }
    harness.update();
    assert_ne!(
        harness.header_root_attrs()["data-generation"],
        original_private_generation
    );
    assert_eq!(
        harness.header_rendered_root_attribute("data-ix-overlay-generation"),
        registration_generation.to_string(),
        "slot projection keeps the registered controller lifetime"
    );
    assert_eq!(
        harness.header_raw(harness.header_message(1, "presence", presence), false),
        (false, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "true");
    assert_eq!(harness.settings.borrow().overflow_requests, [(false, true)]);
    assert_eq!(harness.settings.borrow().overflow_changes, [true]);
    assert_eq!(harness.slot_buttons["default"], default_button);
    assert!(harness.html().contains("default count 1"));
    assert_eq!(
        harness.settings.borrow().slot_state.0.borrow()["default"],
        (1, 0)
    );

    let root = harness.header_root_attrs()["id"].clone();
    let more = format!("{root}-more");
    let transport = harness
        .overlay_input
        .expect("actual Header shared overlay transport");
    let owner = harness.attrs[&transport.raw()]["data-ix-overlay-transport"].clone();
    assert_eq!(harness.header_root_attrs()["data-ix-overlay-owner"], owner);
    let visibility = |sequence| {
        serde_json::json!({
            "owner":owner,"document":"Header hidden More actual-handler regression",
            "sequence":sequence,"kind":"anchor-visibility",
            "nodes":[{"id":root,"generation":registration_generation,"connected":true,
                "discovery_ready":true,"trigger_id":more,"anchor_id":more}],
            "anchor_visibility":{"root":root,"generation":registration_generation,
                "target":more,"intersecting":false}
        })
    };
    assert_eq!(
        menu_dispatch(&harness.dom, "input", transport, visibility(1).to_string()),
        (false, false)
    );
    harness.settle();
    assert_eq!(
        harness.settings.borrow().overflow_requests,
        [(false, true), (true, false)]
    );
    assert_eq!(harness.settings.borrow().overflow_changes, [true]);
    assert_eq!(
        harness.header_root_attrs()["data-open"],
        "true",
        "observer admission honors the root veto"
    );

    harness.settings.borrow_mut().overflow_veto = false;
    assert_eq!(
        menu_dispatch(&harness.dom, "input", transport, visibility(2).to_string()),
        (false, false)
    );
    harness.settle();
    assert_eq!(
        harness.settings.borrow().overflow_requests,
        [(false, true), (true, false), (true, false)]
    );
    assert_eq!(harness.settings.borrow().overflow_changes, [true, false]);
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
    assert_eq!(harness.slot_buttons["default"], default_button);
    assert!(harness.html().contains("default count 1"));
    assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
}

#[test]
fn header_layout_arbitrary_slot_components_mount_once_through_mode_changes() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness =
        AppSwitchHarness::configured(Some(app_switch_config_fixture()), false, |state| {
            state.name = "Persistent heading".into();
            state.suffix = Some("Only large suffix".into());
            state.image = Some("/neutral-app.svg".into());
            state.company = Some("/neutral-company.svg".into());
            state.logo = true;
        });
    harness.settle();
    let state = harness.settings.borrow().slot_state.clone();
    for label in ["default", "secondary", "overflow"] {
        menu_dispatch(&harness.dom, "click", harness.slot_buttons[label], ());
    }
    harness.settle();
    let initial = harness.slot_buttons.clone();
    for (breakpoint, top_layer) in [
        (ApplicationBreakpoint::Small, false),
        (ApplicationBreakpoint::Large, true),
        (ApplicationBreakpoint::Small, true),
        (ApplicationBreakpoint::Large, false),
    ] {
        {
            let mut settings = harness.settings.borrow_mut();
            settings.breakpoint = breakpoint;
            settings.slot_top_layer = top_layer;
        }
        harness.update();
        assert_eq!(
            harness.header_raw(
                harness.header_message(1, "presence", serde_json::json!({})),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert_eq!(
            harness.header_raw(
                harness.header_message(2, "click", serde_json::json!({"more":true})),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "true");
        assert_eq!(
            harness.header_raw(
                harness.header_message(3, "click", serde_json::json!({"inside":true})),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "false");
        let html = harness.html();
        assert!(
            html.contains("Persistent heading")
                && html.contains("Leading compatibility")
                && html.contains("Avatar slot")
        );
        for label in ["default", "secondary", "overflow"] {
            assert_eq!(state.0.borrow()[label], (1, 0));
            assert_eq!(
                harness.slot_buttons[label], initial[label],
                "renderer listener ElementId stays stable"
            );
            assert!(
                html.contains(&format!("{label} count 1")),
                "child local state retained"
            );
        }
        if breakpoint == ApplicationBreakpoint::Small {
            assert!(
                !html.contains("/neutral-app.svg")
                    && !html.contains("/neutral-company.svg")
                    && !html.contains("Only large suffix")
                    && !html.contains("data-app-switch-opener")
            );
            assert!(
                html.contains("Neutral slot"),
                "physical-media logo visibility stays CSS-owned"
            );
        } else {
            assert!(
                html.contains("/neutral-app.svg")
                    && html.contains("/neutral-company.svg")
                    && html.contains("Only large suffix")
                    && html.contains("data-app-switch-opener")
            );
        }
    }
    harness.settings.borrow_mut().visible = false;
    harness.update();
    for label in ["default", "secondary", "overflow"] {
        assert_eq!(state.0.borrow()[label], (1, 1));
    }
}

#[test]
fn app_switch_header_truthy_content_native_labels_and_image_compatibility() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    let html = harness.html();
    assert!(html.contains("role=\"banner\""));
    assert!(!html.contains("<h1"));
    assert!(html.contains("aria-label=\"Apps\""));
    assert!(html.contains("aria-label=\"More Menu\""));
    {
        let mut settings = harness.settings.borrow_mut();
        settings.name = " ".into();
        settings.suffix = Some("Suffix only".into());
        settings.named_icon = Some("application-screen".into());
        settings.image = Some("/neutral.svg".into());
        settings.image_alt = Some(String::new());
        settings.outline = true;
        settings.company = Some("/company.svg".into());
        settings.company_alt = Some("Neutral company".into());
        settings.logo = true;
        settings.switch_label = Some(String::new());
        settings.more_label = Some("More translated".into());
    }
    harness.update();
    let html = harness.html();
    assert!(html.contains("<h1 class=\"ix-application-title\"> </h1>"));
    assert!(html.contains("src=\"/neutral.svg\" alt=\"\""));
    assert!(html.contains("data-outline=\"true\""));
    assert!(html.contains("Neutral company") && html.contains("Neutral slot"));
    assert!(html.contains("aria-label=\"\""));
    assert!(html.contains("More translated"));
    assert!(!html.contains("Switch application") && !html.contains("More header actions"));
    harness.settings.borrow_mut().image = Some(String::new());
    harness.settings.borrow_mut().name.clear();
    harness.update();
    assert!(!harness.html().contains("ix-application-app-icon"));
    assert!(!harness.html().contains("<h1"));
    assert!(harness.html().contains("Suffix only"));
    harness.settings.borrow_mut().image = None;
    harness.settings.borrow_mut().switch_label = None;
    harness.update();
    assert!(harness.html().contains("ix-application-app-icon"));
    assert!(harness.html().contains("aria-label=\"Apps\""));
}

#[test]
fn app_switch_header_actual_request_veto_retry_and_direct_current_config() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    harness.settings.borrow_mut().veto = true;
    assert_eq!(harness.click(), (true, true));
    assert_eq!(harness.settings.borrow().requests, 1);
    assert!(!harness.navigation().modal.is_open());
    assert!(!harness.opening().borrow().pending);
    harness.settle();
    assert!(harness.opening().borrow().attempts.is_empty());
    harness.settings.borrow_mut().veto = false;
    assert_eq!(harness.click(), (false, true));
    assert_eq!(harness.settings.borrow().requests, 2);
    harness.settings.borrow_mut().mutation = AppSwitchRequestMutation::Remove;
    assert_eq!(
        harness.click(),
        (false, true),
        "removal aborts queued opening without canceling the native button default"
    );
    assert!(harness.authority().peek().is_none());
    assert_eq!(harness.settings.borrow().requests, 3);
    harness.set(Some(app_switch_config_fixture()));
    let replacement = AppSwitchConfig {
        label: "Replacement loading".into(),
        ..Default::default()
    };
    harness.settings.borrow_mut().mutation = AppSwitchRequestMutation::Replace(replacement.clone());
    assert_eq!(harness.click(), (false, true));
    assert_eq!(*harness.authority().peek(), Some(replacement.clone()));
    assert!(
        harness.opening().borrow().attempts.is_empty(),
        "no native open before render commits"
    );
    harness.settle();
    assert_eq!(harness.opening().borrow().attempts, vec![replacement]);
    assert!(harness.html().contains("Replacement loading"));
    assert!(harness.html().contains("Loading available applications..."));
    harness.beforetoggle();
    assert!(
        harness.navigation().modal.is_open(),
        "beforetoggle records open synchronously without a bounds reply"
    );
}

#[test]
fn app_switch_queued_open_collapses_requests_and_rechecks_config_after_commit() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    let opening = harness.opening();
    assert_eq!(harness.click(), (false, true));
    let first = opening.borrow().sequence;
    let replacement = AppSwitchConfig {
        label: "Newest committed title".into(),
        ..Default::default()
    };
    harness.settings.borrow_mut().mutation = AppSwitchRequestMutation::Replace(replacement.clone());
    assert_eq!(harness.click(), (false, true));
    assert_eq!(harness.settings.borrow().requests, 2);
    assert!(opening.borrow().sequence > first);
    assert!(opening.borrow().pending);
    assert!(opening.borrow().attempts.is_empty());
    harness.settle();
    assert!(harness.html().contains("Newest committed title"));
    assert!(!harness.html().contains(">Alpha<"));
    assert_eq!(opening.borrow().attempts, vec![replacement]);
    assert!(!opening.borrow().pending);
    harness.settle();
    assert_eq!(
        opening.borrow().attempts.len(),
        1,
        "ordinary rerenders do not reopen"
    );

    // A separate accepted request can be removed before effects run. The
    // native button default stays intact; removal/drop cancels its work.
    harness.set(Some(app_switch_config_fixture()));
    harness.settings.borrow_mut().mutation = AppSwitchRequestMutation::Keep;
    let pending = harness.opening();
    assert_eq!(harness.click(), (false, true));
    let mut authority = harness.authority();
    harness.dom.in_scope(ScopeId::ROOT, || authority.set(None));
    harness.settle();
    assert!(!pending.borrow().alive);
    assert!(!pending.borrow().pending);
    assert_eq!(pending.borrow().attempts.len(), 1);
    assert!(!harness.html().contains("data-app-switch-opener"));
}

#[test]
fn app_switch_queued_open_owner_drop_and_snapshot_request_removal_abort() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut direct = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    let opening = direct.opening();
    assert_eq!(direct.click(), (false, true));
    direct.settings.borrow_mut().visible = false;
    direct.update();
    assert!(!opening.borrow().alive);
    assert!(opening.borrow().attempts.is_empty());
    assert!(opening.borrow().task.is_none());
    assert!(!direct.html().contains("<dialog"));

    // A lifted snapshot cannot change within the old handler, but the
    // committed effect sees its forwarded None and never opens stale UI.
    let mut snapshot = AppSwitchHarness::new(Some(app_switch_config_fixture()), true);
    let snapshot_opening = snapshot.opening();
    snapshot.settings.borrow_mut().mutation = AppSwitchRequestMutation::Remove;
    assert_eq!(snapshot.click(), (false, true));
    assert_eq!(snapshot.settings.borrow().requests, 1);
    assert!(snapshot_opening.borrow().attempts.is_empty());
    snapshot.settle();
    assert!(!snapshot_opening.borrow().alive);
    assert!(snapshot_opening.borrow().attempts.is_empty());
    assert!(!snapshot.html().contains("data-app-switch-opener"));

    snapshot.set(Some(app_switch_config_fixture()));
    let fresh = snapshot.opening();
    assert!(!Rc::ptr_eq(&snapshot_opening, &fresh));
    snapshot.settings.borrow_mut().mutation = AppSwitchRequestMutation::Replace(AppSwitchConfig {
        label: "Snapshot replacement".into(),
        ..Default::default()
    });
    assert_eq!(snapshot.click(), (false, true));
    assert!(fresh.borrow().attempts.is_empty());
    snapshot.settle();
    assert!(snapshot.html().contains("Snapshot replacement"));
    assert_eq!(fresh.borrow().attempts.len(), 1);
    assert_eq!(fresh.borrow().attempts[0].label, "Snapshot replacement");
}

#[test]
fn app_switch_actual_ack_dismisses_before_authorizing_exact_duplicate_id_occurrence() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    let navigation = harness.navigation();
    assert!(!navigation.modal.is_open());
    let html = harness.html();
    assert_eq!(
        html.matches("data-selected=\"true\"").count(),
        2,
        "duplicate source IDs select both matching occurrences"
    );
    assert_eq!(
        html.matches("alt=\"\"").count(),
        2,
        "entry images remain decorative"
    );
    let message = harness.message(1, 1);
    assert_eq!(message["url"], "/beta");
    assert_eq!(message["target"], "named-destination");
    assert_eq!(harness.raw(message.clone()), (true, false));
    assert_eq!(
        navigation.modal.result(),
        Some(crate::ModalResult::Dismissed(
            crate::ModalDismissReason::Programmatic("app-switch".into())
        ))
    );
    assert!(!navigation.modal.is_open());
    assert_eq!(
        navigation.modal.surface_generation(),
        message["generation"].as_u64().unwrap() + 1
    );
    assert_eq!(
        harness.raw(message.clone()),
        (false, false),
        "the same original transaction is consumed only once"
    );
    assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
    harness.settle();
    assert!(
        !navigation.model.borrow().alive,
        "replaced modal subtree drops its entry owner"
    );
    assert_ne!(
        harness.navigation().model.borrow().owner,
        message["owner"].as_str().unwrap()
    );
    assert_eq!(
        harness.raw(message),
        (false, false),
        "old entry descriptor cannot navigate in a replacement mount"
    );
}

#[test]
fn app_switch_actual_ack_rejects_stale_or_mismatched_native_descriptors() {
    let _event_converter_guard = crate::test_event_converter_guard();
    for (field, value) in [
        ("owner", serde_json::json!("other")),
        ("mount", serde_json::json!(0)),
        ("generation", serde_json::json!(100)),
        ("dialog", serde_json::json!("other-dialog")),
        ("connected", serde_json::json!(false)),
        ("native_open", serde_json::json!(false)),
        ("revision", serde_json::json!(100)),
        ("occurrence", serde_json::json!(9)),
        ("url", serde_json::json!("/wrong")),
        ("target", serde_json::json!("_blank")),
    ] {
        let harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        let navigation = harness.navigation();
        let mut message = harness.message(1, 1);
        message[field] = value;
        assert_eq!(harness.raw(message), (false, false), "invalid {field}");
        assert_eq!(navigation.modal.surface_generation(), 0);
        assert_eq!(navigation.modal.result(), None);
        assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
    }
    let unmounted = AppSwitchHarness::unmounted(Some(app_switch_config_fixture()), false);
    assert!(!unmounted.navigation().model.borrow().mounted);
    assert_eq!(
        unmounted.raw(unmounted.message(1, 0)),
        (false, false),
        "the real input handler rejects before mounted delivery"
    );
    let harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    let message = harness.message(1, 0);
    let mut authority = harness.authority();
    let mut next = app_switch_config_fixture();
    next.apps[0].url = "/new-authority".into();
    harness
        .dom
        .in_scope(ScopeId::ROOT, || authority.set(Some(next)));
    assert_eq!(
        harness.raw(message),
        (false, false),
        "direct present config supersedes an old rendered entry before rerender"
    );
}

#[test]
fn app_switch_live_none_retains_display_and_new_config_and_owner_invalidate_stale_entries() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    harness.beforetoggle();
    let old = harness.navigation();
    let stale = harness.message(1, 0);
    harness.set(None);
    assert!(Rc::ptr_eq(&old.model, &harness.navigation().model));
    assert!(old.modal.is_open());
    assert!(harness.html().contains("Alpha"));
    assert!(!harness.html().contains("ix-header-app-switch"));
    let retained = harness.message(2, 1);
    assert_eq!(
        harness.raw(retained),
        (true, false),
        "visible last-present entries remain usable after public None"
    );
    harness.settle();
    let mut replacement = app_switch_config_fixture();
    replacement.label = "Updated title".into();
    replacement.apps[0].name = "Updated Alpha".into();
    harness.set(Some(replacement));
    assert_eq!(harness.raw(stale), (false, false));
    assert!(harness.html().contains("Updated title") && harness.html().contains("Updated Alpha"));
    let current = harness.navigation();
    harness.settings.borrow_mut().visible = false;
    harness.update();
    assert!(!current.model.borrow().alive);
    assert!(!harness.html().contains("<dialog"));
}

#[test]
fn app_switch_snapshot_rsx_caller_keeps_forwarded_authority_current_across_renders() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(None, true);
    assert!(!harness.html().contains("ix-header-app-switch"));
    harness.set(Some(app_switch_config_fixture()));
    assert!(harness.html().contains("ix-header-app-switch"));
    let old = harness.message(1, 0);
    let same_owner = harness.navigation().model.borrow().owner.clone();
    let mut next = app_switch_config_fixture();
    next.label = "Snapshot title".into();
    next.apps[0].url = "/snapshot-new".into();
    harness.set(Some(next.clone()));
    assert_eq!(harness.navigation().model.borrow().owner, same_owner);
    assert_eq!(harness.raw(old), (false, false));
    assert_eq!(
        harness.click(),
        (false, true),
        "context-captured ReadSignal follows the new lifted snapshot"
    );
    harness.beforetoggle();
    harness.set(None);
    assert!(!harness.html().contains("ix-header-app-switch"));
    assert!(harness.navigation().modal.is_open());
    assert_eq!(harness.navigation().model.borrow().display, next);
    assert_eq!(
        harness.raw(harness.message(2, 0)),
        (true, false),
        "retained snapshot navigation remains authorized after None"
    );
}

#[test]
fn app_switch_renders_native_buttons_with_exact_descriptors_and_dialog() {
    fn probe() -> Element {
        rsx! { Application {
            app_switch_config: AppSwitchConfig { current_app_id: "chosen".into(), apps: vec![AppSwitchEntry { id: "chosen".into(), name: "Example app".into(), description: "Example description".into(), icon_src: "data:image/svg+xml,%3Csvg/%3E".into(), url: "https://example.com/".into(), target: "_blank".into() }], ..Default::default() },
            header: rsx! { ApplicationHeader {} }
        } }
    }
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("command=\"show-modal\""));
    assert!(html.contains("data-app-switch-opener"));
    assert!(html.contains("aria-haspopup=\"dialog\""));
    assert!(html.contains("<dialog"));
    assert!(html.contains("data-app-switch-url=\"https://example.com/\""));
    assert!(html.contains("data-app-switch-target=\"_blank\""));
    assert!(!html.contains("href=\"https://example.com/\""));
    assert!(html.contains("data-selected=\"true\""));
    assert!(html.contains("aria-current=\"page\""));
}
