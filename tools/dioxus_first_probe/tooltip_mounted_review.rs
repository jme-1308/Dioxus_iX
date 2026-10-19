//! Excluded explicitly registered Tooltip anchors and form-feedback workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn RegisteredAnchor(
    controller: TooltipController,
    id: String,
    label: String,
    wide: bool,
) -> Element {
    let binding = use_tooltip_anchor(controller, id.clone());
    rsx! {button {id,r#type:"button",class:"ix-button ix-button-secondary",style:if wide{"width:240px"}else{"width:150px"},
        "aria-describedby":binding.describedby("r0239-description"),
        onmounted:binding.onmounted,onresize:binding.onresize,onvisible:binding.onvisible,
        onmouseenter:binding.onmouseenter,onmouseleave:binding.onmouseleave,onfocusin:binding.onfocusin,onfocusout:binding.onfocusout,onkeydown:binding.onkeydown,
        "{label}"
    }}
}

#[component]
pub fn TooltipMountedReview() -> Element {
    let shared = use_tooltip("r0239-shared-tooltip");
    let delayed = use_tooltip("r0239-delayed-tooltip");
    let mut wide = use_signal(|| false);
    let mut anchors = use_signal(|| true);
    let mut delayed_anchor = use_signal(|| true);
    let mut delayed_surface = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut actions = use_signal(|| 0_usize);
    let mut errors = use_signal(Vec::<String>::new);
    let mut input = use_signal(|| "Pump".to_string());
    let mut notes = use_signal(|| "Shift notes".to_string());
    let mut choice = use_signal(String::new);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"tooltip-mounted-review",
            h1 {"Tooltips with Dioxus mounted anchors"}
            p {"Hover or focus an anchor, switch between the registered equipment controls, and press Escape to dismiss. Scroll the clipped host or resize an anchor while help is open."}
            p {id:"r0239-description","Existing equipment description remains associated with each custom anchor."}
            div {class:"tooltip-review-controls",
                Tooltip {id:"r0239-wrapper-tooltip",label:"Wrapper help remains available by hover and keyboard focus",placement:TooltipPlacement::Bottom,onnativeerror:move |error|errors.write().push(error),
                    button {id:"r0239-wrapper",r#type:"button",class:"ix-button ix-button-secondary","Wrapper tooltip"}
                }
                button {id:"r0239-resize",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|wide.toggle(),"Resize first anchor"}
                button {id:"r0239-anchors",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|anchors.toggle(),"Hide / show shared anchors"}
                button {id:"r0239-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|epoch+=1,"Remount shared anchors"}
            }
            section {
                h2 {"Shared interactive help inside clipping"}
                div {id:"r0239-clip",class:"tooltip-review-clip",
                    div {class:"tooltip-review-rail",
                        if anchors() {
                            div {key:"anchors-{epoch()}",class:"tooltip-review-row",
                                RegisteredAnchor {controller:shared.clone(),id:"r0239-first",label:"First equipment",wide:wide()}
                                RegisteredAnchor {controller:shared.clone(),id:"r0239-second",label:"Second equipment",wide:false}
                            }
                        }
                        p {"The scroll host clips its children; native tooltip presentation escapes that clipping."}
                    }
                    TooltipFor {id:"r0239-shared-tooltip",controller:shared.clone(),interactive:true,placement:TooltipPlacement::Bottom,title:"Equipment details",onnativeerror:move |error|errors.write().push(error),
                        p {"Shared help follows the current registered anchor."}
                        button {id:"r0239-action",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|actions+=1,"Inspect equipment"}
                    }
                }
            }
            section {
                h2 {"Delayed help and independent lifetimes"}
                div {class:"tooltip-review-controls",
                    button {id:"r0239-delayed-anchor-toggle",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|delayed_anchor.toggle(),"Hide / show delayed anchor"}
                    button {id:"r0239-delayed-surface-toggle",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|delayed_surface.toggle(),"Hide / show tooltip only"}
                    if delayed_anchor() {RegisteredAnchor {controller:delayed.clone(),id:"r0239-delayed",label:"Delayed help",wide:false}}
                }
                if delayed_surface() {
                    TooltipFor {id:"r0239-delayed-tooltip",controller:delayed.clone(),show_delay:120,placement:TooltipPlacement::Top,onnativeerror:move |error|errors.write().push(error),"Delayed equipment help"}
                }
            }
            section {class:"tooltip-review-form",
                h2 {"Form feedback tooltips"}
                div {id:"r0239-input-host",Input {id:"r0239-input",label:"Equipment name",value:input(),onvaluechange:move |value|input.set(value),show_text_as_tooltip:true,helper_text:"Input mounted feedback"}}
                div {id:"r0239-textarea-host",Textarea {id:"r0239-textarea",label:"Shift notes",rows:2,value:notes(),onvaluechange:move |value|notes.set(value),show_text_as_tooltip:true,helper_text:"Textarea mounted feedback"}}
                div {id:"r0239-select-host",Select {id:"r0239-select",label:"Equipment type",value:choice(),onvaluechange:move |value|choice.set(value),show_text_as_tooltip:true,helper_text:"Select mounted feedback",options:vec![SelectOption::new("pump","Pump"),SelectOption::new("fan","Fan")]}}
            }
            label {r#for:"r0239-outside","Outside focus destination"}
            input {id:"r0239-outside",class:"tooltip-review-outside",value:"Independent focus"}
            output {id:"r0239-status",role:"status","aria-live":"polite","data-actions":actions().to_string(),"data-errors":errors().len().to_string(),"data-error-messages":serde_json::to_string(&errors()).unwrap_or_default(),"data-epoch":epoch().to_string(),"data-anchors":anchors().to_string(),"data-delayed-anchor":delayed_anchor().to_string(),"data-delayed-surface":delayed_surface().to_string(),"data-wide":wide().to_string(),
                "Actions: {actions()}; anchor generation: {epoch()}; native errors: {errors().len()}"
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_TOOLTIP_MOUNTED_CHECK);let _=task.send("tooltip-mounted");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native mounted Tooltip: {value}");window.close();}
                        other=>{eprintln!("Dioxus native mounted Tooltip failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native mounted Tooltip timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.tooltip-mounted-review{box-sizing:border-box;min-height:100vh;padding:24px}.tooltip-mounted-review h1{font-size:28px;margin:0 0 16px}.tooltip-mounted-review h2{font-size:20px;margin:0 0 16px}.tooltip-mounted-review p{max-width:70ch;line-height:1.5}.tooltip-mounted-review section{max-width:900px;margin:28px 0}.tooltip-review-controls,.tooltip-review-row{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:16px 0}.tooltip-review-clip{box-sizing:border-box;height:120px;overflow:auto;border:1px solid var(--ix-color-border);padding:8px}.tooltip-review-rail{min-height:360px;padding:12px 8px;box-sizing:border-box}.tooltip-review-row{margin:0 0 32px}.tooltip-review-form{max-width:440px!important}.tooltip-review-form>div{margin:16px 0}.tooltip-review-outside{display:block;box-sizing:border-box;width:min(100%,440px);min-height:36px;margin:8px 0 20px;padding:6px 8px;background:var(--ix-color-surface);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.tooltip-mounted-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}@media(max-width:600px){.tooltip-mounted-review{padding:16px}}";

// Synthetic Linux events exercise Rust handlers; actual pointer checks run separately.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TOOLTIP_MOUNTED_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),status=()=>byId('r0239-status'),active=()=>document.activeElement;
const open=id=>Boolean(byId(id)?.matches(':popover-open'));
const ready=id=>{const panel=byId(id);return open(id)&&panel.dataset.positioned==='true'&&getComputedStyle(panel).visibility==='visible'&&!panel.inert;};
const described=(id,tip)=>byId(id)?.getAttribute('aria-describedby')?.split(/\s+/).includes(tip);
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}const panel=byId('r0239-shared-tooltip');throw new Error(label+' active='+active()?.id+' shared='+panel?.dataset.anchor+' inert='+panel?.inert+' attr='+panel?.getAttribute('inert')+' open='+open('r0239-shared-tooltip')+' actions='+status()?.dataset.actions);};
const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
const hover=node=>node.dispatchEvent(new MouseEvent('mouseenter',{bubbles:false}));
const leave=node=>node.dispatchEvent(new MouseEvent('mouseleave',{bubbles:false}));
await wait('mounted tooltip fixture',()=>byId('r0239-first')&&byId('r0239-wrapper'));
hover(byId('r0239-wrapper').closest('.ix-tooltip-anchor'));
await wait('wrapper hover opens',()=>open('r0239-wrapper-tooltip'));
key(byId('r0239-wrapper'),'Escape');await wait('wrapper Escape closes',()=>!open('r0239-wrapper-tooltip'));
leave(byId('r0239-wrapper').closest('.ix-tooltip-anchor'));byId('r0239-outside').focus();byId('r0239-wrapper').focus();
await wait('wrapper focus opens',()=>open('r0239-wrapper-tooltip'));
key(byId('r0239-wrapper'),'Escape');await wait('wrapper focus Escape closes',()=>!open('r0239-wrapper-tooltip'));
byId('r0239-first').focus();
await wait('first anchor shows shared help with preserved description',()=>ready('r0239-shared-tooltip')&&described('r0239-first','r0239-description')&&described('r0239-first','r0239-shared-tooltip'));
const tip=()=>byId('r0239-shared-tooltip'),clip=byId('r0239-clip');
if(tip().getBoundingClientRect().bottom<=clip.getBoundingClientRect().bottom)throw new Error('shared tooltip did not escape clipping');
byId('r0239-second').focus();
await wait('shared help switches second anchor',()=>ready('r0239-shared-tooltip')&&tip().dataset.anchor==='r0239-second'&&described('r0239-second','r0239-shared-tooltip')&&described('r0239-first','r0239-description'));
byId('r0239-action').focus();byId('r0239-action').click();
await wait('interactive help retains focus and action',()=>open('r0239-shared-tooltip')&&active()===byId('r0239-action')&&status().dataset.actions==='1');
key(byId('r0239-action'),'Escape');await wait('interactive Escape closes shared help',()=>!open('r0239-shared-tooltip'));
byId('r0239-first').focus();await wait('first anchor reopened',()=>open('r0239-shared-tooltip'));
const beforeWidth=byId('r0239-first').getBoundingClientRect().width;
byId('r0239-resize').click();
await wait('active anchor resize repositions help',()=>status().dataset.wide==='true'&&byId('r0239-first').getBoundingClientRect().width>beforeWidth+50&&open('r0239-shared-tooltip')&&Math.abs((tip().getBoundingClientRect().left+tip().getBoundingClientRect().width/2)-(byId('r0239-first').getBoundingClientRect().left+byId('r0239-first').getBoundingClientRect().width/2))<32);
clip.scrollTop=20;clip.dispatchEvent(new Event('scroll',{bubbles:true}));
await wait('ancestor scroll keeps help anchored',()=>open('r0239-shared-tooltip')&&Math.abs(tip().getBoundingClientRect().top-byId('r0239-first').getBoundingClientRect().bottom-12)<8);
clip.scrollTop=240;clip.dispatchEvent(new Event('scroll',{bubbles:true}));await wait('clipped anchor closes help',()=>!open('r0239-shared-tooltip'));
clip.scrollTop=0;clip.dispatchEvent(new Event('scroll',{bubbles:true}));byId('r0239-outside').focus();
const retired=byId('r0239-first');byId('r0239-anchors').click();await wait('shared anchors retire',()=>!byId('r0239-first')&&!retired.isConnected&&!open('r0239-shared-tooltip'));
byId('r0239-anchors').click();await wait('shared anchors restored',()=>byId('r0239-first'));
const beforeRemount=byId('r0239-first');byId('r0239-remount').click();await wait('shared anchors remount',()=>status().dataset.epoch==='1'&&byId('r0239-first')!==beforeRemount&&!beforeRemount.isConnected);
byId('r0239-first').focus();await wait('fresh anchor shows shared help',()=>open('r0239-shared-tooltip')&&described('r0239-first','r0239-shared-tooltip'));
key(byId('r0239-first'),'Escape');await wait('fresh shared Escape',()=>!open('r0239-shared-tooltip'));
byId('r0239-outside').focus();hover(byId('r0239-delayed'));byId('r0239-delayed-anchor-toggle').click();
await wait('pending delayed anchor removed',()=>!byId('r0239-delayed'));
await new Promise(resolve=>setTimeout(resolve,250));
if(open('r0239-delayed-tooltip'))throw new Error('retired delayed anchor opened help');
byId('r0239-delayed-anchor-toggle').click();await wait('delayed anchor restored',()=>byId('r0239-delayed'));
byId('r0239-delayed').focus();await wait('delayed focus opens help',()=>open('r0239-delayed-tooltip')&&described('r0239-delayed','r0239-delayed-tooltip'));
byId('r0239-delayed-surface-toggle').click();await wait('tooltip-only removal preserves existing description',()=>!byId('r0239-delayed-tooltip')&&described('r0239-delayed','r0239-description')&&!described('r0239-delayed','r0239-delayed-tooltip'));
byId('r0239-delayed-surface-toggle').click();await wait('tooltip-only surface restored',()=>byId('r0239-delayed-tooltip'));
byId('r0239-outside').focus();
for(const [id,message] of [['r0239-input','Input mounted feedback'],['r0239-textarea','Textarea mounted feedback'],['r0239-select','Select mounted feedback']]){
    byId(id).focus();
    await wait(id+' feedback uses mounted anchor',()=>Array.from(document.querySelectorAll('.ix-tooltip')).some(tip=>tip.matches(':popover-open')&&tip.textContent.includes(message)));
    key(byId(id),'Escape');
    await wait(id+' feedback Escape closes',()=>!Array.from(document.querySelectorAll('.ix-tooltip')).some(tip=>tip.matches(':popover-open')&&tip.textContent.includes(message)));
    byId('r0239-outside').focus();
}
if(status().dataset.errors!=='0')throw new Error('native tooltip activation errors: '+status().dataset.errorMessages);
return {ok:true,syntheticEvents:true,wrapperHoverFocusEscape:true,sharedSwitchAndInteractiveAction:true,clippingScrollAndResize:true,anchorRetirementAndRemount:true,delayedRetirement:true,tooltipOnlyAriaCleanup:true,inputTextareaSelectFeedback:true};
"#;
