const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const source = fs.readFileSync(path.join(__dirname,'../src/day_clock.js'),'utf8');
function clock(start) {
 const events=()=>({handlers:{},addEventListener(type,fn,{signal}){this.handlers[type]=fn;signal.addEventListener('abort',()=>delete this.handlers[type]);},fire(type){this.handlers[type]?.();}});
 const root={isConnected:true},document={...events(),body:{},hidden:false,querySelector:()=>root},window=events();
 let now=Date.parse(start),nextId=0,mutation,count=0;const timers=new Map();
 const context=vm.createContext({document,window,AbortController,Date:{now:()=>now},dioxus:{send:()=>count++},
  setTimeout:(fn,ms)=>{timers.set(++nextId,{fn,at:now+ms});return nextId;},clearTimeout:id=>timers.delete(id),
  MutationObserver:class{constructor(fn){mutation=fn;}observe(){}disconnect(){}},
 });
 const lifetime=vm.runInContext('(async()=>{'+source+'})()',context);
 return {document,window,timers,get count(){return count;},advance(ms){now+=ms;for(const [id,t] of [...timers])if(t.at<=now){timers.delete(id);t.fn();}},async close(){root.isConnected=false;mutation();await lifetime;}};
}
test('one timer rolls over at India midnight, independent of host timezone',async()=>{
 const c=clock('2026-11-22T18:29:59Z');assert.equal(c.count,1);assert.equal(c.timers.size,1);
 c.advance(999);assert.equal(c.count,1);c.advance(51);assert.equal(c.count,2);
 assert.equal(c.timers.size,1);assert.equal([...c.timers.values()][0].at,Date.parse('2026-11-23T18:30:00.050Z'));
 await c.close();assert.equal(c.timers.size,0);
});
test('hidden and native pause cancel timers; resume refreshes after multiple missed days without listener growth',async()=>{
 const c=clock('2026-11-22T10:00:00Z');const count=c.count;
 c.document.hidden=true;c.document.fire('visibilitychange');assert.equal(c.timers.size,0);
 c.advance(3*86400000);assert.equal(c.count,count);
 c.document.hidden=false;c.document.fire('visibilitychange');assert.equal(c.count,count+1);
 for(let i=0;i<10;i++){c.window.fire('saksham:pause');assert.equal(c.timers.size,0);c.window.fire('focus');assert.equal(c.timers.size,0);c.window.fire('saksham:resume');assert.equal(c.timers.size,1);}
 assert.equal(Object.keys(c.window.handlers).length,3);
 await c.close();assert.equal(c.timers.size,0);assert.equal(Object.keys(c.window.handlers).length,0);
});
