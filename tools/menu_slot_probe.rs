//! Excluded actual Menu/Category shadow and slot diagnostics.
//! Rust owns public components/props; native code retains actual topology and
//! observes original events and transport delivery without application policy.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("iX Menu shadow inputs")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0)),
                )
                .with_menu(None),
        )
        .launch(Probe);
}
#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Probe);
}
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _fixture = Probe;
    eprintln!("Run the desktop or web Menu slot fixture");
}

fn record(mut rows: Signal<Vec<Value>>, kind: &'static str, details: Value) {
    if let Ok(mut values) = rows.try_write() {
        let seq = values
            .last()
            .and_then(|row| row["seq"].as_u64())
            .unwrap_or(0)
            + 1;
        values.push(json!({"seq":seq,"kind":kind,"details":details}));
    }
}
fn public_id(epoch: usize, role: &str) -> String {
    format!("menu-slot-{epoch}-{role}")
}
fn slotted(case_id: usize) -> bool {
    matches!(case_id, 4 | 5 | 6 | 8)
}

#[component]
fn Probe() -> Element {
    let mut selected = use_signal(|| 1_usize);
    let mut epoch = use_signal(|| 0_usize);
    let readiness =
        use_signal(|| "Native Menu observations remain in memory until completion.".to_string());
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(150)).await;
            eprintln!("Menu slot probe timeout");
            std::process::exit(1);
        });
    });
    rsx! {IxTheme{scheme:ColorScheme::Dark,
        style{{STYLE}}
        main{id:"menu-slot-probe",onmounted:move |_|{
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{let query=dom_query::eval(VERIFY);let _=query.send(());match query.join::<Value>().await{
                Ok(value)if value["ok"]==true=>{println!("Menu slot native observations: {value}");window.close();}
                Ok(value)=>{eprintln!("Menu slot native failure JSON: {value}");std::process::exit(1);}
                Err(error)=>{eprintln!("Menu slot native transport failure: {error:?}");std::process::exit(1);}
            }});}
            #[cfg(target_arch="wasm32")]
            {let mut readiness=readiness;spawn(async move{let query=dioxus::document::eval(VERIFY);if let Err(error)=query.send(json!({"interactive":true})){readiness.set(format!("Menu topology setup failed: {error:?}"));return;}match query.join::<Value>().await{
                Ok(value)=>readiness.set(value.to_string()),Err(error)=>readiness.set(format!("Menu topology setup failed: {error:?}")),
            }});}
        },
            h1{"Menu and Category native topology"}
            p{"Eight fixed input histories distinguish actual Menu ancestry, shadow initialization and native callback delivery."}
            div{class:"menu-slot-controls",for next in 1..=8{button{id:format!("menu-slot-fresh-{next}"),onclick:move |_|{selected.set(next);epoch+=1;},"Fresh M{next:02}"}}}
            input{id:"menu-slot-mode",r#type:"hidden",oninput:move |event|{selected.set(event.value().parse::<usize>().unwrap_or(1).clamp(1,8));epoch+=1;}}
            button{id:"menu-slot-outside","Outside keyboard seed"}
            for current in [epoch()]{Fixture{key:"{current}",epoch:current,case_id:selected()}}
            pre{id:"menu-slot-native-result","{readiness()}"}
        }
    }}
}

#[component]
fn Fixture(epoch: usize, case_id: usize) -> Element {
    let dropdown = use_dropdown();
    let rows = use_signal(Vec::<Value>::new);
    let mut mounted = use_signal(move || case_id != 3);
    let mut category_mounted = use_signal(move || case_id != 5);
    let mut category_slot = use_signal(|| "lead".to_string());
    let mut child_hidden = use_signal(|| false);
    rsx! {section{id:"menu-slot-fixture","data-case":case_id.to_string(),"data-epoch":epoch.to_string(),
        div{id:"menu-slot-shell",class:"ix-theme","data-ix-theme":"brand","data-ix-color-schema":"dark","data-ix-density":"normal",
            div{id:"menu-slot-owner-content",class:"ix-theme","data-ix-theme":"brand","data-ix-color-schema":"dark","data-ix-density":"normal",
                if !slotted(case_id){slot{name:public_id(epoch,"ui")}}
                if mounted(){
                    Application{force_breakpoint:ApplicationBreakpoint::Large,class:"menu-slot-application",
                        menu:rsx!{Menu{label:"Outer Menu",start_expanded:false,pinned:false,
                            onexpandchange:move |expanded|record(rows,"menu-expand",json!({"role":"outer","expanded":expanded})),
                            MenuItem{id:public_id(epoch,"menu-before"),"data-menu-slot-role":if case_id==7{"outer-before"}else{"menu-before"},label:"Before Category",close_on_select:false}
                            if case_id==7{
                                div{"data-menu-slot-role":"inner-menu-wrapper",
                                    Menu{label:"Inner Menu",start_expanded:false,pinned:false,
                                        onexpandchange:move |expanded|record(rows,"menu-expand",json!({"role":"inner","expanded":expanded})),
                                        MenuItem{id:public_id(epoch,"inner-before"),"data-menu-slot-role":"inner-before",label:"Inner before",close_on_select:false}
                                        Category{epoch,category_slot:None,child_hidden:child_hidden()}
                                        MenuItem{id:public_id(epoch,"inner-after"),"data-menu-slot-role":"inner-after",label:"Inner after",close_on_select:false}
                                    }
                                }
                            }else if slotted(case_id){
                                div{id:"menu-slot-consumer",class:"ix-theme","data-ix-theme":"brand","data-ix-color-schema":"dark","data-ix-density":"normal",
                                    div{id:"menu-slot-root-content",
                                        slot{name:public_id(epoch,"ui")}
                                        Dropdown{controller:dropdown,trigger:public_id(epoch,"ordinary-trigger"),label:"Actual Category projection",class:"menu-slot-dropdown",enable_top_layer:true,disable_focus_trap:true,close_behavior:DropdownCloseBehavior::None,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
                                            onbeforechange:move |event:DropdownChangeEvent|record(rows,"dropdown-request",json!({"open":event.open,"previous":event.previous,"veto":event.default_prevented(),"current":dropdown.is_open()})),
                                            onchange:move |open|record(rows,"dropdown-completion",json!({"open":open,"current":dropdown.is_open()})),
                                            div{"data-menu-slot-role":"dropdown-content",
                                                slot{name:public_id(epoch,"lead")}
                                                slot{name:public_id(epoch,"tail")}
                                                div{id:"menu-slot-category-parking","data-menu-slot-role":"category-parking",
                                                    if category_mounted(){Category{epoch,category_slot:Some(public_id(epoch,&category_slot())),child_hidden:child_hidden()}}
                                                }
                                            }
                                        }
                                    }
                                    if case_id==8{button{id:public_id(epoch,"redistribution-peer"),"data-menu-slot-role":"redistribution-peer",slot:public_id(epoch,"tail"),"Redistribution peer"}}
                                }
                            }else{Category{epoch,category_slot:None,child_hidden:child_hidden()}}
                            MenuItem{id:public_id(epoch,"menu-after"),"data-menu-slot-role":if case_id==7{"outer-after"}else{"menu-after"},label:"After Category",close_on_select:false}
                        }},div{}
                    }
                }
            }
        }
        div{id:"menu-slot-public-controls",slot:public_id(epoch,"ui"),class:"menu-slot-controls",
            button{id:public_id(epoch,"ordinary-trigger"),"data-menu-slot-role":"ordinary-dropdown-external-trigger","Open projection"}
            input{id:"menu-slot-command",r#type:"hidden",oninput:move |event|{let action=event.value();record(rows,"command-before",json!({"action":action,"mounted":mounted(),"categoryMounted":category_mounted()}));match action.as_str(){
                "mount-owner"=>mounted.set(true),"mount-category"=>category_mounted.set(true),"category-tail"=>category_slot.set("tail".into()),"hide-child-zero"=>child_hidden.set(true),_=>{}
            }record(rows,"command-return",json!({"action":action,"mounted":mounted(),"categoryMounted":category_mounted()}));}}
            button{onmousedown:move |event|event.prevent_default(),onclick:move |_|category_slot.set("tail".into()),"Assign Category to tail"}
            button{onmousedown:move |event|event.prevent_default(),onclick:move |_|child_hidden.set(true),"Hide child zero"}
            input{id:"menu-slot-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
        }
    }}
}

#[component]
fn Category(epoch: usize, category_slot: Option<String>, child_hidden: bool) -> Element {
    rsx! {MenuCategory{id:public_id(epoch,"category"),"data-menu-slot-role":"category",slot:category_slot,label:"Actual Category",
        MenuItem{id:public_id(epoch,"category-child-0"),"data-menu-slot-role":"category-child-0",label:"Category child zero",hidden:child_hidden,close_on_select:false}
        MenuItem{id:public_id(epoch,"category-child-1"),"data-menu-slot-role":"category-child-1",label:"Category child one",close_on_select:false}
    }}
}

const STYLE: &str = r#"
#menu-slot-probe{padding:20px;min-height:900px;font-family:system-ui,sans-serif}
.menu-slot-controls{display:flex;flex-wrap:wrap;gap:6px;margin:8px 0}
.menu-slot-application{height:600px}
.menu-slot-application nav.ix-menu{position:relative;inset:auto;height:560px;width:52px;overflow:visible}
.menu-slot-dropdown>dialog{padding:0;max-height:360px}
[data-menu-slot-role=dropdown-content]{width:320px;min-height:48px;max-height:240px;overflow:auto}
#menu-slot-native-result{white-space:pre-wrap;max-height:140px;overflow:auto;font-size:12px}
"#;
#[cfg(any(target_arch = "wasm32", feature = "desktop"))]
const VERIFY: &str = include_str!("menu_slot_observe.js");
