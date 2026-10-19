//! Excluded preview and bounded Linux driver of the gallery managed Toast workflows.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../../examples/gallery/toast_review.rs"]
mod gallery_toast;

#[component]
pub fn ToastLifetimePreview() -> Element {
    let mut owner_mounted = use_signal(|| true);
    let mut owner_epoch = use_signal(|| 0_u64);
    // Force only the shipped reduced-motion CSS for a qualified test run; this
    // does not change system preferences or emulate the OS media-query channel.
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let force_reduced_motion = std::env::var_os("DIOXUS_IX_PROBE_REDUCED_MOTION_CSS").is_some();
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let force_reduced_motion = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_toast::STYLES} }
        IxTheme { class: "toast-lifetime-review",
            h1 { "Managed toast lifetime" }
            p { "Queue timed or persistent notifications, pause or resume the latest countdown, or unmount the notification owner scope to verify teardown." }
            div { class: "toast-lifetime-owner-controls",
                Button {
                    id: "toast-owner-toggle",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        if owner_mounted() {
                            owner_mounted.set(false);
                        } else {
                            owner_epoch += 1;
                            owner_mounted.set(true);
                        }
                    },
                    if owner_mounted() {
                        "Unmount notification scope"
                    } else {
                        "Remount notification scope"
                    }
                }
            }
            if owner_mounted() {
                div { key: "toast-owner-{owner_epoch()}",
                    gallery_toast::ToastLifetimeExample {
                        lifetime: Duration::from_millis(650),
                    }
                }
            } else {
                p { id: "toast-owner-unmounted", role: "status",
                    "Notification scope unmounted."
                }
            }
            label { r#for: "toast-lifetime-outside", "Outside focus target" }
            input { id: "toast-lifetime-outside", value: "Station monitor" }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_TOAST_LIFETIME_CHECK);
                        if let Err(error) = query.send("toastlifetime") {
                            eprintln!("Toast lifetime native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native Toast lifetime: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native Toast lifetime failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(Duration::from_secs(30)).await;
                        eprintln!("Dioxus native Toast lifetime timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
        if force_reduced_motion {
            style { id: "toast-forced-reduced-motion", {REDUCED_MOTION_STYLES} }
        }
    }
}

const REDUCED_MOTION_STYLES: &str = ".ix-theme .ix-managed-toast{animation-duration:1ms}.ix-theme .ix-toast-expiry{visibility:hidden}";

const STYLES: &str = "body{margin:0}.toast-lifetime-review{box-sizing:border-box;min-height:100vh;padding:24px}.toast-lifetime-review h1{font-size:28px;margin:0 0 16px}.toast-lifetime-review h2{font-size:20px;margin:24px 0 12px}.toast-lifetime-review p{max-width:68ch;line-height:1.5}.toast-lifetime-review .demo-row{display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin:12px 0}.toast-lifetime-review .demo-label{min-width:130px;font-size:14px;color:var(--ix-color-text-secondary)}.toast-lifetime-owner-controls{display:flex;flex-wrap:wrap;gap:8px;margin:16px 0}.toast-lifetime-review label{display:block;margin:24px 0 8px}.toast-lifetime-review>input{box-sizing:border-box;max-width:100%;width:320px;min-height:32px;padding:6px 8px;color:var(--ix-color-text);background:var(--ix-color-surface);border:1px solid var(--ix-color-border);font:inherit}@media(max-width:600px){.toast-lifetime-review{padding:16px}}";

// Real CSS animation expiry drives auto-close; animationend is never synthesized.
// Synthetic clicks, pointer enter/leave and focus calls are native observations;
// trusted keyboard/pointer behavior is verified separately in Chromium.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TOAST_LIFETIME_CHECK: &str = r#"
let inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const section=()=>get('toast-lifetime'),status=()=>get('toast-lifetime-status');
 const container=()=>section()?.querySelector('.ix-toast-container')||document.querySelector('.ix-toast-container');
 const managedToasts=()=>[...(container()?.querySelectorAll('.ix-managed-toast')??[])];
 const latestManaged=()=>managedToasts().at(-1);
 const toastSurface=node=>node?.querySelector('.ix-toast');
 const expiryBar=node=>node?.querySelector('.ix-toast-expiry');
 const playState=node=>{const bar=expiryBar(node);return bar?getComputedStyle(bar).animationPlayState:null;};
 const closeButton=node=>node?.querySelector('button.ix-message-close[aria-label="Dismiss notification"]')||node?.querySelector('button[aria-label="Dismiss notification"]');
 const actionButton=node=>[...(node?.querySelectorAll('.ix-toast-action button,button')??[])].find(button=>button.textContent?.trim()==='Acknowledge');
 const pointerEnter=node=>node.dispatchEvent(new PointerEvent('pointerenter',{bubbles:false,composed:true}));
 const pointerLeave=node=>node.dispatchEvent(new PointerEvent('pointerleave',{bubbles:false,composed:true}));
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label+'; open='+status()?.dataset.open+'; closed='+status()?.dataset.closed+'; last='+status()?.dataset.last+'; latest='+status()?.dataset.latest);};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<160;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===2)return;
   await hold(20);
  }
  throw Error(label+'; focus='+active()?.id+'; open='+status()?.dataset.open+'; closed='+status()?.dataset.closed+'; last='+status()?.dataset.last+'; latest='+status()?.dataset.latest+'; count='+managedToasts().length);
 };

 await wait('gallery toast fixture mounted',()=>section()&&status()&&get('toast-show-timed')&&get('toast-show-persistent')&&get('toast-pause-latest')&&get('toast-resume-latest')&&get('toast-close-latest')&&get('toast-position-top')&&get('toast-position-bottom')&&get('toast-owner-toggle')&&get('toast-lifetime-outside')&&status().dataset.open==='0'&&status().dataset.closed==='0'&&status().dataset.last==='none'&&status().dataset.latest==='none'&&container()?.dataset.position==='bottom-right');
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 const outside=get('toast-lifetime-outside');
 outside.focus();
 await wait('initial outside focus established',()=>active()===outside);
 inputFocusFacts={documentFocused:document.hasFocus(),active:active()?.id};

 // 1. Actual CSS auto-expiry and single close callback.
 get('toast-show-timed').click();
 await wait('timed toast mounted and running',()=>managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.closed==='0'&&status().dataset.latest==='running'&&toastSurface(latestManaged())?.dataset.type==='success'&&latestManaged()?.textContent?.includes('Configuration saved')&&playState(latestManaged())==='running');
 const reducedMotionCssForced=!!get('toast-forced-reduced-motion');
 if(reducedMotionCssForced)assert(getComputedStyle(expiryBar(latestManaged())).visibility==='hidden'&&getComputedStyle(latestManaged()).animationDuration==='0.001s','shipped reduced-motion CSS is active while countdown still runs');
 await wait('timed toast expires via real CSS animation',()=>managedToasts().length===0&&status().dataset.open==='0'&&status().dataset.closed==='1'&&status().dataset.last==='expired'&&status().dataset.latest==='closed');
 await hold(250);
 assert(status().dataset.closed==='1'&&status().dataset.open==='0','expired toast callback must fire once');

 // 2 & 3. Explicit pause held beyond duration, overlapping hover + explicit resume (hover wins), then resume to expiry.
 get('toast-show-timed').click();
 await wait('second timed toast running',()=>managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.latest==='running'&&playState(latestManaged())==='running');
 get('toast-pause-latest').click();
 await wait('explicit pause suspends expiry animation',()=>status().dataset.latest==='paused'&&playState(latestManaged())==='paused');
 await hold(900);
 assert(managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.closed==='1'&&status().dataset.latest==='paused'&&latestManaged()?.dataset.closing==='false','explicitly paused toast must survive beyond duration');

 pointerEnter(latestManaged());
 get('toast-resume-latest').click();
 await wait('hover keeps toast paused after explicit resume',()=>status().dataset.latest==='paused'&&playState(latestManaged())==='paused');
 await hold(800);
 assert(managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.closed==='1'&&status().dataset.latest==='paused'&&latestManaged()?.dataset.closing==='false','hovered toast must remain paused beyond duration after explicit resume');

 pointerLeave(latestManaged());
 await wait('pointer leave after explicit resume resumes countdown',()=>status().dataset.latest==='running'&&playState(latestManaged())==='running');
 await wait('resumed toast expires via real CSS animation',()=>managedToasts().length===0&&status().dataset.open==='0'&&status().dataset.closed==='2'&&status().dataset.last==='expired'&&status().dataset.latest==='closed');

 // 4. Focused close button pauses a timed notification; outside focus resumes expiry.
 outside.focus();
 await wait('outside focus before focus-pause check',()=>active()===outside);
 get('toast-show-timed').click();
 await wait('third timed toast running',()=>managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.latest==='running'&&closeButton(latestManaged()));
 closeButton(latestManaged()).focus();
 await wait('focusing native close button pauses countdown',()=>active()===closeButton(latestManaged())&&status().dataset.latest==='paused'&&playState(latestManaged())==='paused');
 await hold(900);
 assert(managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.closed==='2'&&status().dataset.latest==='paused'&&latestManaged()?.dataset.closing==='false','focused toast must survive beyond duration');
 outside.focus();
 await wait('moving focus outside resumes countdown',()=>active()===outside&&status().dataset.latest==='running'&&playState(latestManaged())==='running');
 await wait('focus-resumed toast expires via real CSS animation',()=>managedToasts().length===0&&status().dataset.open==='0'&&status().dataset.closed==='3'&&status().dataset.last==='expired'&&status().dataset.latest==='closed');

 // 5. Persistent toast has no progress bar, survives beyond duration, and dismisses via native close button.
 get('toast-show-persistent').click();
 await wait('persistent warning toast mounted without expiry bar',()=>managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.latest==='running'&&toastSurface(latestManaged())?.dataset.type==='warning'&&latestManaged()?.textContent?.includes('Connection review')&&!expiryBar(latestManaged())&&closeButton(latestManaged()));
 await hold(900);
 assert(managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.closed==='3','persistent toast must survive beyond timed duration');
 closeButton(latestManaged()).click();
 await wait('persistent toast dismissed by native close button',()=>managedToasts().length===0&&status().dataset.open==='0'&&status().dataset.closed==='4'&&status().dataset.last==='dismissed'&&status().dataset.latest==='closed');

 // 6. A persistent action receives focus and closes with its own result.
 get('toast-show-persistent').click();
 await wait('persistent toast with Acknowledge action mounted',()=>managedToasts().length===1&&status().dataset.open==='1'&&actionButton(latestManaged()));
 actionButton(latestManaged()).focus();
 await wait('focused persistent action reports paused',()=>active()===actionButton(latestManaged())&&status().dataset.latest==='paused');
 actionButton(latestManaged()).click();
 await wait('Acknowledge action closes with programmatic:acknowledged',()=>managedToasts().length===0&&status().dataset.open==='0'&&status().dataset.closed==='5'&&status().dataset.last==='programmatic:acknowledged'&&status().dataset.latest==='closed');

 // 7 & 8. Top/bottom placement, two stacked entries, and closing latest preserves sibling.
 get('toast-position-top').click();
 await wait('container switches to top-right',()=>container()?.dataset.position==='top-right'&&container().getBoundingClientRect().top<=32);
 get('toast-show-persistent').click();
 await wait('first stacked persistent toast open',()=>managedToasts().length===1&&status().dataset.open==='1');
 get('toast-show-timed').click();
 await wait('second stacked timed toast open',()=>managedToasts().length===2&&status().dataset.open==='2'&&managedToasts()[0]?.textContent?.includes('Connection review')&&managedToasts()[1]?.textContent?.includes('Configuration saved')&&managedToasts()[1].getBoundingClientRect().top>managedToasts()[0].getBoundingClientRect().top);
 get('toast-close-latest').click();
 await wait('closing latest removes only second entry',()=>managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.closed==='6'&&status().dataset.last==='programmatic:closed'&&status().dataset.latest==='closed'&&managedToasts()[0]?.textContent?.includes('Connection review'));
 await hold(900);
 assert(managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.closed==='6'&&managedToasts()[0]?.textContent?.includes('Connection review'),'closed latest toast must not emit stale expiry or remove surviving sibling');
 get('toast-position-bottom').click();
 await wait('container returns to bottom-right',()=>container()?.dataset.position==='bottom-right'&&window.innerHeight-container().getBoundingClientRect().bottom<=32);
 closeButton(managedToasts()[0]).click();
 await wait('surviving sibling dismissed cleanly',()=>managedToasts().length===0&&status().dataset.open==='0'&&status().dataset.closed==='7'&&status().dataset.last==='dismissed');

 // 9. Unmount and remount owner scope via Dioxus parent control without stale callbacks.
 get('toast-show-timed').click();
 await wait('timed toast open before owner unmount',()=>managedToasts().length===1&&status().dataset.open==='1'&&status().dataset.latest==='running');
 get('toast-owner-toggle').click();
 await wait('owner scope unmounted by Dioxus parent toggle',()=>!section()&&!status()&&get('toast-owner-unmounted')&&document.querySelectorAll('.ix-toast-container,.ix-managed-toast').length===0);
 get('toast-owner-toggle').click();
 await wait('fresh keyed owner scope remounted',()=>section()&&status()&&status().dataset.open==='0'&&status().dataset.closed==='0'&&status().dataset.last==='none'&&status().dataset.latest==='none'&&managedToasts().length===0);
 await hold(950);
 assert(status().dataset.open==='0'&&status().dataset.closed==='0'&&status().dataset.last==='none'&&status().dataset.latest==='none'&&managedToasts().length===0,'unmounted scope toast must not fire stale callback after remount');

 return {
  ok:true,
  checks:[
   'real CSS auto-expiry and single callback',
   'explicit pause held beyond duration',
   'overlapping hover wins over explicit resume and expires after pointer leave',
   'focused timed dismiss control pauses beyond duration and outside focus resumes',
   'persistent toast without progress bar survives duration and dismisses natively',
   'persistent Acknowledge focus and payload programmatic:acknowledged',
   'top-right and bottom-right container placement',
   'two stacked entries and close-latest preserves sibling',
   'Dioxus owner scope unmount/remount without stale callbacks'
  ],
  inputFocusFacts,
  reducedMotionCssForced,osPrefersReducedMotion:matchMedia('(prefers-reduced-motion: reduce)').matches,
  input:'synthetic DOM clicks/pointer/focus with real CSS animation expiry',
  trustedInput:'separate Chromium keyboard/pointer verification',
  renderer:navigator.userAgent
 };
} catch(error) {return {ok:false,error:String(error),inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
