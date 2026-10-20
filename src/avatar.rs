//! Static profile faces and explicitly composed Dioxus profile menus.
use crate::{
    ButtonVariant, Divider, Dropdown, DropdownChangeEvent, DropdownNavigationMode,
    DropdownPlacement, DropdownTrigger, Icon, use_dropdown,
};
use dioxus::prelude::*;

/// Profile image, initials, or generic user icon, with an explicitly opted-in
/// profile menu. Static rendering remains the default. `menu` or supplied
/// `children` enables the shared persistent Dropdown; children own their actions.
/// No surrounding Header or arbitrary DOM content is discovered. The menu
/// defaults to bottom-start placement, following the upstream Dropdown default.
///
/// Failed images fall back only for that source; a changed image can load
/// normally. Native `title` uses `tooltip` or username while closed and is
/// suppressed while the menu is open. Custom tooltip styling/labels are not
/// implemented by this native-title adaptation.
#[component]
pub fn Avatar(
    username: Option<String>,
    initials: Option<String>,
    image: Option<String>,
    #[props(default = 32)] size: u32,
    tooltip: Option<String>,
    #[props(default)] menu: bool,
    #[props(default)] extra: Option<String>,
    #[props(default)] children: Option<Element>,
    #[props(default)] disabled: bool,
    #[props(default)] aria_label: Option<String>,
    #[props(default="User menu".to_string())] menu_label: String,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] enable_top_layer: bool,
    #[props(default=DropdownPlacement::BottomStart)] placement: DropdownPlacement,
    #[props(default)] navigation_mode: DropdownNavigationMode,
    #[props(default)] onbeforechange: EventHandler<DropdownChangeEvent>,
    #[props(default)] onchange: EventHandler<bool>,
) -> Element {
    let failed_source = use_signal(|| None::<String>);
    let label = aria_label
        .filter(|label| !label.trim().is_empty())
        .or_else(|| username.clone().filter(|name| !name.trim().is_empty()))
        .unwrap_or_else(|| "User profile".into());
    let title = tooltip.or_else(|| username.clone());
    if menu || children.is_some() {
        rsx! {AvatarMenu{username,initials,image,size,title,extra,children,disabled,label,menu_label,id,class,enable_top_layer,placement,navigation_mode,onbeforechange,onchange,failed_source}}
    } else {
        rsx! {AvatarFace{image,initials,size,label:Some(label),title,id,class,failed_source}}
    }
}

#[component]
fn AvatarFace(
    image: Option<String>,
    initials: Option<String>,
    size: u32,
    failed_source: Signal<Option<String>>,
    #[props(default)] label: Option<String>,
    #[props(default)] title: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] decorative: bool,
) -> Element {
    let visible_image = image.filter(|source| failed_source().as_ref() != Some(source));
    rsx! {span{id,class:"ix-avatar {class}",style:"--ix-avatar-size:{size}px",role:if decorative{"presentation"}else{"img"},"aria-hidden":decorative.then_some("true"),"aria-label":label,title,
        if let Some(source)=visible_image{img{key:"{source}",src:source.clone(),alt:"",onerror:move |_|failed_source.set(Some(source.clone()))}}
        else if let Some(initials)=initials{span{"aria-hidden":"true","{initials}"}}
        else{Icon{name:"user",size:20}}
    }}
}

#[component]
fn AvatarMenu(
    username: Option<String>,
    initials: Option<String>,
    image: Option<String>,
    size: u32,
    title: Option<String>,
    extra: Option<String>,
    children: Option<Element>,
    disabled: bool,
    label: String,
    menu_label: String,
    id: Option<String>,
    class: String,
    enable_top_layer: bool,
    placement: DropdownPlacement,
    navigation_mode: DropdownNavigationMode,
    onbeforechange: EventHandler<DropdownChangeEvent>,
    onchange: EventHandler<bool>,
    failed_source: Signal<Option<String>>,
) -> Element {
    let dropdown = use_dropdown();
    let open = dropdown.is_open();
    let profile = username.filter(|name| !name.trim().is_empty());
    let divider = profile.is_some() && children.is_some();
    rsx! {div{id,class:"ix-avatar-menu {class}",style:"--ix-avatar-size:{size}px",title:(!open).then_some(title).flatten(),
        DropdownTrigger{dropdown,disabled,aria_label:label,variant:ButtonVariant::SubtleTertiary,class:"ix-avatar-trigger",
            AvatarFace{image:image.clone(),initials:initials.clone(),size,failed_source,decorative:true}
        }
        Dropdown{controller:dropdown,label:menu_label,disabled,placement,navigation_mode,enable_top_layer,onbeforechange,onchange,class:"ix-avatar-menu-surface",
            if let Some(username)=profile{div{class:"ix-avatar-profile",onclick:move|event:MouseEvent|event.prevent_default(),
                AvatarFace{image,initials,size,failed_source,decorative:true}
                div{class:"ix-avatar-profile-text",span{class:"ix-avatar-username","{username}"}if let Some(extra)=extra{span{class:"ix-avatar-extra","{extra}"}}}
            }}
            if divider{div{class:"ix-avatar-divider",onclick:move|event:MouseEvent|event.prevent_default(),Divider{}}}
            {children}
        }
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DropdownItem;
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        rc::Rc,
    };
    #[derive(Clone)]
    struct Config {
        username: Option<String>,
        initials: Option<String>,
        image: Option<String>,
        tooltip: Option<String>,
        size: u32,
        menu: bool,
        children: bool,
        disabled: bool,
        top: bool,
    }
    impl Default for Config {
        fn default() -> Self {
            Self {
                username: Some("Operator One".into()),
                initials: Some("OP".into()),
                image: Some("portrait-one.png".into()),
                tooltip: Some("Open operator details".into()),
                size: 32,
                menu: false,
                children: false,
                disabled: false,
                top: false,
            }
        }
    }
    #[derive(Default)]
    struct Fixture {
        config: RefCell<Config>,
        veto: Cell<bool>,
        before: RefCell<Vec<bool>>,
        changes: RefCell<Vec<bool>>,
        actions: Cell<usize>,
        mounts: Cell<usize>,
    }
    impl PartialEq for Fixture {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self, other)
        }
    }
    #[component]
    fn MenuDraft(data: Rc<Fixture>) -> Element {
        let mut note = use_signal(|| String::from("Shift note"));
        rsx! {input{id:String::from("avatar-draft"),value:note(),onmounted:move|_|data.mounts.set(data.mounts.get()+1),oninput:move|event|note.set(event.value()),onclick:move|event:MouseEvent|event.stop_propagation(),onkeydown:move|event:KeyboardEvent|if !matches!(event.key(),Key::Escape|Key::Tab){event.stop_propagation();}}}
    }
    fn fixture(data: Rc<Fixture>) -> Element {
        let config = data.config.borrow().clone();
        let before = data.clone();
        let changed = data.clone();
        let action = data.clone();
        let children=config.children.then(||rsx!{MenuDraft{data:data.clone()}DropdownItem{label:"Preferences",order:0,onitemclick:move|_|action.actions.set(action.actions.get()+1)}});
        rsx! {crate::actions::Avatar{id:"avatar-test",class:"custom-avatar",username:config.username,initials:config.initials,image:config.image,tooltip:config.tooltip,size:config.size,menu:config.menu,children,disabled:config.disabled,enable_top_layer:config.top,extra:"Service shift",menu_label:"Operator menu",
            onbeforechange:move|event:DropdownChangeEvent|{before.before.borrow_mut().push(event.open);if before.veto.get(){event.clone().prevent_default();assert!(event.default_prevented());}},onchange:move|open|changed.changes.borrow_mut().push(open)
        }}
    }
    struct ImageFailure;
    impl dioxus::html::HasImageData for ImageFailure {
        fn load_error(&self) -> bool {
            true
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
    struct RawForm(String);
    impl dioxus::html::HasFileData for RawForm {
        fn files(&self) -> Vec<dioxus::html::FileData> {
            Vec::new()
        }
    }
    impl dioxus::html::HasFormData for RawForm {
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
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event;converter!(@data $group,event) })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Image,$event:ident) => { dioxus::html::ImageData::new(ImageFailure) };
        (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RawKey>().unwrap().clone()) };
        (@data Mounted,$event:ident) => { MountedData::new(()) };
        (@data Form,$event:ident) => { dioxus::html::FormData::new(RawForm($event.downcast::<String>().unwrap().clone())) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected avatar test event") };
    }
    dioxus::html::with_html_event_groups!(converter);
    #[derive(Default)]
    struct Targets {
        attrs: BTreeMap<usize, BTreeMap<String, String>>,
        mounted: Vec<dioxus::core::ElementId>,
    }
    impl Targets {
        fn collect(&mut self, changes: dioxus::core::Mutations) {
            let mut current = dioxus::core::ElementId::ROOT;
            for edit in changes.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
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
                    dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                        self.mounted.push(current)
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
    }
    fn dispatch(
        dom: &VirtualDom,
        name: &str,
        id: dioxus::core::ElementId,
        data: Box<dyn std::any::Any>,
    ) -> bool {
        let event = Event::new(
            Rc::new(dioxus::html::PlatformEventData::new(data)) as Rc<dyn std::any::Any>,
            !matches!(name, "mounted" | "error"),
        );
        dom.runtime().handle_event(name, event.clone(), id);
        event.default_action_enabled()
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

    fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets) {
        let mut dom = VirtualDom::new_with_props(fixture, data);
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        for id in targets.mounted.clone() {
            dispatch(&dom, "mounted", id, Box::new(()));
        }
        settle(&mut dom, &mut targets);
        (dom, targets)
    }
    fn tag<'a>(html: &'a str, name: &str, value: &str) -> &'a str {
        html.split('<')
            .map(|tag| tag.split('>').next().unwrap())
            .find(|tag| tag.contains(&format!("{name}=\"{value}\"")))
            .unwrap_or_else(|| panic!("Missing {name}={value}: {html}"))
    }
    fn trigger(html: &str) -> &str {
        html.split('<')
            .map(|tag| tag.split('>').next().unwrap())
            .find(|tag| tag.starts_with("button ") && tag.contains("ix-avatar-trigger"))
            .unwrap()
    }
    fn open(dom: &VirtualDom) -> bool {
        trigger(&dioxus_ssr::render(dom)).contains("aria-expanded=\"true\"")
    }
    #[test]
    fn avatar_static_precedence_failure_and_changed_source_recovery_preserve_legacy_presentation() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let (mut dom, mut targets) = setup(data.clone());
        let html = dioxus_ssr::render(&dom);
        let face = tag(&html, "id", "avatar-test");
        assert!(face.starts_with("span "));
        assert!(face.contains("role=\"img\""));
        assert!(face.contains("--ix-avatar-size:32px"));
        assert!(face.contains("title=\"Open operator details\""));
        assert!(!html.contains("ix-avatar-menu"));
        assert!(html.contains("src=\"portrait-one.png\""));
        assert!(!html.contains(">OP<"));
        dispatch(
            &dom,
            "error",
            targets.attr("src", "portrait-one.png"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("<img"));
        assert!(html.contains(">OP<"));
        data.config.borrow_mut().image = Some("portrait-two.svg".into());
        data.config.borrow_mut().size = 48;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("src=\"portrait-two.svg\""));
        assert!(!html.contains(">OP<"));
        assert!(tag(&html, "id", "avatar-test").contains("--ix-avatar-size:48px"));
        data.config.borrow_mut().image = None;
        data.config.borrow_mut().initials = None;
        data.config.borrow_mut().username = None;
        data.config.borrow_mut().tooltip = None;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(tag(&html, "id", "avatar-test").contains("aria-label=\"User profile\""));
        assert!(html.contains("<svg"));
        assert!(!html.contains("<button"));
    }
    #[test]
    fn avatar_composed_menu_veto_disabled_actions_and_profile_changes_retain_local_draft() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        data.config.borrow_mut().children = true;
        data.config.borrow_mut().image = None;
        let (mut dom, mut targets) = setup(data.clone());
        let html = dioxus_ssr::render(&dom);
        assert!(trigger(&html).contains("aria-label=\"Operator One\""));
        assert!(html.contains("ix-avatar-profile"));
        assert!(html.contains("Service shift"));
        assert!(html.contains("ix-avatar-divider"));
        assert!(html.contains("data-configured-placement=\"bottom-start\""));
        assert!(html.contains("role=\"presentation\""));
        let button = targets.attr("aria-label", "Operator One");
        data.veto.set(true);
        dispatch(&dom, "click", button, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(!open(&dom));
        assert!(data.changes.borrow().is_empty());
        data.veto.set(false);
        dispatch(&dom, "click", button, Box::new(()));
        settle(&mut dom, &mut targets);
        assert!(open(&dom));
        assert!(!tag(&dioxus_ssr::render(&dom), "id", "avatar-test").contains("title="));
        dispatch(
            &dom,
            "input",
            targets.attr("id", "avatar-draft"),
            Box::new(String::from("Night shift prepared")),
        );
        dispatch(
            &dom,
            "click",
            targets.attr("id", "avatar-draft"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        assert!(open(&dom));
        dispatch(
            &dom,
            "click",
            targets.attr("data-ix-dropdown-label", "Preferences"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        assert_eq!(data.actions.get(), 1);
        assert!(!open(&dom));
        dispatch(&dom, "click", button, Box::new(()));
        settle(&mut dom, &mut targets);
        dispatch(&dom, "keydown", button, Box::new(RawKey(Key::Escape)));
        settle(&mut dom, &mut targets);
        assert!(!open(&dom));
        data.config.borrow_mut().username = Some("Operator Two".into());
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(tag(&html, "id", "avatar-draft").contains("value=\"Night shift prepared\""));
        assert_eq!(data.mounts.get(), 1);
        assert!(trigger(&html).contains("aria-label=\"Operator Two\""));
        data.config.borrow_mut().disabled = true;
        refresh(&mut dom, &mut targets);
        dispatch(
            &dom,
            "click",
            targets.attr("aria-label", "Operator Two"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        assert!(!open(&dom));
        assert!(trigger(&dioxus_ssr::render(&dom)).contains("disabled"));
        data.config.borrow_mut().username = None;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("ix-avatar-profile"));
        assert!(!html.contains("ix-avatar-divider"));
        assert_eq!(data.mounts.get(), 1);
        assert!(html.contains("Night shift prepared"));
    }
    #[test]
    fn avatar_explicit_menu_profile_only_and_shared_image_failure_allow_replacement() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        data.config.borrow_mut().menu = true;
        let (mut dom, mut targets) = setup(data.clone());
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("ix-avatar-profile"));
        assert!(!html.contains("ix-avatar-divider"));
        assert_eq!(html.matches("src=\"portrait-one.png\"").count(), 2);
        dispatch(
            &dom,
            "error",
            targets.attr("src", "portrait-one.png"),
            Box::new(()),
        );
        settle(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("<img"));
        assert_eq!(html.matches(">OP<").count(), 2);
        data.config.borrow_mut().image = Some("portrait-new.png".into());
        data.config.borrow_mut().top = true;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("src=\"portrait-new.png\"").count(), 2);
        assert!(html.contains("popover=\"manual\""));
        let mut empty =
            VirtualDom::new(|| rsx! {crate::Avatar{menu:true,aria_label:"Open account"}});
        empty.rebuild_in_place();
        let html = dioxus_ssr::render(&empty);
        assert!(trigger(&html).contains("aria-label=\"Open account\""));
        assert!(html.contains("aria-label=\"User menu\""));
        assert!(!html.contains("ix-avatar-profile"));
        assert!(!html.contains("ix-avatar-divider"));
    }
}
