//! Excluded native computed-style runner for the shared Rust Typography review.
use dioxus::prelude::*;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
use serde_json::Value;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/theme_review.rs"]
mod theme_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_TYPOGRAPHY_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_typography_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Typography and theme scopes</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Typography"
                } else {
                    "Dioxus iX Typography"
                })
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}
#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Probe);
}
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _fixture = Probe;
    eprintln!("Run the Typography review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_TYPOGRAPHY_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(60)).await;
            eprintln!("Typography native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {
                let window=window.clone();
                spawn(async move {
                    let query=dom_query::eval(VERIFY);let _=query.send(source);
                    match query.join::<Value>().await {
                        Ok(value) if value["ok"]==true=>{
                            println!("Typography native observations: {value}");window.close();
                        }
                        Ok(value)=>{eprintln!("Typography native failure JSON: {value}");std::process::exit(1);}
                        Err(error)=>{eprintln!("Typography native transport failure: {error:?}");std::process::exit(1);}
                    }
                });
            }
        },
        if !source {theme_review::ThemeReview {}}
    }}
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
const source=await dioxus.recv(),errors=[],phases=[],journal=[],assertions=[],objects=new WeakMap();let nextObject=0;
const abort=new AbortController(),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const object=node=>{if(!objects.has(node))objects.set(node,++nextObject);return objects.get(node);};
window.addEventListener('error',event=>errors.push(event.message),{signal:abort.signal});
window.addEventListener('unhandledrejection',event=>errors.push(String(event.reason)),{signal:abort.signal});
const wait=async(predicate,label)=>{const until=performance.now()+25000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const check=(condition,label)=>{assertions.push({ok:!!condition,label});if(!condition)throw Error(label);};
const frames=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{journal.push({kind:'fixed-native-frame',index,timestamp});resolve();})));};
const fields=['fontSize','lineHeight','fontWeight','fontFamily','fontStyle','letterSpacing','color','textDecorationLine','fontFeatureSettings'];
const colorMap={alarm:'text-danger','alarm-contrast':'text-on-danger',contrast:'text-primary','critical-contrast':'text-on-critical','info-contrast':'text-on-information','inv-contrast':'text-inverse','inv-soft':'text-inverse','inv-std':'text-inverse','inv-weak':'text-inverse','neutral-contrast':'text-on-neutral','primary-contrast':'text-on-accent',soft:'text-secondary',std:'text-primary','success-contrast':'text-on-success','warning-contrast':'text-on-warning',weak:'text-disabled'};
const attributes=node=>Object.fromEntries(Array.from(node.attributes,attribute=>[attribute.name,attribute.value]));
const rect=node=>{const value=node.getBoundingClientRect();return{x:value.x,y:value.y,width:value.width,height:value.height};};
const row=wrap=>{
 const node=wrap.querySelector('.ix-typography'),scope=wrap.closest('[data-theme-scope]');
 if(!(node instanceof HTMLElement)||!scope)throw Error('Missing actual Typography '+wrap.id);
 const cs=getComputedStyle(node),format=wrap.dataset.themeKey==='default-format'?null:wrap.dataset.themeFormat;
 const textColor=wrap.dataset.themeColor??null;
 return{id:wrap.id,scope:scope.dataset.themeScope,scopeIndex:Number(scope.dataset.scopeIndex),revision:Number(scope.dataset.revision),kind:wrap.dataset.themeKind,key:wrap.dataset.themeKey,format,textColor,bold:wrap.dataset.themeBold==='true',textDecoration:wrap.dataset.themeDecoration,attrs:attributes(node),class:node.className,computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),numeric:{fontSize:parseFloat(cs.fontSize),lineHeight:parseFloat(cs.lineHeight),fontWeight:parseFloat(cs.fontWeight),letterSpacing:parseFloat(cs.letterSpacing)},rect:rect(node),text:node.textContent,actual:{object:object(node),connected:node.isConnected,tag:node.tagName},wrapperAttrs:attributes(wrap),resolvedTokens:{font:cs.getPropertyValue('--si-sys-typography-'+(format??'body')).trim(),letterSpacing:cs.getPropertyValue('--si-ref-typography-letter-spacing-normal').trim(),bold:cs.getPropertyValue('--si-ref-typography-font-weight-bold').trim(),color:textColor?cs.getPropertyValue('--si-sys-color-'+colorMap[textColor]).trim():null}};
};
const snapshot=id=>({id,rows:Array.from(document.querySelectorAll('[data-theme-row]'),row),scopes:Array.from(document.querySelectorAll('[data-theme-scope]')).filter(scope=>scope.dataset.themeScope!=='nested-isolated').map(scope=>({id:scope.dataset.themeScope,scopeIndex:Number(scope.dataset.scopeIndex),scheme:scope.dataset.scheme,density:scope.dataset.density,revision:Number(scope.dataset.revision),supplied:scope.dataset.supplied==='true'})),settleFrames:2});
try{
 if(source){
   await wait(()=>window.upstreamTypographyReview&&document.getElementById('typography-status')?.dataset.ready==='true','pinned source component readiness');
   const value=await window.upstreamTypographyReview.run();
   check(value?.ok===true,'source four-phase run completed');
   check(value.phases?.length===4,'source four phases');
   for(const phase of value.phases)check(phase.rows.length===193,'source '+phase.id+' 193 actual computed rows');
   return{...value,ok:value.ok&&errors.length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};
 }
 await wait(()=>document.querySelectorAll('[data-theme-row]').length===193,'Rust fixture committed all specimen rows');
 for(const id of ['baseline','revised','reactive','restored']){
   const button=document.getElementById('typography-'+id);
   check(button instanceof HTMLButtonElement,'real Rust '+id+' phase control');
   journal.push({kind:'public-control-before',phase:id,control:button.id,time:performance.now()});
   button.click();
   await wait(()=>document.getElementById('typography-review')?.dataset.themePhase===id,'Rust phase property commit '+id);
   await frames();
   const value=snapshot(id);check(value.rows.length===193,id+' 193 actual computed rows');
   for(const scope of value.scopes){
      const rows=value.rows.filter(row=>row.scope===scope.id);
      check(rows.filter(row=>row.kind==='format').length===26,id+' '+scope.id+' 26 formats');
      check(rows.filter(row=>row.kind==='color').length===16,id+' '+scope.id+' 16 explicit colors');
   }
   phases.push(value);journal.push({kind:'public-control-settled',phase:id,time:performance.now()});
 }
 const fallback=Array.from(document.querySelectorAll('#typography-brand-fallback .ix-typography'),node=>{const cs=getComputedStyle(node);return{attrs:attributes(node),computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),text:node.textContent,rect:rect(node)};});
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'developer-authored system-font semantic-token adapter; not numeric Brand metrics',phases,errors,journal,assertions,fallback,renderer:'Linux desktop WebView Rust',limits:['Computed rows measure actual Rust Typography children; stable specimen IDs belong to developer wrapper elements','Public Rust phase commit is observed before two fixed native RAF plus font readiness; no desired font/color waits','Actual source custom-host and Rust native attributes/geometry are independent renderer adapters','Sentinel tokens are developer-authored system-font integration evidence, not Siemens Brand font metrics','No-token fallback observations remain a separate provisional port diagnostic; no source fallback equivalence claim']};
}catch(error){errors.push(String(error));return{ok:false,phases,assertions,journal,errors};}
finally{abort.abort();}
"#;
