//! Bounded Linux/WebView regression for B10.6. Input/focus events are synthetic;
//! this verifies renderer wiring, not physical keyboard or assistive technology.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/filter_keyboard_review.rs"]
mod filter_keyboard_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX native filter keyboard probe")
                .with_visible(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(900.0, 1000.0)),
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
 const get=id=>document.getElementById(id),root=name=>get(`b10-keyboard-${name}`),input=name=>root(name)?.querySelector('input'),filter=name=>root(name)?.querySelector('.ix-category-filter');
 const options=name=>[...root(name).querySelectorAll('.ix-category-filter-option')],open=name=>filter(name)?.dataset.open==='true';
 const events=()=>JSON.parse(get('b10-keyboard-events').textContent),values=()=>JSON.parse(get('b10-keyboard-values').textContent);
 const focus=()=>({tag:document.activeElement?.tagName,id:document.activeElement?.id,label:document.activeElement?.getAttribute('aria-label'),text:document.activeElement?.textContent?.trim()});
 const state=()=>({focus:focus(),cases:Object.fromEntries(['category','suggestions','top-layer'].map(n=>[n,{open:open(n),input:input(n)?.value,options:options(n).map(o=>o.textContent.trim())}])),events:events()});
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; '+JSON.stringify(state()))};
 const wait=async(test,label)=>{const end=performance.now()+3000;while(!test()){if(performance.now()>end)throw Error(label+'; '+JSON.stringify(state()));await sleep(20)}};
 const key=(node,key,shift=false)=>{const event=new KeyboardEvent('keydown',{bubbles:true,composed:true,cancelable:true,key,code:key===' '?'Space':key,shiftKey:shift});node.dispatchEvent(event);return event.defaultPrevented;};
 const edit=(name,value)=>{const node=input(name);node.value=value;node.dispatchEvent(new InputEvent('input',{bubbles:true,composed:true,data:value,inputType:'insertText'}));};
 const fresh=async()=>{const old=input('category');get('b10-keyboard-reset').click();await wait(()=>input('category')&&input('category')!==old&&events().length===0,'fresh keyed cases');};
 const checks=[];
 await wait(()=>input('category')&&input('suggestions')&&input('top-layer'),'mount');await fresh();
 input('category').focus();await sleep(60);assert(document.activeElement===input('category')&&!open('category'),'focus-only keeps closed');
 assert(key(input('category'),'ArrowDown'),'closed ArrowDown prevented');await wait(()=>open('category')&&document.activeElement===options('category')[0],'closed ArrowDown focuses first native option');assert(options('category').every(n=>n.tagName==='BUTTON'),'options are native buttons');
 let start=events().length;const vendor=options('category')[0],product=options('category')[1];
 for(const name of ['ArrowDown','ArrowUp','Home','End','Enter',' ']){assert(key(vendor,name),'option '+name+' prevented');await sleep(30);assert(document.activeElement===vendor&&events().length===start,'option '+name+' neither traverses nor activates');}
 for(const name of ['ArrowLeft','ArrowRight']){assert(!key(vendor,name),'option '+name+' default retained');assert(document.activeElement===vendor,'lateral arrow keeps native option focus');}
 assert(key(vendor,'Tab',true),'first ShiftTab prevented');await wait(()=>document.activeElement===product,'first ShiftTab wraps last');assert(key(product,'Tab'),'last Tab prevented');await wait(()=>document.activeElement===vendor,'last Tab wraps first');checks.push('native option focus, canceled activation/navigation, popup Tab boundaries');
 vendor.click();await wait(()=>root('category').querySelector('.ix-category-filter-preview')?.textContent==='Vendor'&&document.activeElement===input('category'),'pointer Vendor focuses input');assert(events().slice(start).filter(e=>e.event==='oncategory').length===1,'one category event');
 input('category').focus();assert(!key(input('category'),'ArrowDown'),'open category ArrowDown default preserved');await wait(()=>document.activeElement===options('category')[0],'open category ArrowDown focuses first value');
 const operator=()=>root('category').querySelector('[aria-label="Toggle comparison operator"]');operator().focus();start=events().length;operator().click();await wait(()=>operator()?.getAttribute('aria-pressed')==='true','pointer operator toggles');await sleep(60);assert(document.activeElement===operator()&&events().length===start,'operator retains focus and is silent');
 assert(key(operator(),'Enter')&&key(operator(),' '),'operator activation canceled');await sleep(40);assert(operator()?.getAttribute('aria-pressed')==='true'&&events().length===start,'operator keys do not toggle');
 input('category').focus();assert(!key(input('category'),'Tab'),'input Tab native default retained');await sleep(60);assert(document.activeElement===input('category'),'synthetic Tab causes no forced operator focus');checks.push('open category focus/default flags, pointer operator retained, native input Tab routing');
 edit('category','Draft');key(input('category'),'Enter');await wait(()=>values().category.categories[0]?.value==='Draft','typed category commit');assert(values().category.categories[0].operator==='!=','retained operator commit');assert(events().filter(e=>e.event==='oncategory').length===1,'commit no category(None)');
 // Native text caret changes and native Tab defaults are browser-only evidence;
 // synthetic dispatch here verifies flags and renderer focus wiring only.
 edit('category','abcdef');key(input('category'),'Escape');await wait(()=>!open('category'),'input Escape closes');assert(document.activeElement===input('category'),'input Escape keeps input focus');
 assert(!key(input('category'),'Home')&&!key(input('category'),'End'),'Home/End native defaults preserved');checks.push('typed draft Enter, silent commit preview, input Escape, native caret defaults retained');
 await fresh();input('category').focus();assert(!key(input('category'),'Escape'),'closed input Escape unprevented');assert(key(input('category'),' '),'closed input Space prevented');await wait(()=>open('category')&&document.activeElement===options('category')[0],'closed Space opens/focuses first');input('category').focus();assert(key(input('category'),' '),'open input Space prevented');await sleep(40);assert(input('category').value===''&&events().length===0,'Space inserts no text or input event');
 input('category').focus();assert(key(input('category'),'ArrowUp'),'form ArrowUp prevented');await wait(()=>document.activeElement===get('b10-keyboard-category-before'),'root previous sibling focus');assert(open('category'),'sibling focus does not close source popup');input('category').focus();edit('category','zz-unmatched');await wait(()=>options('category').length===0,'unmatched no options');assert(key(input('category'),'ArrowDown'),'unmatched form ArrowDown prevented');await wait(()=>document.activeElement===get('b10-keyboard-category-after'),'root next sibling focus');assert(open('category'),'unmatched sibling focus retains popup');checks.push('Space source quirks and root-adjacent sibling arrows');
 await fresh();input('suggestions').focus();key(input('suggestions'),'ArrowDown');await wait(()=>open('suggestions')&&document.activeElement===options('suggestions')[0],'closed suggestions first option');input('suggestions').focus();assert(key(input('suggestions'),'ArrowDown'),'open suggestions falls through prevented form');await sleep(60);assert(document.activeElement===input('suggestions'),'open suggestions wrapped root has no sibling target');
 edit('suggestions','Ban');key(input('suggestions'),'Enter');await wait(()=>values().suggestions.tokens[0]==='Ban','typed Ban rather than Banana');assert(values().suggestions.tokens.length===1,'one typed token');checks.push('closed/open suggestion selector distinction and typed Enter');
 await fresh();input('top-layer').focus();key(input('top-layer'),'ArrowDown');await wait(()=>open('top-layer')&&document.activeElement===options('top-layer')[0],'top-layer option focus');await wait(()=>root('top-layer').querySelector('[popover]')?.matches(':popover-open'),'native top-layer membership');start=events().length;assert(key(options('top-layer')[0],'Enter')&&key(options('top-layer')[0],' '),'top-layer activation canceled');await sleep(40);assert(events().length===start,'no top-layer key selection');assert(!key(options('top-layer')[0],'Escape'),'option Escape native default preserved');await wait(()=>!open('top-layer'),'top-layer Escape hides');await sleep(60);assert(document.activeElement!==input('top-layer'),'option Escape does not force input focus');checks.push('top-layer native focus/defaults and Escape without forced restoration');
 // Queue focus, then replace the options/mount before native focus delivery.
 await fresh();input('category').focus();key(input('category'),'ArrowDown');edit('category','zz-unmatched');await wait(()=>open('category')&&options('category').length===0,'queued focus option invalidation');await sleep(80);assert(document.activeElement===input('category'),'removed option never steals focus');
 await fresh();const old=input('category');old.focus();key(old,'ArrowDown');await fresh();await sleep(80);assert(!open('category')&&!document.activeElement?.classList.contains('ix-category-filter-option'),'unmounted pending focus does not hit replacement');checks.push('queued option and keyed unmount focus invalidation');
 return {ok:true,checks,input:'synthetic cancelable composed DOM events; native focus plumbing',notEstablished:['physical OS input','synthetic native Tab/caret default actions','external native propagation equivalence','AT/full Brand/full platform parity'],renderer:navigator.userAgent};
} catch(error) {return {ok:false,error:String(error)}}
"#;

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(40)).await;
            eprintln!("B10.6 native probe timed out");
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
                        println!("B10.6 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("B10.6 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, filter_keyboard_review::FilterKeyboardReview {} }
    } }
}
