//! Render compatibility and actual child-state retention for typed column widths.
use super::*;

fn column_styles(html: &str) -> Vec<&str> {
    html.split('<')
        .filter(|tag| tag.starts_with("div ") && tag.contains("class=\"ix-layout-col\""))
        .map(|tag| {
            tag.split("style=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap()
        })
        .collect()
}
#[test]
fn typed_columns_preserve_exact_legacy_default_numeric_and_clamped_styles() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Col{"Default"} Col{widths:ColWidths::default(),"Typed default"}
            Col{size:6,size_md:4,size_lg:3,"Legacy numeric"}
            Col{widths:ColWidths{base:Some(ColWidth::Span(6)),md:Some(ColWidth::Span(4)),lg:Some(ColWidth::Span(3)),..Default::default()},"Typed numeric"}
            Col{size:0,size_sm:255,"Legacy clamped"}
            Col{widths:ColWidths{base:Some(ColWidth::Span(0)),sm:Some(ColWidth::Span(255)),..Default::default()},"Typed clamped"}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let styles = column_styles(&html);
    assert_eq!(styles.len(), 6);
    assert_eq!(
        styles[0],
        "--ix-col-basis:0%;--ix-col-sm:0%;--ix-col-md:0%;--ix-col-lg:0%;--ix-col-grow:1;--ix-col-grow-sm:1;--ix-col-grow-md:1;--ix-col-grow-lg:1"
    );
    assert_eq!(styles[1], styles[0]);
    assert_eq!(styles[3], styles[2]);
    assert_eq!(styles[5], styles[4]);
    assert_eq!(
        styles[2],
        "--ix-col-basis:calc(6 / var(--ix-grid-columns,12) * 100%);--ix-col-sm:calc(4 / var(--ix-grid-columns,12) * 100%);--ix-col-md:calc(4 / var(--ix-grid-columns,12) * 100%);--ix-col-lg:calc(3 / var(--ix-grid-columns,12) * 100%);--ix-col-grow:0;--ix-col-grow-sm:0;--ix-col-grow-md:0;--ix-col-grow-lg:0"
    );
    assert!(
        styles[4].contains("--ix-col-basis:calc(1 / var(--ix-grid-columns,12) * 100%)")
            && styles[4].contains("--ix-col-sm:calc(12 / var(--ix-grid-columns,12) * 100%)")
    );
}
#[test]
fn typed_column_slots_override_matching_legacy_sizes_and_keep_existing_inheritance() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Col{size:6,size_sm:2,size_md:4,size_lg:10,widths:ColWidths{base:Some(ColWidth::Auto),sm:Some(ColWidth::Span(0)),md:Some(ColWidth::Flexible),lg:None},"Merged"}
            Col{size:12,widths:ColWidths{lg:Some(ColWidth::Auto),..Default::default()},"Inherited auto"}
            Col{widths:ColWidth::Auto,"Scalar auto"}
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let styles = column_styles(&html);
    assert_eq!(
        styles[0],
        "--ix-col-basis:auto;--ix-col-sm:calc(1 / var(--ix-grid-columns,12) * 100%);--ix-col-md:0%;--ix-col-lg:calc(10 / var(--ix-grid-columns,12) * 100%);--ix-col-grow:0;--ix-col-grow-sm:0;--ix-col-grow-md:1;--ix-col-grow-lg:0"
    );
    assert_eq!(
        styles[1],
        "--ix-col-basis:calc(12 / var(--ix-grid-columns,12) * 100%);--ix-col-sm:auto;--ix-col-md:auto;--ix-col-lg:auto;--ix-col-grow:0;--ix-col-grow-sm:0;--ix-col-grow-md:0;--ix-col-grow-lg:0"
    );
    assert_eq!(
        styles[2],
        "--ix-col-basis:auto;--ix-col-sm:auto;--ix-col-md:auto;--ix-col-lg:auto;--ix-col-grow:0;--ix-col-grow-sm:0;--ix-col-grow-md:0;--ix-col-grow-lg:0"
    );
}

#[derive(Default)]
struct ColumnFixture {
    widths: Cell<Option<Signal<ColWidths>>>,
    mounts: Cell<usize>,
}
impl PartialEq for ColumnFixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn PrivateColumnInput(data: Rc<ColumnFixture>) -> Element {
    use_hook(|| data.mounts.set(data.mounts.get() + 1));
    let mut note = use_signal(|| String::from("Prepare equipment"));
    rsx! {crate::Input{id:"column-private-note",label:"Assignment note",value:note(),onvaluechange:move|value|note.set(value)}}
}
fn column_fixture(data: Rc<ColumnFixture>) -> Element {
    let widths = use_signal(|| ColWidths::from(ColWidth::Auto));
    data.widths.set(Some(widths));
    rsx! {LayoutGrid{Row{Col{size:4,widths:widths(),PrivateColumnInput{data:data.clone()}}}}}
}
struct ColumnForm(String);
impl dioxus::html::HasFileData for ColumnForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for ColumnForm {
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
struct ColumnConverter;
macro_rules! column_converter{
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{
        impl dioxus::html::HtmlEventConverter for ColumnConverter{
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{let _=event;column_converter!(@data $group,event)})*
        }
    };
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(ColumnForm($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected column test event")};
}
dioxus::html::with_html_event_groups!(column_converter);
fn column_input_id(changes: dioxus::core::Mutations) -> dioxus::core::ElementId {
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
            } if name == "id" && value == "column-private-note" => return current,
            _ => {}
        }
    }
    panic!("actual private Input ID must be mounted")
}
fn settle_column(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
#[test]
fn changing_column_widths_retains_private_input_state_and_component_scope() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ColumnConverter));
    let data = Rc::new(ColumnFixture::default());
    let mut dom = VirtualDom::new_with_props(column_fixture, data.clone());
    let input = column_input_id(dom.rebuild_to_vec());
    let event = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
        String::from("Retained maintenance note"),
    ))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event("input", Event::new(event, true), input);
    settle_column(&mut dom);
    for (widths, expected_basis) in [
        (ColWidths::from(ColWidth::Flexible), "--ix-col-basis:0%"),
        (
            ColWidths {
                base: Some(ColWidth::Span(8)),
                sm: Some(ColWidth::Span(12)),
                ..Default::default()
            },
            "--ix-col-basis:calc(8 / var(--ix-grid-columns,12) * 100%)",
        ),
        (ColWidths::from(ColWidth::Auto), "--ix-col-basis:auto"),
    ] {
        dom.in_scope(ScopeId::APP, || data.widths.get().unwrap().set(widths));
        settle_column(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert!(column_styles(&html)[0].starts_with(expected_basis));
        assert!(html.contains("value=\"Retained maintenance note\""));
        assert_eq!(data.mounts.get(), 1);
    }
}
