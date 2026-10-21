//! Excluded actual-gallery localized menu workflow; no copied application model.
use super::pane_layers_review::gallery_application;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn MenuLabelsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.menu-labels-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        style { {gallery_application::STYLES} }
        IxTheme { class: "menu-labels-review",
            gallery_application::ApplicationExamples {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_MENU_LABELS_CHECK);
                        if let Err(error) = query.send("menulabels") {
                            eprintln!("Menu labels native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native Menu labels: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native Menu labels failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;
                        eprintln!("Menu labels native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_MENU_LABELS_CHECK: &str = r#"
let snapshots={},checks=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),host=()=>get('application-shell'),app=()=>host()?.querySelector('.ix-application'),menu=()=>host()?.querySelector('[data-menu-root]'),toggle=()=>menu()?.querySelector('.ix-menu-toggle'),utility=label=>menu()?.querySelector('[aria-label="'+label+'"]'),overlay=()=>host()?.querySelector('.ix-menu-overlay-panel'),close=()=>overlay()?.querySelector('.ix-menu-overlay-close'),notification=()=>overlay()?.querySelector('[role=switch][aria-label="Enable notifications"]'),content=()=>host()?.querySelector('.application-demo-content'),scheme=()=>host()?.querySelector('.application-demo-frame>.ix-theme')?.dataset.ixColorSchema;
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label)};
 const focus=async node=>{assert(node,'current control exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait('current native focus',()=>document.activeElement===node)},click=async node=>{await focus(node);node.click()},key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
 const expanded=()=>menu()?.dataset.expanded==='true',setExpanded=async value=>{if(expanded()!==value)await click(host().querySelector('.ix-header-menu-toggle')||toggle());await wait('current expanded state '+value,()=>expanded()===value)},language=async value=>{const node=get('application-menu-language');await focus(node);node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('current translated menu labels',()=>utility(value==='nb'?'Innstillinger':'Settings')&&toggle().getAttribute('aria-label')===(expanded()?(value==='nb'?'Trekk sammen':'Collapse'):(value==='nb'?'Utvid':'Expand')))};
 const snapshot=()=>({language:get('application-menu-language').value,expanded:expanded(),toggle:toggle().getAttribute('aria-label'),header:host().querySelector('.ix-header-menu-toggle')?.getAttribute('aria-label'),overlay:overlay()?.getAttribute('aria-label'),close:close()?.getAttribute('aria-label'),notification:notification()?.getAttribute('aria-checked'),content:content()?.textContent,scheme:scheme()});
 await wait('actual application example ready',()=>app()&&toggle()&&get('application-menu-language'));
 assert(get('application-menu-language').value==='en'&&toggle().getAttribute('aria-label')==='Expand'&&utility('Settings')&&utility('Toggle theme')&&utility('About & legal information'),'English defaults remain available');
 await setExpanded(true);await click(utility('Alarm'));await wait('actual Alarm content selected',()=>content().textContent.includes('Example alarm content'));await setExpanded(true);await click(utility('Settings'));await wait('actual Settings open with current close focus',()=>overlay()?.getAttribute('aria-label')==='Settings'&&document.activeElement===close());await click(notification());await wait('actual notification preference changed',()=>notification()?.getAttribute('aria-checked')==='false');snapshots.english=snapshot();checks.push('default English menu utilities, ordinary Alarm selection and actual Settings notification preference');
 const oldMenu=menu(),oldPanel=overlay(),oldNotification=notification(),oldContent=content();await language('nb');await wait('open Settings labels update live',()=>overlay()?.getAttribute('aria-label')==='Innstillinger'&&close()?.getAttribute('aria-label')==='Lukk innstillinger');assert(menu()===oldMenu&&overlay()===oldPanel&&notification()===oldNotification&&content()===oldContent&&notification().getAttribute('aria-checked')==='false'&&expanded()&&content().textContent.includes('Example alarm content'),'language update retains current menu, open Settings, preference and selected content');assert(utility('Bytt tema')&&utility('Om og juridisk informasjon')&&host().querySelector('.ix-menu-scrim')?.getAttribute('aria-label')==='Lukk navigasjon','Norwegian utilities and scrim');snapshots.norwegian=snapshot();checks.push('live Norwegian labels retain expanded menu/open Settings/current notification and content');
 await click(close());await wait('localized close returns current settings invoker',()=>!overlay()&&document.activeElement===utility('Innstillinger'));const oldScheme=scheme();await click(utility('Bytt tema'));await wait('localized theme action changes actual nested theme',()=>scheme()!==oldScheme);await click(utility('Om og juridisk informasjon'));await wait('localized About opens and focuses current close',()=>overlay()?.getAttribute('aria-label')==='Om og juridisk informasjon'&&close()?.getAttribute('aria-label')==='Lukk Om'&&document.activeElement===close());key(close(),'Escape');await wait('About Escape returns current localized invoker',()=>!overlay()&&document.activeElement===utility('Om og juridisk informasjon'));snapshots.utilities=snapshot();checks.push('localized close/theme/About actions and handled Escape retain normal current focus behavior');
 const breakpoint=host().querySelector('.application-demo-controls [role=combobox]');await focus(breakpoint);key(breakpoint,'ArrowDown');await wait('actual breakpoint selector opened',()=>breakpoint.getAttribute('aria-expanded')==='true');const choices=()=>get(breakpoint.getAttribute('aria-controls'));await wait('actual Small option available',()=>choices()?.querySelector('button[role=option][value=sm]'));await click(choices().querySelector('button[role=option][value=sm]'));await wait('actual forced Small header control',()=>app().classList.contains('ix-app-small')&&host().querySelector('.ix-header-menu-toggle'));
 const header=()=>host().querySelector('.ix-header-menu-toggle');assert(header().getAttribute('aria-label')===(expanded()?'Trekk sammen':'Utvid'),'Small header receives current Norwegian menu labels');if(expanded())await click(header());await wait('Small header collapsed localized label',()=>!expanded()&&header().getAttribute('aria-label')==='Utvid');await click(header());await wait('Small header expands current localized menu',()=>expanded()&&header().getAttribute('aria-label')==='Trekk sammen');await click(host().querySelector('.ix-menu-scrim'));await wait('localized scrim closes Small navigation',()=>!expanded());await language('en');assert(header().getAttribute('aria-label')==='Expand','Small header returns live English default');await setExpanded(true);await click(utility('Settings'));await wait('reopened English Settings retains actual preference',()=>overlay()?.getAttribute('aria-label')==='Settings'&&notification()?.getAttribute('aria-checked')==='false');assert(content().textContent.includes('Example alarm content'),'selected content remains through Small/language/utilities');snapshots.smallRestored=snapshot();checks.push('actual forced Small header/scrim use live labels; English restoration and Settings reopen retain preference/content');
 return{ok:true,checks,snapshots,viewport:{width:innerWidth,height:innerHeight},input:'Excluded synthetic DOM clicks/change/keydown with explicit native focus exercise actual unchanged ApplicationExamples. Forced Small is selected through its actual Breakpoint control, not a narrow-window claim. Escape is a handled component key; native button Enter/Space and physical Tab defaults are separate trusted Chromium evidence. No application/model/controller/callback/CSS injection; unrelated Pane/layout sections remain mounted and visible.'};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:{id:document.activeElement?.id,label:document.activeElement?.getAttribute('aria-label')}}}
"#;
