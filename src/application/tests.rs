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

struct MenuTestVisible(bool);
impl dioxus::html::HasVisibleData for MenuTestVisible {
    fn is_intersecting(&self) -> dioxus::html::VisibleResult<bool> {
        Ok(self.0)
    }
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
    hidden: std::collections::BTreeSet<String>,
    root_keys: Vec<String>,
    child_keys: Vec<String>,
    child_disabled: bool,
    child_negative: bool,
    child_active: bool,
    child_present: bool,
    category_children: bool,
    category_present: bool,
    extra_menu: bool,
    utilities: bool,
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
                application_name:"Example application",enable_toggle_theme:settings.utilities,
                settings:settings.utilities.then(||rsx!{MenuSettings {"Settings content"}}),
                about:settings.utilities.then(||rsx!{MenuAbout {"About content"}}),
                bottom:rsx!{MenuItem {label:"Footer"}},
                input {id:"menu-search-fixture",aria_label:"Navigation search",onkeydown:move |_|{},onfocus:move |_|{}}
                for (order,item) in settings.root_keys.iter().enumerate() {
                    Fragment {key:"{item}",
                        if item=="Alpha" {
                            if settings.alpha {
                                {let activated=activated.clone();rsx!{MenuItem {label:"Alpha",order:Some(order),hidden:settings.hidden.contains(item),disabled:settings.alpha_disabled,onclick:move |_|activated.borrow_mut().activations+=1}}}
                            }
                        } else if item=="Disabled" {MenuItem {label:"Disabled",order:Some(order),disabled:true,hidden:settings.hidden.contains(item)}}
                        else if item=="Link" {a {href:"#wrapped",MenuItem {label:"Link",order:Some(order),hidden:settings.hidden.contains(item)}}}
                        else if item=="Disabled link" {MenuItem {label:"Disabled link",order:Some(order),href:"#disabled",disabled:true}}
                        else if item=="Category" {
                            if settings.category_present {MenuCategory {label:"Category",order:Some(order),
                                input {id:"category-search-fixture",aria_label:"Category search",onkeydown:move |_|{},onfocus:move |_|{}}
                                if settings.category_children {
                                    for (child_order,child) in settings.child_keys.iter().enumerate() {
                                        Fragment {key:"{child}",
                                            if child=="Child" {
                                                if settings.child_present {
                                                    {let activated=child_activated.clone();rsx!{MenuItem {label:"Child",order:Some(child_order),hidden:settings.hidden.contains(child),disabled:settings.child_disabled,tabindex:if settings.child_negative {-1}else{0},active:settings.child_active,onclick:move |_|activated.borrow_mut().activations+=1}}}
                                                }
                                            } else if child=="Disabled child" {MenuItem {label:"Disabled child",order:Some(child_order),hidden:settings.hidden.contains(child),disabled:true}}
                                            else if child=="Two" {a {href:"#category-wrapped",MenuItem {label:"Two",order:Some(child_order),hidden:settings.hidden.contains(child)}}}
                                            else {MenuItem {label:child.clone(),order:Some(child_order),hidden:settings.hidden.contains(child)}}
                                        }
                                    }
                                }
                            }}
                        } else if item=="Second category" {MenuCategory {label:"Second category",order:Some(order),MenuItem {label:"Solo"}}}
                        else {MenuItem {label:item.clone(),order:Some(order),hidden:settings.hidden.contains(item)}}
                    }
                }
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
        (@data Visible,$event:ident) => {dioxus::html::VisibleData::new(MenuTestVisible(*$event.downcast::<bool>().unwrap()))};
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
            "focus" | "blur" | "mounted" | "pointerenter" | "pointerleave" | "visible"
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
        &dom,
        "input",
        ids["$overlay-transport"],
        serde_json::json!({
        "owner":runtime.owner_id(),"document":format!("rendered-focus-cue-{}",runtime.owner_id()),
        "sequence":1,"kind":"observe","nodes":[]})
        .to_string(),
    );
    menu_dispatch(
            &dom, "input", ids["$overlay-transport"],
            serde_json::json!({"owner":runtime.owner_id(),"document":format!("rendered-focus-cue-{}",runtime.owner_id()),"sequence":2,"kind":"document","document_event":{"sequence":1,"action":"input","input_kind":"keydown","key":"Tab"}}).to_string(),
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
            serde_json::json!({"owner":runtime.owner_id(),"document":format!("rendered-focus-cue-{}",runtime.owner_id()),"sequence":3,"kind":"document","document_event":{"sequence":2,"action":"input","input_kind":"mousedown"}}).to_string());
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
fn generic_category_children_publish_own_hidden_state_and_explicit_child_ownership() {
    let capture = GenericMenuItemCapture::default();
    let (mut dom, _) = source_item_dom(capture.clone());
    let category = capture.category.borrow().clone().unwrap();
    let child_id = *category.0.model.borrow().children.keys().next().unwrap();
    let host_id = format!("ix-menu-category-child-{}-{child_id}", category.0.id);
    let html = dioxus_ssr::render(&dom);
    assert!(item_opening_tag(&html, &host_id).contains(" hidden"));
    assert!(item_opening_tag(&html, &host_id).contains("data-menu-category-child"));
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
    mounts: Vec<dioxus::core::ElementId>,
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
            hidden: Default::default(),
            root_keys: [
                "Alpha",
                "Disabled",
                "Link",
                "Disabled link",
                "Category",
                "Omega",
                "Second category",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            child_keys: ["Child", "Disabled child", "Two"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            child_disabled: false,
            child_negative: false,
            child_active,
            child_present: true,
            category_children: true,
            category_present: true,
            extra_menu,
            utilities: true,
            label: "Application Navigation".into(),
            hint: "Use Up and Down arrow keys to navigate between menu items".into(),
            update: None,
            activations: 0,
            bubbled: 0,
        }));
        dioxus::html::set_event_converter(Box::new(MenuTestConverter));
        let mut dom = VirtualDom::new_with_props(menu_navigation_fixture, config.clone());
        let mutations = dom.rebuild_to_vec();
        let context = config.borrow().capture.0.borrow().as_ref().unwrap().clone();
        let mut result = Self {
            dom,
            config,
            context,
            mounts: Vec::new(),
            bar: dioxus::core::ElementId::ROOT,
            toggle: dioxus::core::ElementId::ROOT,
            controls: BTreeMap::new(),
            focuses: Rc::new(RefCell::new(Vec::new())),
            attributes: BTreeMap::new(),
        };
        result.collect(mutations);
        result.bar = result.named_target("Application Navigation");
        result.toggle = result.named_target("Collapse");
        result.mount_pending();
        result
    }
    fn named_target(&self, label: &str) -> dioxus::core::ElementId {
        self.attributes
            .iter()
            .find(|(_, attrs)| attrs.get("aria-label").is_some_and(|value| value == label))
            .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
            .unwrap_or_else(|| panic!("missing target {label}"))
    }
    fn collect(&mut self, mutations: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } => {
                    // Renderer IDs can be reused after a subtree is removed.
                    // A fresh registration must not inherit a retired label.
                    self.controls.retain(|_, (target, _)| *target != id);
                    self.attributes.remove(&id.raw());
                    current = id;
                }
                dioxus::core::Mutation::PushId { id } => current = id,
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    let attrs = self.attributes.entry(current.raw()).or_default();
                    attrs.insert(name, value);
                    if let (Some(label), Some(id)) = (
                        attrs.get("aria-label"),
                        attrs
                            .get("data-menu-control")
                            .or_else(|| attrs.get("data-menu-category-child")),
                    ) {
                        self.controls
                            .insert(label.clone(), (current, id.parse().unwrap()));
                    }
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    self.mounts.push(current)
                }
                _ => {}
            }
        }
    }
    fn mount_pending(&mut self) {
        for target in std::mem::take(&mut self.mounts) {
            let id = self
                .controls
                .values()
                .find(|(control, _)| *control == target)
                .map(|(_, id)| *id)
                .unwrap_or(target.raw());
            let mount = MenuTestMount {
                id,
                focuses: self.focuses.clone(),
                ..Default::default()
            };
            mount.ready.set(true);
            menu_dispatch(&self.dom, "mounted", target, mount);
        }
    }
    fn settle(&mut self) {
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        for _ in 0..32 {
            self.mount_pending();
            // wait_for_work is cancel-safe and drains ready effects/tasks. One
            // poll distinguishes queued work from genuinely blocked mounted
            // focus futures without waiting for them or counting render turns.
            let ready = {
                let mut work = std::pin::pin!(self.dom.wait_for_work());
                std::future::Future::poll(work.as_mut(), &mut context).is_ready()
            };
            if !ready && self.mounts.is_empty() {
                return;
            }
            let mutations = self.dom.render_immediate_to_vec();
            self.collect(mutations);
        }
        panic!("Menu fixture did not settle after 32 ready-work passes");
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
    fn parse_key(key: &str) -> Key {
        match key {
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
        }
    }
    fn root_key(&mut self, key: &str, origin: Option<usize>, excluded: bool) -> (bool, bool) {
        let target = if excluded {
            self.target("Child")
        } else {
            origin
                .and_then(|id| {
                    self.controls
                        .values()
                        .find(|(_, current)| *current == id)
                        .map(|(target, _)| *target)
                })
                .unwrap_or(self.bar)
        };
        self.key(target, Self::parse_key(key))
    }
    fn root_focus(&mut self, origin: Option<usize>) -> (bool, bool) {
        let target = origin
            .and_then(|id| {
                self.controls
                    .values()
                    .find(|(_, current)| *current == id)
                    .map(|(target, _)| *target)
            })
            .unwrap_or(self.bar);
        let result = menu_dispatch(&self.dom, "focus", target, ());
        self.settle();
        result
    }
    fn category_key(&mut self, key: &str, category: &str, child: Option<&str>) -> (bool, bool) {
        self.key(self.target(child.unwrap_or(category)), Self::parse_key(key))
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
        menu_dispatch(&self.dom,"input",transport,serde_json::json!({
            "owner":owner,"document":format!("category-fixture-{owner}"),"sequence":1,"kind":"observe",
            "nodes":[{"id":node.id(),"generation":node.generation()}]}).to_string());
        menu_dispatch(&self.dom, "input", transport, serde_json::json!({
            "owner": owner, "document": format!("category-fixture-{owner}"),
            "sequence": 2, "kind":"document", "document_event":{"sequence":1,"action":"input",
                "input_kind":if keyboard {"keydown"} else {"mousedown"},"key":if keyboard {"Tab"} else {""}}
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
fn menu_navigation_actual_handlers_wrap_restore_and_only_cancel_owned_defaults() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    let alpha = h.id("Alpha");
    let about = h.id("About & legal information");
    assert_eq!(
        h.context.attributes(alpha).2,
        Some(9),
        "Rust sections include all eligible primary/bottom/utility controls"
    );
    assert_eq!(h.root_key("ArrowDown", None, false), (true, false));
    assert_eq!(h.last(), Some(alpha));
    assert_eq!(h.config.borrow().bubbled, 0);
    assert_eq!(h.focuses.borrow().last().copied(), Some(alpha));
    h.root_key("ArrowUp", Some(alpha), false);
    assert_eq!(h.last(), Some(about));
    menu_dispatch(&h.dom, "focusout", h.bar, ());
    h.settle();
    assert!(h.tabs().values().all(|tab| *tab == -1));
    h.root_key("ArrowDown", None, false);
    assert_eq!(h.last(), Some(about), "bar recalls the last eligible item");
    h.root_key("Home", Some(about), false);
    assert_eq!(h.last(), Some(alpha));
    h.root_key("End", Some(alpha), false);
    assert_eq!(h.last(), Some(about));
    assert_eq!(h.root_key("Home", Some(alpha), true), (false, false));
    assert_eq!(h.last(), Some(about));
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn menu_navigation_uses_current_rust_order_and_actual_typed_origin() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    {
        let mut config = h.config.borrow_mut();
        config.root_keys = ["Alpha", "Link", "Omega"]
            .into_iter()
            .map(str::to_string)
            .collect();
        config.utilities = false;
    }
    h.update();
    let alpha = h.id("Alpha");
    let link = h.id("Link");
    let omega = h.id("Omega");
    h.focus_control("Alpha");
    h.root_key("ArrowDown", Some(alpha), false);
    assert_eq!(h.last(), Some(link));
    assert_eq!(
        h.context.model.borrow().actual,
        Some(alpha),
        "a mounted request does not invent a later typed focus event"
    );
    h.root_key("ArrowDown", Some(alpha), false);
    assert_eq!(h.last(), Some(link));
    h.config.borrow_mut().root_keys.swap(1, 2);
    h.update();
    assert_eq!(
        h.id("Link"),
        link,
        "keyed moves preserve the registered owner"
    );
    h.root_key("ArrowDown", Some(alpha), false);
    assert_eq!(h.last(), Some(omega));
    h.config.borrow_mut().hidden.insert("Omega".into());
    h.update();
    h.root_key("ArrowDown", Some(alpha), false);
    assert_eq!(h.last(), Some(link));
    h.config.borrow_mut().alpha_disabled = true;
    h.update();
    h.root_key("Home", None, false);
    assert_eq!(h.last(), Some(link));
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn menu_navigation_typed_focus_retains_last_and_uses_current_rust_positions() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.config.borrow_mut().root_keys = ["Omega", "Category", "Alpha"]
        .into_iter()
        .map(str::to_string)
        .collect();
    h.update();
    let omega = h.id("Omega");
    h.root_key("Home", None, false);
    assert_eq!(h.last(), Some(omega));
    assert_eq!(h.root_focus(None), (false, false));
    assert!(h.tabs().values().all(|tab| *tab == -1));
    assert_eq!(h.last(), Some(omega));
    assert_eq!(h.context.model.borrow().actual, None);
    assert_eq!(h.context.attributes(omega).1, Some(1));
    let category = h.id("Category");
    h.root_focus(Some(category));
    assert_eq!(h.context.model.borrow().actual, Some(category));
    assert_eq!(h.tabs()[&category], 0);
    assert_eq!(h.root_key("Home", Some(category), true), (false, false));
    assert_eq!(h.last(), Some(category));
}

#[test]
fn menu_navigation_category_descendants_and_independent_owners_are_isolated() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::unsettled_with_peer(false, true);
    h.settle();
    let members = h.context.model.borrow().members.len();
    h.click("Category");
    assert_eq!(
        h.context.model.borrow().members.len(),
        members,
        "nested children stay outside the root inventory"
    );
    assert_eq!(h.key(h.target("Child"), Key::Home), (false, false));
    assert_eq!(h.last(), None);
    assert_eq!(
        h.root_key("Home", Some(h.id("Category")), true),
        (false, false)
    );
    assert_eq!(h.last(), None);
    assert!(
        !h.controls.contains_key("Other child")
            || !h
                .context
                .model
                .borrow()
                .members
                .contains_key(&h.id("Other child")),
        "independent menu registrations are isolated"
    );
}

#[test]
fn rust_inventory_keeps_dioxus_navigation_and_custom_field_defaults_in_both_layouts() {
    let _guard = crate::test_event_converter_guard();
    for expanded in [true, false] {
        let mut h = MenuHarness::new();
        if !expanded {
            h.toggle_expanded();
        }

        h.settle();
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
    assert_eq!(
        category
            .0
            .overlay
            .borrow()
            .as_ref()
            .unwrap()
            .surface_metadata(),
        Some(crate::overlay_runtime::SurfaceMetadata {
            trigger_id: String::new(),
            items_host: category.0.host_id.borrow().clone(),
            popover_host: String::new(),
        }),
        "the category anchor is an explicit items host, not a generic trigger"
    );
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

        h.settle();
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

    h.settle();
    h.click("Category");
    assert_eq!(h.key(h.target("Child"), Key::Escape), (false, false));
    assert!(!category.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(category.0.id));
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn menu_category_first_typed_key_initializes_current_active_baseline() {
    let _guard = crate::test_event_converter_guard();
    for active in [false, true] {
        let mut h = MenuHarness::unsettled(active);
        let category = h.category("Category");
        assert!(!category.0.model.borrow().initialized);
        assert_eq!(
            menu_dispatch(&h.dom, "keydown", h.target("Category"), Key::Enter),
            (true, false)
        );
        assert!(category.0.model.borrow().initialized);
        assert_eq!(category.is_open(), !active);
        h.settle();
        assert_eq!(
            category.is_open(),
            !active,
            "late initial effect cannot overwrite accepted input"
        );
        assert_eq!(h.config.borrow().activations, 0);
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
        !panel_tag(&harness).contains(" popover="),
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
    assert!(!expanded.contains(" popover="));
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
        Some(harness.id("Child")),
        "expanded entry: requested={:?}, pending={:?}, generation={}, children={:?}, pending_mounts={:?}",
        category.0.model.borrow().requested,
        category.0.model.borrow().entry_focus,
        category.0.model.borrow().generation,
        category
            .0
            .model
            .borrow()
            .children
            .iter()
            .map(|(id, child)| (*id, child.order, child.hidden, child.node.is_some()))
            .collect::<Vec<_>>(),
        harness.mounts
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

    assert_eq!(
        harness.category_key("Escape", "Second category", Some("Solo")),
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

    harness.settle();
    for (_sequence, key, expanded) in [
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
            harness.category_key(key, "Category", Some("Child")),
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
fn menu_category_rust_order_updates_and_unmount_rejects_retained_focus() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    let category = h.category("Category");
    h.click("Category");
    h.config.borrow_mut().child_keys = ["Two", "Child", "Disabled child"]
        .into_iter()
        .map(str::to_string)
        .collect();
    h.update();
    h.category_key("ArrowUp", "Category", Some("Child"));
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Two")));
    let child = h.id("Child");
    h.config.borrow_mut().category_present = false;
    h.update();
    assert!(!category.0.model.borrow().alive);
    assert!(
        !h.context
            .model
            .borrow()
            .categories
            .contains_key(&category.0.id)
    );
    let before = h.focuses.borrow().clone();
    h.dom.in_runtime(|| category.focus(Some(child)));
    h.settle();
    assert_eq!(*h.focuses.borrow(), before);
}

#[test]
fn menu_category_keyboard_entry_waits_for_its_rust_render_commit() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let child = h.id("Child");
    let delayed = MenuTestMount {
        id: child,
        focuses: h.focuses.clone(),
        ..Default::default()
    };
    menu_dispatch(&h.dom, "mounted", h.target("Child"), delayed.clone());
    h.settle();
    menu_dispatch(&h.dom, "keydown", h.target("Category"), Key::Enter);
    assert!(category.is_open());
    assert_eq!(category.0.model.borrow().entry_focus, Some(child));
    assert!(!delayed.started.get());
    h.settle();
    assert_eq!(category.0.model.borrow().entry_focus, None);
    assert!(
        delayed.started.get(),
        "render commit admits the mounted focus once"
    );
    h.config.borrow_mut().child_present = false;
    h.update();
    assert!(delayed.read_dropped.get());
    h.release(&delayed);
    assert!(!h.focuses.borrow().contains(&child));
}

#[test]
fn menu_category_pending_focus_rejects_new_hidden_or_disabled_props() {
    let _guard = crate::test_event_converter_guard();
    for hidden in [false, true] {
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let child = h.id("Child");
        let delayed = MenuTestMount {
            id: child,
            focuses: h.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(&h.dom, "mounted", h.target("Child"), delayed.clone());
        h.settle();
        h.category_key("Enter", "Category", None);
        assert!(delayed.started.get());
        if hidden {
            h.config.borrow_mut().hidden.insert("Child".into());
        } else {
            h.config.borrow_mut().child_disabled = true;
        }
        h.update();
        assert!(delayed.read_dropped.get());
        h.release(&delayed);
        assert!(!h.focuses.borrow().contains(&child));
        assert_eq!(category.0.model.borrow().entry_focus, None);
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

    for (_sequence, key, target) in [
        (1, "ArrowDown", "Child"),
        (2, "ArrowUp", "Two"),
        (3, "Home", "Child"),
        (4, "End", "Two"),
    ] {
        assert_eq!(harness.category_key(key, "Category", None), (true, false));
        assert_eq!(
            category.0.model.borrow().requested,
            Some(Some(harness.id(target)))
        );
        assert!(category.is_open());
    }
    assert_eq!(
        harness.category_key("Escape", "Category", None),
        (false, false)
    );
    assert!(!category.is_open());
    assert_eq!(
        harness.focuses.borrow().last().copied(),
        Some(category.0.id)
    );

    assert_eq!(
        harness.category_key("ArrowDown", "Category", None),
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
        harness.category_key("Escape", "Category", None),
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

        h.settle();
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

        assert_eq!(h.category_key("ArrowDown", "Category", None), (true, false));
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
        {
            let mut config = h.config.borrow_mut();
            config.category_children = false;
            config.root_keys = ["Alpha", "Category", "Second category", "Omega"]
                .into_iter()
                .map(str::to_string)
                .collect();
        }
        h.update();
        if !expanded {
            h.toggle_expanded();
        }

        h.settle();
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
    for (_sequence, child, expected) in [
        (1, "Two", "Child"),
        (2, "Child", "Two"),
        (3, "Two", "Child"),
    ] {
        harness.category_key("ArrowDown", "Category", Some(child));
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
fn HeaderStatefulSlot(
    state: HeaderSlotCapture,
    label: String,
    #[props(default)] nested: bool,
    #[props(default)] empty: bool,
    #[props(default)] form_controls: bool,
    #[props(default)] form_revision: usize,
) -> Element {
    use_hook(|| {
        state.0.borrow_mut().entry(label.clone()).or_default().0 += 1;
    });
    let cleanup = state.clone();
    let cleanup_label = label.clone();
    use_drop(move || cleanup.0.borrow_mut().get_mut(&cleanup_label).unwrap().1 += 1);
    let count = use_signal(|| 0_usize);
    if empty {
        return VNode::empty();
    }
    rsx! {div {"data-header-slot-state":label.clone(),
        HeaderSlotControls {label:label.clone(),count}
        if form_controls { HeaderFormControls {key:"{form_revision}",revision:form_revision} }
        if nested {crate::Dropdown {label:"Header nested",suppress_trigger_visibility_check:true,
            crate::DropdownItem {label:"Nested choice"}
        }}
    }}
}
#[component]
fn HeaderSlotControls(label: String, count: Signal<usize>) -> Element {
    let mut count = count;
    let input_id = use_hook(|| {
        format!(
            "header-slot-input-{}",
            NAV_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let input_mount = crate::use_focus_mount(input_id.clone());
    let button_id = use_hook(|| {
        format!(
            "header-slot-button-{}",
            NAV_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let button_mount = crate::use_focus_mount(button_id.clone());
    rsx! {
        input {id:input_id,onmounted:input_mount,value:"initial uncontrolled"}
        button {id:button_id,onmounted:button_mount,r#type:"button","data-header-slot-button":label.clone(),onclick:move |_|count+=1,"{label} count {count}"}
    }
}
#[component]
fn HeaderFormControls(revision: usize) -> Element {
    rsx! {
        crate::Input {id:format!("header-form-input-{revision}"),value:"Input value"}
        crate::Textarea {id:format!("header-form-textarea-{revision}"),value:"Textarea value"}
        crate::Select {id:format!("header-form-select-{revision}"),options:vec![crate::SelectOption::new("a","Alpha")]}
        crate::Checkbox {id:format!("header-form-checkbox-{revision}"),label:"Confirm inspection",name:format!("header-checkbox-mirror-{revision}"),required:true}
        crate::Radio {id:format!("header-form-radio-{revision}"),label:"Primary schedule",name:format!("header-radio-{revision}"),value:"primary",checked:true}
        crate::Toggle {id:format!("header-form-toggle-{revision}"),label:"Remote monitoring",name:format!("header-toggle-mirror-{revision}"),required:true}
        crate::Checkbox {id:format!("header-form-disabled-checkbox-{revision}"),label:"Unavailable confirmation",disabled:true}
        crate::Radio {id:format!("header-form-disabled-radio-{revision}"),label:"Unavailable schedule",name:format!("header-disabled-radio-{revision}"),value:"disabled",disabled:true}
        crate::Toggle {id:format!("header-form-disabled-toggle-{revision}"),label:"Unavailable monitoring",disabled:true}
        crate::RadioGroup {name:format!("header-form-radio-group-{revision}"),label:"Inspection route",value:"group-a",
            options:vec![crate::SelectOption::new("group-a","Route A"),crate::SelectOption::new("group-b","Route B")]}
    }
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
    slot_presence: [Option<bool>; 4],
    slot_empty: bool,
    slot_form_controls: bool,
    slot_form_revision: usize,
    overflow_present: bool,
    slot_nested: bool,
    overflow_veto: bool,
    overflow_requests: Vec<(bool, bool)>,
    overflow_changes: Vec<bool>,
    overflow_listener_revision: usize,
    overflow_deliveries: Vec<(usize, bool)>,
    overflow_reentrant: Option<Rc<dyn Fn(bool)>>,
    overlay_runtime: Option<crate::overlay_runtime::RuntimeHandle>,
    overflow_peers: Option<HeaderPeerCapture>,
    show_menu: bool,
    show_legacy_apps: bool,
    sidebar: bool,
    sidebar_visible: bool,
    sidebar_empty: bool,
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
        show_menu:show_authority,show_app_switch:state.show_legacy_apps,enable_top_layer:state.slot_top_layer,
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
        actions:rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"default",empty:state.slot_empty,form_controls:state.slot_form_controls,form_revision:state.slot_form_revision}},
        actions_present:state.slot_presence[1],secondary_present:state.slot_presence[0],overflow_present:state.slot_presence[2],avatar_present:state.slot_presence[3],
        secondary:rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"secondary",empty:state.slot_empty}},
        overflow:state.overflow_present.then(||rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"overflow",nested:state.slot_nested,empty:state.slot_empty}}),
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
    let sidebar=state.sidebar.then(||if state.sidebar_empty {VNode::empty()}else{rsx!{div {input {id:format!("sidebar-child-{}","input"),value:"Sidebar",oninput:move |_|{}}}}});
    rsx! {div {oninput:move |event:FormEvent|{input.borrow_mut().ancestor_inputs+=1;event.prevent_default();},
        if state.visible {
            if !state.contained {{header}}
            else if state.snapshot {Application {app_switch_config:authority(),header,menu,menu_expanded,force_breakpoint:state.breakpoint,
                sidebar_visible:state.sidebar_visible,sidebar}}
            else {Application {app_switch_config:authority,header,menu,menu_expanded,force_breakpoint:state.breakpoint,
                sidebar_visible:state.sidebar_visible,sidebar}}
        }
        if let Some(capture) = peers { HeaderOverflowPeers { capture } }
    }}
}
struct AppSwitchHarness {
    dom: VirtualDom,
    settings: Rc<RefCell<AppSwitchFixture>>,
    attrs: BTreeMap<usize, BTreeMap<String, String>>,
    entries: BTreeMap<String, dioxus::core::ElementId>,
    trigger: Option<dioxus::core::ElementId>,
    dialog: Option<dioxus::core::ElementId>,
    menu_trigger: Option<dioxus::core::ElementId>,
    menu_control: Option<dioxus::core::ElementId>,
    sidebar_child_input: Option<dioxus::core::ElementId>,
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
            slot_presence: [None; 4],
            slot_empty: false,
            slot_form_controls: false,
            slot_form_revision: 0,
            overflow_present: true,
            slot_nested: false,
            overflow_veto: false,
            overflow_requests: vec![],
            overflow_changes: vec![],
            overflow_listener_revision: 0,
            overflow_deliveries: vec![],
            overflow_reentrant: None,
            overlay_runtime: None,
            overflow_peers: None,
            show_menu: false,
            show_legacy_apps: false,
            sidebar: false,
            sidebar_visible: true,
            sidebar_empty: false,
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
            entries: BTreeMap::new(),
            trigger: None,
            dialog: None,
            menu_trigger: None,
            menu_control: None,
            sidebar_child_input: None,
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
                    if name == "href" {
                        self.entries.insert(value.clone(), current);
                    }
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
                                attrs
                                    .get("id")
                                    .is_some_and(|id| id == "sidebar-child-input")
                            }) {
                                self.sidebar_child_input = Some(current);
                            } else if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                attrs.contains_key("data-header-layout-observer")
                            }) {
                                self.header_input = Some(current);
                            } else if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                attrs.contains_key("data-ix-overlay-transport")
                            }) {
                                self.overlay_input = Some(current);
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
                    if self.sidebar_child_input == Some(current) {
                        self.sidebar_child_input = None;
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
    fn header_more_target(&self) -> dioxus::core::ElementId {
        let owner = &self.header_root_attrs()["id"];
        let (id, _) = self
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-header-more-button") == Some(owner))
            .expect("current More control");
        dioxus::core::ElementId::from_raw(*id)
    }
    fn header_more_click(&self, prevented: bool) -> (bool, bool) {
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
        let event = Event::new(data, true);
        if prevented {
            event.prevent_default();
        }
        self.dom
            .runtime()
            .handle_event("click", event.clone(), self.header_more_target());
        (!event.default_action_enabled(), event.propagates())
    }
    fn header_key(
        &self,
        key: Key,
        more: bool,
        modifiers: Modifiers,
        prevented: bool,
    ) -> (bool, bool) {
        let target = if more {
            self.header_more_target()
        } else {
            let root = self.header_root_attrs();
            let (id, _) = self
                .attrs
                .iter()
                .find(|(_, attrs)| attrs.get("data-ix-overlay-host") == root.get("id"))
                .expect("current active Header popup");
            dioxus::core::ElementId::from_raw(*id)
        };
        let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
            (key, modifiers),
        )));
        let event = Event::new(data, true);
        if prevented {
            event.prevent_default();
        }
        self.dom
            .runtime()
            .handle_event("keydown", event.clone(), target);
        (!event.default_action_enabled(), event.propagates())
    }
    fn header_background_click(&self) -> (bool, bool) {
        let root = self.header_root_attrs();
        let (id, _) = self
            .attrs
            .iter()
            .find(|(_, attrs)| attrs.get("data-ix-overlay-host") == root.get("id"))
            .expect("current popup");
        menu_dispatch(
            &self.dom,
            "click",
            dioxus::core::ElementId::from_raw(*id),
            (),
        )
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
        let mut report = serde_json::json!({"owner":owner,"generation":generation,"sequence":sequence,"kind":kind,"viewport_width":1200.0});
        report
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        report
    }
    fn header_raw(&self, report: serde_json::Value) -> (bool, bool) {
        menu_dispatch(
            &self.dom,
            "input",
            self.header_input.unwrap(),
            report.to_string(),
        )
    }
    fn header_peer_nodes(&self) -> Vec<serde_json::Value> {
        (0..4).map(|index| {
                let class = format!("header-test-peer-{index}");
                let attrs = self.attrs.values().find(|attrs| attrs.get("class").is_some_and(|value| value.split_whitespace().any(|value| value == class))).expect("rendered actual peer root");
                serde_json::json!({"id":attrs["data-ix-overlay-root"],"generation":attrs["data-ix-overlay-generation"].parse::<u64>().unwrap()})
            }).collect()
    }
    fn header_peer_observe(&self, sequence: u64, child: bool) -> String {
        let peers = self.settings.borrow().overflow_peers.clone().unwrap();
        let runtime = peers.0.borrow().runtime.clone().unwrap();
        let header = self.header_root_attrs()["id"].clone();
        let mut nodes = self.header_peer_nodes();
        let first = nodes[0]["id"].as_str().unwrap().to_string();
        let first_node = runtime.node_handle(&first).unwrap();
        let header_node = runtime.node_handle(&header).unwrap();
        first_node.set_ownership(
            if child {
                vec![header_node.clone()]
            } else {
                Vec::new()
            },
            true,
        );
        first_node.set_mounted();
        header_node.set_mounted();
        nodes.push(serde_json::json!({"id":header,"generation":self.header_rendered_root_attribute("data-ix-overlay-generation").parse::<u64>().unwrap()}));
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
    fn entry_click(&self, occurrence: usize) -> (bool, bool) {
        let navigation = self.navigation();
        let url = navigation.model.borrow().display.apps[occurrence]
            .url
            .clone();
        menu_dispatch(&self.dom, "click", self.entries[&url], ())
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
        !harness.html().contains("ix-header-menu-toggle"),
        "Rust sidebar slot presence resets the override"
    );
    assert!(!harness.html().contains("data-application-sidebar-observer"));
    harness.write_show_menu(true);
    harness.settings.borrow_mut().name = "Another unrelated title".into();
    harness.update();
    assert!(
        harness.html().contains("ix-header-menu-toggle"),
        "unchanged effective sidebar presence does not emit a context change"
    );
    assert_eq!(
        menu_dispatch(
            &harness.dom,
            "input",
            harness.sidebar_child_input.unwrap(),
            "ordinary edit".to_string()
        ),
        (true, true)
    );
    assert_eq!(
        harness.settings.borrow().ancestor_inputs,
        1,
        "ordinary sidebar input reaches the consumer callback"
    );
    harness.settings.borrow_mut().sidebar_visible = false;
    harness.update();
    assert!(
        !harness.html().contains("ix-header-menu-toggle"),
        "hiding the slot changes effective presence"
    );
    harness.write_show_menu(true);
    harness.settings.borrow_mut().sidebar = false;
    harness.update();
    assert!(
        harness.html().contains("ix-header-menu-toggle"),
        "removing an already hidden slot preserves unchanged context"
    );
    {
        let mut state = harness.settings.borrow_mut();
        state.sidebar_visible = true;
        state.sidebar = true;
        state.sidebar_empty = true;
    }
    harness.update();
    assert!(
        !harness.html().contains("ix-header-menu-toggle"),
        "explicit Some(empty Dioxus content) is a present Rust slot without DOM discovery"
    );
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
fn header_layout_dioxus_overflow_keys_preserve_veto_defaults_and_once_only_actions() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(None, false);
    assert_eq!(
        harness.header_raw(harness.header_message(1, "layout", serde_json::json!({}))),
        (false, false)
    );
    harness.settle();
    harness.settings.borrow_mut().overflow_veto = true;
    assert_eq!(
        harness.header_key(Key::Enter, true, Modifiers::empty(), false),
        (true, false),
        "opening key prevents default and stops virtual bubbling under veto"
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
        harness.header_key(Key::ArrowUp, true, Modifiers::empty(), false),
        (true, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "true");
    assert!(
        !harness
            .header_root_attrs()
            .contains_key("data-focus-target"),
        "Header does not publish native focus commands"
    );
    assert_eq!(
        harness.header_key(Key::Escape, false, Modifiers::empty(), false),
        (true, false),
        "handled local Escape prevents default and stops virtual bubbling"
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
    assert_eq!(
        harness.settings.borrow().overflow_changes,
        vec![true, false]
    );
    let stale = harness.header_message(3, "key", serde_json::json!({"more":true,"key":"Enter"}));
    assert_eq!(
        harness.header_raw(stale),
        (false, false),
        "retired native key reports do not dispatch keyboard actions"
    );
    harness.settle();
    assert_eq!(
        harness.settings.borrow().overflow_changes,
        vec![true, false]
    );
    assert_eq!(harness.header_more_click(false), (false, false));
    harness.settle();
    assert_eq!(harness.header_background_click(), (false, false));
    harness.settle();
    assert_eq!(
        harness.header_root_attrs()["data-open"],
        "true",
        "popup background keeps native defaults and does not dismiss"
    );
    menu_dispatch(&harness.dom, "click", harness.slot_buttons["overflow"], ());
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
    assert_eq!(
        harness.header_key(Key::Enter, true, Modifiers::empty(), false),
        (true, false)
    );
    harness.settle();
    assert_eq!(harness.header_root_attrs()["data-open"], "true");
    let requests_before_keys = harness.settings.borrow().overflow_requests.len();
    let changes_before_keys = harness.settings.borrow().overflow_changes.len();
    for key in [Key::Enter, Key::Character(" ".into())] {
        assert_eq!(
            harness.header_key(key, true, Modifiers::empty(), false),
            (false, true),
            "already-open More preserves native button activation"
        );
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
    assert_eq!(harness.header_more_click(false), (false, false));
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
    let stale = harness.header_message(4, "key", serde_json::json!({"inside":true,"key":"Escape"}));
    harness.settings.borrow_mut().slot_top_layer = true;
    harness.update();
    assert_eq!(
        harness.header_raw(stale),
        (false, false),
        "retired generation key report cannot mutate new presentation"
    );
    harness.settle();
    assert_eq!(harness.settings.borrow().overflow_requests.len(), requests);
    assert_eq!(harness.header_root_attrs()["data-open"], "false");
}

#[test]
fn header_completion_keeps_each_payload_and_resolves_current_committed_listener() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::new(None, false);
    h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
    for _ in 0..3 {
        h.header_more_click(false);
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
    h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
    let owner_runtime = h.dom.in_scope(ScopeId::ROOT, || {
        dioxus::core::Runtime::try_current().unwrap()
    });
    let weak_runtime = Rc::downgrade(&owner_runtime);
    let target = h.header_more_target();
    let step = Rc::new(Cell::new(0_usize));
    let observed_step = step.clone();
    h.settings.borrow_mut().overflow_reentrant = Some(Rc::new(move |open| {
        let index = step.get();
        if index >= 2 || open != (index == 0) {
            return;
        }
        step.set(index + 1);
        let runtime = weak_runtime.upgrade().unwrap();
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(())));
        runtime.handle_event("click", Event::new(data, true), target);
    }));
    h.header_more_click(false);
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
    h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
    h.header_more_click(false);
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
    h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
    h.header_more_click(false);
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
    h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
    h.header_more_click(false);
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
        h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
        h.header_peer_observe(1, false);
        if previous {
            h.header_more_click(false);
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
        assert_eq!(h.header_more_click(false), (false, false));
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
        h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
        h.header_peer_observe(1, false);
        let peers = h.seed_header_peers();
        h.settings.borrow_mut().overflow_veto = veto;
        assert_eq!(
            h.header_key(
                Key::ArrowDown,
                true,
                if alt {
                    Modifiers::ALT
                } else {
                    Modifiers::empty()
                },
                false
            ),
            (!alt, false)
        );
        assert_eq!(h.settings.borrow().overflow_requests, [(false, true)]);
        assert_eq!(
            peers.0.borrow().requests,
            [(0, false), (1, false)],
            "closed activation post-phase: Alt={alt}, veto={veto}"
        );
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
fn header_more_dismissal_skips_open_navigation_and_consumer_prevented_input() {
    let _event_converter_guard = crate::test_event_converter_guard();
    for case in ["open-enter", "prevented-click", "prevented-key"] {
        let capture = HeaderPeerCapture::default();
        let mut h =
            AppSwitchHarness::configured(None, false, |state| state.overflow_peers = Some(capture));
        h.settle();
        h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
        h.header_peer_observe(1, false);
        let previous = case == "open-enter";
        if previous {
            h.header_more_click(false);
            h.settle();
        }
        let peers = h.seed_header_peers();
        h.settings.borrow_mut().overflow_requests.clear();
        h.settings.borrow_mut().overflow_changes.clear();
        if case == "prevented-click" {
            assert_eq!(h.header_more_click(true), (true, true));
        } else {
            let prevented = case == "prevented-key";
            let key = if previous { Key::Enter } else { Key::ArrowDown };
            assert_eq!(
                h.header_key(key, true, Modifiers::empty(), prevented),
                (prevented, true)
            );
        }
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
fn actual_nested_dropdown_clicks_do_not_toggle_the_header_surface() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::configured(None, false, |state| state.slot_nested = true);
    h.settle();
    h.header_raw(h.header_message(1, "layout", serde_json::json!({})));
    let (id, _) = h
        .attrs
        .iter()
        .find(|(_, attrs)| {
            attrs
                .get("aria-label")
                .is_some_and(|value| value == "Header nested")
                && attrs.contains_key("data-ix-overlay-root")
        })
        .expect("actual nested Dropdown root");
    let target = dioxus::core::ElementId::from_raw(*id);
    // Actual Dioxus scope providers establish this nested relationship.
    // No native hierarchy snapshot is supplied by this renderer.
    assert_eq!(h.header_more_click(false), (false, false));
    h.settle();
    for open in [true, false] {
        assert_eq!(menu_dispatch(&h.dom, "click", target, ()), (false, false));
        h.settle();
        assert_eq!(h.attrs[&target.raw()]["aria-expanded"], open.to_string());
        assert_eq!(
            h.header_root_attrs()["data-open"],
            "true",
            "child activation never reaches the Header group handler"
        );
    }
    assert_eq!(h.settings.borrow().overflow_requests, [(false, true)]);
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
    let layout = serde_json::json!({});
    assert_eq!(
        harness.header_raw(harness.header_message(1, "layout", layout.clone())),
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
    assert_eq!(harness.header_more_click(false), (false, false));
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
        harness.header_raw(harness.header_message(1, "layout", layout)),
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
    let more_target = harness
        .attrs
        .iter()
        .find(|(_, attributes)| attributes.get("id") == Some(&more))
        .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
        .unwrap();
    menu_dispatch(&harness.dom, "visible", more_target, false);
    harness.settle();
    assert_eq!(
        harness.settings.borrow().overflow_requests,
        [(false, true), (true, false)]
    );
    assert_eq!(harness.settings.borrow().overflow_changes, [true]);
    assert_eq!(
        harness.header_root_attrs()["data-open"],
        "true",
        "Dioxus visibility admission honors the root veto"
    );

    harness.settings.borrow_mut().overflow_veto = false;
    menu_dispatch(&harness.dom, "visible", more_target, false);
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
    for (iteration, (breakpoint, top_layer)) in [
        (ApplicationBreakpoint::Small, false),
        (ApplicationBreakpoint::Large, true),
        (ApplicationBreakpoint::Small, true),
        (ApplicationBreakpoint::Large, false),
    ]
    .into_iter()
    .enumerate()
    {
        {
            let mut settings = harness.settings.borrow_mut();
            settings.breakpoint = breakpoint;
            settings.slot_top_layer = top_layer;
        }
        harness.update();
        assert_eq!(
            harness.header_raw(harness.header_message(1, "layout", serde_json::json!({}))),
            (false, false)
        );
        harness.settle();
        assert_eq!(harness.header_more_click(false), (false, false));
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "true");
        assert_eq!(
            menu_dispatch(&harness.dom, "click", harness.slot_buttons["overflow"], ()),
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
                html.contains(&format!(
                    "{label} count {}",
                    if label == "overflow" {
                        iteration + 2
                    } else {
                        1
                    }
                )),
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
fn app_switch_entry_click_dismisses_and_preserves_exact_duplicate_id_destinations() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    let navigation = harness.navigation();
    let (owner, revision) = {
        let model = navigation.model.borrow();
        (model.owner.clone(), model.revision)
    };
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
    assert!(html.contains("href=\"/alpha\" target=\"_self\""));
    assert!(html.contains("href=\"/beta\" target=\"named-destination\""));
    assert_eq!(
        harness.entry_click(1),
        (true, true),
        "closed dialog entries cannot navigate"
    );
    assert_eq!(navigation.modal.result(), None);
    harness.beforetoggle();
    assert!(navigation.modal.is_open());
    let result = harness.entry_click(1);
    assert_eq!(
        result,
        (
            cfg!(all(feature = "desktop", not(target_arch = "wasm32"))),
            true
        ),
        "web uses the native anchor default; desktop handles its exact target"
    );
    assert_eq!(
        navigation.modal.result(),
        Some(crate::ModalResult::Dismissed(
            crate::ModalDismissReason::Programmatic("app-switch".into())
        ))
    );
    assert!(!navigation.modal.is_open());
    assert_eq!(
        harness.entry_click(1),
        (true, true),
        "closed owner rejects a repeated click"
    );
    assert_eq!(
        harness.settings.borrow().ancestor_inputs,
        0,
        "entry clicks have no private input transport"
    );
    harness.settle();
    assert!(
        !navigation.model.borrow().alive,
        "dismissed subtree drops its entry owner"
    );
    assert_ne!(harness.navigation().model.borrow().owner, owner);
    assert!(
        harness
            .dom
            .in_runtime(|| navigation.activate(revision, 1))
            .is_none(),
        "retained old owner cannot authorize a replacement mount"
    );
}

#[test]
fn app_switch_entry_handlers_reject_unmounted_and_changed_authority() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let unmounted = AppSwitchHarness::unmounted(Some(app_switch_config_fixture()), false);
    assert!(!unmounted.navigation().model.borrow().mounted);
    unmounted.beforetoggle();
    assert_eq!(
        unmounted.entry_click(0),
        (true, true),
        "entry cannot act before its Dioxus mounted delivery"
    );
    assert!(unmounted.navigation().modal.is_open());
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    harness.beforetoggle();
    let navigation = harness.navigation();
    let revision = navigation.model.borrow().revision;
    assert!(
        harness
            .dom
            .in_runtime(|| navigation.activate(revision, 99))
            .is_none(),
        "missing occurrence cannot dismiss or navigate"
    );
    let mut authority = harness.authority();
    let mut replacement = app_switch_config_fixture();
    replacement.apps[0].url = "/new-authority".into();
    harness
        .dom
        .in_scope(ScopeId::ROOT, || authority.set(Some(replacement)));
    assert_eq!(
        harness.entry_click(0),
        (true, true),
        "latest direct config invalidates an old rendered entry before rerender"
    );
    assert!(navigation.modal.is_open());
    assert_eq!(navigation.modal.result(), None);
    harness.settle();
    assert!(
        harness
            .dom
            .in_runtime(|| navigation.activate(revision, 0))
            .is_none(),
        "prior render revision stays invalid"
    );
    assert!(harness.html().contains("href=\"/new-authority\""));
    assert_eq!(
        harness.entry_click(0).0,
        cfg!(all(feature = "desktop", not(target_arch = "wasm32")))
    );
    assert!(!navigation.modal.is_open());
}

#[test]
fn app_switch_live_none_retains_display_and_new_config_and_owner_invalidate_stale_entries() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
    harness.beforetoggle();
    let old = harness.navigation();
    let revision = old.model.borrow().revision;
    harness.set(None);
    assert!(Rc::ptr_eq(&old.model, &harness.navigation().model));
    assert!(old.modal.is_open());
    assert!(harness.html().contains("Alpha"));
    assert!(!harness.html().contains("ix-header-app-switch"));
    assert_eq!(
        harness.entry_click(1).0,
        cfg!(all(feature = "desktop", not(target_arch = "wasm32"))),
        "visible retained entries stay usable after public None"
    );
    assert!(!old.modal.is_open());
    harness.settle();
    let mut replacement = app_switch_config_fixture();
    replacement.label = "Updated title".into();
    replacement.apps[0].name = "Updated Alpha".into();
    harness.set(Some(replacement));
    assert!(
        harness
            .dom
            .in_runtime(|| old.activate(revision, 0))
            .is_none()
    );
    assert!(harness.html().contains("Updated title") && harness.html().contains("Updated Alpha"));
    let current = harness.navigation();
    let revision = current.model.borrow().revision;
    harness.settings.borrow_mut().visible = false;
    harness.update();
    assert!(!current.model.borrow().alive);
    assert!(!harness.html().contains("<dialog"));
    assert!(
        harness
            .dom
            .in_runtime(|| current.activate(revision, 0))
            .is_none(),
        "removed owner rejects retained work"
    );
}

#[test]
fn app_switch_snapshot_rsx_caller_keeps_forwarded_authority_current_across_renders() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let mut harness = AppSwitchHarness::new(None, true);
    assert!(!harness.html().contains("ix-header-app-switch"));
    harness.set(Some(app_switch_config_fixture()));
    assert!(harness.html().contains("ix-header-app-switch"));
    let old = harness.navigation();
    let (same_owner, revision) = {
        let model = old.model.borrow();
        (model.owner.clone(), model.revision)
    };
    let mut next = app_switch_config_fixture();
    next.label = "Snapshot title".into();
    next.apps[0].url = "/snapshot-new".into();
    harness.set(Some(next.clone()));
    assert_eq!(harness.navigation().model.borrow().owner, same_owner);
    assert!(
        harness
            .dom
            .in_runtime(|| old.activate(revision, 0))
            .is_none()
    );
    assert_eq!(
        harness.click(),
        (false, true),
        "captured ReadSignal follows the new lifted snapshot"
    );
    harness.beforetoggle();
    harness.set(None);
    assert!(!harness.html().contains("ix-header-app-switch"));
    assert!(harness.navigation().modal.is_open());
    assert_eq!(harness.navigation().model.borrow().display, next);
    assert_eq!(
        harness.entry_click(0).0,
        cfg!(all(feature = "desktop", not(target_arch = "wasm32"))),
        "retained snapshot entries stay usable after None"
    );
}

#[test]
fn app_switch_renders_exact_target_anchors_without_native_descriptors() {
    fn probe() -> Element {
        rsx! {Application {app_switch_config:AppSwitchConfig {current_app_id:"chosen".into(),apps:vec![AppSwitchEntry {id:"chosen".into(),name:"Example app".into(),description:"Example description".into(),icon_src:"data:image/svg+xml,%3Csvg/%3E".into(),url:"https://example.com/".into(),target:"_blank".into()}],..Default::default()},header:rsx!{ApplicationHeader{}}}}
    }
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("command=\"show-modal\""));
    assert!(html.contains("data-app-switch-opener"));
    assert!(html.contains("aria-haspopup=\"dialog\""));
    assert!(html.contains("<dialog"));
    assert!(html.contains("<a class=\"ix-application-switch-entry\""));
    assert!(html.contains("href=\"https://example.com/\""));
    assert!(html.contains("target=\"_blank\""));
    assert!(
        !html.contains("data-app-switch-url")
            && !html.contains("data-app-switch-target")
            && !html.contains("data-app-switch-observer")
    );
    assert!(html.contains("data-selected=\"true\""));
    assert!(html.contains("aria-current=\"page\""));
}

fn header_registered_control_id(harness: &AppSwitchHarness, prefix: &str) -> Option<String> {
    let html = harness.html();
    let suffix = html.split_once(&format!(" id=\"{prefix}"))?.1;
    Some(format!("{prefix}{}", suffix.split('"').next()?))
}

#[test]
fn header_optional_menu_focus_registration_expires_with_its_control_scope() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::configured(None, false, |state| {
        state.contained = false;
        state.show_menu = true;
    });
    h.settle();
    let runtime = h.settings.borrow().overlay_runtime.clone().unwrap();
    let first_id = header_registered_control_id(&h, "ix-header-menu-toggle-").unwrap();
    let first = runtime.capture_registered_focus(&first_id).unwrap();
    // Keep the backing handle alive deliberately: scope cleanup must invalidate
    // registration even when some other consumer retained the mounted element.
    let retained = first.live_node().unwrap();
    h.settings.borrow_mut().name = "Updated application".into();
    h.update();
    assert_eq!(
        header_registered_control_id(&h, "ix-header-menu-toggle-").as_ref(),
        Some(&first_id)
    );
    assert!(first.live_node().is_some());
    h.write_show_menu(false);
    assert!(header_registered_control_id(&h, "ix-header-menu-toggle-").is_none());
    assert!(runtime.capture_registered_focus(&first_id).is_none());
    assert!(first.live_node().is_none());
    h.write_show_menu(true);
    let replacement_id = header_registered_control_id(&h, "ix-header-menu-toggle-").unwrap();
    assert_ne!(replacement_id, first_id);
    assert!(runtime.capture_registered_focus(&replacement_id).is_some());
    assert!(first.live_node().is_none());
    assert!(first.matches_node(&retained));
    h.settings.borrow_mut().visible = false;
    h.update();
    assert!(runtime.capture_registered_focus(&replacement_id).is_none());
}

#[test]
fn header_legacy_apps_registration_tracks_responsive_hiding_and_configured_replacement() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::configured(None, false, |state| state.show_legacy_apps = true);
    h.settle();
    let runtime = h.settings.borrow().overlay_runtime.clone().unwrap();
    let first_id = header_registered_control_id(&h, "ix-header-apps-").unwrap();
    let first = runtime.capture_registered_focus(&first_id).unwrap();
    let retained = first.live_node().unwrap();
    let target = h
        .attrs
        .iter()
        .find(|(_, attrs)| attrs.get("id") == Some(&first_id))
        .unwrap()
        .0;
    assert_eq!(
        menu_dispatch(
            &h.dom,
            "click",
            dioxus::core::ElementId::from_raw(*target),
            ()
        ),
        (false, true)
    );
    assert_eq!(h.settings.borrow().requests, 1);
    h.settings.borrow_mut().breakpoint = ApplicationBreakpoint::Small;
    h.update();
    assert!(header_registered_control_id(&h, "ix-header-apps-").is_none());
    assert!(runtime.capture_registered_focus(&first_id).is_none());
    assert!(first.live_node().is_none());
    h.settings.borrow_mut().breakpoint = ApplicationBreakpoint::Large;
    h.update();
    let restored_id = header_registered_control_id(&h, "ix-header-apps-").unwrap();
    let restored = runtime.capture_registered_focus(&restored_id).unwrap();
    assert_ne!(restored_id, first_id);
    h.set(Some(app_switch_config_fixture()));
    let configured_id = header_registered_control_id(&h, "ix-app-switch-opener-").unwrap();
    let configured = runtime.capture_registered_focus(&configured_id).unwrap();
    assert!(header_registered_control_id(&h, "ix-header-apps-").is_none());
    assert!(runtime.capture_registered_focus(&restored_id).is_none());
    assert!(restored.live_node().is_none());
    h.set(None);
    assert!(configured.live_node().is_none());
    let replacement_id = header_registered_control_id(&h, "ix-header-apps-").unwrap();
    assert!(runtime.capture_registered_focus(&replacement_id).is_some());
    assert_ne!(replacement_id, restored_id);
    h.settings.borrow_mut().show_legacy_apps = false;
    h.update();
    assert!(runtime.capture_registered_focus(&replacement_id).is_none());
    assert!(first.live_node().is_none());
    assert!(first.matches_node(&retained));
}

fn header_group_populated(harness: &AppSwitchHarness, group: &str) -> bool {
    let html = harness.html();
    let start = html
        .find(&format!("data-header-group=\"{group}\""))
        .expect("Header group");
    let tag = &html[start..start + html[start..].find('>').unwrap()];
    tag.contains("data-populated=\"true\"")
}

#[test]
fn header_slot_presence_uses_rust_options_and_explicit_empty_wrapper_declarations() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::configured(None, false, |state| {
        state.breakpoint = ApplicationBreakpoint::Small;
        state.overflow_present = false;
    });
    h.settle();
    assert!(header_group_populated(&h, "secondary"));
    assert!(header_group_populated(&h, "default"));
    assert!(!header_group_populated(&h, "overflow"));
    // Historical native presence data cannot override the authored Rust slots.
    h.header_raw(h.header_message(
        1,
        "layout",
        serde_json::json!({"presence":[false,false,true,false]}),
    ));
    h.settle();
    assert!(header_group_populated(&h, "default"));
    assert!(!header_group_populated(&h, "overflow"));
    {
        let mut state = h.settings.borrow_mut();
        state.slot_empty = true;
        state.slot_presence = [Some(false); 4];
    }
    h.update();
    assert!(!header_group_populated(&h, "secondary"));
    assert!(!header_group_populated(&h, "default"));
    assert!(!header_group_populated(&h, "overflow"));
    assert!(!h.html().contains("default count"));
    assert_eq!(h.settings.borrow().slot_state.0.borrow()["default"], (1, 0));
    // Removing a declaration restores Some/None semantics, even when a custom
    // wrapper currently returns empty content; no DOM discovery substitutes.
    h.settings.borrow_mut().slot_presence = [None; 4];
    h.update();
    assert!(header_group_populated(&h, "default"));
    assert!(!header_group_populated(&h, "overflow"));
    {
        let mut state = h.settings.borrow_mut();
        state.slot_empty = false;
        state.overflow_present = true;
    }
    h.update();
    assert!(header_group_populated(&h, "overflow"));
    assert_eq!(h.settings.borrow().slot_state.0.borrow()["default"], (1, 0));
}

#[test]
fn header_standard_fields_register_their_native_controls_and_invalidate_replaced_scopes() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::configured(None, false, |state| state.slot_form_controls = true);
    h.settle();
    let runtime = h.settings.borrow().overlay_runtime.clone().unwrap();
    let ids = [
        "header-form-input-0",
        "header-form-textarea-0",
        "header-form-select-0",
    ];
    let captured: Vec<_> = ids
        .iter()
        .map(|id| {
            runtime
                .capture_registered_focus(id)
                .expect("mounted native focus control")
        })
        .collect();
    let retained: Vec<_> = captured
        .iter()
        .map(|target| target.live_node().unwrap())
        .collect();
    assert!(
        runtime
            .capture_registered_focus("header-form-select-0-control")
            .is_none(),
        "presentation wrapper is not the Select focus target"
    );
    h.settings.borrow_mut().slot_form_revision = 1;
    h.update();
    for (id, target) in ids.iter().zip(&captured) {
        assert!(runtime.capture_registered_focus(id).is_none());
        assert!(target.live_node().is_none());
    }
    let replacements: Vec<_> = [
        "header-form-input-1",
        "header-form-textarea-1",
        "header-form-select-1",
    ]
    .iter()
    .map(|id| runtime.capture_registered_focus(id).unwrap())
    .collect();
    h.settings.borrow_mut().slot_form_controls = false;
    h.update();
    assert!(
        replacements
            .iter()
            .all(|target| target.live_node().is_none())
    );
    assert!(
        captured
            .iter()
            .zip(&retained)
            .all(|(target, node)| target.matches_node(node))
    );
}

fn rendered_control_tag(harness: &AppSwitchHarness, id: &str) -> String {
    let html = harness.html();
    let marker = format!(" id=\"{id}\"");
    let position = html.find(&marker).expect("rendered registered control ID");
    let start = html[..position].rfind('<').unwrap();
    let end = position + html[position..].find('>').unwrap();
    html[start..end].to_string()
}

#[test]
fn header_choice_controls_register_actual_native_targets_and_expire_on_replacement() {
    let _guard = crate::test_event_converter_guard();
    let mut h = AppSwitchHarness::configured(None, false, |state| state.slot_form_controls = true);
    h.settle();
    let runtime = h.settings.borrow().overlay_runtime.clone().unwrap();
    let ids = [
        "header-form-checkbox-0",
        "header-form-radio-0",
        "header-form-toggle-0",
    ];
    let captures: Vec<_> = ids
        .iter()
        .map(|id| {
            runtime
                .capture_registered_focus(id)
                .expect("mounted interactive choice control")
        })
        .collect();
    let retained: Vec<_> = captures
        .iter()
        .map(|target| target.live_node().unwrap())
        .collect();
    assert!(rendered_control_tag(&h, ids[0]).starts_with("<button "));
    assert!(rendered_control_tag(&h, ids[0]).contains("role=\"checkbox\""));
    assert!(rendered_control_tag(&h, ids[1]).starts_with("<input "));
    assert!(rendered_control_tag(&h, ids[1]).contains("type=\"radio\""));
    assert!(rendered_control_tag(&h, ids[2]).starts_with("<button "));
    assert!(rendered_control_tag(&h, ids[2]).contains("role=\"switch\""));
    for kind in ["checkbox", "radio", "toggle"] {
        let id = format!("header-form-disabled-{kind}-0");
        let tag = rendered_control_tag(&h, &id);
        assert!(
            tag.contains(" disabled"),
            "disabled eligibility stays on the registered native control"
        );
    }
    let html = h.html();
    for mirror in ["header-checkbox-mirror-0", "header-toggle-mirror-0"] {
        let marker = format!("name=\"{mirror}\"");
        let position = html.find(&marker).unwrap();
        let start = html[..position].rfind('<').unwrap();
        let end = position + html[position..].find('>').unwrap();
        let tag = &html[start..end];
        assert!(
            tag.starts_with("<input ")
                && tag.contains("ix-check-native")
                && tag.contains("aria-hidden=\"true\"")
        );
        assert!(
            !tag.contains(" id="),
            "submission mirror does not take the public focus ID"
        );
    }
    // RadioGroup composes the same native Radio component. Its generated ID
    // must register without requiring explicit IDs for every option.
    let group_ids: Vec<String> = html
        .split("<input ")
        .skip(1)
        .map(|tail| tail.split('>').next().unwrap())
        .filter(|tag| tag.contains("name=\"header-form-radio-group-0\""))
        .map(|tag| {
            tag.split_once("id=\"")
                .unwrap()
                .1
                .split('"')
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(group_ids.len(), 2);
    let group_captures: Vec<_> = group_ids
        .iter()
        .map(|id| {
            runtime
                .capture_registered_focus(id)
                .expect("RadioGroup native option registration")
        })
        .collect();
    h.settings.borrow_mut().slot_form_revision = 1;
    h.update();
    for (id, target) in ids.iter().zip(&captures) {
        assert!(runtime.capture_registered_focus(id).is_none());
        assert!(target.live_node().is_none());
    }
    assert!(
        group_captures
            .iter()
            .all(|target| target.live_node().is_none())
    );
    let replacements: Vec<_> = [
        "header-form-checkbox-1",
        "header-form-radio-1",
        "header-form-toggle-1",
    ]
    .iter()
    .map(|id| runtime.capture_registered_focus(id).unwrap())
    .collect();
    h.settings.borrow_mut().slot_form_controls = false;
    h.update();
    assert!(
        replacements
            .iter()
            .all(|target| target.live_node().is_none())
    );
    assert!(
        captures
            .iter()
            .zip(&retained)
            .all(|(target, node)| target.matches_node(node))
    );
}
