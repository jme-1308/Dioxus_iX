//! Excluded genuine assigned-slot fixture. Rust owns components, public props,
//! requests and commands; the companion native driver moves retained DOM nodes
//! into one consumer shadow tree and observes real distribution and events.
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
                        .with_title("iX genuine assigned slots")
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
    eprintln!("Run the desktop or web genuine slot probe");
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

fn roving(case_id: usize) -> bool {
    matches!(case_id, 2 | 6 | 7 | 10 | 14)
}
fn shared(case_id: usize) -> bool {
    matches!(case_id, 9 | 10 | 13 | 14)
}
fn public_id(epoch: usize, role: &str) -> String {
    format!("slot-{epoch}-{role}")
}
fn slot_name(epoch: usize, role: &str, name: &str) -> String {
    format!("slot-{epoch}-{role}-{name}")
}

#[component]
fn Probe() -> Element {
    let mut epoch = use_signal(|| 0_usize);
    let mut selected = use_signal(|| 1_usize);
    let readiness = use_signal(|| {
        "Native observations stay in memory until the fixed history ends.".to_string()
    });
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(150)).await;
            eprintln!("Assigned-slot probe watchdog timeout");
            std::process::exit(1);
        });
    });
    rsx! {IxTheme{scheme:ColorScheme::Dark,
        style{{STYLE}}
        main{id:"slot-probe",onmounted:move |_|{
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(());
                match query.join::<Value>().await{
                    Ok(value)if value["ok"]==true=>{println!("Dropdown slot native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Dropdown slot native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Dropdown slot transport failure: {error:?}");std::process::exit(1);}
                }
            });}
            #[cfg(target_arch="wasm32")]
            {let mut readiness=readiness;spawn(async move{
                let task=dioxus::document::eval(VERIFY);
                if let Err(error)=task.send(json!({"interactive":true})){
                    readiness.set(format!("Slot topology setup failed: {error:?}"));return;
                }
                match task.join::<Value>().await{
                    Ok(value)if value["ok"]==true=>readiness.set("Genuine slot topology ready. Use a Fresh history button to reset it before interacting.".to_string()),
                    Ok(value)=>readiness.set(format!("Slot topology setup failed: {value}")),
                    Err(error)=>readiness.set(format!("Slot topology setup failed: {error:?}")),
                }
            });}
        },
            h1{"Genuine native assigned slots"}
            p{"Fourteen public-input histories, each operation followed by twelve fixed native frames. One shared consumer shadow tree is an explicit port topology adapter."}
            div{class:"slot-mode-controls",for next in 1..=14{
                button{id:format!("slot-fresh-{next}"),onclick:move |_|{selected.set(next);epoch+=1;},"Fresh S{next:02}"}
            }}
            input{id:"slot-mode",r#type:"hidden",oninput:move |event|{selected.set(event.value().parse::<usize>().unwrap_or(1).clamp(1,14));epoch+=1;}}
            button{id:"slot-outside","Outside keyboard seed"}
            for current in [epoch()]{Fixture{key:"{current}",epoch:current,case_id:selected()}}
            pre{id:"slot-native-result","{readiness()}"}
        }
    }}
}

#[component]
fn Fixture(epoch: usize, case_id: usize) -> Element {
    let root = use_dropdown();
    let child = use_dropdown();
    let grand = use_dropdown();
    let rows = use_signal(Vec::<Value>::new);
    let mut mode = use_signal(move || {
        if roving(case_id) {
            DropdownNavigationMode::RovingTabindex
        } else {
            DropdownNavigationMode::ActiveDescendant
        }
    });
    let mut assigned_a_slot = use_signal(|| "lead".to_string());
    let mut branch_hidden = use_signal(|| false);
    let root_role = if shared(case_id) { "parent" } else { "root" };
    let command_root = root;
    let child_role = if case_id == 13 { "sub1" } else { "child" };
    let grand_role = "grand";
    rsx! {section{id:"slot-fixture","data-case":case_id.to_string(),"data-epoch":epoch.to_string(),
        div{id:"slot-consumer",class:"ix-theme","data-ix-theme":"brand","data-ix-color-schema":"dark","data-ix-density":"normal","data-slot-consumer":"true",
            div{id:"slot-shadow-content",
                slot{name:public_id(epoch,"ui")}
                div{"data-slot-supply":"ui",slot:public_id(epoch,"ui"),class:"slot-public-controls",
                    button{id:public_id(epoch,&format!("{root_role}-trigger")),"data-slot-role":format!("{root_role}-trigger"),"Open root"}
                    input{id:"slot-command",r#type:"hidden",oninput:move |event|{
                        let action=event.value();record(rows,"command-before",json!({"action":action,"root":command_root.is_open(),"child":child.is_open(),"grand":grand.is_open()}));
                        match action.as_str(){
                            "assigned-a-tail"=>assigned_a_slot.set("tail".into()),
                            "branch-hidden"=>branch_hidden.set(true),
                            "mode-ad"=>mode.set(DropdownNavigationMode::ActiveDescendant),
                            "root-open"=>command_root.open(),"root-close"=>command_root.close(false),
                            "child-open"=>child.open(),"child-close"=>child.close(false),_=>{}
                        }
                        record(rows,"command-return",json!({"action":action,"root":command_root.is_open(),"child":child.is_open(),"grand":grand.is_open()}));
                    }}
                    button{onmousedown:move |event|event.prevent_default(),onclick:move |_|assigned_a_slot.set("tail".into()),"Assign A to tail"}
                    button{onmousedown:move |event|event.prevent_default(),onclick:move |_|branch_hidden.set(true),"Hide branch root"}
                    button{onmousedown:move |event|event.prevent_default(),onclick:move |_|mode.set(DropdownNavigationMode::ActiveDescendant),"Use active descendant"}
                    input{id:"slot-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
                }
                if matches!(case_id,4|12){
                    Application{force_breakpoint:ApplicationBreakpoint::Large,class:"slot-application",
                        menu:rsx!{Menu{label:"Actual Category context",start_expanded:false,pinned:false,
                            if case_id==4{
                                RootSurface{controller:root,root_role,epoch,case_id,mode:mode(),assigned_a_slot:assigned_a_slot(),branch_hidden:branch_hidden(),rows}
                            }else{
                                MenuCategory{id:public_id(epoch,"category"),"data-slot-role":"category",label:"Own Category",
                                    MenuItem{id:public_id(epoch,"category-child-0"),"data-slot-role":"category-child-0",label:"Category child zero",close_on_select:false}
                                    MenuItem{id:public_id(epoch,"category-child-1"),"data-slot-role":"category-child-1",label:"Category child one",close_on_select:false}
                                }
                            }
                        }},div{}
                    }
                }else if case_id==11{
                    div{"data-slot-supply":"ui",slot:public_id(epoch,"ui"),"data-slot-role":"dropdown-button-composition",
                        DropdownButton{label:"Slot review",class:"slot-natural-button",navigation_mode:DropdownNavigationMode::ActiveDescendant,close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,disable_focus_trap:true,focus_checked_item:false,
                            onbeforechange:move |event:DropdownChangeEvent|record(rows,"request",json!({"role":"dropdown-button","open":event.open,"previous":event.previous,"veto":false})),
                            onchange:move |open|record(rows,"completion",json!({"role":"dropdown-button","open":open})),
                            DropdownItem{id:public_id(epoch,"button-a"),"data-slot-role":"button-a",tabindex:"0",label:"Button A"}
                            DropdownItem{id:public_id(epoch,"button-b"),"data-slot-role":"button-b",tabindex:"0",label:"Button B"}
                            DropdownItem{id:public_id(epoch,"button-c"),"data-slot-role":"button-c",tabindex:"0",label:"Button C"}
                        }
                    }
                }else{
                    RootSurface{controller:root,root_role,epoch,case_id,mode:mode(),assigned_a_slot:assigned_a_slot(),branch_hidden:branch_hidden(),rows}
                    if shared(case_id){
                        RootSurface{controller:child,root_role:child_role,epoch,case_id,mode:mode(),assigned_a_slot:"lead".to_string(),branch_hidden:false,rows}
                        if case_id==13{RootSurface{controller:grand,root_role:grand_role,epoch,case_id,mode:mode(),assigned_a_slot:"lead".to_string(),branch_hidden:false,rows}}
                    }
                }
            }
        }
    }}
}

#[component]
fn RootSurface(
    controller: DropdownHandle,
    root_role: &'static str,
    epoch: usize,
    case_id: usize,
    mode: DropdownNavigationMode,
    assigned_a_slot: String,
    branch_hidden: bool,
    rows: Signal<Vec<Value>>,
) -> Element {
    let trigger = match root_role {
        "child" | "sub1" => public_id(epoch, "parent-linked"),
        "grand" => public_id(epoch, "sub1-linked"),
        _ => public_id(epoch, &format!("{root_role}-trigger")),
    };
    let links_next = shared(case_id) && matches!(root_role, "parent" | "sub1");
    let fallback = matches!(case_id, 5 | 6);
    let category = case_id == 4;
    let lead = slot_name(epoch, root_role, "lead");
    let tail = slot_name(epoch, root_role, "tail");
    let request_handle = controller;
    rsx! {Dropdown{controller,trigger,label:format!("Actual {root_role} slots"),class:format!("slot-root-{root_role}"),navigation_mode:mode,
        close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,disable_focus_trap:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
        focus_checked_item:case_id==3,
        onbeforechange:move |event:DropdownChangeEvent|{
            let veto=(case_id==13&&root_role=="grand"&&event.open)||(case_id==14&&root_role=="child"&&!event.open);
            if veto{event.prevent_default();}
            record(rows,"request",json!({"role":root_role,"open":event.open,"previous":event.previous,"veto":veto,"current":request_handle.is_open()}));
        },
        onchange:move |open|record(rows,"completion",json!({"role":root_role,"open":open,"current":controller.is_open()})),
        div{"data-slot-root-role":root_role,class:"slot-root-content",
            if links_next{
                slot{name:lead.clone()}
                DropdownItem{id:public_id(epoch,&format!("{root_role}-leaf")),"data-slot-role":format!("{root_role}-leaf"),tabindex:"0",label:"Parent leaf",style:ROW}
                div{class:"slot-parking",
                    DropdownItem{id:public_id(epoch,&format!("{root_role}-linked")),"data-slot-role":format!("{root_role}-linked"),"data-slot-supply":root_role,slot:lead.clone(),label:"Linked submenu",style:ROW}
                }
            }else if category{
                slot{name:lead.clone()}
                SlotItem{epoch,root_role,name:"direct-after",tabindex:Some("2"),label:"After Category"}
                div{class:"slot-parking",
                    MenuCategory{id:public_id(epoch,"category"),"data-slot-role":"category","data-slot-supply":root_role,slot:lead.clone(),label:"Assigned Category",style:ROW,
                        MenuItem{id:public_id(epoch,"category-child-0"),"data-slot-role":"category-child-0",label:"Category child zero",close_on_select:false}
                        MenuItem{id:public_id(epoch,"category-child-1"),"data-slot-role":"category-child-1",label:"Category child one",close_on_select:false}
                    }
                }
            }else if fallback{
                SlotItem{epoch,root_role,name:"direct-before",tabindex:Some("5"),label:"Before fallback"}
                slot{name:slot_name(epoch,root_role,"unused"),
                    SlotItem{epoch,root_role,name:"fallback-item",tabindex:Some("3"),label:"Real fallback item"}
                }
                SlotItem{epoch,root_role,name:"direct-after",tabindex:Some("2"),label:"After fallback"}
            }else{
                SlotItem{epoch,root_role,name:"direct-before",tabindex:Some("5"),label:"Direct before"}
                slot{name:lead.clone()}
                SlotItem{epoch,root_role,name:"direct-between",tabindex:Some("0"),label:"Direct between"}
                slot{name:tail.clone()}
                SlotItem{epoch,root_role,name:"direct-after",tabindex:Some("2"),label:"Direct after"}
                div{class:"slot-parking",
                    if case_id==3{
                        input{id:public_id(epoch,&format!("{root_role}-checked-input")),"data-slot-role":"checked-input","data-slot-supply":root_role,slot:lead.clone(),r#type:"checkbox",checked:true,initial_checked:true,style:ROW}
                    }
                    DropdownItem{id:public_id(epoch,&format!("{root_role}-assigned-a")),"data-slot-role":"assigned-a","data-slot-supply":root_role,slot:slot_name(epoch,root_role,&assigned_a_slot),label:"Assigned A",style:ROW}
                    div{id:public_id(epoch,&format!("{root_role}-assigned-wrapper")),"data-slot-role":"assigned-wrapper","data-slot-supply":root_role,slot:lead.clone(),
                        SlotItem{epoch,root_role,name:"assigned-b",tabindex:None,label:"Assigned B descendant"}
                    }
                    div{id:public_id(epoch,&format!("{root_role}-branch-root")),"data-slot-role":"branch-root","data-slot-supply":root_role,slot:tail.clone(),"data-ix-roving-item":"",tabindex:"-1",hidden:branch_hidden,style:ROW,
                        SlotItem{epoch,root_role,name:"branch-child",tabindex:None,label:"Branch child"}
                    }
                }
            }
        }
    }}
}

#[component]
fn SlotItem(
    epoch: usize,
    root_role: &'static str,
    name: &'static str,
    tabindex: Option<&'static str>,
    label: &'static str,
) -> Element {
    rsx! {DropdownItem{id:public_id(epoch,&format!("{root_role}-{name}")),"data-slot-role":name,tabindex,style:ROW,label}}
}

const ROW: &str = "display:block;box-sizing:border-box;min-height:36px;margin:0";
const STYLE: &str = r#"
#slot-probe{padding:20px;min-height:900px;font-family:system-ui,sans-serif}
.slot-mode-controls,.slot-public-controls{display:flex;flex-wrap:wrap;gap:6px;margin:8px 0}
.slot-root-content{width:320px;max-height:240px;overflow:auto}
[class*="slot-root-"]>dialog{padding:0;max-height:380px}
:host([data-slot-interactive]) [class*="slot-root-"]>dialog{position:fixed;position-anchor:auto;position-area:none;inset:330px auto auto 24px;margin:0;max-width:calc(100vw - 48px)}
.slot-application{height:560px}
.slot-application nav.ix-menu{position:relative;inset:auto;height:540px;width:52px;overflow:visible}
#slot-native-result{font-size:12px;max-height:100px;overflow:auto;white-space:pre-wrap}
"#;

#[cfg(any(feature = "desktop", target_arch = "wasm32"))]
const VERIFY: &str = include_str!("dropdown_slot_observe.js");
