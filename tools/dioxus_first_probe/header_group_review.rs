//! Excluded Header groups and standard form-control workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn HeaderEquipmentFields(
    visible: bool,
    note: Signal<String>,
    details: Signal<String>,
    selection: Signal<String>,
    applications: Signal<usize>,
) -> Element {
    let mut note = note;
    let mut details = details;
    let mut selection = selection;
    let mut applications = applications;
    // The helper itself stays mounted. Its caller declares whether its current
    // output is populated; Header never inspects this component's DOM.
    rsx! {
        if visible {
            div {class:"r0245-fields",
                Input {id:"r0245-input",label:"Equipment name",value:note(),
                    onvaluechange:move |value|note.set(value)}
                Textarea {id:"r0245-textarea",label:"Inspection note",value:details(),rows:Some(2),resize:"vertical",
                    onvaluechange:move |value|details.set(value)}
                Select {id:"r0245-select",label:"Equipment state",value:selection(),
                    options:vec![SelectOption::new("ready","Ready"),SelectOption::new("service","Needs service"),SelectOption::new("hold","On hold")],
                    onvaluechange:move |value|selection.set(value)}
                Button {id:"r0245-apply",onclick:move |_|applications+=1,"Apply equipment details"}
            }
        }
    }
}

#[component]
pub fn HeaderGroupReview() -> Element {
    let mut small = use_signal(|| false);
    let mut fields_visible = use_signal(|| true);
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let note = use_signal(|| "Pump P-101".to_string());
    let details = use_signal(|| "Inspect the inlet pressure.".to_string());
    let selection = use_signal(|| "ready".to_string());
    let applications = use_signal(|| 0_usize);
    let mut changes = use_signal(Vec::<bool>::new);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"header-group-review",
            h1 {"Dioxus-owned Header groups"}
            p {"Standard form controls register their mounted focus handles. The Header uses explicit slot presence while keeping values and component trees across responsive layouts."}
            div {class:"r0245-controls",
                Button {id:"r0245-layout",variant:ButtonVariant::Secondary,onclick:move |_|small.toggle(),
                    if small(){"Use large layout"}else{"Use small layout"}}
                Button {id:"r0245-fields-toggle",variant:ButtonVariant::Secondary,onclick:move |_|fields_visible.toggle(),
                    if fields_visible(){"Empty the helper"}else{"Restore form controls"}}
                Button {id:"r0245-mounted",variant:ButtonVariant::Secondary,onclick:move |_|mounted.toggle(),
                    if mounted(){"Remove Header"}else{"Restore Header"}}
                Button {id:"r0245-remount",variant:ButtonVariant::Secondary,onclick:move |_|{epoch+=1;mounted.set(true);},"Remount Header"}
            }
            section {
                h2 {"Equipment workspace"}
                p {"Open More with Home or End. Use Tab and Shift+Tab inside the form. F2 selects the small layout and F3 the large layout while keeping focus in the field."}
                div {id:"r0245-stage",class:"r0245-stage",
                    onkeydown:move |event:KeyboardEvent|match event.key() {
                        Key::F2=>{event.prevent_default();small.set(true);},
                        Key::F3=>{event.prevent_default();small.set(false);},
                        _=>{},
                    },
                    Application {class:"r0245-app",force_breakpoint:if small(){ApplicationBreakpoint::Small}else{ApplicationBreakpoint::Large},
                        header:mounted().then(||rsx! {ApplicationHeader {key:"{epoch()}",name:"Equipment",enable_top_layer:true,
                            aria_label_more_menu_icon_button:"Equipment form actions",
                            overflow_present:Some(fields_visible()),
                            onoverflowchange:move |open|changes.write().push(open),
                            overflow:rsx! {HeaderEquipmentFields {visible:fields_visible(),note,details,selection,applications}}
                        }}),
                        div {class:"r0245-body",
                            p {"Equipment values belong to Rust state. Emptying the helper hides More, and restoring it creates fresh mounted controls."}
                            Button {id:"r0245-outside",variant:ButtonVariant::Secondary,"Independent workspace action"}
                        }
                    }
                }
            }
            output {id:"r0245-state",role:"status","aria-live":"polite",
                "data-small":small().to_string(),"data-visible":fields_visible().to_string(),"data-mounted":mounted().to_string(),
                "data-epoch":epoch().to_string(),"data-note":note(),"data-details":details(),"data-selection":selection(),
                "data-applications":applications().to_string(),"data-changes":format!("{:?}",changes()),
                "Equipment: {note()}; state: {selection()}; applied: {applications()}"
            }
            p {class:"r0245-evidence-note","The Linux driver uses synthetic keys for focus boundaries and ordinary Dioxus events for editing. Physical Tab traversal is checked separately in Chromium."}
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_HEADER_GROUP_CHECK);let _=query.send("header-group-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus Header groups native: {value}");window.close();}
                        other=>{eprintln!("Dioxus Header groups native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus Header groups native timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.header-group-review{box-sizing:border-box;min-height:100vh;padding:24px}.header-group-review>h1{font-size:28px;margin:0 0 16px}.header-group-review h2{font-size:20px;margin:0 0 12px}.header-group-review p{max-width:70ch;line-height:1.5}.r0245-controls{display:flex;flex-wrap:wrap;gap:10px;margin:20px 0}.header-group-review section{margin:28px 0;max-width:1000px}.r0245-stage{max-width:100%;border:1px solid var(--ix-color-border)}.header-group-review .r0245-app{height:240px;min-height:240px}.r0245-body{padding:16px}.r0245-body p{margin-top:0}.r0245-fields{box-sizing:border-box;display:flex;flex-direction:column;gap:14px;width:min(270px,calc(100vw - 64px));max-width:100%;padding:8px}.r0245-fields>.ix-field,.r0245-fields>.ix-select{box-sizing:border-box;width:100%;max-width:100%;min-width:0}.r0245-fields .ix-select-control,.r0245-fields .ix-input-wrap{box-sizing:border-box;max-width:100%;min-width:0}.header-group-review output{display:block;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}.r0245-evidence-note{font-size:14px;color:var(--ix-color-soft-text)}@media(max-width:600px){.header-group-review{padding:16px}.r0245-body{padding:12px}}";

// Development-only synthetic Linux driver. Interior Tab is checked only for
// noninterception; synthetic events do not perform physical browser traversal.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_HEADER_GROUP_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement,state=()=>byId('r0245-state');
const root=()=>byId('r0245-stage').querySelector('.ix-header-layout'),more=()=>root()?.querySelector('.ix-header-more-button');
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id+' state='+state()?.textContent);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const edit=(id,value)=>{const node=byId(id);node.focus();node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));};
const open=async(last=false)=>{more().focus();key(more(),last?'End':'Home');await wait(last?'End entry to Apply':'Home entry to Input',()=>root().dataset.open==='true'&&active()===byId(last?'r0245-apply':'r0245-input'));};
const close=async()=>{byId('r0245-apply').focus();key(active(),'Escape');await wait('Escape returns to More',()=>root().dataset.open==='false'&&active()===more());};
await wait('Header registered and ready',()=>root()?.dataset.ixOverlayReady==='true'&&more()&&!more().hidden);
await open();
const firstInput=byId('r0245-input');
if(key(firstInput,'Tab').defaultPrevented)throw new Error('Interior Input Tab intercepted');
byId('r0245-apply').focus();if(!key(active(),'Tab').defaultPrevented)throw new Error('Forward boundary not claimed');
await wait('forward wraps to Input',()=>active()===firstInput);
if(!key(firstInput,'Tab',true).defaultPrevented)throw new Error('Reverse boundary not claimed');
await wait('reverse wraps to Apply',()=>active()===byId('r0245-apply'));
edit('r0245-input','Pump retained');await wait('Input edit retained',()=>state().dataset.note==='Pump retained');
edit('r0245-textarea','Inspection retained');await wait('Textarea edit retained',()=>state().dataset.details==='Inspection retained');
const select=byId('r0245-select');select.focus();key(select,'ArrowDown');await wait('Select opens',()=>select.getAttribute('aria-expanded')==='true');
key(select,'Home');await wait('Select first option committed',()=>select.getAttribute('aria-activedescendant')==='r0245-select-list-0');
key(select,'ArrowDown');await wait('Select second option committed',()=>select.getAttribute('aria-activedescendant')==='r0245-select-list-1');
key(select,'Enter');await wait('Select selects service',()=>state().dataset.selection==='service'&&select.getAttribute('aria-expanded')==='false');
firstInput.focus();key(firstInput,'F2');await wait('small layout preserves Input element and value',()=>root().dataset.small==='true'&&active()===firstInput&&firstInput===byId('r0245-input')&&firstInput.value==='Pump retained');
key(firstInput,'F3');await wait('large layout preserves Input element and value',()=>root().dataset.small==='false'&&active()===firstInput&&byId('r0245-textarea').value==='Inspection retained'&&state().dataset.selection==='service');
await close();await open(true);
const before=+state().dataset.applications;byId('r0245-apply').click();await wait('Apply action once',()=>+state().dataset.applications===before+1);
if(root().dataset.open==='true')await close();
byId('r0245-fields-toggle').click();await wait('empty helper hides More',()=>state().dataset.visible==='false'&&more().hidden&&!byId('r0245-input'));
byId('r0245-fields-toggle').click();await wait('helper restores fresh form controls',()=>state().dataset.visible==='true'&&!more().hidden&&byId('r0245-input')&&byId('r0245-input')!==firstInput);
await open();if(byId('r0245-input').value!=='Pump retained')throw new Error('Restored controlled value lost');await close();
const oldInput=byId('r0245-input'),oldMore=more();byId('r0245-remount').click();await wait('Header remount replaces registration',()=>state().dataset.epoch==='1'&&more()!==oldMore&&byId('r0245-input')!==oldInput);
await open();await close();
byId('r0245-mounted').click();await wait('Header removed',()=>state().dataset.mounted==='false'&&!root());
byId('r0245-mounted').click();await wait('Header restored',()=>state().dataset.mounted==='true'&&more()&&!more().hidden);
await open();await close();
return {ok:true,homeInput:true,endApply:true,forwardBoundary:true,reverseBoundary:true,inputEdit:true,textareaEdit:true,selectSelection:true,responsiveSameElement:true,emptyHelperPresence:true,freshHelperControls:true,headerRemount:true,headerRemoval:true,applyOnce:true,syntheticInteriorTab:false};
"#;
