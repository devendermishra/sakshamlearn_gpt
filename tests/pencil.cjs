const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(require('node:path').join(__dirname,'../src/pencil.js'),'utf8');
function workspace(initial=[]) {
 const events=()=>({handlers:{},addEventListener(type,fn){this.handlers[type]=fn;},fire(type,event={}){this.handlers[type]?.({preventDefault(){},...event});}});
 const context={stroke(){},fill(){},beginPath(){},moveTo(){},lineTo(){},arc(){}};
 let bounds={left:0,top:0,width:1000,height:200};
 const canvas={...events(),getContext:()=>context,getBoundingClientRect:()=>bounds,setPointerCapture(){}};
 const image=events(), undo=events(), clear=events();
 const root={isConnected:true,querySelector:q=>q==='canvas'?canvas:q.includes('undo')?undo:q.includes('clear')?clear:image};
 const document={...events(),body:{},getElementById:()=>root};
 const window={...events(),devicePixelRatio:1};
 const messages=[];let mutation,resize;
 const scope=vm.createContext({config:{initial,allowMouse:false},document,window,AbortController,performance,
  dioxus:{send:value=>messages.push(structuredClone(value))},
  ResizeObserver:class{constructor(fn){resize=fn;}observe(){}disconnect(){}},
  MutationObserver:class{constructor(fn){mutation=fn;}observe(){}disconnect(){}},
 });
 const pending=vm.runInContext('(async()=>{'+source+'})()',scope);
 return {canvas,undo,clear,messages,resize(w,h){bounds={...bounds,width:w,height:h};resize();},async close(){root.isConnected=false;mutation();await pending;}};
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
