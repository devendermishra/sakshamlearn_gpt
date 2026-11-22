const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(require('node:path').join(__dirname,'../src/pencil.js'),'utf8');
function workspace(initial=[], ratio=1) {
 const metrics={lines:0,resets:0};
 let now=0,frameId=0;const frames=new Map();
 const events=()=>({handlers:{},addEventListener(type,fn,{signal}={}){this.handlers[type]=fn;signal?.addEventListener('abort',()=>delete this.handlers[type]);},fire(type,event={}){this.handlers[type]?.({preventDefault(){},...event});}});
 const context={stroke(){},fill(){},beginPath(){},moveTo(){},lineTo(){metrics.lines++;},arc(){},clearRect(){}};
 let bounds={left:0,top:0,width:1000,height:200};
 const canvas={...events(),getContext:()=>context,getBoundingClientRect:()=>bounds,setPointerCapture(){}};
 let width=300,height=150;
 Object.defineProperties(canvas,{width:{get:()=>width,set:v=>{width=v;metrics.resets++;}},height:{get:()=>height,set:v=>{height=v;metrics.resets++;}}});
 const image=events(), undo=events(), clear=events();
 const root={isConnected:true,querySelector:q=>q==='canvas'?canvas:q.includes('undo')?undo:q.includes('clear')?clear:image};
 const document={...events(),body:{},getElementById:()=>root};
 const window={...events(),devicePixelRatio:ratio};
 const messages=[];let mutation,resize;
 const scope=vm.createContext({config:{initial,allowMouse:false},document,window,AbortController,performance:{now:()=>now},
  requestAnimationFrame:fn=>{frames.set(++frameId,fn);return frameId;},cancelAnimationFrame:id=>frames.delete(id),
  dioxus:{send:value=>messages.push(structuredClone(value))},
  ResizeObserver:class{constructor(fn){resize=fn;}observe(){}disconnect(){}},
  MutationObserver:class{constructor(fn){mutation=fn;}observe(){}disconnect(){}},
 });
 const pending=vm.runInContext('(async()=>{'+source+'})()',scope);
 return {canvas,undo,clear,messages,metrics,window,document,frames, tick(ms){now+=ms;},frame(){for(const [id,fn] of [...frames]){frames.delete(id);fn();}},resize(w,h){bounds={...bounds,width:w,height:h};resize();},async close(){root.isConnected=false;mutation();await pending;}};
}
const pen=(id,x,y)=>({pointerType:'pen',pointerId:id,clientX:x,clientY:y});
test('pencil rejects touch and mouse, and palm release does not interrupt the pen',async()=>{
 const w=workspace();
 w.canvas.fire('pointerdown',{pointerType:'touch',pointerId:1});
 w.canvas.fire('pointerdown',{pointerType:'mouse',pointerId:2,button:0});
 assert.equal(w.messages.length,0);
 w.canvas.fire('pointerdown',pen(3,100,50));
 w.canvas.fire('pointerdown',pen(4,200,50));
 w.canvas.fire('pointerup',{pointerType:'touch',pointerId:1});
 w.canvas.fire('pointermove',pen(3,500,100));
 w.canvas.fire('pointerup',pen(3,500,100));
 assert.equal(w.messages.at(-1).length,1);
 assert.deepEqual(w.messages.at(-1)[0],[[.1,.25],[.5,.5]]);
 await w.close();
});
test('cancelled strokes persist; resize, undo and clear preserve normalized coordinates',async()=>{
 const w=workspace([[[.2,.3],[.4,.5]]]);
 w.resize(2000,400);
 w.canvas.fire('pointerdown',pen(1,1000,200));
 w.canvas.fire('pointermove',pen(1,1800,360));
 w.canvas.fire('pointercancel',pen(1,1800,360));
 assert.deepEqual(w.messages.at(-1)[1],[[.5,.5],[.9,.9]]);
 w.undo.fire('click');assert.equal(w.messages.at(-1).length,1);
 assert.deepEqual(w.messages.at(-1)[0],[[.2,.3],[.4,.5]]);
 w.clear.fire('click');assert.deepEqual(w.messages.at(-1),[]);
 await w.close();
});

test('movement draws linearly without resetting the canvas or saving per frame',async()=>{
 const w=workspace([Array.from({length:1000},(_,i)=>[i/1000,.2])]);
 const lines=w.metrics.lines,resets=w.metrics.resets;
 w.canvas.fire('pointerdown',pen(1,10,10));
 for(let i=0;i<1000;i++){
  w.canvas.fire('pointermove',pen(1,i,100));w.frame();
 }
 assert.equal(w.messages.length,0);
 assert.equal(w.metrics.resets,resets);
 assert.equal(w.metrics.lines-lines,1000);
 w.canvas.fire('pointerup',pen(1,999,100));
 assert.equal(w.messages.length,1);
 assert.equal(w.messages[0][1].length,1001);
 await w.close();
});
test('empty coalesced events fall back to the event; long strokes checkpoint and unmount flushes',async()=>{
 const w=workspace();w.canvas.fire('pointerdown',pen(1,100,50));w.tick(2100);
 w.canvas.fire('pointermove',{...pen(1,500,100),getCoalescedEvents:()=>[]});
 assert.equal(w.messages.length,1);
 assert.equal(w.messages[0][0].length,2);
 w.canvas.fire('pointermove',pen(1,600,100));
 await w.close();
 assert.equal(w.messages.length,2);
 assert.equal(w.messages[1][0].length,3);
});
test('canvas backing storage is bounded at high pixel ratios and zoom',async()=>{
 const w=workspace([],3);w.resize(4000,4000);
 assert.ok(w.canvas.width*w.canvas.height<=4*1024*1024);
 await w.close();
});

test('native pause saves once, suspends drawing and resumes without listener growth',async()=>{
 const w=workspace();w.canvas.fire('pointerdown',pen(1,100,50));
 w.canvas.fire('pointermove',pen(1,200,100));
 w.window.fire('saksham:pause');assert.equal(w.messages.length,1);assert.equal(w.frames.size,0);
 w.canvas.fire('pointerdown',pen(2,300,100));w.canvas.fire('pointermove',pen(2,400,100));
 assert.equal(w.messages.length,1);assert.equal(w.frames.size,0);
 w.window.fire('saksham:pause');assert.equal(w.messages.length,1);
 w.window.fire('saksham:resume');w.canvas.fire('pointerdown',pen(3,400,100));
 w.document.fire('click');assert.equal(w.messages.length,2);
 await w.close();assert.equal(Object.keys(w.window.handlers).length,0);
});
