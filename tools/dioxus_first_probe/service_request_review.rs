//! Excluded actual-gallery service request and bounded Linux form/style driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/service_request.rs"]
mod gallery_request;

#[component]
pub fn ServiceRequestPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_request::STYLES} }
        IxTheme { class: "service-request-review",
            h1 { "Service request" }
            gallery_request::ServiceRequestExample {}
            section { class: "service-pill-review",
                h2 { "Compact status treatments" }
                p { "Outlined labels follow readable text tokens; filled and application colors remain distinct." }
                for (scheme, name, outline_color) in [(ColorScheme::Light, "light", "#145b70"), (ColorScheme::Dark, "dark", "#c7f4ff")] {
                    IxTheme { scheme, class: "service-pill-scheme",
                        div { id: "service-pills-{name}",
                            h3 { "{name}" }
                            div { class: "service-pill-row",
                                span { id: "service-pill-{name}-outline-neutral", Pill { variant: StatusVariant::Neutral, outline: true, icon: "info", "Neutral" } }
                                span { id: "service-pill-{name}-outline-warning", Pill { variant: StatusVariant::Warning, outline: true, icon: "warning", "Warning" } }
                                span { id: "service-pill-{name}-outline-custom", Pill { variant: StatusVariant::Custom, outline: true,
                                    background: "#e0f5fa", color: outline_color, icon: "info", "Custom outline" } }
                                span { id: "service-pill-{name}-filled-neutral", Pill { variant: StatusVariant::Neutral, "Neutral" } }
                                span { id: "service-pill-{name}-filled-warning", Pill { variant: StatusVariant::Warning, "Warning" } }
                                span { id: "service-pill-{name}-filled-custom", Pill { variant: StatusVariant::Custom,
                                    background: "#145b70", color: "#ffffff", "Custom filled" } }
                            }
                            div { class: "service-pill-tokens", aria_hidden: "true",
                                span { id: "service-token-{name}-text", style: "color:var(--ix-color-text)", "Text" }
                                span { id: "service-token-{name}-border", style: "color:var(--ix-color-border)", "Border" }
                                span { id: "service-token-{name}-surface", style: "color:var(--ix-color-surface-2)", "Surface" }
                                span { id: "service-token-{name}-warning", style: "color:var(--ix-color-warning)", "Warning" }
                                span { id: "service-token-{name}-warning-text", style: "color:var(--ix-color-warning-text)", "Warning icon" }
                                span { id: "service-token-{name}-on-warning", style: "color:var(--ix-color-on-warning,#fff)", "On warning" }
                            }
                        }
                    }
                }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_SERVICE_REQUEST_CHECK);
                        if let Err(error) = query.send("servicerequest") {
                            eprintln!("Service request native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native service request: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native service request failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Service request native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.service-request-review{box-sizing:border-box;min-height:100vh;padding:24px}.service-request-review h1{font-size:28px;margin:0 0 24px}.service-request-review h2{font-size:22px;margin:24px 0 16px}.service-request-review h3{font-size:18px}.service-request-review p{max-width:72ch;line-height:1.5}.service-pill-review{margin-top:32px}.service-pill-scheme{padding:16px;margin:12px 0;max-width:100%;box-sizing:border-box}.service-pill-scheme h3{margin:0 0 16px}.service-pill-row{display:flex;align-items:center;flex-wrap:wrap;gap:12px}.service-pill-tokens{display:none}@media(max-width:600px){.service-request-review{padding:16px}}";

// Driver-only JS uses actual requestSubmit/FormData, explicit native focus and
// synthetic input/keyboard/click events. Rust gallery controls own validation,
// payload preview, read-only state and reset. Programmatically changing read-only
// native values would bypass browser semantics, so this driver does not do that.
// Actual browser typing, keyboard defaults and pointer input are separate web
// evidence, without claiming physical-device certification.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SERVICE_REQUEST_CHECK: &str = r#"
let snapshots={},pills={},layout={},inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('service-request'),form=()=>get('service-form'),result=()=>get('service-result');
 const asset=()=>get('service-asset'),priority=()=>get('service-priority'),technicians=()=>get('service-technicians'),notes=()=>get('service-notes');
 const shutdown=()=>get('service-shutdown'),lock=()=>get('service-lock'),preview=()=>get('service-preview');
 const checked=node=>node.getAttribute('aria-checked')==='true';
 const nativePriority=()=>form()?.querySelector('select[name="priority"]');
 const numberBox=()=>technicians()?.closest('.ix-number-field');
 const plus=()=>numberBox()?.querySelector('button[aria-label="increment number"]');
 const statusPill=()=>host()?.querySelector('.ix-pill');
 const state=()=>result()?.dataset.status,submissions=()=>Number(result()?.dataset.submissions);
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; result='+result()?.textContent+'; focus='+(active()?.id||active()?.tagName));
 };
 const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true}));
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setValue=async(node,value,label)=>{
  const prototype=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(prototype,'value').set.call(node,value);
  node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(label+' controlled value applied',()=>node.isConnected&&node.value===value);
 };
 const submit=()=>form().requestSubmit(get('service-submit'));
 const payload=()=>Object.fromEntries(new FormData(form()).entries());
 const snapshot=()=>({status:state(),submissions:submissions(),asset:asset().value,priority:nativePriority().value,
  technicians:technicians().value,notes:notes().value,shutdown:checked(shutdown()),locked:checked(lock()),preview:preview()?{...preview().dataset}:null});
 const pillFacts=node=>{
  const label=node.querySelector('.ix-status-label'),css=getComputedStyle(node),box=node.getBoundingClientRect();
  const icon=node.querySelector('svg');
  return {label:label.textContent.trim(),text:getComputedStyle(label).color,background:css.backgroundColor,
   border:css.borderTopColor,icon:icon?getComputedStyle(icon).color:null,height:box.height,width:box.width};
 };
 const luminance=color=>{
  const channels=color.match(/[\d.]+/g).slice(0,3).map(Number).map(value=>{
   const channel=value/255;return channel<=0.04045?channel/12.92:Math.pow((channel+0.055)/1.055,2.4);
  });return channels[0]*0.2126+channels[1]*0.7152+channels[2]*0.0722;
 };
 const contrast=(foreground,background)=>{
  const a=luminance(foreground),b=luminance(background);return (Math.max(a,b)+0.05)/(Math.min(a,b)+0.05);
 };
 const actualStatusPill=label=>{
  const facts=pillFacts(statusPill());assert(facts.label.includes(label)&&Math.abs(facts.height-20)<=1,'actual '+label+' status Pill preserves20px geometry');
  const theme=statusPill().closest('.ix-theme'),surface=getComputedStyle(theme).backgroundColor;
  assert(contrast(facts.text,surface)>=4.5,'actual '+label+' outlined status remains readable against theme surface');return facts;
 };

 await wait('actual controlled service form mounted',()=>host()&&form()&&result()&&asset()&&priority()&&technicians()&&notes()&&shutdown()&&lock()&&plus()&&statusPill());
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 assert(form().noValidate&&get('service-submit').type==='submit','native noValidate form delegates acceptance to Rust host validation');
 assert(state()==='draft'&&submissions()===0&&!preview()&&asset().value===''&&nativePriority().value==='normal'&&technicians().value==='2'&&notes().value===''&&!checked(shutdown())&&!checked(lock()),'initial request values/modes are a clean draft');
 pills.actualDraft=actualStatusPill('Draft');await focus(get('service-submit'),'submit before required validation');submit();
 await wait('required asset rejection and controller focus',()=>state()==='invalid'&&submissions()===0&&!preview()&&active()===asset()&&asset().getAttribute('aria-invalid')==='true');
 inputFocusFacts={documentFocused:document.hasFocus(),active:active()?.id};pills.actualInvalid=actualStatusPill('Needs attention');
 snapshots.emptyInvalid=snapshot();
 await setValue(asset(),'MTR-42','asset tag');await setValue(notes(),'Inspect bearings before restart.','work notes');
 await wait('field changes return request to draft',()=>state()==='draft'&&!preview());
 await focus(priority(),'Priority combobox');key(priority(),'ArrowDown');
 await wait('keyboard opens actual Select popup',()=>priority().getAttribute('aria-expanded')==='true'&&get(priority().getAttribute('aria-controls')));
 key(priority(),'End');
 await wait('keyboard moves to High priority option',()=>get(priority().getAttribute('aria-activedescendant'))?.textContent.includes('High'));
 key(priority(),'Enter');
 await wait('keyboard selects High in native form mirror',()=>nativePriority().value==='high'&&priority().getAttribute('aria-expanded')==='false');
 plus().click();await wait('numeric stepper increments controlled value',()=>technicians().value==='3');
 await focus(technicians(),'Technicians');key(technicians(),'ArrowUp');await wait('numeric ArrowUp increments controlled value',()=>technicians().value==='4');
 await setValue(technicians(),'7','out-of-range technicians');await focus(get('service-submit'),'submit before numeric validation');submit();
 await wait('host rejects out-of-range integer and focuses numeric control',()=>state()==='invalid'&&submissions()===0&&!preview()&&active()===technicians()&&technicians().getAttribute('aria-invalid')==='true');
 snapshots.rangeInvalid=snapshot();await setValue(technicians(),'4','valid technicians');
 shutdown().click();await wait('custom Checkbox updates native shutdown form mirror',()=>checked(shutdown())&&form().querySelector('input[type="checkbox"][name="shutdown"]')?.checked);
 const actualPayload=payload();
 assert(actualPayload.asset==='MTR-42'&&actualPayload.priority==='high'&&actualPayload.technicians==='4'&&actualPayload.notes==='Inspect bearings before restart.'&&actualPayload.shutdown==='on','real native FormData has the five intended field names and current values');
 submit();
 await wait('native submit creates successful payload preview',()=>state()==='ready'&&submissions()===1&&preview());
 assert(preview().tagName==='DL'&&preview().dataset.asset===actualPayload.asset&&preview().dataset.priority===actualPayload.priority&&preview().dataset.technicians===actualPayload.technicians&&preview().dataset.notes===actualPayload.notes&&preview().dataset.shutdown==='true','actual preview reflects accepted native named payload');
 pills.actualReady=actualStatusPill('Preview ready');snapshots.ready=snapshot();
 await setValue(notes(),'Inspect compressor after shutdown.','changed work notes');
 await wait('changed fields invalidate old preview without erasing submission count',()=>state()==='draft'&&!preview()&&submissions()===1);
 lock().click();
 await wait('read-only lock applies to all controls and submit',()=>checked(lock())&&asset().readOnly&&priority().readOnly&&technicians().readOnly&&notes().readOnly&&shutdown().disabled&&get('service-submit').disabled);
 const locked=snapshot();await focus(technicians(),'readonly numeric input');key(technicians(),'ArrowUp');
 await wait('readonly numeric input blocks stepping',()=>technicians().value===locked.technicians&&numberBox().querySelector('.ix-number-steppers').dataset.hidden==='true');
 await focus(priority(),'readonly Select');key(priority(),'ArrowDown');priority().click();
 await wait('readonly Select does not open or change selected value',()=>priority().getAttribute('aria-expanded')==='false'&&nativePriority().value===locked.priority);
 shutdown().click();get('service-submit').click();
 await wait('disabled shutdown and submit produce no state changes',()=>checked(shutdown())===locked.shutdown&&state()==='draft'&&submissions()===1&&!preview());
 snapshots.locked=snapshot();lock().click();
 await wait('read-only lock releases normal controls',()=>!checked(lock())&&!asset().readOnly&&!priority().readOnly&&!technicians().readOnly&&!notes().readOnly&&!shutdown().disabled&&!get('service-submit').disabled);
 await focus(priority(),'Select before reset');key(priority(),'ArrowDown');
 await wait('Select is open before Dioxus-owned reset',()=>priority().getAttribute('aria-expanded')==='true'&&get(priority().getAttribute('aria-controls')));
 get('service-reset').focus();get('service-reset').click();
 await wait('reset replaces form-owned state and removes stale Select popup',()=>state()==='draft'&&submissions()===0&&!preview()&&asset()?.value===''&&nativePriority()?.value==='normal'&&technicians()?.value==='2'&&notes()?.value===''&&!checked(shutdown())&&!checked(lock())&&priority()?.getAttribute('aria-expanded')==='false'&&!document.querySelector('.ix-select-backdrop')&&active()===get('service-reset'));
 assert(!asset().readOnly&&!technicians().readOnly&&!get('service-submit').disabled,'reset restores editable mode');snapshots.reset=snapshot();

 for(const scheme of ['light','dark']){
  const root=get('service-pills-'+scheme),theme=root.closest('.ix-theme'),surface=getComputedStyle(theme).backgroundColor;
  const token=name=>getComputedStyle(get('service-token-'+scheme+'-'+name)).color;
  const facts={};
  for(const name of ['outline-neutral','outline-warning','outline-custom','filled-neutral','filled-warning','filled-custom']){
   facts[name]=pillFacts(get('service-pill-'+scheme+'-'+name).querySelector('.ix-pill'));
   assert(Math.abs(facts[name].height-20)<=1,scheme+' '+name+' Pill retains20px height');
  }
  for(const name of ['outline-neutral','outline-warning']){
   facts[name].contrast=contrast(facts[name].text,surface);
   assert(facts[name].text===token('text')&&facts[name].background==='rgba(0, 0, 0, 0)'&&facts[name].contrast>=4.5,scheme+' '+name+' outlined label uses readable normal text');
  }
  assert(facts['outline-neutral'].border===token('border'),'neutral outline preserves visible border in '+scheme);
  assert(facts['outline-warning'].border===token('warning')&&facts['outline-warning'].icon===token('warning-text'),'warning outline preserves semantic border and readable icon in '+scheme);
  const customText=scheme==='light'?'rgb(20, 91, 112)':'rgb(199, 244, 255)';
  assert(facts['outline-custom'].text===customText&&facts['outline-custom'].border==='rgb(224, 245, 250)','custom outlined foreground/border remain application-authored in '+scheme);
  assert(facts['filled-neutral'].background===token('surface')&&facts['filled-neutral'].text===token('text'),'filled neutral palette preserved in '+scheme);
  assert(facts['filled-warning'].background===token('warning')&&facts['filled-warning'].text===token('on-warning'),'filled warning palette preserved in '+scheme);
  assert(facts['filled-custom'].background==='rgb(20, 91, 112)'&&facts['filled-custom'].text==='rgb(255, 255, 255)','filled custom palette preserved in '+scheme);
  pills[scheme]={surface,...facts};
 }
 const box=form().getBoundingClientRect();layout={formWidth:box.width,formHeight:box.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(layout.formWidth>100&&layout.pageScroll<=layout.pageClient+1,'actual form and compact Pill examples contain horizontal page overflow');
 return {ok:true,snapshots,pills,layout,inputFocusFacts,
  checks:['required asset and range validation with Dioxus controller focus','Select keyboard high priority and NumberInput stepper/ArrowUp','actual native named FormData and successful requestSubmit payload preview','edits invalidate old preview retaining accepted count','readonly properties and blocked component interactions','reset clears data/modes/count with open Select cleanup','real draft/invalid/ready status Pill','bounded light/dark outlined/filled/custom Pill color and20px geometry'],
  input:'actual native requestSubmit/FormData with synthetic click/input/keyboard events and explicit focus',
  trustedInput:'separate Chromium actual keyboard typing/activation and native readonly defaults; not physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,pills,layout,inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
