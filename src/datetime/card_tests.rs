use super::*;

fn root<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag.starts_with("div ") && tag.contains(&format!("id=\"{id}\"")))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn classes(tag: &str) -> BTreeSet<&str> {
    tag.split_once("class=\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .split_whitespace()
        .collect()
}
fn card<'a>(html: &'a str, id: &str) -> &'a str {
    html.split_once(&format!("id=\"{id}\""))
        .unwrap()
        .1
        .split("</section>")
        .next()
        .unwrap()
}
#[test]
fn card_footer_defaults_remain_compatible_and_explicit_presence_is_independent() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            section {DateTimeCard {id:"plain","Plain content"}}
            section {DateTimeCard {id:"inferred",header:rsx!{"Legacy header"},footer:rsx!{"Legacy footer"},"Legacy content"}}
            section {DateTimeCard {id:"suppressed",has_footer:false,footer:rsx!{"Suppressed action"},"Suppressed footer"}}
            section {DateTimeCard {id:"reserved",has_footer:true,"Reserved footer"}}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        classes(root(&html, "plain")),
        BTreeSet::from(["ix-date-time-card", "ix-date-corners-rounded"])
    );
    assert!(!card(&html, "plain").contains("ix-date-card-header"));
    assert!(!card(&html, "plain").contains("ix-date-card-footer"));
    assert!(card(&html, "inferred").contains("Legacy header"));
    assert!(card(&html, "inferred").contains("Legacy footer"));
    assert!(!card(&html, "suppressed").contains("Suppressed action"));
    assert!(!card(&html, "suppressed").contains("ix-date-card-footer"));
    assert!(card(&html, "reserved").contains("ix-date-card-footer"));
    assert!(html.contains("Legacy content") && html.contains("Plain content"));
}
#[test]
fn card_time_mode_composes_with_header_padding_embedding_and_each_corner() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            for (id,corners) in [("rounded",DateTimeCorners::Rounded),("left",DateTimeCorners::Left),("right",DateTimeCorners::Right),("straight",DateTimeCorners::Straight)] {
                section {DateTimeCard {id,corners,time_picker_appearance:true,embedded:true,hide_header:true,no_padding:id=="straight",class:"consumer-card",header:rsx!{"Hidden header"},"Time content"}}
            }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    for corner in ["rounded", "left", "right", "straight"] {
        let tokens = classes(root(&html, corner));
        assert!(tokens.contains("ix-date-time-card") && tokens.contains("consumer-card"));
        assert!(
            tokens.contains("ix-date-time-picker-appearance")
                && tokens.contains("ix-date-embedded")
        );
        assert!(tokens.contains(format!("ix-date-corners-{corner}").as_str()));
        assert_eq!(tokens.contains("ix-date-no-padding"), corner == "straight");
    }
    assert!(!html.contains("Hidden header") && !html.contains("ix-date-card-header"));
    assert_eq!(html.matches("Time content").count(), 4);
}

#[derive(Default)]
struct Capture {
    revision: Cell<usize>,
    visible: Cell<bool>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
    deliveries: RefCell<Vec<(usize, Rc<MountedData>)>>,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn Draft(data: Rc<Capture>) -> Element {
    let mut draft = use_signal(|| "Initial note".to_string());
    use_hook({
        let data = data.clone();
        move || data.mounts.set(data.mounts.get() + 1)
    });
    use_drop(move || data.drops.set(data.drops.get() + 1));
    rsx! {input {value:draft(),oninput:move|event|draft.set(event.value())}}
}
#[component]
fn ChangingCard(data: Rc<Capture>, revision: usize) -> Element {
    let mounted = data.clone();
    rsx! {DateTimeCard {
        id:format!("card-{revision}"),class:format!("consumer-{revision}"),
        time_picker_appearance:revision!=0,no_padding:revision==2,embedded:revision!=0,
        hide_header:revision==2,corners:if revision==0 {DateTimeCorners::Rounded}else{DateTimeCorners::Left},
        header:if revision==0 {None}else{Some(rsx!{"Current header"})},
        footer:rsx!{"Footer action"},has_footer:revision!=1,
        onmounted:move|event:MountedEvent|mounted.deliveries.borrow_mut().push((revision,event.data())),
        Draft {data}
    }}
}
fn fixture(data: Rc<Capture>) -> Element {
    rsx! {if data.visible.get(){ChangingCard {data:data.clone(),revision:data.revision.get()}}}
}

fn refresh(dom: &mut VirtualDom) -> dioxus::core::Mutations {
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec()
}
struct CardMount(Rc<Cell<usize>>);
impl dioxus::html::RenderedElementBacking for CardMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Form(String);
impl dioxus::html::HasFileData for Form {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for Form {
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
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{let _=event;converter!(@data $group,event)})*}
    };
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(Form($event.downcast::<String>().unwrap().clone()))};
    (@data Mounted,$event:ident)=>{MountedData::new(CardMount($event.downcast::<Rc<Cell<usize>>>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("unexpected card test event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    target: dioxus::core::ElementId,
    payload: impl std::any::Any,
) {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event(name, Event::new(data, name != "mounted"), target);
}
#[test]
fn card_forwards_actual_root_mount_and_keeps_native_child_draft_through_prop_changes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let data = Rc::new(Capture::default());
    data.visible.set(true);
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let edits = dom.rebuild_to_vec();
    let inputs = listener_ids_ref(&edits, "input");
    let mounts = listener_ids_ref(&edits, "mounted");
    assert_eq!(inputs.len(), 1);
    assert_eq!(mounts.len(), 1);
    let backing = Rc::new(Cell::new(0usize));
    dispatch(&dom, "mounted", mounts[0], backing.clone());
    assert!(Rc::ptr_eq(
        &data.deliveries.borrow()[0]
            .1
            .downcast::<CardMount>()
            .unwrap()
            .0,
        &backing
    ));
    dispatch(&dom, "input", inputs[0], "Retained native note".to_string());
    dom.render_immediate_to_vec();
    for revision in [1, 2, 0] {
        data.revision.set(revision);
        let edits = refresh(&mut dom);
        assert!(listener_ids_ref(&edits, "mounted").is_empty());
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Retained native note"));
        assert!(
            classes(root(&html, &format!("card-{revision}")))
                .contains(format!("consumer-{revision}").as_str())
        );
        assert_eq!(html.contains("Current header"), revision == 1);
        assert_eq!(html.contains("Footer action"), revision != 1);
    }
    data.revision.set(1);
    refresh(&mut dom);
    dispatch(&dom, "mounted", mounts[0], backing.clone());
    assert_eq!(
        data.deliveries.borrow()[1].0,
        1,
        "current prop callback receives the actual root event"
    );
    assert!(Rc::ptr_eq(
        &data.deliveries.borrow()[1]
            .1
            .downcast::<CardMount>()
            .unwrap()
            .0,
        &backing
    ));
    assert_eq!((data.mounts.get(), data.drops.get()), (1, 0));
    data.visible.set(false);
    refresh(&mut dom);
    assert!(!dioxus_ssr::render(&dom).contains("ix-date-time-card"));
    assert_eq!(data.drops.get(), 1);
}
fn listener_ids_ref(edits: &dioxus::core::Mutations, name: &str) -> Vec<dioxus::core::ElementId> {
    let mut current = dioxus::core::ElementId::ROOT;
    let mut result = Vec::new();
    for edit in &edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = *id
            }
            dioxus::core::Mutation::NewEventListener { name: event } if event == name => {
                result.push(current)
            }
            _ => {}
        }
    }
    result
}
