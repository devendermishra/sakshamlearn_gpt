const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {chromium} = require(path.join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright'));
const root = path.resolve(__dirname, '..');
const output = path.join(root, 'target/english-spacing-review');
const button = (page, name) => page.getByRole('button', {name, exact:true});
async function run(profile, check) {
 const server=net.createServer();
 await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const port=server.address().port;
 await new Promise(r=>server.close(r));
 const endpoint='http://127.0.0.1:'+port;
 const webview=await fs.mkdtemp(path.join(profile,'webview-'));
 const child=spawn(path.join(root,'target/debug/saksham.exe'),[],{cwd:profile,windowsHide:true,stdio:'ignore',env:{...process.env,LOCALAPPDATA:profile,WEBVIEW2_USER_DATA_FOLDER:webview,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port='+port}});
 const closed=new Promise(r=>child.once('exit',r));
 let browser;
 try {
  for(let i=0;i<160;i++) {
   if(child.exitCode!==null) throw Error('Preview exited early');
   if(await fetch(endpoint+'/json/version').then(r=>r.ok,()=>false)) break;
   await new Promise(r=>setTimeout(r,250));
  }
  browser=await chromium.connectOverCDP(endpoint);
  const context=browser.contexts()[0];
  const page=context.pages()[0]||await context.waitForEvent('page');
  page.setDefaultTimeout(15000);
  await button(page,'Daily exercise plans · रोज़ के अभ्यास').waitFor();
  // Block internet access while retaining the desktop shell's loopback IPC.
  await context.route('**/*', route=>{
   const url=new URL(route.request().url());
   const local=url.hostname==='dioxus.index.html'||url.hostname==='localhost'||url.hostname==='127.0.0.1'||url.hostname.endsWith('.localhost');
   return /^https?:$/.test(url.protocol)&&!local ? route.abort() : route.continue();
  });
  try { await check(page); }
  catch(error) {
   await page.screenshot({path:path.join(output,'failure.png'),fullPage:true}).catch(()=>{});
   console.error((await page.locator('body').innerText()).slice(0,5000));
   throw error;
  }
 } finally {
  await browser?.close().catch(()=>{});
  if(child.exitCode===null) child.kill();
  await closed;
 }
}
async function waitSave(file,predicate) {
 for(let i=0;i<80;i++) {
  const data=await fs.readFile(file,'utf8').then(JSON.parse,()=>null);
  if(data&&predicate(data)) return data;
  await new Promise(r=>setTimeout(r,100));
 }
 throw Error('Expected saved daily work: '+file);
}
const open=async page=>{
 await button(page,'Daily exercise plans · रोज़ के अभ्यास').click();
 await page.getByRole('heading',{name:'Daily exercise plans · रोज़ के अभ्यास',exact:true}).waitFor();
};
(async()=>{
 await fs.mkdir(output,{recursive:true});
 for(const scale of [96/25.4,6]) {
  const profile=await fs.mkdtemp(path.join(root,'target/english-spacing-ui-'));
  const folder=path.join(profile,'Saksham/learner-work');
  await fs.mkdir(folder,{recursive:true});
  await fs.writeFile(path.join(profile,'Saksham/progress.json'),JSON.stringify({scale:4,vertical_scale:scale}));
  const rough=path.join(folder,'rough-en.json');
  const old=[[[[.2,30/236],[.4,30/236]]]];
  await fs.writeFile(rough,JSON.stringify(old));
  await run(profile,async page=>{
   await page.locator('button[data-subject="en"]').click();
   await page.getByRole('button',{name:/Rough Sheet/}).click();
   const canvas=page.locator('#rough-ink canvas');await canvas.waitFor();
   const geometry=await page.locator('#rough-ink').evaluate(el=>{
    const grid=el.querySelector('.ink-grid'),style=getComputedStyle(grid);
    return {height:parseFloat(style.getPropertyValue('--height')),gap:parseFloat(style.getPropertyValue('--english-gap')),total:grid.getBoundingClientRect().height};
   });
   assert.ok(Math.abs(geometry.height/scale/4-4)<.001);
   assert.ok(Math.abs((geometry.height/4+geometry.gap)/scale-5.6)<.001);
   assert.ok(Math.abs(geometry.total/scale-209.6)<.02);
   const aligned=await canvas.evaluate(c=>{
    const ctx=c.getContext('2d'),y=Math.round(c.height*27.6/209.6),x=Math.round(c.width*.3);
    return ctx.getImageData(x,y,1,1).data[3]>0;
   });assert.ok(aligned,'legacy ink stays on the second baseline');
   await canvas.scrollIntoViewIfNeeded();const box=await canvas.boundingBox();
   await page.mouse.move(box.x+box.width*.5,box.y+Math.min(70,box.height*.1));
   await page.mouse.down();await page.mouse.move(box.x+box.width*.6,box.y+Math.min(70,box.height*.1),{steps:5});await page.mouse.up();
   const saved=await waitSave(rough,p=>p[0]?.length===2);
   assert.ok(Math.abs(saved[0][0][0][1]-30/236)<1e-6,'canonical coordinates retained');
   await button(page,'Undo last stroke').click();await waitSave(rough,p=>p[0]?.length===1);
   await page.screenshot({path:path.join(output,'rough-'+scale.toFixed(2)+'.png'),fullPage:true});
   await page.locator('header > button').first().click();
   await page.getByRole('button',{name:/Tracing Practice/}).click();
   await button(page,'Short sentences').click();
   await page.locator('.ink-english canvas').first().waitFor();
   const gap=await page.locator('.ink-english .ink-grid').evaluate(el=>parseFloat(getComputedStyle(el).rowGap));
   assert.ok(Math.abs(gap/scale-1.6)<.001);
   await page.screenshot({path:path.join(output,'sentence-'+scale.toFixed(2)+'.png'),fullPage:true});
  });
  await run(profile,async page=>{
   await page.locator('button[data-subject="en"]').click();
   await page.getByRole('button',{name:/Rough Sheet/}).click();
   await page.locator('#rough-ink canvas').waitFor();
   assert.ok(Math.abs((JSON.parse(await fs.readFile(rough,'utf8')))[0][0][0][1]-30/236)<1e-6);
  });
 }
 // Exercise the actual multiline renderer and read-only sample path with the app CSS.
 const scripts=(await Promise.all(['trace.js','trace_guide.js','ink.js'].map(name=>fs.readFile(path.join(root,'src',name),'utf8')))).join('\n');
 const css=await fs.readFile(path.join(root,'src/style.css'),'utf8');
 const font=await fs.readFile(path.join(root,'assets/PatrickHand-Regular.ttf'),'base64');
 const browser=await chromium.launch({channel:'msedge',headless:true});
 try {
  const page=await browser.newPage({viewport:{width:1280,height:900}});
  for(const readonly of [false,true]) {
   await page.setContent('<style>'+css+'@font-face{font-family:"Patrick Hand";src:url(data:font/ttf;base64,'+font+')}</style><section id="multiline" class="ink ink-english" style="--height:80px;--cell:600px;--columns:1;--english-gap:8px"><p class="instructions"></p><div class="ink-grid"></div><p role="status"></p><button data-undo>Undo</button><button data-clear>Clear</button></section>');
   const config={id:'multiline',model:'We play at home.\nWe walk to school.',initial:[[],[[[.2,26/32],[.4,26/32]]]],readonly,square:false,plain:false,englishPaper:true,fullSheet:false,lines:1,allowMouse:true,english:true,sentenceWidth:600};
   await page.evaluate(({config,scripts})=>{
    const dioxus={send:()=>{}};
    // Editable pads keep their channel open until removed; wait for the canvas instead.
    void new Function('config','dioxus','return (async()=>{'+scripts+'})()')(config,dioxus);
   },{config,scripts});
   await page.locator('#multiline canvas').nth(1).waitFor();
   const result=await page.evaluate(()=>{
    const el=document.querySelector('#multiline'),canvases=el.querySelectorAll('canvas');
    const copy=canvases[1],ctx=copy.getContext('2d');
    const alpha=ctx.getImageData(Math.round(copy.width*.3),Math.round(copy.height*27.6/33.6),1,1).data[3];
    return {rows:Number(el.style.getPropertyValue('--sentence-lines')),height:copy.getBoundingClientRect().height,
      pitch:canvases[1].getBoundingClientRect().top-canvases[0].getBoundingClientRect().top,alpha};
   },{config,scripts});
   assert.equal(result.rows,2);
   assert.equal(result.height,168);
   assert.equal(result.pitch,176);
   assert.ok(result.alpha>0,'multiline saved ink remains on its baseline, readonly='+readonly);
   await page.screenshot({path:path.join(output,'multiline-'+readonly+'.png'),fullPage:true});
  }
 } finally {await browser.close();}
 console.log('English spacing, legacy ink, multiline tracing, read-only samples, undo and restart passed.');
})().catch(e=>{console.error(e);process.exitCode=1;});
