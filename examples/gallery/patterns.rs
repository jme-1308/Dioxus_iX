//! Source-comparable composite component preview examples.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "layout_auto_form.rs"]
mod layout_auto_form;

fn filter_categories() -> Vec<FilterCategory> {
    vec![
        FilterCategory::new(
            "ID_1",
            "Vendor",
            vec!["Apple".into(), "MS".into(), "Siemens".into()],
        ),
        FilterCategory::new(
            "ID_2",
            "Product",
            vec!["iPhone X".into(), "Windows".into(), "APS".into()],
        ),
    ]
}
fn initial_filter() -> FilterState {
    FilterState {
        tokens: vec!["Custom filter text".into()],
        categories: vec![CategoryToken {
            id: "ID_1".into(),
            value: "IBM".into(),
            operator: LogicalFilterOperator::NotEqual,
        }],
    }
}
fn filter_summary(state: &FilterState) -> String {
    let mut values = state.tokens.clone();
    values.extend(
        state
            .categories
            .iter()
            .map(|token| format!("{} {} {}", token.id, token.operator.symbol(), token.value)),
    );
    if values.is_empty() {
        "No active criteria".into()
    } else {
        values.join("; ")
    }
}
#[component]
fn FlipExample(variant: FlipTileVariant) -> Element {
    let mut index = use_signal(|| 0usize);
    rsx! {div{class:"pattern-flip-example",
        FlipTile{variant,index:index(),ontoggle:move|next|index.set(next),header:rsx!{"Flip header"},
            footer:rsx!{div{div{"Predicted maintenance date"}div{class:"pattern-flip-footer-date",Icon{name:"info",size:16}"2021-06-22"}}},
            pages:vec![rsx!{FlipTileContent{"Example 1"}},rsx!{FlipTileContent{"Example 2"}}],
        }
        p{role:"status","Visible content: Example {index()+1}"}
    }}
}
#[component]
pub fn PatternExamples() -> Element {
    let mut filter = use_signal(initial_filter);
    let mut suggestions_filter = use_signal(FilterState::default);
    let mut top_layer_filter = use_signal(FilterState::default);
    let mut native_limitation = use_signal(String::new);
    let mut draft = use_signal(FilterInputState::default);
    let mut unique = use_signal(|| false);
    let mut protect_filter = use_signal(|| false);
    let mut cleared = use_signal(|| 0usize);
    let mut report_error = use_signal(|| true);
    let mut expanded_form = use_signal(|| false);
    let summary = filter_summary(&filter());
    let suggestions_summary = filter_summary(&suggestions_filter());
    let input_summary = if let Some(category) = draft().category {
        format!("{category}: {}", draft().token)
    } else {
        draft().token
    };
    let templates = GridTemplates {
        lg: Some(
            GridTemplate::from_rows(&[
                &["header", "header", "header"],
                &["nav", "content", "content"],
                &["footer", "footer", "footer"],
            ])
            .expect("valid static large template"),
        ),
        md: Some(
            GridTemplate::from_rows(&[
                &["header", "header"],
                &["nav", "content"],
                &["footer", "footer"],
            ])
            .expect("valid static medium template"),
        ),
        sm: Some(
            GridTemplate::from_rows(&[&["header"], &["nav"], &["content"], &["footer"]])
                .expect("valid static small template"),
        ),
    };
    rsx! {
        section{class:"demo-section",id:"category-filter",
            h2{"Category filter"}
            p{class:"pattern-description","The upstream initial state and Vendor/Product categories. Select a category, choose or type a value, and switch between equal and not-equal. Enter also adds free text."}
            label{class:"pattern-option",input{r#type:"checkbox",checked:unique(),onchange:move|event|unique.set(event.checked())}"Use each category once"}
            label{class:"pattern-option",input{r#type:"checkbox",checked:protect_filter(),onchange:move|event|protect_filter.set(event.checked())}"Prevent filter reset"}
            CategoryFilter{categories:filter_categories(),state:filter(),unique_categories:unique(),placeholder:"Filter by",onchange:move|state|filter.set(state),oninput:move|state|draft.set(state),onclear:move|_|cleared+=1,onbeforeclear:move|event:FilterClearEvent|if protect_filter(){event.prevent_default();}}
            p{class:"pattern-status",role:"status","{summary}"}
            p{class:"pattern-status","Last input event: {input_summary}. Clear actions: {cleared}"}
            h3{"Top layer options"}
            p{class:"pattern-description","This filter is inside a container that clips overflow. Native popover options remain visible beyond its edge and follow the input when scrolling."}
            div {class:"pattern-top-layer-clip",
                CategoryFilter {categories:filter_categories(),state:top_layer_filter(),enable_top_layer:true,label:"Top layer category filter",onchange:move |state|top_layer_filter.set(state),onnativeerror:move |error|native_limitation.set(format!("Native popover unavailable; using inline options. {error}"))}
            }
            if !native_limitation().is_empty() {p{class:"pattern-status",role:"status","{native_limitation}"}}
        }
        section{class:"demo-section",id:"category-filter-suggestions",
            h2{"Category filter suggestions"}
            CategoryFilter{suggestions:vec!["Item 1".into(),"Item 2".into()],state:suggestions_filter(),onchange:move|state|suggestions_filter.set(state),placeholder:"Filter by",label:"Suggested item filter"}
            p{class:"pattern-status",role:"status","{suggestions_summary}"}
            h3{"Readonly and disabled states"}
            div{class:"pattern-state-grid",CategoryFilter{categories:filter_categories(),default_state:initial_filter(),readonly:true,label:"Readonly filter"}CategoryFilter{categories:filter_categories(),default_state:initial_filter(),disabled:true,label:"Disabled filter"}}
        }
        section{class:"demo-section",id:"flip-tile",
            h2{"Flip tile"}
            p{class:"pattern-description","The five upstream semantic variants, each cycling between Example 1 and Example 2. Use the eye control with a pointer, Enter or Space."}
            div{class:"pattern-flip-examples",for variant in [FlipTileVariant::Filled,FlipTileVariant::Primary,FlipTileVariant::Info,FlipTileVariant::Warning,FlipTileVariant::Alarm]{FlipExample{variant}}}
        }
        section{class:"demo-section",id:"info-page",
            h2{"Info page"}
            if report_error(){InfoPage{title_text:"Report could not be generated",copy_text:"An error occurred while generating the report.",instructions:"Contact your administrator for more information.",actions:rsx!{Button{onclick:move|_|report_error.set(false),"Return to overview"}}}}
            else{div{class:"pattern-report-overview",h3{"Report overview"}p{"The recovery action returned to the overview."}Button{variant:ButtonVariant::Tertiary,onclick:move|_|report_error.set(true),"Review error state"}}}
        }
        section{class:"demo-section",id:"css-grid",
            h2{"CSS grid"}
            p{class:"pattern-description","Named areas use the upstream small and medium viewport breakpoints (576px and 768px), with larger-template fallback. Resize to review the layout."}
            CssGrid{templates,
                GridArea{name:"header",div{class:"pattern-grid-region","Header"}}
                GridArea{name:"nav",div{class:"pattern-grid-region","Navigation"}}
                GridArea{name:"content",div{class:"pattern-grid-region pattern-grid-content","Content"}}
                GridArea{name:"footer",div{class:"pattern-grid-region","Footer"}}
            }
        }
        section{class:"demo-section",id:"layout-auto",
            h2{"Layout auto"}
            p{class:"pattern-description","The upstream numbered layout: one column below 48em, two columns above it. Item 4 spans both columns when space is available."}
            LayoutAuto{for number in 1..=6{LayoutAutoItem{colspan:if number==4{2}else{1},div{class:"pattern-layout-number","{number}"}}}}
            h3{"Layout auto custom"}
            p{class:"pattern-description","Custom breakpoints: one column, two at 560px, and four at 800px."}
            LayoutAuto{layout:vec![AutoLayoutBreakpoint::new("0",1),AutoLayoutBreakpoint::new("560px",2),AutoLayoutBreakpoint::new("800px",4)],for number in 1..=6{LayoutAutoItem{colspan:if number==4{2}else{1},div{class:"pattern-layout-number","{number}"}}}}
            h3{"Responsive form content"}
            Button{variant:ButtonVariant::Tertiary,onclick:move|_|expanded_form.toggle(),if expanded_form(){"Hide optional field"}else{"Add optional field"}}
            LayoutAuto{
                LayoutAutoItem{label{class:"pattern-form-field","First name" input{r#type:"text",name:"first-name",autocomplete:"given-name"}}}
                LayoutAutoItem{label{class:"pattern-form-field","Last name" input{r#type:"text",name:"last-name",autocomplete:"family-name"}}}
                LayoutAutoItem{colspan:2,label{class:"pattern-form-field","Address" input{r#type:"text",name:"address",autocomplete:"street-address"}}}
                if expanded_form(){LayoutAutoItem{colspan:2,label{class:"pattern-form-field","Additional information" textarea{name:"additional-information",rows:"3"}}}}
            }
            layout_auto_form::LayoutAutoFormExample {}
        }
    }
}

pub const STYLES: &str = include_str!("styles/patterns.css");
