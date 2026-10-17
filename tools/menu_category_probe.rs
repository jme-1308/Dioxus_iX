//! Visible Linux/WebView G04.4 replay. Synthetic keys measure native focus and
//! cancellation plumbing; browser-native Enter/Space defaults are reviewed separately.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/menu_category_review.rs"]
mod menu_category_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX category state and keyboard probe")
                .with_visible(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(1000.0, 1100.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms)),get=id=>document.getElementById('g04-category-'+id),stage=()=>get('stage'),menu=()=>stage()?.querySelector('nav.ix-menu');
 const item=name=>[...(stage()?.querySelectorAll('[role=menuitem]')??[])].find(n=>n.getAttribute('aria-label')===name),category=name=>item(name)?.closest('.ix-menu-category'),opened=name=>item(name)?.getAttribute('aria-expanded')==='true';
 const events=()=>JSON.parse(get('events').textContent),state=()=>JSON.parse(get('state').textContent),selected=()=>events().filter(e=>e.event==='select');
 const active=()=>document.activeElement?.getAttribute('aria-label')??document.activeElement?.id??document.activeElement?.tagName;
 const trace=[],record=entry=>{trace.push(entry);if(trace.length>80)trace.shift()};
 document.addEventListener('input',event=>{if(event.target.matches?.('input[data-menu-observer]')){try{const m=JSON.parse(event.target.value);record({event:'input',kind:m.kind,key:m.key,category:m.category,parent:m.category_parent,sequence:m.sequence,focus:active()})}catch{}}},true);
 document.addEventListener('click',event=>record({event:'click',target:event.target.getAttribute?.('aria-label')??event.target.id,trusted:event.isTrusted}),true);
 document.addEventListener('keydown',event=>record({event:'keydown',key:event.key,target:event.target.getAttribute?.('aria-label'),trusted:event.isTrusted}),true);
 const changes=new MutationObserver(records=>{for(const m of records)if(m.target.matches?.('.ix-menu-category,[aria-haspopup=menu]'))record({event:'mutation',attribute:m.attributeName,value:m.target.getAttribute(m.attributeName),label:m.target.getAttribute('aria-label'),focus:active()})});changes.observe(document.body,{subtree:true,attributes:true,attributeFilter:['hidden','aria-expanded','data-open','data-menu-category-mode']});
 const snapshot=()=>({focus:active(),state:state(),categories:['Category','Second category'].map(name=>({name,open:opened(name)})),events:events(),trace});
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; '+JSON.stringify(snapshot()))};
 const wait=async(test,label)=>{const end=performance.now()+3500;while(!test()){if(performance.now()>end)throw Error(label+'; '+JSON.stringify(snapshot()));await sleep(20)}};
 const key=(node,name,shift=false,alt=false)=>{const event=new KeyboardEvent('keydown',{key:name,code:name,bubbles:true,composed:true,cancelable:true,shiftKey:shift,altKey:alt});node.dispatchEvent(event);return event.defaultPrevented};
 const command=id=>{get(id).focus();get(id).click()};
 const ready=()=>menu()?.dataset.menuNative==='true'&&category('Category')?.dataset.categoryNative==='true'&&category('Second category')?.dataset.categoryNative==='true';
 const fresh=async(mode)=>{const old=menu();command(mode);await wait(()=>menu()&&menu()!==old&&ready()&&selected().length===0,'fresh '+mode+' ready')};
 const focus=async(name)=>{item(name).focus();await wait(()=>document.activeElement===item(name),'focus '+name)};
 const go=async(from,action,to,prevented=true,shift=false)=>{await focus(from);assert(key(item(from),action,shift)===prevented,action+' cancellation');await wait(()=>document.activeElement===item(to),action+' focus '+to)};
 const checks=[],bubble=[];document.addEventListener('keydown',e=>bubble.push({key:e.key,prevented:e.defaultPrevented}));
 await wait(()=>ready(),'category native observers');await fresh('expanded');
 assert(!opened('Category')&&!opened('Second category'),'fresh panels closed');
 await go('Category','Enter','One');assert(opened('Category'),'parent Enter opens');
 item('Second category').click();await wait(()=>opened('Second category'),'second independent panel');assert(opened('Category'),'two expanded panels retain state');
 await go('Category','Escape','Category',false);assert(opened('Category')&&opened('Second category'),'expanded parent Escape remains unhandled');
 await go('Category','ArrowDown','One');assert(opened('Category')&&opened('Second category'),'open expanded parent Down ends in its child domain');
 await go('One','ArrowDown','Disabled child');assert(!item('Disabled child').disabled&&item('Disabled child').getAttribute('aria-disabled')==='true','nested disabled button is ARIA-only native focus target');
 await go('Disabled child','ArrowDown','Two');await go('Two','Tab','Wrapped child');await go('Wrapped child','Tab','One');await go('One','Tab','Wrapped child',true,true);
 await go('One','Home','One',false);await go('One','End','One',false);await go('One','ArrowLeft','One',false);await go('One','ArrowRight','One',false);
 assert(get('wrapper').tabIndex===-1,'nested anchor wrapper no extra tab stop');assert(selected().length===0,'navigation does not select');
 checks.push('independent expanded panels; raw disabled/wrapped DOM order; local Tab wrapping; Home/End/Left/Right defaults; wrapper suppression');
 const before=selected().length;item('One').click();await wait(()=>selected().length===before+1,'pointer selects once');assert(opened('Category')&&opened('Second category'),'pointer retains panels');
 await focus('One');assert(!key(item('One'),'Enter'),'expanded child Enter unprevented');assert(document.activeElement===item('Category'),'Enter synchronously returns parent before dispatch completes');await sleep(50);assert(opened('Category'),'synthetic Enter does not manufacture parent click');
 await focus('One');assert(!key(item('One'),' '),'expanded child Space unprevented');assert(document.activeElement===item('Category'),'Space synchronously returns parent');await sleep(50);assert(opened('Category'),'Space retains expanded panel');
 await go('One','Escape','Category');await wait(()=>!opened('Category'),'Escape closes own panel');assert(opened('Second category')&&state().expanded,'Escape retains other panel and outer menu');assert(bubble.some(e=>e.key==='Escape'&&e.prevented),'expanded handled Escape reaches document bubble');
 checks.push('pointer retention; synchronous focus-only Enter/Space acknowledgement without invented clicks; local Escape ownership/default-only propagation');
 await fresh('expanded');item('Category').click();await wait(()=>opened('Category'),'open dynamic panel');const one=item('One');command('move');await wait(()=>state().order[0]==='wrapped','move wrapped first');assert(item('One')===one,'keyed child native identity retained');await go('One','ArrowUp','Wrapped child');
 command('wrapped-toggle');await wait(()=>!state().wrapped_present,'remove wrapped');await go('Two','ArrowDown','One');command('empty');await wait(()=>state().empty,'empty category');await go('Category','ArrowDown','Second category');assert(opened('Category'),'empty expanded parent preserves ordinary outer navigation');await focus('Category');assert(key(item('Category'),'Enter'),'empty parent close prevented');await wait(()=>!opened('Category'),'empty closes');assert(key(item('Category'),'Enter'),'empty parent open prevented');await wait(()=>opened('Category'),'empty opens');await sleep(60);assert(document.activeElement===item('Category'),'empty open retains parent focus');
 checks.push('current keyed child order/removal, stable native identity and empty open without focus target');
 await fresh('expanded');command('active');await wait(()=>state().one_active&&opened('Category'),'live active opens panel');menu().querySelector('.ix-menu-toggle').click();await wait(()=>!state().expanded&&!opened('Category'),'global collapse clears panel');menu().querySelector('.ix-menu-toggle').click();await wait(()=>state().expanded&&opened('Category'),'active reexpand opens');
 command('mount');await wait(()=>!menu(),'unmount active menu');command('mount');await wait(()=>ready()&&opened('Category'),'initial active opens on remount');
 checks.push('initial/live active state and global collapse/reexpand recomputation');
 await fresh('collapsed');await go('Category','ArrowDown','Second category');assert(!opened('Category'),'closed collapsed parent Down follows the outer menu without presenting');await go('Category','Enter','One');assert(opened('Category'),'collapsed Enter presents');
 const geometry=()=>{const child=item('One').getBoundingClientRect(),parent=item('Category').getBoundingClientRect(),panel=category('Category').querySelector(':scope > [role=menu]').getBoundingClientRect();return{child:child.toJSON(),parent:parent.toJSON(),panel:panel.toJSON(),gap:panel.left-parent.right,scroll:scrollY,topLayer:category('Category').querySelector(':scope > [role=menu]').matches(':popover-open')}};
 const positioned=()=>{const g=geometry();return g.topLayer&&Math.abs(g.panel.top-g.parent.top)<1&&Math.abs(g.gap-3)<1&&g.child.width>0&&g.child.height>0&&document.elementFromPoint(g.child.x+g.child.width/2,g.child.y+g.child.height/2)?.closest('[data-menu-category-child]')===item('One')};
 window.scrollTo({top:0,behavior:'instant'});await wait(()=>scrollY===0&&positioned(),'collapsed right-start placement and native hit');const collapsedGeometry=geometry();
 window.scrollTo({top:120,behavior:'instant'});await wait(()=>scrollY>0&&scrollY!==collapsedGeometry.scroll,'native page really scrolls');await wait(positioned,'scrolled collapsed flyout remains anchored and hit');const scrolledGeometry=geometry();
 window.scrollTo({top:0,behavior:'instant'});await wait(()=>scrollY===0&&positioned(),'restored collapsed flyout placement');
 checks.push('native top-layer right-start/3px placement and child pointer hit before/after real document scroll');
 await focus('Category');assert(key(item('Category'),'Enter'),'repeated collapsed parent activation prevented');await sleep(60);assert(opened('Category')&&document.activeElement===item('Category'),'repeated present retains parent focus');
 await go('Category','ArrowDown','One');await go('Category','ArrowUp','Wrapped child');await go('Category','Home','One');await go('Category','End','Wrapped child');
 await go('Category','Escape','Category',false);await wait(()=>!opened('Category'),'parent Escape closes open collapsed flyout');await go('Category','Enter','One');
 await go('One','ArrowDown','Two');await go('Two','End','Wrapped child');
 await go('One','ArrowLeft','One',false);await go('One','ArrowRight','One',false);assert(opened('Category'),'collapsed horizontal keys unchanged');
 assert(!key(item('One'),'ArrowDown',false,true),'collapsed Alt+Down stays native');await sleep(40);assert(document.activeElement===item('One')&&opened('Category'),'collapsed Alt+Down retains focus and flyout');
 await focus('One');assert(key(item('One'),'Enter'),'collapsed Enter canceled separately');assert(document.activeElement===item('Category'),'collapsed Enter synchronously returns parent');await wait(()=>!opened('Category'),'collapsed Enter closes');
 await go('Category',' ','One');await focus('One');assert(key(item('One'),' '),'collapsed Space canceled separately');assert(document.activeElement===item('Category'),'collapsed Space returns parent');await wait(()=>!opened('Category'),'collapsed Space closes');
 await go('Category','Enter','One');await go('One','Escape','Category',false);await wait(()=>!opened('Category'),'collapsed Escape closes');
 assert(selected().length===0,'collapsed Enter and Space do not select a child');
 await go('Category','Enter','One');item('One').click();await wait(()=>!opened('Category'),'collapsed pointer closes');await wait(()=>document.activeElement===document.body,'collapsed pointer hide releases child focus');assert(selected().length===1,'collapsed pointer selects once');
 await go('Category','Enter','One');item('Second category').click();await wait(()=>opened('Second category')&&!opened('Category'),'single flyout ownership');
 checks.push('collapsed present/focus, disabled skip/endpoints, separate Enter/Space cancellation/closure, Escape and single ownership');
 await fresh('expanded');await focus('Category');key(item('Category'),'Enter');command('mount');await wait(()=>!menu(),'unmount pending category focus');await sleep(80);assert(document.activeElement===get('mount'),'pending category focus never steals outside focus');command('mount');await wait(()=>ready(),'remount new category observer');await sleep(80);assert(document.activeElement===get('mount'),'old request does not target new owner');
 checks.push('pending opening/unmount guard and new observer ownership');
 return {ok:true,checks,collapsedGeometry,scrolledGeometry,input:'synthetic composed cancelable DOM keys; real native focus and cancellation',limits:['native Tab default exit and Enter parent click require browser replay','physical input/AT/platform/Brand equivalence unproved','parent Down intermediate-focus timing remains renderer-dependent'],renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error)}}
"#;

const CAPABILITIES: &str = r#"
await dioxus.recv();
return {ok:true,capabilities:{anchorName:CSS.supports('anchor-name','--ix-test'),anchorTop:CSS.supports('top','anchor(top)'),anchorRight:CSS.supports('left','anchor(right)'),popover:typeof HTMLElement.prototype.showPopover==='function'}};
"#;

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(55)).await;
            eprintln!("G04.4 native probe timed out");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let desktop=desktop.clone();spawn(async move {
                let script=if std::env::var("DIOXUS_IX_PROBE_SCOPE").as_deref()==Ok("capabilities"){CAPABILITIES}else{VERIFY};
                let query=dom_query::eval(script);let _=query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"]==true => {println!("G04.4 native category checks: {result}");desktop.close();}
                    result => {eprintln!("G04.4 native category probe failed: {result:?}");std::process::exit(1);}
                }
            });
        },menu_category_review::MenuCategoryReview {} div {style:"height:24rem", "aria-hidden":"true"} }
    } }
}
