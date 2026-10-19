//! Excluded two-surface workflow for the shared typed overlay stream lifetime.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn EquipmentPanel(
    name: String,
    title: String,
    field_label: String,
    popover: PopoverHandle,
    value: Signal<String>,
    actions: Signal<usize>,
) -> Element {
    let mut value = value;
    let mut actions = actions;
    rsx! {
        section {class:"r0254-panel",
            h2 {"{title}"}
            div {id:format!("r0254-{name}-open"),PopoverTrigger {popover,"Open {title}"}}
            Popover {popover,label:title.clone(),
                PopoverHeader {"{title}"}
                PopoverContent {
                    div {class:"r0254-form",
                        Input {id:format!("r0254-{name}-field"),label:field_label,value:value(),onvaluechange:move |next|value.set(next)}
                        Button {id:format!("r0254-{name}-action"),variant:ButtonVariant::Secondary,onclick:move |_|actions+=1,"Apply {title}"}
                    }
                }
            }
        }
    }
}

#[component]
pub fn TransportLifetimeReview() -> Element {
    let primary = use_popover();
    let peer = use_popover();
    let mut primary_visible = use_signal(|| true);
    let mut peer_visible = use_signal(|| true);
    let equipment = use_signal(|| "Pump P-101".to_string());
    let inspection = use_signal(|| "Inspect the inlet pressure".to_string());
    let primary_actions = use_signal(|| 0_usize);
    let peer_actions = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"transport-lifetime-review",
            h1 {"Independent equipment panels"}
            p {"Open either panel to edit its equipment details. Hiding a panel removes its controls; restoring it keeps the values. Both panels share document focus handling while they are mounted."}
            div {class:"r0254-controls",
                Button {id:"r0254-primary-toggle",variant:ButtonVariant::Secondary,onclick:move |_|primary_visible.toggle(),if primary_visible(){"Hide equipment panel"}else{"Restore equipment panel"}}
                Button {id:"r0254-peer-toggle",variant:ButtonVariant::Secondary,onclick:move |_|peer_visible.toggle(),if peer_visible(){"Hide inspection panel"}else{"Restore inspection panel"}}
                Button {id:"r0254-restore",variant:ButtonVariant::Secondary,onclick:move |_|{primary_visible.set(true);peer_visible.set(true);},"Restore both panels"}
            }
            div {class:"r0254-workspace",
                if primary_visible() {
                    EquipmentPanel {name:"primary",title:"Equipment details",field_label:"Equipment name",popover:primary,value:equipment,actions:primary_actions}
                } else {p {id:"r0254-primary-removed","Equipment panel hidden."}}
                if peer_visible() {
                    EquipmentPanel {name:"peer",title:"Inspection details",field_label:"Inspection note",popover:peer,value:inspection,actions:peer_actions}
                } else {p {id:"r0254-peer-removed","Inspection panel hidden."}}
            }
            output {id:"r0254-state",role:"status","aria-live":"polite",
                "data-primary-visible":primary_visible().to_string(),"data-peer-visible":peer_visible().to_string(),
                "data-equipment":equipment(),"data-inspection":inspection(),"data-primary-actions":primary_actions().to_string(),"data-peer-actions":peer_actions().to_string(),
                "Equipment: {equipment()}; inspection: {inspection()}; applied: {primary_actions()} / {peer_actions()}"
            }
            p {class:"r0254-note","The Linux driver uses synthetic input. Browser verification covers ordinary typing, Tab boundaries and Escape return."}
            div {onmounted:move |_|{
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_TRANSPORT_CHECK);let _=task.send("transport");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus typed overlay lifetime native: {value}");window.close();}
                        other=>{eprintln!("Dioxus typed overlay lifetime native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(35)).await;eprintln!("Dioxus typed overlay lifetime native timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.transport-lifetime-review{box-sizing:border-box;min-height:100vh;padding:24px}.transport-lifetime-review h1{font-size:28px;margin:0 0 16px}.transport-lifetime-review h2{font-size:20px;margin:0 0 12px}.transport-lifetime-review p{max-width:70ch;line-height:1.5}.r0254-controls{display:flex;flex-wrap:wrap;gap:12px;margin:20px 0}.r0254-workspace{display:flex;flex-wrap:wrap;gap:36px;margin:32px 0;max-width:900px}.r0254-panel{min-width:0;max-width:100%}.r0254-form{display:flex;flex-direction:column;gap:16px;width:280px;max-width:calc(100vw - 64px)}.transport-lifetime-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}.r0254-note{color:var(--ix-color-text-secondary)}@media(max-width:600px){.transport-lifetime-review{padding:16px}.r0254-workspace{flex-direction:column;gap:24px}}";

// No hidden inputs or native hub internals are used as a readiness handshake.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TRANSPORT_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement,state=()=>byId('r0254-state');
const trigger=name=>byId('r0254-'+name+'-open')?.querySelector('button');
const panel=name=>document.getElementById(trigger(name)?.getAttribute('aria-controls'));
const field=name=>byId('r0254-'+name+'-field'),action=name=>byId('r0254-'+name+'-action');
const opened=name=>panel(name)?.matches(':popover-open')&&panel(name)?.dataset.ixOverlayOpen==='true';
const closed=name=>panel(name)&&!panel(name).matches(':popover-open')&&panel(name).dataset.ixOverlayOpen==='false';
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' active='+active()?.id+' state='+state()?.textContent);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const open=async name=>{trigger(name).focus();trigger(name).click();await wait(name+' opening focuses registered standard Input',()=>opened(name)&&active()===field(name));};
const edit=async(name,value,property)=>{field(name).value=value;field(name).dispatchEvent(new Event('input',{bubbles:true}));await wait(name+' controlled edit',()=>state().dataset[property]===value);};
const escape=async name=>{key(active(),'Escape');await wait(name+' Escape returns to current trigger',()=>closed(name)&&active()===trigger(name));};
const noHiddenTransport=()=>!document.querySelector('input[data-ix-overlay-transport],input[data-ix-overlay-native],[data-ix-overlay-registry-native]');
await wait('two independent surfaces mounted',()=>trigger('primary')&&trigger('peer')&&field('primary')&&field('peer'));
if(!noHiddenTransport())throw new Error('hidden overlay transport remains');
await open('primary');await edit('primary','Pump P-102','equipment');await escape('primary');
const oldPrimary=panel('primary'),oldPrimaryTrigger=trigger('primary');
byId('r0254-primary-toggle').click();await wait('original surface retires while peer remains',()=>!trigger('primary')&&!oldPrimary.isConnected&&!oldPrimaryTrigger.isConnected&&trigger('peer'));
await open('peer');await edit('peer','Inspect the discharge pressure','inspection');await escape('peer');
const oldPeer=panel('peer');byId('r0254-peer-toggle').click();
await wait('last surface removed',()=>!trigger('primary')&&!trigger('peer')&&!oldPeer.isConnected&&document.querySelectorAll('.ix-popover').length===0);
if(!noHiddenTransport())throw new Error('transport input survived last release');
byId('r0254-primary-toggle').click();await wait('first surface remounted after last release',()=>trigger('primary')&&panel('primary')!==oldPrimary&&field('primary')?.value==='Pump P-102');
await open('primary');
if(key(field('primary'),'Tab').defaultPrevented)throw new Error('interior Input Tab intercepted after restart');
action('primary').focus();action('primary').click();await wait('remounted panel action remains useful',()=>state().dataset.primaryActions==='1');
if(!key(action('primary'),'Tab').defaultPrevented)throw new Error('forward Popover boundary not claimed after restart');
await wait('forward boundary wraps to header close',()=>active()===panel('primary').querySelector('.ix-dialog-close'));
await escape('primary');
byId('r0254-peer-toggle').click();await wait('peer restored with retained value',()=>trigger('peer')&&field('peer')?.value==='Inspect the discharge pressure');
await open('peer');await escape('peer');await open('primary');await escape('primary');
if(!noHiddenTransport())throw new Error('hidden transport reintroduced on restart');
return {ok:true,syntheticInput:true,openingStandardInput:true,retainedValues:true,escapeReturn:true,originalSurfaceRemovalPreservesPeer:true,lastSurfaceReleaseAndRestart:true,restartedTabBoundary:true,restoredPeerIndependent:true,hiddenTransportAbsent:true};
"#;
