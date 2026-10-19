//! Linux/WebView B18.1 replay. DOM stimuli are synthetic; window.open is a
//! non-forwarding spy. Trusted defaults and real destinations require browser QA.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
// Destination routing/styles are browser-only; this probe reuses the controls.
#[allow(dead_code)]
#[path = "../examples/gallery/header_switch_review.rs"]
mod header_switch_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX header and application-switch probe")
                .with_visible(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1100.0, 1100.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const get=id=>document.getElementById('header-review-'+id),stage=()=>get('stage'),header=()=>stage()?.querySelector('header.ix-application-header'),dialog=()=>stage()?.querySelector('dialog.ix-modal');
 const switcher=()=>header()?.querySelector('.ix-header-app-switch button'),entry=name=>[...(dialog()?.querySelectorAll('.ix-application-switch-entry')??[])].find(el=>el.textContent.includes(name));
 const state=()=>JSON.parse(get('state').textContent),events=()=>JSON.parse(get('events').textContent),opens=[],clicks=[];
 const active=()=>document.activeElement?.getAttribute('aria-label')??document.activeElement?.id??document.activeElement?.tagName;
 const snapshot=()=>({state:state(),events:events().slice(-8),opens,focus:active(),dialog:dialog()?{open:dialog().open,text:dialog().textContent}:null});
 const assert=(ok,name)=>{if(!ok)throw Error(name+'; '+JSON.stringify(snapshot()))};
 const wait=async(test,name)=>{const deadline=performance.now()+4500;while(!test()){if(performance.now()>deadline)throw Error(name+'; '+JSON.stringify(snapshot()));await sleep(20)}};
 const choose=(id,value)=>{const el=get(id);el.value=value;el.dispatchEvent(new Event('input',{bubbles:true,cancelable:true}));};
 const command=id=>{get(id).focus();get(id).click()};
 const nativeOpen=window.open;
 window.open=function(...args){opens.push({args,nativeDialogOpen:dialog()?.open??false,connected:dialog()?.isConnected??false,active:active(),userActivation:navigator.userActivation?.isActive??null});return {}};
 document.addEventListener('click',event=>{if(event.target.closest?.('.ix-header-app-switch,.ix-application-switch-entry'))clicks.push({text:event.target.textContent,prevented:event.defaultPrevented,trusted:event.isTrusted})});
 const close=async()=>{const d=dialog();if(!d?.open)return;const event=new Event('cancel',{cancelable:true});d.dispatchEvent(event);assert(event.defaultPrevented,'native cancel prevented by Rust');await wait(()=>!dialog()?.open,'dismissed');};
 const open=async()=>{switcher().focus();switcher().click();await wait(()=>dialog()?.open,'native invoker opens');await wait(()=>dialog()?.querySelector('a.ix-application-switch-entry,.ix-application-switch-loading'),'Rust entry content ready');};
 const fresh=async()=>{await close();choose('request-policy','allow');command('populated');await wait(()=>switcher(),'configured switch');const previous=header();command('reset');await wait(()=>header()&&header()!==previous&&switcher(),'fresh owner');};
 const arm=async(action,delay=1000)=>{choose('arm-action',action);choose('delay',String(delay));command('arm');await wait(()=>state().armed_action===action,'armed action');};
 const checks=[];
 await wait(()=>header()&&switcher(),'review application');
 assert(header().getAttribute('role')==='banner'&&header().querySelector('h1')?.textContent==='Operations workspace','native heading/banner');
 assert(header().querySelector('img')?.getAttribute('alt')==='Neutral application symbol','native image alt');
 assert(switcher().getAttribute('aria-label')==='Apps','icon fallback native label');
 const retainedHeader=header();choose('name-mode','empty');await wait(()=>!header().querySelector('h1'),'empty name removes heading');assert(header()===retainedHeader,'live name keeps header node');
 choose('name-mode','provided');await wait(()=>header().querySelector('h1'),'restore title');choose('label-mode','provided');await wait(()=>switcher().getAttribute('aria-label')==='Switch review application','custom native label');
 checks.push('native heading/banner/image/label and stable live title');
 choose('request-policy','veto');switcher().click();await wait(()=>events().some(e=>e.event==='request-veto'),'veto request');assert(!dialog()?.open,'veto keeps closed');
 choose('request-policy','remove');switcher().click();await wait(()=>!state().configuration_present&&!switcher(),'request-time removal');assert(!dialog()?.open,'removed config aborts native opening');
 command('populated');await wait(()=>switcher(),'reconfigure');choose('request-policy','replace');await open();await wait(()=>dialog()?.textContent.includes('Loading review applications'),'request-time loading commit');await close();
 checks.push('request veto, retry and synchronous direct-Signal removal/replacement');
 await fresh();command('loading');await arm('populate');await open();assert(dialog().textContent.includes('Loading review applications'),'loading before live timer');await wait(()=>entry('Beta application'),'live population');assert(dialog().querySelectorAll('a.ix-application-switch-entry').length===2,'entries are native anchors');assert([...dialog().querySelectorAll('.ix-application-switch-entry img')].every(img=>img.getAttribute('alt')===''),'decorative images');assert(entry('Beta application').getAttribute('href')==='/?header-review-destination=beta'&&entry('Beta application').getAttribute('target')==='header-review-target','real anchor destination and target');await close();
 await fresh();await arm('remove-config');await open();await wait(()=>!state().configuration_present,'config removed while open');assert(dialog()?.open&&entry('Alpha application'),'last config survives while open');const before=opens.length;entry('Alpha application').click();await wait(()=>opens.length===before+1,'retained config entry navigates');assert(JSON.stringify(opens.at(-1).args)===JSON.stringify(['/?header-review-destination=alpha','header-review-target']),'retained config exact navigation');await wait(()=>!dialog()?.open,'entry dismisses');
 checks.push('live population, real decorative anchors and retained-None navigation');
 for(const target of ['header-review-target','_blank','_self','_parent','_top','']){
  await fresh();choose('target',target);command('populated');await open();const count=opens.length;const original=entry('Beta application');original.click();await wait(()=>opens.length===count+1,'exact target '+target);assert(JSON.stringify(opens.at(-1).args)===JSON.stringify(['/?header-review-destination=beta',target]),'exact two arguments '+target);assert(original.getAttribute('href')==='/?header-review-destination=beta'&&original.getAttribute('target')===target,'anchor exact URL/target '+target);await wait(()=>!dialog()?.open,'dismiss target '+target);
 }
 checks.push('all exact targets including empty; real anchor attributes; nonforwarding desktop operation spy');
 await fresh();await arm('remove-application');await open();await wait(()=>!state().owner_mounted&&!header(),'owner removal');assert(!document.querySelector('#header-review-stage dialog[open]'),'removed owner leaves no modal');command('owner-mount');await wait(()=>header()&&switcher(),'owner restored');await open();await close();await wait(()=>document.activeElement===switcher(),'dismiss returns native switch focus');
 checks.push('owner teardown/remount and native focus restoration');
 window.open=nativeOpen;
 return {ok:true,checks,opens,clicks,input:'synthetic DOM click/input/cancel with native dialog opening/focus and target-aware desktop navigation',limits:['window.open spy does not forward or prove popup success','trusted Enter/link default actions and real navigation are browser evidence','physical/AT/platform/Brand/full-responsive parity unproved'],renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(75)).await;
            eprintln!("B18.1 native header probe timed out");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let desktop=desktop.clone();spawn(async move {
                let query=dom_query::eval(VERIFY);let _=query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"]==true => {println!("B18.1 native header checks: {result}");desktop.close();}
                    result => {eprintln!("B18.1 native header probe failed: {result:?}");std::process::exit(1);}
                }
            });
        }, header_switch_review::HeaderSwitchReview {} }
    } }
}
