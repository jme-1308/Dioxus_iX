//! Excluded actual-gallery operator profile preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/operator_profile.rs"]
mod gallery_profile;

#[component]
pub fn OperatorProfilePreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_profile::STYLES} }
        IxTheme { class: "operator-profile-review",
            h1 { "Operator profile menu" }
            Button { id: "operator-profile-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "operator-profile-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_profile::OperatorProfileExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_OPERATOR_PROFILE_CHECK);
                        if let Err(error) = query.send("operatorprofile") {
                            eprintln!("Operator profile native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native operator profile: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native operator profile failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Operator profile native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.operator-profile-review{box-sizing:border-box;min-height:100vh;padding:24px}.operator-profile-review>h1{font-size:28px;margin:0 0 24px}.operator-profile-review h2{font-size:22px;margin:24px 0 16px}.operator-profile-review p{max-width:72ch;line-height:1.5}#operator-profile-frame{margin-top:24px}@media(max-width:600px){.operator-profile-review{padding:16px}}";

// Excluded replay only. Untrusted native Enter/Space/Tab do not execute browser
// defaults; form setup uses explicit events, activation uses DOM clicks. Shared
// Dropdown action keys execute the real Rust handlers. Normal page scrolling
// positions native controls before focus; trusted typing/Tab is separate evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_OPERATOR_PROFILE_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},imageFacts={},tooltipFacts={},failureGeometry={},stopObserveImages=()=>{};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const avatar=()=>get('profile-avatar'),trigger=()=>avatar()?.querySelector('button.ix-avatar-trigger');
 const menu=()=>get(trigger()?.getAttribute('aria-controls')),result=()=>get('profile-result');
 const note=()=>get('profile-note'),notifications=()=>get('profile-notifications');
 const opened=()=>trigger()?.getAttribute('aria-expanded')==='true'&&menu()&&!menu().hidden;
 const count=name=>Number(result()?.dataset[name]);
 const record=id=>get('profile-record-'+id),field=(id,name)=>record(id)?.querySelector('[data-field="'+name+'"]')?.textContent.trim();
 const faces=()=>[...avatar()?.querySelectorAll('.ix-avatar')??[]],images=()=>[...avatar()?.querySelectorAll('.ix-avatar img')??[]];
 const descriptions=()=>String(trigger()?.getAttribute('aria-describedby')||'').split(/\s+/).filter(Boolean),tooltip=()=>descriptions().map(get).find(node=>node?.getAttribute('role')==='tooltip');
 const tooltipShown=()=>{const panel=tooltip();return !!panel&&panel.dataset.open==='true'&&panel.matches(':popover-open')&&getComputedStyle(panel).visibility==='visible'&&panel.getBoundingClientRect().width>0};
 const tooltipText=()=>tooltip()?.querySelector('.ix-tooltip-content')?.textContent.trim(),tooltipState=()=>({id:tooltip()?.id,description:trigger()?.getAttribute('aria-describedby'),shown:tooltipShown(),text:tooltipText(),label:tooltip()?.getAttribute('aria-label')});
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.className))};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const hover=node=>node.dispatchEvent(new MouseEvent('mouseenter',{bubbles:false})),leave=node=>node.dispatchEvent(new MouseEvent('mouseleave',{bubbles:false}));
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const setSelect=async(id,value)=>{const node=get(id);node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' native select value applied',()=>node.value===value)};
 const setChecked=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' explicit form-event setup applied',()=>node.checked===value)};
 const setNote=async text=>{const node=note();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual retained draft updated',()=>note()===node&&node.value===text)};
 const hitTestable=(node,insideFrame=true)=>{
  const box=node?.getBoundingClientRect(),frame=get('operator-profile-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id||node?.className,active:active()?.id,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className}:null};
  return !!box&&box.width>0&&box.height>0&&(!insideFrame||(box.left>=frame.left-1&&box.right<=frame.right+1))&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const open=async()=>{const previousTooltip=tooltip();if(!opened()){await focus(trigger(),'profile trigger');trigger().click()}await wait('actual profile menu open hides its tooltip',()=>opened()&&result().dataset.menuOpen==='true'&&!tooltipShown()&&(!previousTooltip?.isConnected||!previousTooltip.matches(':popover-open')))};
 const close=async()=>{if(opened()){await focus(trigger(),'profile Escape trigger');key(trigger(),'Escape')}await wait('actual profile Escape closed and native focus returned',()=>!opened()&&result().dataset.menuOpen==='false'&&active()===trigger())};
 const apply=async(id,text)=>{const before=count('updates');get('profile-apply').click();await wait('Apply saves actual selected operator record and closes menu',()=>count('updates')===before+1&&field(id,'note')===text&&!opened())};
 const snapshot=()=>({operator:result()?.dataset.operator,menu:opened(),updates:count('updates'),statusActions:count('statusActions'),preferences:count('preferenceChanges'),requests:count('changeRequests'),draft:note()?.value,notifications:notifications()?.checked,records:['alex','jo'].map(id=>({id,status:field(id,'status'),notifications:field(id,'notifications'),note:field(id,'note')}))});

 await wait('actual Avatar/profile/form/records mounted',()=>avatar()&&trigger()&&menu()&&note()&&notifications()&&result()&&record('alex')&&record('jo'));
 await wait('native WebView can receive focus',()=>document.hasFocus());
 assert(!opened()&&result().dataset.operator==='alex'&&['updates','statusActions','preferenceChanges','changeRequests'].every(name=>count(name)===0)&&field('alex','status')==='Available'&&field('jo','status')==='Away'&&field('alex','notifications')==='On'&&field('jo','notifications')==='Off'&&['alex','jo'].every(id=>field(id,'note')==='No preparation note'),'actual initial operator records and counters');
 assert(trigger().tagName==='BUTTON'&&trigger().getAttribute('aria-label')==='Open operator profile'&&trigger().getAttribute('aria-haspopup')==='menu'&&menu().getAttribute('role')==='menu'&&menu().getAttribute('aria-label')==='Operator profile actions'&&!avatar().hasAttribute('title')&&!trigger().hasAttribute('title'),'native trigger/menu relationship has no duplicate native title');
 await focus(trigger(),'closed profile tooltip anchor');await wait('closed focus exposes styled described tooltip',()=>tooltipShown()&&tooltipText()==='Alex Taylor · Line 1 operator'&&tooltip().getAttribute('aria-label')==='Operator profile information');
 assert(tooltip().classList.contains('ix-tooltip')&&tooltip().dataset.positioned==='true'&&descriptions().includes(tooltip().id),'actual styled panel is positioned and linked by trigger description');tooltipFacts.focus=tooltipState();key(trigger(),'Escape');await wait('closed tooltip Escape hides help without opening menu or moving focus',()=>!tooltipShown()&&!opened()&&active()===trigger());
 await focus(get('profile-operator'),'tooltip blur target');hover(avatar());await wait('closed hover shows actual tooltip without stealing selector focus',()=>tooltipShown()&&active()===get('profile-operator'));tooltipFacts.hover=tooltipState();leave(avatar());await wait('closed pointer leave dismisses tooltip',()=>!tooltipShown());
 await focus(trigger(),'tooltip focus reopening');await wait('new focus reopens tooltip',()=>tooltipShown());await focus(get('profile-operator'),'tooltip ordinary blur');await wait('native blur dismisses styled help',()=>!tooltipShown());
 await open();
 assert(!avatar().hasAttribute('title')&&avatar().querySelector('.ix-avatar-username').textContent==='Alex Taylor'&&avatar().querySelector('.ix-avatar-extra').textContent==='Line 1 operator'&&avatar().querySelector('.ix-avatar-divider [role="separator"]')&&faces().length===2&&faces().every(node=>node.getAttribute('aria-hidden')==='true'&&node.textContent==='AT')&&note().value==='Inspect Motor 001'&&notifications().checked,'actual open profile information/divider/paired decorative initials and draft defaults');
 layout.faceWidths=faces().map(node=>node.getBoundingClientRect().width);assert(layout.faceWidths.every(width=>Math.abs(width-32)<1),'legacy 32px default profile faces preserved');
 await focus(note(),'menu descendant tooltip guard');hover(avatar());hover(note());await wait('menu descendant focus and hover cannot revive tooltip',()=>opened()&&active()===note()&&!tooltipShown());leave(note());
 await close();snapshots.semantics=snapshot();

 await focus(trigger(),'AD opening trigger');key(trigger(),'ArrowDown');await wait('real shared AD ArrowDown opens profile',()=>opened());
 await focus(trigger(),'AD action owner');key(trigger(),'End');await wait('AD last enabled status action represented on trigger',()=>trigger().getAttribute('aria-activedescendant')==='profile-status');
 key(trigger(),'Enter');await wait('semantic AD Enter changes actual Alex status once and closes',()=>field('alex','status')==='Away'&&count('statusActions')===1&&!opened()&&active()===trigger());
 await open();await focus(note(),'retained native note Escape');key(note(),'Escape');await wait('Escape from composed field closes and returns trigger',()=>!opened()&&active()===trigger());
 await open();await focus(get('profile-operator'),'outside native operator selector');get('operator-profile').querySelector('h2').click();await wait('ordinary outside click dismisses without stealing native external focus',()=>!opened()&&active()===get('profile-operator'));
 snapshots.keyboardDismissal=snapshot();

 await open();const retainedNote=note(),retainedNotifications=notifications();
 await focus(note(),'actual preparation field');await wait('actual note receives pointer hit',()=>hitTestable(note(),false));hitFacts.note=failureGeometry.hit;
 for(const value of ['Home','ArrowLeft',' ','Tab']){const event=key(note(),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native note '+value+' default remains available')}
 await setNote('Inspect pump seal');await focus(notifications(),'native shift notifications');const space=key(notifications(),' ');keyPermissions.checkboxSpace=!space.defaultPrevented;assert(!space.defaultPrevented,'native checkbox Space default available');
 notifications().click();await wait('Alex preference changes actual record without dismissing menu',()=>opened()&&!notifications().checked&&field('alex','notifications')==='Off'&&count('preferenceChanges')===1);
 await apply('alex','Inspect pump seal');await open();assert(note()===retainedNote&&notifications()===retainedNotifications&&note().value==='Inspect pump seal','close/reopen retains actual menu-local native nodes and draft');snapshots.draftPreference=snapshot();

 await setSelect('profile-operator','jo');await wait('actual selected operator profile and record preference change',()=>result().dataset.operator==='jo'&&get('profile-current-name').textContent==='Jo Rivera'&&avatar().querySelector('.ix-avatar-username').textContent==='Jo Rivera'&&avatar().querySelector('.ix-avatar-extra').textContent==='Shift coordinator'&&!notifications().checked);await open();
 assert(note()===retainedNote&&note().value==='Inspect pump seal'&&faces().every(node=>node.textContent==='JR'),'operator change retains shared draft and updates both initials');
 notifications().click();await wait('Jo preference is independent of Alex',()=>notifications().checked&&field('jo','notifications')==='On'&&field('alex','notifications')==='Off'&&count('preferenceChanges')===2);
 await setNote('Prepare Jo shift');await apply('jo','Prepare Jo shift');await focus(get('profile-operator'),'updated Jo tooltip blur');await focus(trigger(),'updated Jo tooltip anchor');await wait('operator change updates actual tooltip text',()=>tooltipShown()&&tooltipText()==='Jo Rivera · Shift coordinator'&&tooltip().getAttribute('aria-label')==='Operator profile information');tooltipFacts.operator=tooltipState();await setSelect('profile-operator','alex');await wait('live operator text updates the same closed tooltip without focus theft',()=>tooltipShown()&&tooltipText()==='Alex Taylor · Line 1 operator'&&active()===trigger());await wait('Alex restored without overwriting records',()=>result().dataset.operator==='alex'&&field('alex','note')==='Inspect pump seal'&&field('jo','note')==='Prepare Jo shift');await open();
 assert(note()===retainedNote&&note().value==='Prepare Jo shift'&&!notifications().checked,'menu shared draft and per-operator notifications have distinct ownership');await close();snapshots.operatorSwitch=snapshot();

 const options=get('profile-options');options.querySelector('summary').click();await wait('native profile options expanded',()=>options.open);
 await setChecked('profile-action-disabled',true);await open();await wait('actual status action disabled',()=>get('profile-status').getAttribute('aria-disabled')==='true');
 const guardedStatus=count('statusActions');get('profile-status').click();await wait('disabled status cannot mutate real model',()=>count('statusActions')===guardedStatus&&field('alex','status')==='Away'&&opened());
 await focus(trigger(),'AD disabled-skip owner');key(trigger(),'End');await wait('AD End skips disabled status action',()=>trigger().getAttribute('aria-activedescendant')==='profile-apply');await close();
 await setChecked('profile-disabled',true);await wait('native trigger disabled',()=>trigger().disabled);const blockedRequests=count('changeRequests');trigger().click();await wait('disabled trigger rejects before change request',()=>!opened()&&count('changeRequests')===blockedRequests);await setChecked('profile-disabled',false);await wait('native trigger restored',()=>!trigger().disabled);
 await setChecked('profile-veto',true);let before=count('changeRequests');trigger().click();await wait('vetoed opening records attempt without opening',()=>count('changeRequests')===before+1&&!opened());await setChecked('profile-veto',false);await open();
 // Form-event setup keeps the menu open; clicking this outside option would also
 // request ordinary outside dismissal before changing the option itself.
 await setChecked('profile-veto',true);before=count('changeRequests');await focus(trigger(),'closing veto trigger');key(trigger(),'Escape');await wait('vetoed Escape preserves actual open state',()=>count('changeRequests')===before+1&&opened());
 await setChecked('profile-veto',false);await close();await setChecked('profile-action-disabled',false);
 await focus(get('profile-operator'),'tooltip settings blur');await focus(trigger(),'live tooltip settings anchor');await wait('current tooltip shown before label change',()=>tooltipShown());await setChecked('profile-tooltip-german',true);await wait('accessible tooltip label updates without replacing draft or stealing focus',()=>tooltipShown()&&tooltip().getAttribute('aria-label')==='Profilinformationen'&&active()===trigger());tooltipFacts.localized=tooltipState();const removedTooltipId=tooltip().id;await setChecked('profile-tooltip-hidden',true);await wait('explicit empty tooltip removes panel and stale trigger description',()=>!tooltip()&&!get(removedTooltipId)&&descriptions().length===0&&!avatar().hasAttribute('title')&&!trigger().hasAttribute('title'));await focus(get('profile-operator'),'empty tooltip blur');await focus(trigger(),'empty tooltip focus');hover(avatar());await wait('empty tooltip remains absent on focus and hover',()=>!tooltipShown()&&descriptions().length===0);leave(avatar());await setChecked('profile-tooltip-hidden',false);await focus(get('profile-operator'),'restored tooltip blur');await focus(trigger(),'restored tooltip focus');await wait('restored text and translated label produce a fresh described panel',()=>tooltipShown()&&tooltipText()==='Alex Taylor · Line 1 operator'&&tooltip().getAttribute('aria-label')==='Profilinformationen');assert(note()===retainedNote&&note().value==='Prepare Jo shift'&&field('alex','note')==='Inspect pump seal'&&field('jo','note')==='Prepare Jo shift'&&field('alex','notifications')==='Off'&&field('jo','notifications')==='On','tooltip changes retain draft and independent saved settings');tooltipFacts.restored=tooltipState();await setChecked('profile-tooltip-german',false);await wait('English tooltip label restored',()=>tooltip()?.getAttribute('aria-label')==='Operator profile information');await focus(get('profile-operator'),'tooltip settings done');snapshots.guards=snapshot();

 await setChecked('profile-top-layer',true);await open();await wait('actual profile uses native top layer',()=>menu().matches(':popover-open'));
 let nativeErrors=0;const observeError=event=>{if(event.target instanceof HTMLImageElement&&avatar().contains(event.target))nativeErrors++};document.addEventListener('error',observeError,true);stopObserveImages=()=>document.removeEventListener('error',observeError,true);
 await setSelect('profile-image','invalid');await wait('actual invalid image error falls both faces back to initials',()=>result().dataset.imageMode==='invalid'&&nativeErrors>0&&images().length===0&&faces().every(node=>node.textContent==='AT'));
 imageFacts.invalid={nativeErrors,faces:faces().map(node=>node.textContent)};
 await setSelect('profile-image','valid');await wait('changed valid image loads in both actual profile faces',()=>result().dataset.imageMode==='valid'&&images().length===2&&images().every(image=>image.complete&&image.naturalWidth>0&&image.naturalHeight>0));
 imageFacts.recovered=images().map(image=>({width:image.naturalWidth,height:image.naturalHeight,alt:image.alt}));assert(note()===retainedNote&&note().value==='Prepare Jo shift'&&menu().matches(':popover-open'),'image recovery retains profile menu and local draft');stopObserveImages();snapshots.imagesTopLayer=snapshot();await close();

 const oldTooltipId=tooltip()?.id,oldAvatar=avatar(),oldNote=note(),reset=get('profile-reset'),operatorSelect=get('profile-operator');await focus(reset,'Restore operator profiles');reset.click();
 await wait('Restore resets actual records/options/counts and owned draft',()=>result().dataset.operator==='alex'&&!opened()&&result().dataset.imageMode==='initials'&&['updates','statusActions','preferenceChanges','changeRequests'].every(name=>count(name)===0)&&field('alex','status')==='Available'&&field('jo','status')==='Away'&&field('alex','notifications')==='On'&&field('jo','notifications')==='Off'&&['alex','jo'].every(id=>field(id,'note')==='No preparation note')&&note()?.value==='Inspect Motor 001'&&notifications().checked&&['profile-disabled','profile-action-disabled','profile-veto','profile-top-layer','profile-tooltip-german','profile-tooltip-hidden'].every(id=>!get(id).checked));
 assert((!oldTooltipId||!get(oldTooltipId))&&!oldAvatar.isConnected&&!oldNote.isConnected&&avatar()!==oldAvatar&&note()!==oldNote&&get('profile-reset')===reset&&get('profile-operator')===operatorSelect&&get('profile-options')===options&&active()===reset,'Restore replaces only Avatar-owned menu scope and preserves toolbar/options/focus');snapshots.reset=snapshot();

 get('operator-profile-narrow').click();await wait('actual 340px frame and contained page',()=>get('operator-profile-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);
 await focus(trigger(),'narrow native profile trigger');await wait('narrow native trigger fits and is hit-testable',()=>hitTestable(trigger()));hitFacts.narrowTrigger=failureGeometry.hit;await wait('narrow styled tooltip has correct reset text and viewport fit',()=>{const panel=tooltip(),box=panel?.getBoundingClientRect();return tooltipShown()&&tooltipText()==='Alex Taylor · Line 1 operator'&&panel.getAttribute('aria-label')==='Operator profile information'&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1});tooltipFacts.narrow=tooltipState();
 await setChecked('profile-top-layer',true);await open();await wait('narrow native top layer opened',()=>menu().matches(':popover-open'));
 await focus(note(),'narrow actual preparation');await wait('narrow actual field fits viewport and receives hit',()=>hitTestable(note(),false));hitFacts.narrowNote=failureGeometry.hit;
 await setNote('Narrow inspection note');await focus(notifications(),'narrow actual notifications');await wait('narrow actual checkbox receives pointer hit',()=>hitTestable(notifications(),false));notifications().click();await wait('narrow notification changes real current record',()=>field('alex','notifications')==='Off'&&count('preferenceChanges')===1&&opened());
 await wait('narrow Apply receives actual pointer hit',()=>hitTestable(get('profile-apply'),false));hitFacts.narrowApply=failureGeometry.hit;await apply('alex','Narrow inspection note');await open();await focus(trigger(),'narrow AD action owner');key(trigger(),'End');await wait('narrow AD status selected',()=>trigger().getAttribute('aria-activedescendant')==='profile-status');key(trigger(),'Enter');
 await wait('narrow semantic action updates actual status once',()=>field('alex','status')==='Away'&&count('statusActions')===1&&!opened());await open();
 const frame=get('operator-profile-frame').getBoundingClientRect(),popup=menu().getBoundingClientRect(),input=note().getBoundingClientRect();layout={...layout,frameWidth:frame.width,triggerWidth:trigger().getBoundingClientRect().width,inputWidth:input.width,inputHeight:input.height,popup:{left:popup.left,right:popup.right,width:popup.width},pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(note().value==='Narrow inspection note'&&!notifications().checked&&field('alex','note')==='Narrow inspection note'&&field('jo','note')==='No preparation note'&&count('updates')===1&&layout.inputHeight>=31&&popup.left>=0&&popup.right<=innerWidth+1&&layout.pageScroll<=layout.pageClient+1,'narrow retained draft/preferences/actions produce real records and usable contained fields');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,imageFacts,tooltipFacts,
 checks:['styled described tooltip focus/hover/Escape/blur/leave; menu suppression; native profile semantics and paired32px faces','shared AD action changes actual status; field Escape and outside dismissal preserve focus','native draft editing/default permission and actual immediate preference/Apply records','operator switching updates tooltip text while preserving shared draft and independent saved settings','disabled/vetoed menu guards plus live tooltip labels and explicit-empty ARIA cleanup without data loss','actual image error yields paired initials and different valid source loads both faces in native top layer','Restore resets owned menu/draft/model/options and preserves toolbar focus','340px actual trigger/field/checkbox/action hit-testing and real saved settings/status workflow'],
 input:'synthetic form/click/key events and explicit native focus/normal page scroll setup; native Enter/Space/Tab defaults are not simulated; closing-veto uses explicit form-change setup',
 trustedInput:'separate Chromium trusted typing/Tab/ShiftTab/pointer/default activation and visual evidence; no physical-device certification',renderer:navigator.userAgent};
} catch(error){stopObserveImages();return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,imageFacts,tooltipFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;
