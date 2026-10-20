//! Excluded actual-gallery slider content preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/slider_limits.rs"]
mod gallery_limits;

#[component]
pub fn SliderContentPreview() -> Element {
    let mut narrow = use_signal(|| false);
    let mut plain = use_signal(|| 25.0);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_limits::STYLES} }
        IxTheme { class: "slider-content-review",
            h1 { "Slider content" }
            Button { id: "slider-content-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "slider-content-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                div { class: "slider-content-columns",
                    gallery_limits::SliderLimitsExample {}
                    section { id: "slider-content-plain",
                        h2 { "Plain endpoint labels" }
                        Slider { label: "Plain slider", value: plain(), min: 0.0, max: 100.0, step: 1.0,
                            label_start: "0", label_end: "100", onvaluechange: move |value| plain.set(value) }
                    }
                }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_SLIDER_CONTENT_CHECK);
                        if let Err(error) = query.send("slidercontent") {
                            eprintln!("Slider content native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native slider content: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native slider content failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Slider content native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.slider-content-review{box-sizing:border-box;min-height:100vh;padding:24px}.slider-content-review>h1{font-size:28px;margin:0 0 24px}.slider-content-review #slider-content-plain>h2{font-size:22px;margin:0 0 16px}.slider-content-columns{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:32px}.slider-content-columns>section{min-width:0}#slider-content-frame{margin-top:24px}#slider-content-frame[style*=\"340px\"] .slider-content-columns{grid-template-columns:minmax(0,1fr)}@media(max-width:600px){.slider-content-review{padding:16px}}";

// Excluded native replay; native range keyboard defaults are separate trusted-browser evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SLIDER_CONTENT_CHECK: &str = r#"
let snapshots={},hitFacts={},layout={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('slider-limits-result'),form=()=>get('slider-limits-form'),range=()=>get('slider-limits')?.querySelector('input[type="range"]'),note=()=>get('slider-limits-note'),plain=()=>get('slider-content-plain')?.querySelector('input[type="range"]');
 const idle=()=>get('slider-limits-idle'),rated=()=>get('slider-limits-rated'),save=()=>get('slider-limits-save'),reset=()=>get('slider-limits-reset'),field=name=>get('slider-limits-saved')?.querySelector('[data-field="'+name+'"]')?.textContent,endpoint=()=>get('slider-limits')?.querySelector('.ix-slider-endpoints'),saves=()=>Number(result()?.dataset.saves);
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const input=async(node,value,label)=>{await focus(node,label);Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,String(value));node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' controlled value',()=>node.value===String(value))};
 const disabled=async value=>{const node=get('slider-limits-disabled');node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('actual disabled state '+value,()=>result().dataset.disabled===String(value))};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const hit=async(node,label)=>{await focus(node,label);await wait(label+' actual native hit/containment',()=>{const r=node.getBoundingClientRect(),frame=get('slider-content-frame').getBoundingClientRect(),point=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);failureGeometry={node:node.id||node.className,rect:rect(node),frame:rect(get('slider-content-frame')),hit:point?.id||point?.className||point?.tagName,viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};return r.width>0&&r.height>0&&r.left>=frame.left-1&&r.right<=frame.right+1&&r.top>=0&&r.bottom<=innerHeight+1&&(point===node||node.contains(point))});hitFacts[label]={...failureGeometry}};
 const snapshot=()=>({state:{...result().dataset},range:range().value,note:note().value,saved:{setpoint:field('setpoint'),note:field('note')},focus:active()?.id});

 await wait('actual composed gallery and legacy plain Slider mounted',()=>range()&&note()&&idle()&&rated()&&form()&&result()&&plain());await wait('native document focused',()=>document.hasFocus());
 const originalRange=range(),originalNote=note(),plainEndpoints=get('slider-content-plain').querySelector('.ix-slider-endpoints');
 assert(range().form===form()&&range().name==='setpoint'&&range().min==='0'&&range().max==='100'&&range().step==='5'&&range().value==='25'&&range().getAttribute('aria-label')==='Fan setpoint (%)'&&note().form===form()&&note().name==='note'&&note().value==='Prepare fan inspection'&&saves()===0&&get('slider-limits-empty'),'actual initial draft, native form/range semantics and empty saved state');
 assert(endpoint().contains(idle())&&endpoint().contains(rated())&&idle().type==='button'&&rated().type==='button'&&endpoint().textContent.includes('0% idle')&&endpoint().textContent.includes('100% rated')&&!endpoint().textContent.includes('Idle setpoint')&&!endpoint().textContent.includes('Rated setpoint'),'actual endpoint composition takes precedence without accidental submit controls');
 assert(plain().value==='25'&&plain().step==='1'&&plainEndpoints.children.length===2&&[...plainEndpoints.children].every(node=>node.children.length===1&&node.firstElementChild.tagName==='SPAN')&&plainEndpoints.children[0].textContent==='0'&&plainEndpoints.children[1].textContent==='100','side-by-side legacy string endpoints preserve span anatomy');await input(range(),55,'actual native range synthetic input');await wait('controlled range request reaches actual draft without Save',()=>result().dataset.value==='55'&&range().getAttribute('aria-valuenow')==='55'&&saves()===0);snapshots.nativeRange=snapshot();

 await input(note(),'Retain fan preparation note','retained composed form note');await focus(idle(),'native Set idle endpoint');idle().click();await wait('Set idle changes actual controlled setpoint to0 without submission',()=>range().value==='0'&&result().dataset.value==='0'&&saves()===0&&note().value==='Retain fan preparation note');await focus(rated(),'native Set rated endpoint');rated().click();await wait('Set rated changes actual controlled setpoint to100 without submission',()=>range().value==='100'&&result().dataset.value==='100'&&saves()===0&&note().value==='Retain fan preparation note');assert(range()===originalRange&&note()===originalNote,'endpoint actions preserve actual native range/note ownership');snapshots.endpointActions=snapshot();

 await input(range(),75,'native range setpoint before actual form Save');await wait('actual controlled75 setpoint',()=>result().dataset.value==='75');const payload=new FormData(form());assert(payload.getAll('setpoint').length===1&&payload.get('setpoint')==='75'&&payload.get('note')==='Retain fan preparation note','actual successful controls provide one exact native setpoint and note');await focus(save(),'actual native form Save');save().click();await wait('native FormEvent commits exact submitted setpoint/note',()=>saves()===1&&field('setpoint')==='75'&&field('note')==='Retain fan preparation note');await focus(reset(),'actual native form Reset');reset().click();await wait('native Reset restores controlled initial draft while preserving saved record',()=>range().value==='25'&&result().dataset.value==='25'&&note().value==='Prepare fan inspection'&&Number(result().dataset.resets)===1&&saves()===1&&field('setpoint')==='75'&&field('note')==='Retain fan preparation note');assert(range()===originalRange&&note()===originalNote,'native Reset preserves current mounted controls');snapshots.savedAndReset=snapshot();

 await disabled(true);assert(range().disabled&&note().disabled&&idle().disabled&&rated().disabled&&save().disabled&&!reset().disabled,'consumer disables composed endpoint controls, range/note and native Save while retaining Reset');idle().click();rated().click();save().click();await wait('disabled endpoint/Save attempts preserve actual draft and saved record',()=>range().value==='25'&&note().value==='Prepare fan inspection'&&saves()===1&&field('setpoint')==='75');await disabled(false);get('slider-content-narrow').click();await wait('340px actual preview frame',()=>get('slider-content-frame').getBoundingClientRect().width<=341);await hit(range(),'narrow actual native range');await hit(idle(),'narrow Set idle');idle().click();await wait('narrow idle actual value',()=>range().value==='0');await hit(rated(),'narrow Set rated');rated().click();await wait('narrow rated actual value',()=>range().value==='100');await hit(note(),'narrow actual note');await input(note(),'Narrow fan inspection','narrow note input');await hit(save(),'narrow actual Save');save().click();await wait('narrow native form saves actual endpoint value and note',()=>saves()===2&&field('setpoint')==='100'&&field('note')==='Narrow fan inspection');await hit(plain(),'narrow legacy plain range');layout={frame:rect(get('slider-content-frame')),composed:rect(get('slider-limits')),legacy:rect(get('slider-content-plain')),endpoints:rect(endpoint()),viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};assert(layout.composed.width<=341&&layout.legacy.width<=341&&layout.pageWidth<=innerWidth+1,'actual composed controls/plain endpoints fit340px without page overflow');snapshots.disabledAndNarrow=snapshot();
 return {ok:true,snapshots,layout,hitFacts,checks:['actual native form/range and composed content precedence beside legacy plain endpoint anatomy','native Set idle/Set rated actions retain note and never submit','actual native FormData/FormEvent saves exact setpoint/note and Reset preserves saved record','disabled consumer controls/Save guards and340px actual range/endpoint/note/Save hits with real saved record'],input:'excluded synthetic native value assignment plus input events, explicit DOM clicks, real native form/reset defaults and actual native focus; no Rust callbacks or fabricated event payloads are invoked. Untrusted range keyboard events cannot prove browser default range stepping.',trustedInput:'lead Chromium supplies trusted native range keys/Tab/pointer and paired visual evidence',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,hitFacts,failureGeometry}}
"#;
