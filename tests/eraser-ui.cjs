const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(path.join(path.dirname(process.execPath),'../node_modules/playwright'));
const root=path.resolve(__dirname,'..');
const source=name=>fs.readFileSync(path.join(root,'src',name),'utf8');
const strokes=[[[.1,.2],[.9,.2]],[[.1,.4],[.9,.4]],[[.1,.8],[.9,.8]],[[.95,.95]]];
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true});
 try {
  const page=await browser.newPage();
  const errors=[];page.on('pageerror',error=>errors.push(error.message));
  for(const engine of ['pencil','ink']) {
   async function mount(initial=strokes,extra={}) {
    await page.goto('about:blank');
    await page.setContent(engine==='pencil'?
     '<style>canvas{width:1000px;height:200px}</style><div id="pencil-workspace"><button data-pencil-action="pen">Pen</button><button data-pencil-action="eraser">Eraser</button><button data-pencil-action="undo">Undo</button><button data-pencil-action="clear">Clear</button><div class="pencil-surface"><img><canvas></canvas></div></div>':
     '<style>canvas{width:1000px;height:200px}</style><section id="ink"><p class="instructions"></p><div class="ink-grid"></div><p role="status"></p><button data-ink-tool="pen">Pen</button><button data-ink-tool="eraser">Eraser</button><button data-undo>Undo</button><button data-clear>Clear</button><button data-done>Done</button></section>');
    const config=engine==='pencil'?{initial,allowMouse:false}:{id:'ink',model:'',initial:[initial],readonly:false,square:false,plain:true,englishPaper:false,lines:1,allowMouse:false,english:true,...extra};
    const scripts=engine==='pencil'?source('pencil.js'):['trace.js','trace_guide.js','ink.js'].map(source).join('\n');
    await page.evaluate(({config,scripts})=>{
     window.messages=[];HTMLCanvasElement.prototype.setPointerCapture=()=>{};
     window.lifetime=new Function('config','dioxus',`return (async()=>{${scripts}})()`)(config,{send:value=>messages.push(structuredClone(value))});
    },{config,scripts});
    await page.locator('canvas').first().waitFor();
   }
   const saved=()=>page.evaluate(engine=>{const message=messages.at(-1);return engine==='pencil'?message:message?.ink[0];},engine);
   async function gesture(points,type='pen',index=0) {
    await page.evaluate(({points,type,index})=>{
     const canvas=document.querySelectorAll('canvas')[index],r=canvas.getBoundingClientRect();
     const fire=(name,p)=>canvas.dispatchEvent(new PointerEvent(name,{pointerType:type,pointerId:1,button:0,clientX:r.left+p[0]*r.width,clientY:r.top+p[1]*r.height}));
     fire('pointerdown',points[0]);for(const p of points.slice(1))fire('pointermove',p);fire('pointerup',points.at(-1));
    },{points,type,index});
   }
   await mount();await page.getByRole('button',{name:'Eraser',exact:true}).click();
   await gesture([[.5,.1],[.5,.5]]);
   assert.deepEqual(await saved(),strokes.slice(2),engine+' swept segment erases crossed strokes');
   await page.getByRole('button',{name:'Undo',exact:true}).click();assert.deepEqual(await saved(),strokes,engine+' one undo restores entire gesture in order');
   await page.getByRole('button',{name:'Undo',exact:true}).click();assert.deepEqual(await saved(),strokes.slice(0,3),engine+' earlier draw undo remains valid');
   await gesture([[.5,.2]],'touch');assert.deepEqual(await saved(),strokes.slice(0,3),engine+' rejects touch');
   await gesture([[.5,.2]],'mouse');assert.deepEqual(await saved(),strokes.slice(0,3),engine+' rejects disabled mouse');
   const before=await page.evaluate(()=>messages.length);await gesture([[.5,.6]]);assert.equal(await page.evaluate(()=>messages.length),engine==='pencil'?before:before+1);
   await mount();await page.getByRole('button',{name:'Eraser',exact:true}).click();
   await gesture([[.95,.95]]);assert.deepEqual(await saved(),strokes.slice(0,3),engine+' single point');
   await page.getByRole('button',{name:'Pen',exact:true}).click();await gesture([[.2,.7],[.3,.7]]);assert.equal((await saved()).length,4);
   const erased=await saved();await mount(erased);await page.getByRole('button',{name:'Eraser',exact:true}).click();
   await page.locator('canvas').first().evaluate(c=>{c.style.width='500px';c.style.height='100px';});
   await gesture([[.5,.27]]);assert.equal((await saved()).some(s=>s[0][1]===.2),false,engine+' radius remains eight CSS pixels after resize');
   // Interrupting an eraser gesture flushes the changed draft on pause.
   await mount();await page.getByRole('button',{name:'Eraser',exact:true}).click();
   await page.locator('canvas').first().evaluate(c=>{const r=c.getBoundingClientRect();c.dispatchEvent(new PointerEvent('pointerdown',{pointerType:'pen',pointerId:9,button:0,clientX:r.left+r.width*.5,clientY:r.top+r.height*.2}));window.dispatchEvent(new Event('saksham:pause'));});
   assert.deepEqual(await saved(),strokes.slice(1),engine+' interrupted erasure saves');
   await page.evaluate(()=>window.dispatchEvent(new Event('saksham:resume')));
   await page.getByRole('button',{name:'Undo',exact:true}).click();assert.deepEqual(await saved(),strokes);
   if(engine==='ink') {
    // Eraser capture can sweep into adjacent writing cells, with one global undo.
    await mount([], {plain:false,model:'',lines:2,initial:[strokes,strokes]});
    await page.getByRole('button',{name:'Eraser',exact:true}).click();
    const initial=await page.evaluate(()=>[...document.querySelectorAll('canvas')].map(c=>c.toDataURL()));
    await page.evaluate(()=>{
     const [a,b]=document.querySelectorAll('canvas'),r=a.getBoundingClientRect(),s=b.getBoundingClientRect();
     const fire=(name,x,y)=>a.dispatchEvent(new PointerEvent(name,{pointerType:'pen',pointerId:2,button:0,clientX:x,clientY:y}));
     fire('pointerdown',r.left+r.width*.5,r.top+r.height*.2);fire('pointermove',s.left+s.width*.5,s.top+s.height*.4);fire('pointerup',s.left+s.width*.5,s.top+s.height*.4);
    });
    assert.ok(await page.evaluate(()=>messages.at(-1).ink.every(cell=>cell.length<4)),'sweep edits adjacent cells');
    await page.getByRole('button',{name:'Undo',exact:true}).click();
    assert.deepEqual(await page.evaluate(()=>messages.at(-1).ink),[strokes,strokes]);
    assert.deepEqual(await page.evaluate(()=>[...document.querySelectorAll('canvas')].map(c=>c.toDataURL())),initial,'undo restores rendered ink');
    await mount([], {model:'A',plain:false,lineTrace:true,initial:[],englishPaper:true,lines:1});
    const model=await page.locator('canvas').first().evaluate(c=>c.toDataURL());
    await gesture([[.2,.3],[.4,.3]]);await page.getByRole('button',{name:'Eraser',exact:true}).click();await gesture([[.3,.3]]);
    assert.equal(await page.locator('canvas').first().evaluate(c=>c.toDataURL()),model,'tracing guide survives erasure');
    await page.locator('[data-done]').evaluate(b=>b.disabled=true);await page.locator('[data-done]').evaluate(b=>b.onclick());
    assert.equal(await page.evaluate(()=>messages.some(m=>m.done)),false,'disabled submission callback cannot finish');
    await mount(strokes,{readonly:true});await page.getByRole('button',{name:'Eraser',exact:true}).click();await gesture([[.5,.2]]);
    assert.equal(await page.evaluate(()=>messages.length),0,'read-only sample rejects erasing');
   }
   console.log(engine+': swept erasing, undo, input rejection, resize, persistence, and lifecycle checks passed.');
  }
  assert.deepEqual(errors,[]);
 } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
