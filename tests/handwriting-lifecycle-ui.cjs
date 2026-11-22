const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(path.join(path.dirname(process.execPath), '../node_modules/playwright'));
const scripts=['trace.js','ink.js'].map(name=>fs.readFileSync(path.join(__dirname,'../src',name),'utf8')).join('\n');
(async()=>{
 const browser=await chromium.launch({channel:'msedge',headless:true});
 try {
  const page=await browser.newPage();
  await page.setContent('<main></main>');
  await page.evaluate(()=>{
   window.messages=[];
   HTMLCanvasElement.prototype.setPointerCapture=()=>{};
   window.mount=()=>{document.querySelector('main').innerHTML='<section id="ink"><p class="instructions"></p><div class="ink-grid"></div><p role="status"></p><button data-undo>Undo</button><button data-clear>Clear</button></section>';};
   mount();
  });
  const config={id:'ink',model:'',initial:[],readonly:false,square:false,plain:true,englishPaper:false,lines:1,allowMouse:false,english:true};
  async function mount(){await page.evaluate(({config,scripts})=>{window.lifetime=new Function('config','dioxus',`return (async()=>{${scripts}})()`)(config,{send:value=>messages.push(structuredClone(value))});},{config,scripts});await page.waitForSelector('#ink canvas');}
  async function stroke(){await page.evaluate(()=>{const canvas=document.querySelector('#ink canvas');canvas.dispatchEvent(new PointerEvent('pointerdown',{pointerType:'pen',pointerId:1,button:0,clientX:10,clientY:10}));canvas.dispatchEvent(new PointerEvent('pointermove',{pointerType:'pen',pointerId:1,clientX:20,clientY:20}));});}
  await mount();await stroke();
  await page.evaluate(()=>window.dispatchEvent(new Event('saksham:pause')));
  assert.equal(await page.evaluate(()=>messages.length),1);
  assert.equal(await page.evaluate(()=>messages[0].ink[0][0].length),2);
  for(let i=0;i<5;i++)await page.evaluate(()=>{window.dispatchEvent(new Event('saksham:resume'));window.dispatchEvent(new Event('saksham:pause'));});
  assert.equal(await page.evaluate(()=>messages.length),1);
  await page.evaluate(()=>window.dispatchEvent(new Event('saksham:resume')));await stroke();
  // Capture-phase save precedes the button's navigation handler/unmount.
  await page.evaluate(()=>{document.querySelector('[data-clear]').addEventListener('click',()=>{window.beforeNavigation=messages.length;document.querySelector('#ink').remove();},{capture:true});document.querySelector('[data-clear]').click();});
  assert.equal(await page.evaluate(()=>beforeNavigation),2);
  await page.evaluate(()=>lifetime);
  await page.evaluate(()=>mount());await mount();await stroke();
  await page.evaluate(()=>window.dispatchEvent(new Event('saksham:pause')));
  assert.equal(await page.evaluate(()=>messages.filter(m=>m.ink.some(c=>c.length)).length),3);
  await page.evaluate(()=>document.querySelector('#ink').remove());await page.evaluate(()=>lifetime);
  console.log('Handwriting saves active strokes before pause/navigation; repeated lifecycle events and remounts do not duplicate saves.');
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
