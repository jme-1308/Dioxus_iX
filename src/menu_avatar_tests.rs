// Reuses the application test converter/mounted mock; no second event transport.
#[derive(Clone)]
struct AccountConfig {
    top: String,
    logout: String,
    custom: bool,
    hide_logout: bool,
    top_layer: bool,
    disabled: bool,
    tooltip: Option<String>,
    tooltip_label: Option<String>,
}
impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            top: "Operator One".into(),
            logout: "Logout".into(),
            custom: true,
            hide_logout: false,
            top_layer: false,
            disabled: false,
            tooltip: None,
            tooltip_label: None,
        }
    }
}
#[derive(Default)]
struct AccountFixture {
    config: RefCell<AccountConfig>,
    mounts: Cell<usize>,
    actions: Cell<usize>,
    logouts: Cell<usize>,
    veto: Cell<bool>,
    keep_action_open: Cell<bool>,
    changes: RefCell<Vec<bool>>,
}
impl PartialEq for AccountFixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn AccountDraft(data: Rc<AccountFixture>) -> Element {
    let mut draft = use_signal(|| String::from("Service shift"));
    let binding = crate::use_dropdown_item("account-draft");
    rsx! {input{id:String::from("account-draft"),aria_label:"Shift note",value:draft(),tabindex:binding.tabindex,
        onmounted:move|event|{binding.onmounted.call(event);data.mounts.set(data.mounts.get()+1)},
        oninput:move|event|draft.set(event.value()),..binding.attributes()
    }}
}
fn account_fixture(data: Rc<AccountFixture>) -> Element {
    let config = data.config.borrow().clone();
    let action = data.clone();
    let logout = data.clone();
    let before = data.clone();
    let changed = data.clone();
    let children = config.custom.then(||rsx!{
        AccountDraft {data:data.clone()}
        MenuAvatarItem {label:"Preferences",icon:"settings",disabled:config.disabled,onclick:move|event:MouseEvent|{
            action.actions.set(action.actions.get()+1);
            if action.keep_action_open.get(){event.prevent_default();}
        }}
        MenuAvatarItem {label:"Unavailable",disabled:true}
    });
    rsx! {MenuAvatar{id:"account-test",top:config.top,bottom:"Maintenance team",children,hide_logout_button:config.hide_logout,
        i18n_logout:config.logout,enable_top_layer:config.top_layer,tooltip_text:config.tooltip,aria_label_tooltip:config.tooltip_label,
        onbeforechange:move|event:crate::DropdownChangeEvent|if before.veto.get(){event.prevent_default()},
        onchange:move|open|changed.changes.borrow_mut().push(open),
        onlogout:move|_|logout.logouts.set(logout.logouts.get()+1)
    }}
}
struct AccountHarness {
    dom: VirtualDom,
    data: Rc<AccountFixture>,
    ids: BTreeMap<String, dioxus::core::ElementId>,
    mounts: Vec<dioxus::core::ElementId>,
    focuses: Rc<RefCell<Vec<usize>>>,
}
impl AccountHarness {
    fn new() -> Self {
        let data = Rc::new(AccountFixture::default());
        let mut dom = VirtualDom::new_with_props(account_fixture, data.clone());
        let changes = dom.rebuild_to_vec();
        let mut rig = Self {
            dom,
            data,
            ids: BTreeMap::new(),
            mounts: Vec::new(),
            focuses: Rc::default(),
        };
        rig.collect(changes);
        rig.settle();
        rig
    }
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
                } if name == "id" => {
                    self.ids.insert(value, current);
                }
                dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                    self.mounts.push(current)
                }
                _ => {}
            }
        }
    }
    fn settle(&mut self) {
        for _ in 0..8 {
            for target in std::mem::take(&mut self.mounts) {
                let mount = MenuTestMount {
                    id: target.raw(),
                    focuses: self.focuses.clone(),
                    ..Default::default()
                };
                mount.ready.set(true);
                menu_dispatch(&self.dom, "mounted", target, mount);
            }
            let changes = self.dom.render_immediate_to_vec();
            self.collect(changes);
            self.dom.process_events();
        }
    }
    fn refresh(&mut self) {
        self.dom.mark_dirty(ScopeId::APP);
        self.settle();
    }
    fn html(&self) -> String {
        dioxus_ssr::render(&self.dom)
    }
    fn button(&self, label: &str) -> dioxus::core::ElementId {
        let html = self.html();
        let button = html
            .split("<button ")
            .skip(1)
            .find(|part| part.split("</button>").next().unwrap().contains(label))
            .unwrap_or_else(|| panic!("Missing native button {label}: {html}"));
        let id = button
            .split("id=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        self.ids[id]
    }
    fn trigger(&self) -> dioxus::core::ElementId {
        self.button(&self.data.config.borrow().top)
    }
    fn event(
        &mut self,
        name: &str,
        target: dioxus::core::ElementId,
        payload: impl std::any::Any,
    ) -> (bool, bool) {
        let result = menu_dispatch(&self.dom, name, target, payload);
        self.settle();
        result
    }
    fn click(&mut self, label: &str) {
        let id = self.button(label);
        self.event("click", id, ());
    }
    fn open(&self) -> bool {
        self.html()
            .split('<')
            .find(|tag| tag.starts_with("button ") && tag.contains("ix-menu-avatar-trigger"))
            .unwrap()
            .contains("aria-expanded=\"true\"")
    }
}
#[test]
fn menu_avatar_shared_surface_retains_draft_across_close_and_account_updates() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(MenuTestConverter));
    let mut rig = AccountHarness::new();
    assert_eq!(
        rig.data.mounts.get(),
        1,
        "persistent children mount before opening"
    );
    assert!(
        rig.html()
            .contains("data-configured-placement=\"top-start\"")
    );
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    assert!(rig.open());
    let draft = rig.ids["account-draft"];
    rig.event("input", draft, String::from("Night shift prepared"));
    assert!(
        !rig.event("keydown", draft, Key::ArrowLeft).0,
        "native editing keys retain defaults"
    );
    rig.event("keydown", draft, Key::Escape);
    assert!(!rig.open());
    rig.data.config.borrow_mut().top = "Operator Two".into();
    rig.refresh();
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    assert!(rig.open());
    assert!(
        item_opening_tag(&rig.html(), "account-draft").contains("value=\"Night shift prepared\"")
    );
    assert_eq!(rig.data.mounts.get(), 1);
    assert!(!rig.html().contains("ix-avatar-backdrop"));
}
#[test]
fn menu_avatar_native_actions_preserve_keys_cancellation_and_disabled_guards() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(MenuTestConverter));
    let mut rig = AccountHarness::new();
    rig.data.veto.set(true);
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    assert!(!rig.open());
    rig.data.veto.set(false);
    rig.event("click", trigger, ());
    assert!(rig.open());
    let action = rig.button("Preferences");
    assert!(!rig.event("keydown", action, Key::Enter).0);
    assert!(!rig.event("keydown", action, Key::Character(" ".into())).0);
    assert_eq!(
        rig.data.actions.get(),
        0,
        "native keys do not fabricate Rust clicks"
    );
    rig.focuses.borrow_mut().clear();
    let logout = rig.button("Logout");
    rig.event("focus", action, ());
    assert!(rig.event("keydown", action, Key::ArrowDown).0);
    assert_eq!(
        rig.focuses.borrow().last().copied(),
        Some(logout.raw()),
        "shared arrows skip disabled Unavailable"
    );
    rig.data.keep_action_open.set(true);
    rig.click("Preferences");
    assert_eq!(rig.data.actions.get(), 1);
    assert!(rig.open());
    rig.data.keep_action_open.set(false);
    rig.click("Preferences");
    assert_eq!(rig.data.actions.get(), 2);
    assert!(!rig.open());
    rig.data.config.borrow_mut().disabled = true;
    rig.refresh();
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    rig.click("Preferences");
    assert_eq!(
        rig.data.actions.get(),
        2,
        "synthetic disabled click cannot call consumer"
    );
    // Tab releases the shared roving menu without replacing native input keys.
    let trigger = rig.trigger();
    if !rig.open() {
        rig.event("click", trigger, ());
    }
    let draft = rig.ids["account-draft"];
    rig.event("keydown", draft, Key::Tab);
    assert!(!rig.open());
}
#[test]
fn menu_avatar_localized_logout_top_layer_tooltip_and_empty_membership_follow_live_props() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(MenuTestConverter));
    let mut rig = AccountHarness::new();
    {
        let mut config = rig.data.config.borrow_mut();
        config.logout = "Abmelden".into();
        config.top_layer = true;
        config.tooltip = Some("Account actions".into());
        config.tooltip_label = Some("Kontoinformationen".into());
    }
    rig.refresh();
    let html = rig.html();
    assert!(html.contains("data-ix-overlay-top-layer=\"true\""));
    assert!(html.contains("popover=\"manual\""));
    assert!(html.contains("aria-label=\"Kontoinformationen\""));
    assert!(html.contains("Account actions"));
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    assert!(rig.open());
    assert!(!rig.html().contains("role=\"tooltip\""));
    rig.click("Abmelden");
    assert_eq!(rig.data.logouts.get(), 1);
    assert!(!rig.open());
    rig.data.config.borrow_mut().hide_logout = true;
    rig.refresh();
    assert!(!rig.html().contains("Abmelden"));
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    assert!(rig.open(), "custom actions still available without logout");
    rig.data.config.borrow_mut().custom = false;
    rig.refresh();
    assert!(!rig.open());
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    assert!(!rig.open());
    let html = rig.html();
    let trigger_tag = html
        .split('<')
        .find(|tag| tag.starts_with("button ") && tag.contains("ix-menu-avatar-trigger"))
        .unwrap();
    assert!(!trigger_tag.contains("aria-haspopup=") && !trigger_tag.contains("aria-controls="));
    rig.data.config.borrow_mut().hide_logout = false;
    rig.refresh();
    let trigger = rig.trigger();
    rig.event("click", trigger, ());
    assert!(rig.open());
    rig.click("Abmelden");
    assert_eq!(rig.data.logouts.get(), 2);
}
