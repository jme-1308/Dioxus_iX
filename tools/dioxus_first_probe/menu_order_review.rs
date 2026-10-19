//! Excluded explicitly ordered Menu/Category lists and independent ownership.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn OrderedCategoryChild(
    name: String,
    order: usize,
    hidden: bool,
    onselect: EventHandler<String>,
) -> Element {
    rsx! {div {class:"menu-order-wrapper",MenuItem {label:name.clone(),order:Some(order),hidden,close_on_select:false,onclick:move |_|onselect.call(name.clone())}}}
}

#[component]
fn OrderedEntry(
    name: String,
    order: usize,
    hidden: bool,
    disabled: bool,
    child_reversed: bool,
    child_hidden: bool,
    onselect: EventHandler<String>,
) -> Element {
    if name == "Maintenance" {
        let mut children = vec!["Task one", "Task two", "Task three"];
        if child_reversed {
            children.reverse();
        }
        rsx! {div {class:"menu-order-wrapper",MenuCategory {label:name,icon:"cogwheel",order:Some(order),hidden,
            for (index,child) in children.into_iter().enumerate() {
                OrderedCategoryChild {key:"{child}",name:child.to_string(),order:index,hidden:child=="Task two"&&child_hidden,onselect}
            }
        }}}
    } else {
        rsx! {div {class:"menu-order-wrapper",MenuItem {label:name.clone(),icon:"home",order:Some(order),hidden,disabled,close_on_select:false,onclick:move |_|onselect.call(name.clone())}}}
    }
}

#[component]
pub fn MenuOrderReview() -> Element {
    let mut reversed = use_signal(|| false);
    let mut inserted = use_signal(|| false);
    let mut alpha_present = use_signal(|| true);
    let mut alpha_hidden = use_signal(|| false);
    let mut beta_disabled = use_signal(|| true);
    let mut child_reversed = use_signal(|| false);
    let mut child_hidden = use_signal(|| false);
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut expanded = use_signal(|| true);
    let mut selections = use_signal(|| 0_usize);
    let mut last = use_signal(String::new);
    let mut secondary_selections = use_signal(|| 0_usize);
    let selected = use_callback(move |name: String| {
        last.set(name);
        selections += 1;
    });
    let mut entries = vec!["Alpha", "Beta", "Maintenance"];
    if inserted() {
        entries.push("Gamma");
    }
    if reversed() {
        entries.reverse();
    }
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"menu-order-review",
            h1 {"Menu lists ordered in Rust"}
            p {"Wrapped keyed entries forward their current order and hidden state. Primary items, bottom actions and utilities keep separate sections; categories own their child lists. The second menu is independent."}
            div {class:"menu-order-controls",
                button {id:"r0242-reverse",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|reversed.toggle(),"Reverse primary list"}
                button {id:"r0242-insert",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|inserted.toggle(),"Insert / remove Gamma"}
                button {id:"r0242-remove",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|alpha_present.toggle(),"Remove / restore Alpha"}
                button {id:"r0242-hidden",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|alpha_hidden.toggle(),"Hide / show Alpha"}
                button {id:"r0242-disabled",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|beta_disabled.toggle(),"Disable / enable Beta"}
                button {id:"r0242-child-reverse",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|child_reversed.toggle(),"Reverse category children"}
                button {id:"r0242-child-hidden",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|child_hidden.toggle(),"Hide / show Task two"}
                button {id:"r0242-expanded",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|expanded.toggle(),"Collapse / expand primary menu"}
                button {id:"r0242-mount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|mounted.toggle(),"Hide / show primary menu"}
                button {id:"r0242-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|epoch+=1,"Remount primary menu"}
            }
            div {class:"menu-order-workspaces",
                section {h2 {"Primary equipment menu"}div {id:"r0242-main",class:"menu-order-stage",
                    Application {menu_expanded:expanded,start_expanded:true,pinned_menu:true,force_breakpoint:ApplicationBreakpoint::Large,
                        menu:rsx! {if mounted(){Menu {key:"primary-{epoch()}",label:"Equipment navigation",application_name:"Equipment",start_expanded:true,pinned:true,enable_toggle_theme:true,
                            settings:rsx! {MenuSettings {p {"Equipment settings"}}},about:rsx! {MenuAbout {p {"Equipment application information"}}},
                            bottom:rsx! {MenuItem {label:"Footer action",order:Some(0),icon:"info",close_on_select:false,onclick:move |_|selected.call("Footer action".into())}},
                            MenuItem {label:"Dashboard",order:Some(0),icon:"home",close_on_select:false,onclick:move |_|selected.call("Dashboard".into())}
                            for (index,name) in entries.into_iter().enumerate().filter(|(_,name)|*name!="Alpha"||alpha_present()) {
                                OrderedEntry {key:"{name}",name:name.to_string(),order:index+1,hidden:name=="Alpha"&&alpha_hidden(),disabled:name=="Beta"&&beta_disabled(),child_reversed:child_reversed(),child_hidden:child_hidden(),onselect:move |name|selected.call(name)}
                            }
                        }}},
                        div {class:"menu-order-content",p {"Open Maintenance to navigate its explicitly registered tasks."}button {id:"r0242-main-outside",r#type:"button",class:"ix-button ix-button-secondary","Workspace"}}
                    }
                }}
                section {h2 {"Independent static menu"}div {id:"r0242-secondary",class:"menu-order-stage menu-order-secondary",
                    Application {start_expanded:true,pinned_menu:true,force_breakpoint:ApplicationBreakpoint::Large,menu:rsx! {Menu {label:"Independent navigation",start_expanded:true,pinned:true,
                        MenuItem {label:"Second one",icon:"home",close_on_select:false,onclick:move |_|secondary_selections+=1}
                        MenuItem {label:"Second two",icon:"info",close_on_select:false,onclick:move |_|secondary_selections+=1}
                    }},p {"Independent registration and focus ownership."}}
                }}
            }
            label {r#for:"r0242-outside","Outside focus destination"}input {id:"r0242-outside",class:"menu-order-field",value:"Independent focus"}
            output {id:"r0242-state",role:"status","aria-live":"polite","data-reversed":reversed().to_string(),"data-inserted":inserted().to_string(),"data-alpha-present":alpha_present().to_string(),"data-hidden":alpha_hidden().to_string(),"data-disabled":beta_disabled().to_string(),"data-child-reversed":child_reversed().to_string(),"data-child-hidden":child_hidden().to_string(),"data-expanded":expanded().to_string(),"data-mounted":mounted().to_string(),"data-epoch":epoch().to_string(),"data-selections":selections().to_string(),"data-last":last(),"data-secondary-selections":secondary_selections().to_string(),
                "Primary selections: {selections()} ({last()}); independent selections: {secondary_selections()}; generation: {epoch()}"
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {let task=super::dom_query::eval(NATIVE_MENU_ORDER_CHECK);let _=task.send("menu-order");match task.join::<serde_json::Value>().await {Ok(value) if value["ok"]==true=>{println!("Dioxus native Rust Menu order: {value}");window.close();}other=>{eprintln!("Dioxus native Rust Menu order failed: {other:?}");std::process::exit(1);}}});spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native Rust Menu order timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.menu-order-review{box-sizing:border-box;min-height:100vh;padding:24px}.menu-order-review h1{font-size:28px;margin:0 0 16px}.menu-order-review h2{font-size:20px;margin:0 0 16px}.menu-order-review p{max-width:70ch;line-height:1.5}.menu-order-controls{display:flex;flex-wrap:wrap;gap:10px;max-width:960px;margin:20px 0}.menu-order-workspaces{display:flex;flex-wrap:wrap;gap:24px}.menu-order-workspaces>section{flex:1 1 400px;min-width:0;max-width:100%}.menu-order-stage{height:660px;border:1px solid var(--ix-color-border)}.menu-order-stage>.ix-application{height:100%}.menu-order-secondary{height:320px}.menu-order-content{padding:12px;min-width:0;overflow-wrap:anywhere}.menu-order-secondary .ix-application-main{padding:12px}.menu-order-field{display:block;box-sizing:border-box;width:min(100%,360px);min-height:34px;margin:8px 0 16px;padding:6px 8px;background:var(--ix-color-surface);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.menu-order-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}@media(max-width:600px){.menu-order-review{padding:16px}.menu-order-stage>.ix-application[data-menu-expanded=true] .ix-application-body{flex-direction:column}.menu-order-stage>.ix-application[data-menu-pinned=true][data-menu-expanded=true] .ix-application-menu-slot{width:100%;height:500px;position:relative;flex-shrink:0}.menu-order-stage>.ix-application[data-menu-expanded=true] .ix-application-menu-slot .ix-menu{width:100%;height:100%;position:relative}.menu-order-secondary>.ix-application[data-menu-pinned=true][data-menu-expanded=true] .ix-application-menu-slot{height:210px}}";

// Synthetic events verify mounted focus and presentation, not browser key defaults.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_MENU_ORDER_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),state=()=>byId('r0242-state'),main=()=>byId('r0242-main').querySelector('nav.ix-menu'),secondary=()=>byId('r0242-secondary').querySelector('nav.ix-menu');
const bar=menu=>menu?.querySelector('[role="menubar"]'),item=(name,menu=main())=>Array.from(menu?.querySelectorAll('[role="menuitem"]')??[]).find(node=>node.getAttribute('aria-label')===name);
const category=()=>item('Maintenance')?.closest('.ix-menu-category'),opened=()=>item('Maintenance')?.getAttribute('aria-expanded')==='true';
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+document.activeElement?.getAttribute('aria-label')+' state='+state()?.textContent);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const go=async(from,value,name,shift=false,menu=main())=>{from.focus();key(from,value,shift);await wait(value+' focuses '+name,()=>document.activeElement===item(name,menu));};
const click=id=>byId(id).click();
await wait('two independent mounted menus',()=>bar(main())&&item('Alpha')&&item('Second one',secondary()));
await go(bar(main()),'Home','Dashboard');await go(item('Dashboard'),'ArrowDown','Alpha');await go(item('Alpha'),'ArrowDown','Maintenance');
await go(item('Maintenance'),'End','About & legal information');await go(item('About & legal information'),'ArrowUp','Toggle theme');await go(item('Toggle theme'),'ArrowUp','Settings');await go(item('Settings'),'ArrowUp','Footer action');
const alpha=item('Alpha');click('r0242-reverse');await wait('keyed primary reverse',()=>state().dataset.reversed==='true');if(item('Alpha')!==alpha)throw new Error('keyed Alpha was replaced');
await go(bar(main()),'Home','Dashboard');await go(item('Dashboard'),'ArrowDown','Maintenance');await go(item('Maintenance'),'ArrowDown','Alpha');
click('r0242-insert');await wait('Gamma inserted',()=>item('Gamma'));await go(item('Dashboard'),'ArrowDown','Gamma');click('r0242-insert');await wait('Gamma removed',()=>!item('Gamma'));
click('r0242-hidden');click('r0242-disabled');await wait('Rust hidden and disabled props updated',()=>state().dataset.hidden==='true'&&state().dataset.disabled==='false');await go(item('Maintenance'),'ArrowDown','Beta');
click('r0242-remove');await wait('Alpha removed',()=>!item('Alpha'));click('r0242-remove');click('r0242-hidden');await wait('Alpha restored visible',()=>item('Alpha')&&state().dataset.hidden==='false');
await go(item('Maintenance'),'Enter','Task one');await wait('expanded category opened',()=>opened());await go(item('Task one'),'Tab','Task two');await go(item('Task two'),'Tab','Task one',true);
click('r0242-child-reverse');await wait('wrapped child order reversed',()=>state().dataset.childReversed==='true');await go(item('Task one'),'ArrowDown','Task three');
click('r0242-child-hidden');await wait('wrapped child hidden prop',()=>state().dataset.childHidden==='true');await go(item('Task three'),'Tab','Task one');await go(item('Task one'),'Escape','Maintenance');await wait('expanded category Escape closes',()=>!opened());
click('r0242-expanded');await wait('primary menu collapsed',()=>main().dataset.expanded==='false'&&!opened());byId('r0242-outside').focus();category().scrollIntoView({block:'center'});await wait('hover anchor visible without focus',()=>category().getBoundingClientRect().top>=0&&category().getBoundingClientRect().bottom<=innerHeight);
category().dispatchEvent(new PointerEvent('pointerenter',{pointerType:'mouse',bubbles:false}));await wait('pointer hover opens native collapsed flyout',()=>opened()&&category().querySelector('[role="menu"]:popover-open'));
category().dispatchEvent(new PointerEvent('pointerleave',{pointerType:'mouse',bubbles:false}));await wait('pointer departure dismisses flyout',()=>!opened());
await go(item('Maintenance'),'Enter','Task three');await wait('collapsed keyboard presents native flyout',()=>opened()&&category().querySelector('[role="menu"]:popover-open'));
item('Task three').click();await wait('collapsed pointer selection dismisses once',()=>!opened()&&state().dataset.selections==='1'&&state().dataset.last==='Task three');
await go(bar(secondary()),'Home','Second one',false,secondary());await go(item('Second one',secondary()),'ArrowDown','Second two',false,secondary());if(state().dataset.selections!=='1')throw new Error('independent navigation selected in primary');
item('Second two',secondary()).click();await wait('independent selection remains isolated',()=>state().dataset.secondarySelections==='1'&&state().dataset.selections==='1');
byId('r0242-outside').focus();const retired=main();click('r0242-mount');await wait('primary owner removed',()=>!main()&&!retired.isConnected);click('r0242-mount');await wait('primary owner restored',()=>main()&&item('Dashboard'));
const beforeRemount=main();click('r0242-remount');await wait('keyed primary owner remount',()=>state().dataset.epoch==='1'&&main()!==beforeRemount&&!beforeRemount.isConnected);await go(bar(main()),'Home','Dashboard');
return {ok:true,syntheticEvents:true,primaryBottomUtilities:true,keyedOrderInsertRemove:true,explicitHiddenDisabled:true,wrappedCategoryOrderAndTab:true,expandedAndCollapsedPresentation:true,pointerHoverDismissal:true,selectionAndIndependentOwner:true,ownerCleanupAndRemount:true};
"#;
