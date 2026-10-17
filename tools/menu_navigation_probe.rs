//! Bounded visible Linux/WebView replay of G04.3. Keyboard events are synthetic;
//! native focus and cancellation wiring are measured, not physical OS input.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/menu_navigation_review.rs"]
mod menu_navigation_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX outer menu navigation probe")
                .with_visible(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1000.0, 1000.0)),
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
 const get=id=>document.getElementById('g04-menu-'+id),stage=()=>get('stage'),menu=()=>stage()?.querySelector('nav.ix-menu'),bar=()=>menu()?.querySelector('[role=menubar]');
 const allNodes=()=>[...(bar()?.querySelectorAll('[role=menuitem],[role=menuitemcheckbox]')??[])];
 const nodes=()=>allNodes().filter(node=>!node.closest('[role=menu]'));
 const item=name=>allNodes().find(n=>(n.getAttribute('aria-label')??n.textContent.trim())===name),events=()=>JSON.parse(get('events').textContent),state=()=>JSON.parse(get('state').textContent);
 const selected=()=>events().filter(e=>e.event==='select');
 const current=()=>({tag:document.activeElement?.tagName,label:document.activeElement?.getAttribute('aria-label'),text:document.activeElement?.textContent?.trim().slice(0,80)});
 const snapshot=()=>({focus:current(),state:state(),items:nodes().map(n=>({label:n.getAttribute('aria-label')??n.textContent.trim(),tab:n.tabIndex,pos:n.getAttribute('aria-posinset'),size:n.getAttribute('aria-setsize')})),events:events()});
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; '+JSON.stringify(snapshot()))};
 const wait=async(test,label)=>{const end=performance.now()+3500;while(!test()){if(performance.now()>end)throw Error(label+'; '+JSON.stringify(snapshot()));await sleep(20)}};
 const bubble=[];document.addEventListener('keydown',event=>bubble.push({key:event.key,prevented:event.defaultPrevented,target:event.target?.getAttribute('aria-label')}));
 const key=(node,name,shift=false)=>{const event=new KeyboardEvent('keydown',{key:name,code:name,bubbles:true,composed:true,cancelable:true,shiftKey:shift});node.dispatchEvent(event);return event.defaultPrevented;};
 const command=id=>{get(id).focus();get(id).click();};
 const ready=()=>menu()?.dataset.menuNative==='true';
 const fresh=async()=>{const old=menu();command('reset');await wait(()=>menu()&&menu()!==old&&ready()&&events().length===0,'fresh ready fixture');};
 const go=async(node,name,label,shift=false)=>{node.focus();const prevented=key(node,name,shift);await wait(()=>document.activeElement===item(label)&&item(label)?.tabIndex===0,name+' focuses committed roving target '+label);assert(prevented,name+' prevented');};
 const checks=[];
 await wait(()=>bar()&&ready(),'native observer mount');await fresh();
 assert(bar().tabIndex===0&&bar().getAttribute('aria-orientation')==='vertical','native vertical menubar entry');
 assert(nodes().every(n=>n.tabIndex===-1),'initial eligible controls -1');
 assert(item('Disabled link')?.hasAttribute('disabled')&&!item('Disabled link').hasAttribute('href')&&item('Disabled link').tabIndex===-1,'normal disabled anchor marker survives native rendering');
 const toggle=menu().querySelector('.ix-menu-toggle');assert(toggle&&!bar().contains(toggle),'toggle outside traversal');toggle.focus();assert(!key(toggle,'ArrowDown'),'toggle ArrowDown not consumed');await sleep(60);assert(document.activeElement===toggle,'toggle key no navigation');
 await go(bar(),'ArrowDown','Alpha');assert(item('Alpha').tabIndex===0&&item('Alpha').getAttribute('aria-posinset')==='1'&&item('Alpha').getAttribute('aria-setsize')==='9','first roving position metadata');
 await go(item('Alpha'),'ArrowDown','Link');assert(item('Disabled').tabIndex===-1,'disabled markup skipped');assert(get('link-wrapper').tabIndex===-1,'anchor wrapper not extra tab stop');
 await go(item('Link'),'End','About & legal information');await go(item('About & legal information'),'ArrowDown','Alpha');await go(item('Alpha'),'ArrowUp','About & legal information');await go(item('About & legal information'),'Home','Alpha');
 assert(selected().length===0,'navigation never selects');assert(bubble.some(e=>e.key==='ArrowDown'&&e.prevented),'handled key still reaches document bubble');checks.push('native entry, toggle boundary, source wrapping/endpoints, wrapper and disabled markup, metadata, default-only propagation');
 item('Link').focus();await sleep(40);bar().focus();await sleep(40);assert(nodes().every(n=>n.tabIndex===-1),'focusout resets eligible indices');await go(bar(),'ArrowUp','Link');
 await fresh();await go(bar(),'ArrowUp','About & legal information');checks.push('separate actual focus and remembered/absent container entry');
 const alpha=item('Alpha');command('move');await wait(()=>state().reversed,'move accepted');assert(item('Alpha')===alpha,'stable keyed native node');await go(bar(),'Home','Omega');assert(item('Omega').getAttribute('aria-posinset')==='1','moved existing occurrence position');await go(item('Omega'),'ArrowDown','Alpha');
 command('link-toggle');await wait(()=>!state().link_present,'remove Link');await go(item('Alpha'),'ArrowDown','Category');assert(item('Category').getAttribute('aria-setsize')==='8','removed occurrence updates size');
 command('hide');await wait(()=>state().omega_hidden,'hidden Omega');await go(bar(),'Home','Alpha');
 command('disable');await wait(()=>!state().disabled,'normal disabled bool cleared');await go(item('Alpha'),'ArrowDown','Disabled');checks.push('current DOM order on keyed move, removal, hidden and normal disabled markup changes');
 command('link-toggle');await wait(()=>state().link_present,'restore Link');command('inline-hide');await wait(()=>state().link_inline_hidden,'hide direct wrapper');
 item('Disabled').focus();assert(key(item('Disabled'),'ArrowDown'),'hidden wrapper request handled');await wait(()=>item('Link').tabIndex===0,'hidden Link still gets roving state');await sleep(80);assert(document.activeElement===item('Disabled'),'hidden wrapper native focus fails');
 assert(key(item('Disabled'),'ArrowDown'),'repeat actual-origin key handled');await sleep(80);assert(document.activeElement===item('Disabled')&&item('Link').tabIndex===0,'actual origin stays distinct from requested memory');checks.push('source hidden-wrapper failure preserves actual-origin versus requested target distinction');
 command('labels');await wait(()=>bar()?.getAttribute('aria-label')==='Review navigation','live menu label');assert(bar().getAttribute('aria-description')==='Choose a menu item with Up and Down','live navigation hint');assert(selected().length===0,'commands and label update silent');
 await fresh();item('Category').click();await wait(()=>item('Category').getAttribute('aria-expanded')==='true'&&item('One')?.getClientRects().length,'existing category visible');item('One').focus();assert(!key(item('One'),'Home'),'category Home excluded from outer traversal');await sleep(60);assert(document.activeElement===item('One'),'nested Home no root target');checks.push('live labels and category role-menu isolation');
 await fresh();bar().focus();key(bar(),'ArrowDown');command('mount');await wait(()=>!menu(),'unmount pending request');await sleep(100);assert(document.activeElement===get('mount'),'unmount does not steal outside focus');command('mount');await wait(()=>menu()&&ready(),'remount rebound observer');await sleep(100);assert(document.activeElement===get('mount'),'stale request never targets new menu');await go(bar(),'ArrowDown','Alpha');checks.push('queued focus unmount guard, new ownership and rebound observer');
 return {ok:true,checks,input:'synthetic cancelable composed keys; native focus/default plumbing',notEstablished:['physical OS input','synthetic native Tab default action','source property-only disabled updates from one Rust bool','category-local G04.4','full AT/Brand/platform parity'],renderer:navigator.userAgent};
} catch(error) {return {ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(45)).await;
            eprintln!("G04.3 native probe timed out");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let desktop = desktop.clone();
            spawn(async move {
                let query = dom_query::eval(VERIFY);
                let _ = query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"] == true => {
                        println!("G04.3 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("G04.3 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, menu_navigation_review::MenuNavigationReview {} }
    } }
}
