//! Excluded actual-gallery slider content preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/slider_limits.rs"]
mod gallery_limits;

#[component]
fn NestedThemeForm(dark: bool) -> Element {
    let mut note = use_signal(|| "Nested theme draft".to_string());
    let mut saved = use_signal(String::new);
    let mut actions = use_signal(|| 0_usize);
    rsx! {
        IxTheme { class: "slider-theme-nested", scheme: if dark { ColorScheme::Dark } else { ColorScheme::Light }, density: if dark { Density::Normal } else { Density::Compact },
            form { id: "slider-theme-nested-form", onsubmit: move |event: FormEvent| {
                event.prevent_default();
                if let Some(value) = event.values().into_iter().find_map(|(name, value)| match value {
                    dioxus::html::FormValue::Text(value) if name == "theme-note" => Some(value),
                    _ => None,
                }) { saved.set(value); }
            },
                Input { id: "slider-theme-nested-note", label: "Nested theme preparation", name: "theme-note", required: true, value: note(), onvaluechange: move |value| note.set(value) }
                Button { id: "slider-theme-nested-save", button_type: ButtonType::Submit, "Save nested preparation" }
                output { id: "slider-theme-nested-saved", "{saved()}" }
                IxTheme { class: "slider-theme-inner", scheme: ColorScheme::Dark, density: Density::Normal,
                    Input { id: "slider-theme-inner-note", label: "Independent inner appearance", value: "Dark normal density", readonly: true }
                    Button { id: "slider-theme-inner-action", button_type: ButtonType::Button, onclick: move |_| actions += 1, "Inspect inner scope" }
                    output { id: "slider-theme-inner-actions", "{actions()}" }
                }
            }
        }
    }
}

#[component]
fn IndependentThemePeer() -> Element {
    let mut note = use_signal(|| "Independent peer draft".to_string());
    rsx! {
        IxTheme { class: "slider-theme-peer", scheme: ColorScheme::Dark, density: Density::Normal,
            form { id: "slider-theme-peer-form",
                Input { id: "slider-theme-peer-note", label: "Independent peer preparation", name: "peer-note", value: note(), onvaluechange: move |value| note.set(value) }
            }
        }
    }
}

#[component]
pub fn SliderContentPreview() -> Element {
    let mut narrow = use_signal(|| false);
    let mut plain = use_signal(|| 25.0);
    let mut nested_dark = use_signal(|| false);
    let mut peer = use_signal(|| true);
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
            section {
                h2 { "Independent theme scopes" }
                Button { id: "slider-theme-switch", button_type: ButtonType::Button, onclick: move |_| nested_dark.toggle(), "Switch nested scheme and density" }
                Button { id: "slider-theme-peer-toggle", button_type: ButtonType::Button, onclick: move |_| peer.toggle(), "Hide / show independent peer" }
                NestedThemeForm { dark: nested_dark() }
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
        if peer() { div { id: "slider-theme-peer-host", IndependentThemePeer {} } }
    }
}

const STYLES: &str = "body{margin:0}.slider-content-review{box-sizing:border-box;min-height:100vh;padding:24px}.slider-content-review>h1{font-size:28px;margin:0 0 24px}.slider-content-review #slider-content-plain>h2{font-size:22px;margin:0 0 16px}.slider-content-columns{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:32px}.slider-content-columns>section{min-width:0}#slider-content-frame{margin-top:24px}#slider-content-frame[style*=\"340px\"] .slider-content-columns{grid-template-columns:minmax(0,1fr)}@media(max-width:600px){.slider-content-review{padding:16px}}";

// Excluded native replay; native range keyboard defaults are separate trusted-browser evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SLIDER_CONTENT_CHECK: &str = r#"
let snapshots={},hitFacts={},layout={},tooltipFacts={},themeFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('slider-limits-result'),form=()=>get('slider-limits-form'),range=()=>get('slider-limits')?.querySelector('input[type="range"]'),note=()=>get('slider-limits-note'),plain=()=>get('slider-content-plain')?.querySelector('input[type="range"]');
 const idle=()=>get('slider-limits-idle'),rated=()=>get('slider-limits-rated'),save=()=>get('slider-limits-save'),reset=()=>get('slider-limits-reset'),field=name=>get('slider-limits-saved')?.querySelector('[data-field="'+name+'"]')?.textContent,endpoint=()=>get('slider-limits')?.querySelector('.ix-slider-endpoints'),saves=()=>Number(result()?.dataset.saves);
 const sliderField=()=>range()?.closest('.ix-slider-field'),valueTooltip=()=>sliderField()?.querySelector('.ix-slider-value-tooltip'),thumb=()=>sliderField()?.querySelector('.ix-slider-thumb-anchor'),tooltipOpen=()=>!!valueTooltip()&&valueTooltip().matches(':popover-open')&&valueTooltip().dataset.open==='true';
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const input=async(node,value,label)=>{await focus(node,label);Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,String(value));node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' controlled value',()=>node.value===String(value))};
 const disabled=async value=>{const node=get('slider-limits-disabled');node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('actual disabled state '+value,()=>result().dataset.disabled===String(value))};
 const showTooltip=async value=>{const node=get('slider-limits-show-tooltip');node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('actual show-tooltip option '+value,()=>node.checked===value&&(value?!!valueTooltip():!valueTooltip()))};
 const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const hit=async(node,label)=>{await focus(node,label);await wait(label+' actual native hit/containment',()=>{const r=node.getBoundingClientRect(),frame=get('slider-content-frame').getBoundingClientRect(),point=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);failureGeometry={node:node.id||node.className,rect:rect(node),frame:rect(get('slider-content-frame')),hit:point?.id||point?.className||point?.tagName,viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};return r.width>0&&r.height>0&&r.left>=frame.left-1&&r.right<=frame.right+1&&r.top>=0&&r.bottom<=innerHeight+1&&(point===node||node.contains(point))});hitFacts[label]={...failureGeometry}};
 const checkValueTooltip=async(value,label,clipped=false)=>{await wait(label+' current shared native value tooltip is placed near moving thumb',()=>{const panel=valueTooltip(),anchor=thumb();if(!panel||!anchor)return false;const p=panel.getBoundingClientRect(),a=anchor.getBoundingClientRect(),control=range().closest('.ix-slider-control').getBoundingClientRect();tooltipFacts[label]={value:panel.querySelector('.ix-tooltip-content')?.textContent.trim(),panel:rect(panel),anchor:rect(anchor),control:rect(range().closest('.ix-slider-control')),open:tooltipOpen(),active:active()?.id,viewport:{width:innerWidth,height:innerHeight}};return tooltipOpen()&&panel.dataset.positioned==='true'&&panel.querySelector('.ix-tooltip-content')?.textContent.trim()===String(value)&&p.width>0&&p.height>0&&p.left>=-1&&p.right<=innerWidth+1&&p.top>=-1&&p.bottom<=innerHeight+1&&Math.abs((p.left+p.right)/2-(a.left+a.right)/2)<=Math.max(16,p.width/2)&&Math.min(Math.abs(p.bottom-a.top),Math.abs(p.top-a.bottom))<=32&&(!clipped||p.top<control.top)});assert(active()===range(),'value tooltip preserves actual native range focus');assert(valueTooltip().getAttribute('role')==='tooltip'&&valueTooltip().getAttribute('data-anchor')===thumb().id&&(range().getAttribute('aria-describedby')||'').split(/\s+/).includes(valueTooltip().id)&&thumb().tabIndex<0,'value popup uses shared tooltip semantics and explicit nonfocusable thumb anchor')};
 const snapshot=()=>({state:{...result().dataset},range:range().value,note:note().value,saved:{setpoint:field('setpoint'),note:field('note')},focus:active()?.id});

 await wait('actual composed gallery and legacy plain Slider mounted',()=>range()&&note()&&idle()&&rated()&&form()&&result()&&plain());await wait('native document focused',()=>document.hasFocus());
 const originalRange=range(),originalNote=note(),plainEndpoints=get('slider-content-plain').querySelector('.ix-slider-endpoints');
 assert(range().form===form()&&range().name==='setpoint'&&range().min==='0'&&range().max==='100'&&range().step==='5'&&range().value==='25'&&range().getAttribute('aria-label')==='Fan setpoint (%)'&&note().form===form()&&note().name==='note'&&note().value==='Prepare fan inspection'&&saves()===0&&get('slider-limits-empty'),'actual initial draft, native form/range semantics and empty saved state');
 assert(endpoint().contains(idle())&&endpoint().contains(rated())&&idle().type==='button'&&rated().type==='button'&&endpoint().textContent.includes('0% idle')&&endpoint().textContent.includes('100% rated')&&!endpoint().textContent.includes('Idle setpoint')&&!endpoint().textContent.includes('Rated setpoint'),'actual endpoint composition takes precedence without accidental submit controls');
 assert(plain().value==='25'&&plain().step==='1'&&plainEndpoints.children.length===2&&[...plainEndpoints.children].every(node=>node.children.length===1&&node.firstElementChild.tagName==='SPAN')&&plainEndpoints.children[0].textContent==='0'&&plainEndpoints.children[1].textContent==='100','side-by-side legacy string endpoints preserve span anatomy');assert(get('slider-limits-show-tooltip').checked&&valueTooltip()&&!tooltipOpen(),'default value tooltip stays closed before interaction');const clippingControl=range().closest('.ix-slider-control'),priorOverflow=clippingControl.style.overflow;clippingControl.style.overflow='hidden';await focus(range(),'initial native range tooltip');await checkValueTooltip(25,'initial focused25 escapes consumer clipping',true);const initialThumbX=thumb().getBoundingClientRect().left;await input(range(),55,'actual native range synthetic input');await wait('controlled range request reaches actual draft without Save',()=>result().dataset.value==='55'&&range().getAttribute('aria-valuenow')==='55'&&saves()===0);await checkValueTooltip(55,'controlled55 tooltip follows moving thumb',true);assert(thumb().getBoundingClientRect().left>initialThumbX+1,'controlled range update moves actual authored thumb anchor');clippingControl.style.overflow=priorOverflow;key(range(),'Escape');await wait('Escape dismisses value tooltip without changing value or native focus',()=>!tooltipOpen()&&active()===range()&&range().value==='55');await focus(note(),'blur tooltip after Escape');await focus(range(),'reopen value tooltip');await checkValueTooltip(55,'reopened55 native range tooltip');await showTooltip(false);assert(!valueTooltip()&&active()===range()&&range().value==='55'&&saves()===0,'show-tooltip false removes value popup without stealing range focus or submitting');await showTooltip(true);await focus(note(),'focus away before tooltip re-enabled');await focus(range(),'focus range after tooltip re-enabled');await checkValueTooltip(55,'re-enabled55 native range tooltip');await disabled(true);await wait('disabling focused range removes value tooltip',()=>!valueTooltip()&&range().disabled&&!(range().getAttribute('aria-describedby')||'').split(/\s+/).some(id=>id.endsWith('-value-tooltip')));await disabled(false);await focus(note(),'actual blur target');await wait('native blur leaves value tooltip closed',()=>!tooltipOpen()&&active()===note()&&range().value==='55');snapshots.nativeRange=snapshot();

 await input(note(),'Retain fan preparation note','retained composed form note');await focus(idle(),'native Set idle endpoint');idle().click();await wait('Set idle changes actual controlled setpoint to0 without submission',()=>range().value==='0'&&result().dataset.value==='0'&&saves()===0&&note().value==='Retain fan preparation note');await focus(rated(),'native Set rated endpoint');rated().click();await wait('Set rated changes actual controlled setpoint to100 without submission',()=>range().value==='100'&&result().dataset.value==='100'&&saves()===0&&note().value==='Retain fan preparation note');assert(range()===originalRange&&note()===originalNote,'endpoint actions preserve actual native range/note ownership');snapshots.endpointActions=snapshot();

 await input(range(),75,'native range setpoint before actual form Save');await wait('actual controlled75 setpoint',()=>result().dataset.value==='75');const payload=new FormData(form());assert(payload.getAll('setpoint').length===1&&payload.get('setpoint')==='75'&&payload.get('note')==='Retain fan preparation note','actual successful controls provide one exact native setpoint and note');await focus(save(),'actual native form Save');save().click();await wait('native FormEvent commits exact submitted setpoint/note',()=>saves()===1&&field('setpoint')==='75'&&field('note')==='Retain fan preparation note');await focus(reset(),'actual native form Reset');reset().click();await wait('native Reset restores controlled initial draft while preserving saved record',()=>range().value==='25'&&result().dataset.value==='25'&&note().value==='Prepare fan inspection'&&Number(result().dataset.resets)===1&&saves()===1&&field('setpoint')==='75'&&field('note')==='Retain fan preparation note');assert(range()===originalRange&&note()===originalNote,'native Reset preserves current mounted controls');snapshots.savedAndReset=snapshot();

 await disabled(true);assert(range().disabled&&note().disabled&&idle().disabled&&rated().disabled&&save().disabled&&!reset().disabled,'consumer disables composed endpoint controls, range/note and native Save while retaining Reset');idle().click();rated().click();save().click();await wait('disabled endpoint/Save attempts preserve actual draft and saved record',()=>range().value==='25'&&note().value==='Prepare fan inspection'&&saves()===1&&field('setpoint')==='75');await disabled(false);get('slider-content-narrow').click();await wait('340px actual preview frame',()=>get('slider-content-frame').getBoundingClientRect().width<=341);await hit(range(),'narrow actual native range');await checkValueTooltip(25,'narrow focused25 contained tooltip');await hit(idle(),'narrow Set idle');idle().click();await wait('narrow idle actual value',()=>range().value==='0');await hit(rated(),'narrow Set rated');rated().click();await wait('narrow rated actual value',()=>range().value==='100');await hit(note(),'narrow actual note');await input(note(),'Narrow fan inspection','narrow note input');await hit(save(),'narrow actual Save');save().click();await wait('narrow native form saves actual endpoint value and note',()=>saves()===2&&field('setpoint')==='100'&&field('note')==='Narrow fan inspection');await hit(plain(),'narrow legacy plain range');layout={frame:rect(get('slider-content-frame')),composed:rect(get('slider-limits')),legacy:rect(get('slider-content-plain')),endpoints:rect(endpoint()),viewport:{width:innerWidth,height:innerHeight},pageWidth:document.documentElement.scrollWidth};assert(layout.composed.width<=341&&layout.legacy.width<=341&&layout.pageWidth<=innerWidth+1,'actual composed controls/plain endpoints fit340px without page overflow');snapshots.disabledAndNarrow=snapshot();
 const nested=()=>document.querySelector('.slider-theme-nested'),inner=()=>document.querySelector('.slider-theme-inner'),peerRoot=()=>document.querySelector('.slider-theme-peer');
 const styleBundles=()=>[...document.querySelectorAll('style')].filter(node=>node.textContent.trimStart().startsWith('.ix-theme {\n --ix-font:')).map(base=>{const styles=[];for(let node=base;node&&node.tagName==='STYLE'&&styles.length<34;node=node.nextElementSibling)styles.push(node.textContent);return styles});
 const sameBundle=(a,b)=>a.length===34&&b.length===34&&a.every((text,index)=>text===b[index]);
 const styleSummary=()=>styleBundles().map(styles=>({count:styles.length,characters:styles.reduce((sum,text)=>sum+text.length,0),sheets:styles.map(text=>{let hash=2166136261;for(let index=0;index<text.length;index++)hash=Math.imul(hash^text.charCodeAt(index),16777619);return {characters:text.length,hash:(hash>>>0).toString(16)}})}));
 const themeSnapshot=()=>({nested:{scheme:nested()?.dataset.ixColorSchema,density:nested()?.dataset.ixDensity,color:getComputedStyle(nested()).color,background:getComputedStyle(nested()).backgroundColor,controlHeight:getComputedStyle(nested()).getPropertyValue('--ix-control-height').trim(),buttonHeight:get('slider-theme-nested-save').getBoundingClientRect().height,inputHeight:get('slider-theme-nested-note').getBoundingClientRect().height},inner:{scheme:inner()?.dataset.ixColorSchema,density:inner()?.dataset.ixDensity,color:getComputedStyle(inner()).color,background:getComputedStyle(inner()).backgroundColor,controlHeight:getComputedStyle(inner()).getPropertyValue('--ix-control-height').trim(),buttonHeight:get('slider-theme-inner-action').getBoundingClientRect().height,inputHeight:get('slider-theme-inner-note').getBoundingClientRect().height},bundles:styleSummary()});
 await wait('nested and independent peer theme fixture mounted',()=>nested()&&inner()&&peerRoot()&&get('slider-theme-nested-note')&&get('slider-theme-peer-note'));
 const originalBundle=styleBundles()[0],nestedInput=get('slider-theme-nested-note'),innerInput=get('slider-theme-inner-note'),peerInput=get('slider-theme-peer-note');
 assert(styleBundles().length===2&&styleBundles().every(styles=>sameBundle(styles,originalBundle))&&!nested().querySelector('style')&&!inner().querySelector('style')&&get('slider-theme-peer-host').querySelectorAll('style').length===34,'one exact34-sheet bundle per independent root with no nested duplicates');
 themeFacts.initial=themeSnapshot();assert(themeFacts.initial.nested.scheme==='light'&&themeFacts.initial.nested.density==='compact'&&themeFacts.initial.inner.scheme==='dark'&&themeFacts.initial.inner.density==='normal'&&themeFacts.initial.nested.color!==themeFacts.initial.inner.color&&themeFacts.initial.nested.background!==themeFacts.initial.inner.background&&themeFacts.initial.nested.controlHeight==='32px'&&themeFacts.initial.inner.controlHeight==='40px'&&themeFacts.initial.nested.buttonHeight<themeFacts.initial.inner.buttonHeight&&themeFacts.initial.nested.inputHeight<themeFacts.initial.inner.inputHeight,'nested light/compact and inner dark/normal scopes apply independent real color and density tokens');
 await input(nestedInput,'Retain nested theme preparation','actual nested native preparation');await focus(get('slider-theme-switch'),'nested scheme and density switch');get('slider-theme-switch').click();await wait('nested theme updates without remounting native fields or child draft',()=>nested().dataset.ixColorSchema==='dark'&&nested().dataset.ixDensity==='normal'&&get('slider-theme-nested-note')===nestedInput&&get('slider-theme-inner-note')===innerInput&&nestedInput.value==='Retain nested theme preparation'&&active()===get('slider-theme-switch'));
 themeFacts.updated=themeSnapshot();assert(themeFacts.updated.nested.color===themeFacts.updated.inner.color&&themeFacts.updated.nested.background===themeFacts.updated.inner.background&&themeFacts.updated.nested.controlHeight==='40px'&&themeFacts.updated.nested.buttonHeight===themeFacts.updated.inner.buttonHeight&&themeFacts.updated.nested.inputHeight===themeFacts.updated.inner.inputHeight&&styleBundles().length===2&&styleBundles().every(styles=>sameBundle(styles,originalBundle)),'scheme/density rerender preserves bundle order/text and independent inner styling');
 await focus(get('slider-theme-nested-save'),'actual nested native form Save');get('slider-theme-nested-save').click();await wait('nested native form submits retained current value',()=>get('slider-theme-nested-saved').textContent==='Retain nested theme preparation'&&new FormData(get('slider-theme-nested-form')).get('theme-note')==='Retain nested theme preparation');get('slider-theme-inner-action').click();await wait('nested independent button still acts without submitting',()=>get('slider-theme-inner-actions').textContent==='1'&&get('slider-theme-nested-saved').textContent==='Retain nested theme preparation');
 await input(peerInput,'Independent retained peer note','actual peer native preparation');assert(new FormData(get('slider-theme-peer-form')).get('peer-note')==='Independent retained peer note','peer native form owns its own current preparation');await focus(get('slider-theme-peer-toggle'),'independent peer lifetime toggle');get('slider-theme-peer-toggle').click();await wait('peer removal retires only its own theme bundle and fields',()=>!peerRoot()&&!get('slider-theme-peer-host')&&!peerInput.isConnected&&styleBundles().length===1);assert(sameBundle(styleBundles()[0],originalBundle)&&get('slider-theme-nested-note')===nestedInput&&nestedInput.value==='Retain nested theme preparation'&&range()===originalRange&&note()===originalNote&&range().value==='100'&&note().value==='Narrow fan inspection'&&saves()===2&&field('setpoint')==='100'&&field('note')==='Narrow fan inspection','retained root bundle, native drafts and exact actual Slider saved record survive peer removal');themeFacts.peerRemoved=themeSnapshot();get('slider-theme-peer-toggle').click();await wait('fresh independent peer owns fresh exact stylesheet bundle',()=>peerRoot()&&get('slider-theme-peer-note')!==peerInput&&get('slider-theme-peer-note').value==='Independent peer draft'&&styleBundles().length===2);assert(styleBundles().every(styles=>sameBundle(styles,originalBundle))&&get('slider-theme-nested-note')===nestedInput&&get('slider-theme-nested-saved').textContent==='Retain nested theme preparation'&&getComputedStyle(peerRoot()).color===getComputedStyle(inner()).color&&getComputedStyle(peerRoot()).getPropertyValue('--ix-control-height').trim()==='40px','peer remount restores standalone styling without replacing retained siblings or native form result');themeFacts.peerRestored=themeSnapshot();
 return {ok:true,snapshots,layout,hitFacts,tooltipFacts,themeFacts,checks:['independent roots own exact34-sheet bundles while nested themes reuse ancestor styles; native fields retain drafts/forms through scheme/density updates and peer lifetime','shared native value tooltip follows actual controlled thumb, escapes consumer clipping and fits viewport without moving focus','Escape/blur/disabled/show-tooltip option remove value presentation while retaining setpoint/form drafts','actual native form/range and composed content precedence beside legacy plain endpoint anatomy','native Set idle/Set rated actions retain note and never submit','actual native FormData/FormEvent saves exact setpoint/note and Reset preserves saved record','disabled consumer controls/Save guards and340px actual range/endpoint/note/Save hits with real saved record'],input:'excluded synthetic native value assignment plus input events, explicit DOM clicks, real native form/reset defaults and actual native focus; no Rust callbacks or fabricated event payloads are invoked. Untrusted range keyboard events cannot prove browser default range stepping.',trustedInput:'not established by this excluded synthetic replay; any current trusted browser evidence is recorded separately by the lead',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,hitFacts,tooltipFacts,themeFacts,failureGeometry}}
"#;
