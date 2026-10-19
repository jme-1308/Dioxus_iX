//! Excluded PaneLayout mounted geometry, standalone sizing and core interaction workflow.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn RegisteredPaneContent(value: Signal<String>, onaction: EventHandler<()>) -> Element {
    let field_mount = use_focus_mount("r0240-note");
    let action_mount = use_focus_mount("r0240-pane-action");
    let mut value = value;
    rsx! {
        label {r#for:"r0240-note","Equipment name"}
        input {id:"r0240-note",class:"pane-layout-field",value:value(),autofocus:true,onmounted:field_mount,oninput:move |event|value.set(event.value())}
        button {id:"r0240-pane-action",r#type:"button",class:"ix-button ix-button-secondary",onmounted:action_mount,onclick:move |_|onaction.call(()),"Apply equipment name"}
    }
}

#[component]
pub fn PaneLayoutReview() -> Element {
    let mut narrow = use_signal(|| false);
    let mut short = use_signal(|| false);
    let mut size = use_signal(|| "33%".to_string());
    let mut floating = use_signal(|| false);
    let mut horizontal = use_signal(|| false);
    let mut left_open = use_signal(|| true);
    let mut top_open = use_signal(|| false);
    let mut veto = use_signal(|| false);
    let mut requests = use_signal(|| 0_usize);
    let mut changes = use_signal(|| 0_usize);
    let mut actions = use_signal(|| 0_usize);
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut explicit_large = use_signal(|| false);
    let note = use_signal(|| "Pump P-101".to_string());
    let opener_mount = use_focus_mount("r0240-open");
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"pane-layout-review",
            h1 {"Panes sized by their Dioxus layout"}
            p {"Resize the workspace, change its pane size, or use floating presentation. The nested layout owns independent measurements. Standalone panes use explicit available space or the shared viewport."}
            div {class:"pane-layout-controls",
                button {id:"r0240-width",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|narrow.toggle(),"Toggle workspace width"}
                button {id:"r0240-height",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|short.toggle(),"Toggle workspace height"}
                label {r#for:"r0240-size","Left pane size"}
                select {id:"r0240-size",value:size(),onchange:move |event|size.set(event.value()),option {value:"33%","33%"} option {value:"50%","50%"} option {value:"600px","600px (capped)"}}
                button {id:"r0240-mode",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |event:MouseEvent|{event.stop_propagation();floating.toggle();},"Toggle floating / inline"}
                button {id:"r0240-order",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|horizontal.toggle(),"Toggle layout order"}
                button {id:"r0240-open",r#type:"button",class:"ix-button ix-button-primary",onmounted:opener_mount,onclick:move |event:MouseEvent|{event.stop_propagation();left_open.set(true);},"Open equipment pane"}
                button {id:"r0240-veto",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":veto().to_string(),onclick:move |event:MouseEvent|{event.stop_propagation();veto.toggle();},"Toggle close veto (F6 inside pane)"}
                button {id:"r0240-mount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|mounted.toggle(),"Hide / show layout"}
                button {id:"r0240-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|{epoch+=1;left_open.set(false);},"Remount layout"}
            }
            section {
                h2 {"Resizable equipment workspace"}
                div {id:"r0240-stage",style:format!("width:{}px;height:{}px",if narrow(){480}else{840},if short(){320}else{440}),onkeydown:move |event:KeyboardEvent|if event.key()==Key::F6{event.prevent_default();veto.toggle();},
                    if mounted() {
                        PaneLayout {key:"layout-{epoch()}",variant:if floating(){PaneVariant::Floating}else{PaneVariant::Inline},layout:if horizontal(){PaneLayoutOrder::FullHorizontal}else{PaneLayoutOrder::FullVertical},
                            left:rsx! {Pane {heading:"Equipment details",composition:PaneComposition::Right,size:size(),expanded:left_open(),hide_on_collapse:true,close_on_click_outside:floating(),aria_label_collapse_close_button:"Close equipment pane",
                                onbeforeexpandedchange:move |event:PaneExpandedChangeEvent|{requests+=1;if veto()&&!event.expanded{event.prevent_default();}},
                                onexpandedchange:move |open|{left_open.set(open);changes+=1;},
                                RegisteredPaneContent {value:note,onaction:move |_|actions+=1}
                            }},
                            top:rsx! {Pane {heading:"Workspace status",size:"50%",expanded:top_open(),onexpandedchange:move |open|top_open.set(open),p {"Top slot uses half the measured layout height."}}},
                            div {class:"pane-layout-content",
                                h3 {"Independent nested layout"}
                                div {id:"r0240-nested",PaneLayout {
                                    right:rsx! {Pane {heading:"Nested information",size:"33%",expanded:true,p {"One third of this nested layout."}}},
                                    p {"Nested workspace"}
                                }}
                                button {id:"r0240-content-action",r#type:"button",class:"ix-button ix-button-secondary","Workspace outside destination"}
                            }
                        }
                    } else {p {"Layout removed; its mounted measurements are retired."}}
                }
            }
            section {
                h2 {"Standalone available space"}
                button {id:"r0240-explicit-size",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|explicit_large.toggle(),"Toggle explicit available width"}
                div {class:"pane-layout-standalone-row",
                    div {id:"r0240-explicit",Pane {heading:"Explicit available size",composition:PaneComposition::Right,size:"50%",available_size:Some((if explicit_large(){600.0}else{360.0},240.0)),expanded:true,p {"Explicit size controls the available axis."}}}
                    div {id:"r0240-viewport",Pane {heading:"Viewport fallback",composition:PaneComposition::Right,size:"33%",expanded:true,p {"Without a layout, shared viewport width determines this pane size."}}}
                }
            }
            label {r#for:"r0240-outside","Outside focus destination"}
            input {id:"r0240-outside",class:"pane-layout-field",value:"Independent focus"}
            output {id:"r0240-state",role:"status","aria-live":"polite","data-narrow":narrow().to_string(),"data-short":short().to_string(),"data-size":size(),"data-floating":floating().to_string(),"data-horizontal":horizontal().to_string(),"data-open":left_open().to_string(),"data-veto":veto().to_string(),"data-requests":requests().to_string(),"data-changes":changes().to_string(),"data-actions":actions().to_string(),"data-mounted":mounted().to_string(),"data-epoch":epoch().to_string(),"data-explicit-large":explicit_large().to_string(),
                "Expansion requests: {requests()}; changes: {changes()}; actions: {actions()}; equipment: {note()}"
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_PANE_LAYOUT_CHECK);let _=task.send("pane-layout");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native mounted Pane layout: {value}");window.close();}
                        other=>{eprintln!("Dioxus native mounted Pane layout failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native mounted Pane layout timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.pane-layout-review{box-sizing:border-box;min-height:100vh;padding:24px}.pane-layout-review h1{font-size:28px;margin:0 0 16px}.pane-layout-review h2{font-size:20px;margin:0 0 16px}.pane-layout-review h3{font-size:18px;margin:0 0 12px}.pane-layout-review p{max-width:70ch;line-height:1.5}.pane-layout-review section{margin:28px 0;max-width:940px}.pane-layout-controls{display:flex;flex-wrap:wrap;align-items:center;gap:10px;max-width:940px;margin:16px 0}#r0240-stage{max-width:100%;box-sizing:border-box;border:1px solid var(--ix-color-border)}.pane-layout-content{padding:12px}.pane-layout-content p{padding:8px}#r0240-nested{width:300px;max-width:100%;height:280px}.pane-layout-standalone-row{display:flex;flex-wrap:wrap;gap:20px;margin:16px 0}.pane-layout-standalone-row>div{position:relative;width:400px;max-width:100%;height:240px;border:1px solid var(--ix-color-border);overflow:hidden}.pane-layout-field{display:block;box-sizing:border-box;width:min(100%,300px);min-height:34px;margin:8px 0 16px;padding:6px 8px;background:var(--ix-color-surface);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.pane-layout-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}@media(max-width:600px){.pane-layout-review{padding:16px}}";

// Synthetic Linux events; real keyboard/pointer interaction is checked separately.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PANE_LAYOUT_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),status=()=>byId('r0240-state');
const layout=()=>byId('r0240-stage')?.querySelector(':scope > .ix-pane-layout');
const pane=slot=>layout()?.querySelector(':scope > .ix-pane-slot-'+slot+' > .ix-pane');
const nested=()=>byId('r0240-nested')?.querySelector(':scope > .ix-pane-layout');
const nestedPane=()=>nested()?.querySelector(':scope > .ix-pane-slot-right > .ix-pane');
const size=node=>Number.parseFloat(node?.style.getPropertyValue('--ix-pane-size'));
const close=slot=>pane(slot)?.querySelector('.ix-pane-header > button');
const open=()=>pane('left')?.dataset.expanded==='true';
const near=(actual,expected)=>Number.isFinite(actual)&&Math.abs(actual-expected)<=2;
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' size='+size(pane('left'))+' open='+open()+' focus='+document.activeElement?.id);};
const click=id=>byId(id).click();
const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
const choose=async value=>{byId('r0240-size').value=value;byId('r0240-size').dispatchEvent(new Event('change',{bubbles:true}));await wait('chosen '+value,()=>status().dataset.size===value);};
await wait('mounted left and top slots use layout axes',()=>layout()&&near(size(pane('left')),Math.round(layout().getBoundingClientRect().width/3))&&near(size(pane('top')),Math.round(layout().getBoundingClientRect().height/2)));
if(!pane('left').classList.contains('ix-pane-left'))throw new Error('layout did not override supplied composition');
const initialWidth=layout().getBoundingClientRect().width;
click('r0240-width');await wait('real container width updates percent sizing',()=>status().dataset.narrow==='true'&&layout().getBoundingClientRect().width<initialWidth-100&&near(size(pane('left')),Math.round(layout().getBoundingClientRect().width/3)));
await choose('50%');await wait('half width size',()=>near(size(pane('left')),Math.round(layout().getBoundingClientRect().width/2)));
await choose('600px');await wait('pixel size capped at measured half width',()=>near(size(pane('left')),Math.round(layout().getBoundingClientRect().width/2)));
const initialHeight=layout().getBoundingClientRect().height;
click('r0240-height');await wait('real container height updates top axis',()=>status().dataset.short==='true'&&layout().getBoundingClientRect().height<initialHeight-80&&near(size(pane('top')),Math.round(layout().getBoundingClientRect().height/2)));
close('top').click();await wait('top pane expands to rendered half height',()=>pane('top').dataset.expanded==='true'&&near(pane('top').getBoundingClientRect().height,size(pane('top'))));close('top').click();
await wait('nested layout measures independently',()=>nested()&&near(size(nestedPane()),Math.round(nested().getBoundingClientRect().width/3))&&nested().getBoundingClientRect().width<layout().getBoundingClientRect().width);
click('r0240-order');await wait('layout order updates',()=>layout().dataset.layout==='full-horizontal');
const explicit=()=>byId('r0240-explicit').querySelector('.ix-pane'),fallback=()=>byId('r0240-viewport').querySelector('.ix-pane');
await wait('standalone explicit and viewport sizes',()=>near(size(explicit()),180)&&near(size(fallback()),Math.round((window.visualViewport?.width??innerWidth)/3)));
click('r0240-explicit-size');await wait('explicit available width updates independently',()=>near(size(explicit()),300)&&near(size(fallback()),Math.round((window.visualViewport?.width??innerWidth)/3)));
close('left').click();await wait('inline pane closes',()=>!open());
click('r0240-mode');await wait('floating layout settings propagate',()=>pane('left').dataset.floating==='true');
byId('r0240-open').focus();click('r0240-open');await wait('floating opening focuses registered field',()=>open()&&document.activeElement===byId('r0240-note'));
click('r0240-pane-action');await wait('registered pane action',()=>status().dataset.actions==='1');
key(byId('r0240-note'),'F6');await wait('close veto enabled',()=>status().dataset.veto==='true');const requests=Number(status().dataset.requests);close('left').click();await wait('close veto preserves expanded pane',()=>Number(status().dataset.requests)>requests&&open());
key(byId('r0240-note'),'F6');await wait('close veto disabled',()=>status().dataset.veto==='false');close('left').click();await wait('close accepted after veto removed',()=>!open());
click('r0240-open');await wait('reopen for outside dismissal',()=>open());click('r0240-content-action');await wait('layout content dismisses floating pane',()=>!open());
const retired=layout();click('r0240-mount');await wait('layout retires',()=>status().dataset.mounted==='false'&&!layout()&&!retired.isConnected);
click('r0240-mount');await wait('layout restored with current geometry',()=>layout()&&near(size(pane('left')),Math.round(layout().getBoundingClientRect().width/2)));
const beforeRemount=layout();click('r0240-remount');await wait('keyed remount owns fresh measurements',()=>status().dataset.epoch==='1'&&layout()!==beforeRemount&&!beforeRemount.isConnected&&near(size(pane('left')),Math.round(layout().getBoundingClientRect().width/2)));
return {ok:true,syntheticEvents:true,layoutWidthAndHeight:true,percentAndPixelCap:true,nestedIndependent:true,standaloneExplicitAndViewport:true,slotAndVariantSettings:true,registeredFocusAndAction:true,vetoAndOutsideClose:true,hideAndRemount:true,mobile:pane('left').dataset.mobile};
"#;
