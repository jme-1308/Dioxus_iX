use super::*;
use crate::{ModalDismissReason, use_modal};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Default)]
struct Config {
    message: String,
    custom_guard: bool,
    revision: usize,
    centered: bool,
    size: ModalSize,
}
#[derive(Default)]
struct Observation {
    modal: Cell<Option<ModalHandle>>,
    config: Cell<Option<Signal<Config>>>,
    draft: Cell<Option<Signal<String>>>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
    requests: RefCell<Vec<(usize, ModalDismissRequest)>>,
    results: RefCell<Vec<ModalResult>>,
    open: RefCell<Vec<bool>>,
}
impl PartialEq for Observation {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn PrivateBody(data: Rc<Observation>) -> Element {
    let draft = use_signal(|| "Initial task note".to_string());
    data.draft.set(Some(draft));
    use_hook({
        let data = data.clone();
        move || data.mounts.set(data.mounts.get() + 1)
    });
    use_drop(move || data.drops.set(data.drops.get() + 1));
    rsx! {p {"Private note: {draft}"}}
}
fn fixture(data: Rc<Observation>) -> Element {
    let modal = use_modal();
    data.modal.set(Some(modal));
    let config = use_signal(|| Config {
        message: "Preparing inspection".into(),
        ..Default::default()
    });
    data.config.set(Some(config));
    let config = config();
    let requests = data.clone();
    let results = data.clone();
    let open = data.clone();
    let revision = config.revision;
    let guard = config.custom_guard.then(|| {
        EventHandler::new(move |request| requests.requests.borrow_mut().push((revision, request)))
    });
    rsx! {LoadingDialog {modal,message:config.message,centered:config.centered,size:config.size,ondismissrequest:guard,
        onresult:move|result|results.results.borrow_mut().push(result),
        onopenchange:move|value|open.open.borrow_mut().push(value),
        PrivateBody {data}
    }}
}
struct Toggle;
impl dioxus::html::HasToggleData for Toggle {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {
            $(fn $convert(&self,_:&dioxus::html::PlatformEventData)->dioxus::html::$data {converter!(@data $group)})*
        }
    };
    (@data Toggle)=>{dioxus::html::ToggleData::new(Toggle)};
    (@data $other:ident)=>{panic!("unexpected loading dialog test event")};
}
dioxus::html::with_html_event_groups!(converter);
fn beforetoggle_target(edits: &dioxus::core::Mutations) -> Option<dioxus::core::ElementId> {
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in &edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = *id
            }
            dioxus::core::Mutation::NewEventListener { name } if *name == "beforetoggle" => {
                return Some(current);
            }
            _ => {}
        }
    }
    None
}
fn settle(dom: &mut VirtualDom, target: &mut dioxus::core::ElementId) {
    for _ in 0..4 {
        let edits = dom.render_immediate_to_vec();
        if let Some(replacement) = beforetoggle_target(&edits) {
            *target = replacement;
        }
        dom.process_events();
    }
}
fn setup() -> (VirtualDom, Rc<Observation>, dioxus::core::ElementId) {
    let data = Rc::new(Observation::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut target = beforetoggle_target(&dom.rebuild_to_vec()).unwrap();
    settle(&mut dom, &mut target);
    (dom, data, target)
}
fn open(dom: &VirtualDom, target: dioxus::core::ElementId) {
    let data = Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event("beforetoggle", Event::new(data, false), target);
}
#[test]
fn loading_dialog_message_options_and_description_update_without_remounting_task_content() {
    let (mut dom, data, mut target) = setup();
    let modal = data.modal.get().unwrap();
    let description = format!("{}-loading-message", modal.id());
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("aria-label=\"Loading\""));
    assert!(html.contains(&format!("aria-describedby=\"{description}\"")));
    assert!(html.contains(&format!("id=\"{description}\"")));
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("data-centered=\"false\""));
    assert!(html.contains("ix-modal-size-360"));
    dom.in_runtime(|| {
        data.draft.get().unwrap().set("Keep this task note".into());
        let mut config = data.config.get().unwrap();
        config.with_mut(|value| {
            value.message = "Checking <motor> & drive".into();
            value.centered = true;
            value.size = ModalSize::Md;
        });
    });
    settle(&mut dom, &mut target);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Checking &#60;motor&#62; &#38; drive"));
    assert!(!html.contains("Preparing inspection"));
    assert!(html.contains("data-centered=\"true\""));
    assert!(html.contains("ix-modal-size-600"));
    assert!(html.contains("Keep this task note"));
    assert!(html.contains(&format!("id=\"{description}\"")));
    assert_eq!((data.mounts.get(), data.drops.get()), (1, 0));
}
#[test]
fn loading_dialog_denies_ordinary_dismissals_and_explicit_finish_preserves_its_result() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut target) = setup();
    let modal = data.modal.get().unwrap();
    open(&dom, target);
    dom.in_runtime(|| {
        assert!(modal.is_open());
        for reason in [
            ModalDismissReason::Escape,
            ModalDismissReason::Backdrop,
            ModalDismissReason::Cancel,
            ModalDismissReason::Programmatic("external".into()),
        ] {
            modal.dismiss(reason);
            assert!(modal.is_open());
            assert!(!modal.is_dismiss_pending());
        }
        assert!(data.results.borrow().is_empty());
        modal.close("inspection-ready");
        modal.close("duplicate");
        assert_eq!(
            modal.result(),
            Some(ModalResult::Closed("inspection-ready".into()))
        );
    });
    assert_eq!(
        *data.results.borrow(),
        [ModalResult::Closed("inspection-ready".into())]
    );
    assert_eq!(*data.open.borrow(), [true, false]);
    settle(&mut dom, &mut target);
    assert_eq!(
        data.drops.get(),
        1,
        "existing Modal replacement retires the old task child"
    );
}
#[test]
fn loading_dialog_custom_guards_track_current_props_and_finish_invalidates_pending_requests() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut target) = setup();
    let modal = data.modal.get().unwrap();
    dom.in_runtime(|| {
        data.config.get().unwrap().with_mut(|value| {
            value.custom_guard = true;
            value.revision = 1;
        })
    });
    settle(&mut dom, &mut target);
    open(&dom, target);
    dom.in_runtime(|| {
        modal.dismiss(ModalDismissReason::Cancel);
        modal.dismiss(ModalDismissReason::Escape);
        assert_eq!(data.requests.borrow().len(), 1);
        let (revision, request) = data.requests.borrow_mut().pop().unwrap();
        assert_eq!(revision, 1);
        assert_eq!(request.reason, ModalDismissReason::Cancel);
        assert!(request.deny());
        data.config
            .get()
            .unwrap()
            .with_mut(|value| value.revision = 2);
    });
    settle(&mut dom, &mut target);
    dom.in_runtime(|| {
        modal.dismiss(ModalDismissReason::Escape);
        let (revision, stale) = data.requests.borrow_mut().pop().unwrap();
        assert_eq!(revision, 2);
        modal.close("finished-through-guard");
        assert!(!stale.allow());
    });
    settle(&mut dom, &mut target);
    open(&dom, target);
    dom.in_runtime(|| {
        modal.dismiss(ModalDismissReason::Programmatic("approve".into()));
        assert!(data.requests.borrow_mut().pop().unwrap().1.allow());
        assert_eq!(
            modal.result(),
            Some(ModalResult::Dismissed(ModalDismissReason::Programmatic(
                "approve".into()
            )))
        );
    });
    assert_eq!(data.results.borrow().len(), 2);
}
