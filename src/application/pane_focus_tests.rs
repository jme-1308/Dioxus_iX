
    use super::*;
    use dioxus::core::{ElementId, Mutation};

    #[derive(Clone)]
    struct Mount {
        id: String,
        log: Rc<RefCell<Vec<String>>>,
    }
    impl dioxus::html::RenderedElementBacking for Mount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            let this = self.clone();
            Box::pin(async move {
                assert!(focus);
                this.log.borrow_mut().push(this.id);
                Ok(())
            })
        }
    }
    struct Converter;
    macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*
            }
        };
        (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<Mount>().unwrap().clone())};
        (@data $other:ident,$event:ident)=>{panic!("unexpected Pane test event")};
    }
    dioxus::html::with_html_event_groups!(converter);
    #[derive(Default)]
    struct Config {
        capture: PaneFocusCapture,
        expanded: Option<Signal<bool>>,
        floating: Option<Signal<bool>>,
        visible: Option<Signal<bool>>,
    }
    #[component]
    fn Field() -> Element {
        let id = "pane-field".to_string();
        let mounted = crate::dropdown::use_focus_mount(id.clone());
        rsx! { input {id,onmounted:mounted,autofocus:true} }
    }
    fn fixture(config: Rc<RefCell<Config>>) -> Element {
        let expanded = use_signal(|| false);
        let floating = use_signal(|| true);
        let visible = use_signal(|| true);
        let capture = config.borrow().capture.clone();
        use_context_provider(|| capture);
        {
            let mut config = config.borrow_mut();
            config.expanded = Some(expanded);
            config.floating = Some(floating);
            config.visible = Some(visible);
        }
        rsx! {
            crate::Button {id:"pane-opener", "Open"}
            if visible() { Pane {heading:"Test Pane",expanded:expanded(),variant:if floating(){PaneVariant::Floating}else{PaneVariant::Inline},hide_on_collapse:true,
                Field {}
                crate::Button {id:"pane-action","Action"}
            }}
        }
    }
    struct Harness {
        dom: VirtualDom,
        config: Rc<RefCell<Config>>,
        focus: PaneFocusController,
        ids: BTreeMap<String, ElementId>,
        toggle: String,
        log: Rc<RefCell<Vec<String>>>,
    }
    impl Harness {
        fn new() -> Self {
            dioxus::html::set_event_converter(Box::new(Converter));
            let config = Rc::new(RefCell::new(Config::default()));
            let mut dom = VirtualDom::new_with_props(fixture, config.clone());
            let mut ids = BTreeMap::new();
            let mut current = ElementId::ROOT;
            for edit in dom.rebuild_to_vec().edits {
                match edit {
                    Mutation::SetId { id } | Mutation::PushId { id } => current = id,
                    Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } if name == "id" => {
                        ids.insert(value, current);
                    }
                    _ => {}
                }
            }
            let focus = config.borrow().capture.0.borrow().clone().unwrap();
            let toggle = ids
                .keys()
                .find(|id| id.ends_with("-toggle"))
                .unwrap()
                .clone();
            let result = Self {
                dom,
                config,
                focus,
                ids,
                toggle,
                log: Rc::new(RefCell::new(Vec::new())),
            };
            for id in [
                "pane-opener",
                "pane-field",
                "pane-action",
                result.toggle.as_str(),
            ] {
                result.mount(id, id);
            }
            result
        }
        fn mount(&self, id: &str, record: &str) {
            let data: Rc<dyn std::any::Any> =
                Rc::new(dioxus::html::PlatformEventData::new(Box::new(Mount {
                    id: record.into(),
                    log: self.log.clone(),
                })));
            self.dom
                .runtime()
                .handle_event("mounted", Event::new(data, false), self.ids[id]);
        }
        fn settle(&mut self) {
            for _ in 0..6 {
                self.dom.render_immediate_to_vec();
            }
        }
        fn expanded(&mut self, value: bool) {
            self.dom.in_scope(ScopeId::ROOT, || {
                self.config.borrow().expanded.unwrap().set(value)
            });
            self.settle();
        }
        fn candidate(&self, id: &str, autofocus: bool, inside: bool) -> crate::popover_focus::NativeFocusCandidate {
            serde_json::from_value(serde_json::json!({
                "target":id,"tab_index":0,"autofocus":autofocus,
                "hosts":if inside {vec![self.focus.root_id.clone()]} else {Vec::new()}
            })).unwrap()
        }
        fn facts(&self, active: Option<&str>, inside: bool) -> crate::overlay_runtime::ScopedFocusFacts {
            crate::overlay_runtime::ScopedFocusFacts {
                observation: crate::overlay_runtime::FocusObservation {
                    candidates: vec![
                        self.candidate(&self.toggle, false, true),
                        self.candidate("pane-field", true, true),
                        self.candidate("pane-action", false, true),
                        self.candidate("pane-opener", false, false),
                    ],
                    active_id: active.unwrap_or_default().to_string(),
                    active_default: active.is_none(),
                },
                active_hosts: if inside {vec![self.focus.root_id.clone()]} else {Vec::new()},
                eligible_focus_ids: vec![self.toggle.clone(),"pane-field".into(),"pane-action".into(),"pane-opener".into()],
                nested: false,
            }
        }
        fn report_pending(&self, facts: crate::overlay_runtime::ScopedFocusFacts) {
            let epoch = self.focus.model.borrow().epoch;
            self.dom
                .in_scope(ScopeId::ROOT, || self.focus.observe(crate::overlay_runtime::ScopedFocusEvent::Requested {request:epoch,facts}));
        }
        fn report(&mut self, facts: crate::overlay_runtime::ScopedFocusFacts) {
            self.report_pending(facts);
            self.settle();
        }
        fn tab(&mut self, backwards: bool) -> bool {
            let prevented = self
                .dom
                .in_scope(ScopeId::ROOT, || self.focus.tab(backwards));
            self.settle();
            prevented
        }
    }
    #[test]
    fn floating_pane_opens_registered_autofocus_and_wraps_only_native_boundaries() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        let mut facts = h.facts(Some("pane-opener"), false);
        facts.observation.candidates.insert(0, h.candidate("unregistered-autofocus", true, true));
        h.report(facts);
        assert_eq!(*h.log.borrow(), ["pane-field"]);
        h.report(h.facts(Some("pane-field"), true));
        assert!(!h.tab(false), "interior Tab keeps its native default");
        assert_eq!(h.log.borrow().len(), 1);
        h.report(h.facts(Some("pane-action"), true));
        assert!(h.tab(false));
        assert_eq!(h.log.borrow().last(), Some(&h.toggle));
        h.report(h.facts(Some(&h.toggle), true));
        assert!(h.tab(true));
        assert_eq!(
            h.log.borrow().last().map(String::as_str),
            Some("pane-action")
        );
        let mut unsupported = h.facts(Some("pane-action"), true);
        unsupported.observation.candidates.insert(0, h.candidate("unregistered-boundary", false, true));
        h.report(unsupported);
        assert!(
            !h.tab(false),
            "an unsupported destination cannot cancel native Tab"
        );
        let mut nested = h.facts(Some("pane-action"), true);
        nested.nested = true;
        h.report(nested);
        assert!(
            !h.tab(false),
            "nested overlays retain their own keyboard handling"
        );
        h.dom.in_scope(ScopeId::ROOT, || {
            h.config.borrow().floating.unwrap().set(false)
        });
        h.settle();
        h.report(h.facts(Some("pane-action"), true));
        assert!(!h.tab(false), "inline panes never trap Tab");
    }
    #[test]
    fn pane_close_returns_live_owned_opener_and_preserves_independent_outside_focus() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        h.expanded(false);
        h.report(h.facts(None, false));
        assert_eq!(
            h.log.borrow().last().map(String::as_str),
            Some("pane-opener")
        );
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        h.log.borrow_mut().clear();
        h.expanded(false);
        h.report(h.facts(Some("outside-destination"), false));
        assert!(
            h.log.borrow().is_empty(),
            "outside close must preserve an independent focus destination"
        );
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        h.log.borrow_mut().clear();
        h.expanded(false);
        let mut facts = h.facts(None, false);
        facts.observation.candidates.retain(|candidate|candidate.target != "pane-opener");
        facts.eligible_focus_ids.retain(|id| id != "pane-opener");
        h.report(facts);
        assert!(
            h.log.borrow().is_empty(),
            "a hidden or disabled opener is not a return destination"
        );
    }

    #[test]
    fn pane_close_returns_registered_programmatic_opener_without_enrolling_it_in_tab_order() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        let mut facts = h.facts(Some("pane-opener"), false);
        facts.observation.candidates.retain(|candidate| candidate.target != "pane-opener");
        h.report(facts);
        assert_eq!(*h.log.borrow(), ["pane-field"]);
        h.expanded(false);
        let mut facts = h.facts(None, false);
        facts.observation.candidates.retain(|candidate| candidate.target != "pane-opener");
        h.report(facts);
        assert_eq!(h.log.borrow().last().map(String::as_str), Some("pane-opener"));
    }
    #[test]
    fn pane_mounted_focus_rejects_replacement_close_and_scope_teardown() {
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        h.report_pending(h.facts(Some("pane-opener"), false));
        h.mount("pane-field", "replacement-field");
        h.settle();
        assert!(
            h.log.borrow().is_empty(),
            "a snapshot cannot focus a replacement control"
        );
        h.expanded(false);
        h.report(h.facts(Some("outside"), false));
        h.expanded(true);
        h.report(h.facts(Some("pane-opener"), false));
        assert_eq!(
            h.log.borrow().last().map(String::as_str),
            Some("replacement-field")
        );
        h.log.borrow_mut().clear();
        h.report_pending(h.facts(Some("pane-action"), true));
        h.dom.in_scope(ScopeId::ROOT, || {
            assert!(h.focus.tab(false));
            h.config.borrow().expanded.unwrap().set(false);
        });
        h.settle();
        assert!(
            h.log.borrow().is_empty(),
            "closing invalidates a deferred wrap"
        );
        h.expanded(true);
        h.report_pending(h.facts(Some("pane-opener"), false));
        let retained = h.focus.clone();
        let log = h.log.clone();
        drop(h);
        assert!(!retained.model.borrow().alive);
        assert!(log.borrow().is_empty());
    }

    #[test]
    fn pane_passive_facts_update_cache_without_consuming_current_or_stale_intents() {
        use crate::overlay_runtime::ScopedFocusEvent;
        let _guard = crate::test_event_converter_guard();
        let mut h = Harness::new();
        h.expanded(true);
        let opening_epoch = h.focus.model.borrow().epoch;
        h.dom.in_scope(ScopeId::ROOT, || {
            h.focus.observe(ScopedFocusEvent::Observe(h.facts(Some("pane-opener"), false)));
        });
        h.settle();
        assert!(h.log.borrow().is_empty());
        assert!(matches!(h.focus.model.borrow().intent, Some(PaneFocusIntent::Open)));
        assert_eq!(h.focus.model.borrow().facts.active.as_deref(), Some("pane-opener"));
        h.dom.in_scope(ScopeId::ROOT, || {
            h.focus.observe(ScopedFocusEvent::Requested {
                request: opening_epoch.wrapping_sub(1),
                facts: h.facts(Some("pane-action"), true),
            });
        });
        assert_eq!(h.focus.model.borrow().facts.active.as_deref(), Some("pane-opener"));
        h.report(h.facts(Some("pane-opener"), false));
        assert_eq!(*h.log.borrow(), ["pane-field"]);
        h.expanded(false);
        h.dom.in_scope(ScopeId::ROOT, || {
            h.focus.observe(ScopedFocusEvent::Observe(h.facts(None, false)));
            h.focus.observe(ScopedFocusEvent::Requested {
                request: opening_epoch,
                facts: h.facts(None, false),
            });
        });
        h.settle();
        assert_eq!(h.log.borrow().len(), 1, "old opening reply cannot release the return intent");
        assert!(matches!(h.focus.model.borrow().intent, Some(PaneFocusIntent::Return)));
        h.report(h.facts(None, false));
        assert_eq!(h.log.borrow().last().map(String::as_str), Some("pane-opener"));
    }
