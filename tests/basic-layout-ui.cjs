const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {chromium} = require(path.join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright'));
const root = path.resolve(__dirname, '..');
const output = path.join(root, 'target', 'basic-layout-review');
const button = (page, name) => page.getByRole('button', {name, exact:true});
async function port() {
 const server = net.createServer();
 await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
 const result = server.address().port;
 await new Promise(resolve => server.close(resolve)); return result;
}
async function run(profile, check) {
 const endpoint = 'http://127.0.0.1:' + await port();
 const processProfile = await fs.mkdtemp(path.join(profile, 'webview-'));
 const child = spawn(path.join(root, 'target/debug/saksham.exe'), [], {
  cwd:profile, windowsHide:true, stdio:'ignore',
  env:{...process.env, LOCALAPPDATA:profile, WEBVIEW2_USER_DATA_FOLDER:processProfile,
   WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port='+new URL(endpoint).port},
 });
 const closed = new Promise(resolve=>child.once('exit',resolve));
 let browser;
 try {
  for(let i=0;i<160;i++) {
   if(child.exitCode!==null) throw Error('Preview exited early');
   if(await fetch(endpoint+'/json/version').then(r=>r.ok,()=>false)) break;
   await new Promise(resolve=>setTimeout(resolve,250));
  }
  browser=await chromium.connectOverCDP(endpoint);
  const context=browser.contexts()[0];
  const page=context.pages()[0] || await context.waitForEvent('page');
  page.setDefaultTimeout(15000);
  await check(page);
 } finally {
  await browser?.close().catch(()=>{});
  if(child.exitCode===null) child.kill();
  await closed;
 }
}
async function saved(file) {return JSON.parse(await fs.readFile(file,'utf8'));}
async function waitForSave(file, predicate) {
 for(let i=0;i<60;i++) {
  const value=await saved(file).catch(()=>null);
  if(value && predicate(value)) return value;
  await new Promise(r=>setTimeout(r,100));
 }
 throw Error('Expected saved state not found: '+file);
}
async function stroke(page) {
 const canvas=page.locator('#pencil-workspace canvas');
 await canvas.scrollIntoViewIfNeeded();
 const box=await canvas.boundingBox();
 await page.mouse.move(box.x+box.width*.12,box.y+box.height*.3);
 await page.mouse.down();
 await page.mouse.move(box.x+box.width*.7,box.y+box.height*.3,{steps:12});
 await page.mouse.up();
}
(async()=>{
 await fs.mkdir(output,{recursive:true});
 const profile=await fs.mkdtemp(path.join(root,'target/basic-layout-ui-'));
 const basic=path.join(profile,'Saksham/learner-work/basic-pencil.json');
 await run(profile,async page=>{
  await button(page,'Basic').click();
  await page.getByRole('heading',{name:'Basic',exact:true}).waitFor();
  await button(page,'+').click();
  for(const viewport of [{width:1280,height:800},{width:1024,height:768},{width:800,height:600}]) {
   await page.setViewportSize(viewport);
   for(let number=9;number<=14;number++) {
    await button(page,'All pages').click();
    await button(page,'Page '+number).click();
    await page.locator('.pencil-surface img').evaluate(i=>i.decode());
    await page.waitForFunction(()=>{
     const section=document.querySelector('.basic-pattern-mode');
     const frame=section.querySelector('.pencil-frame').getBoundingClientRect();
     const scroll=section.querySelector('.pencil-scroll').getBoundingClientRect();
     return frame.width>100 && frame.height>100 && frame.top>=scroll.top && frame.bottom<=scroll.bottom && frame.left>=scroll.left && frame.right<=scroll.right;
    });
    const metrics=await page.evaluate(()=>{
     const section=document.querySelector('.basic-pattern-mode');
     const scroll=section.querySelector('.pencil-scroll');
     const frame=section.querySelector('.pencil-frame').getBoundingClientRect();
     return {bottom:section.getBoundingClientRect().bottom, height:innerHeight, frameHeight:frame.height, scrollHeight:scroll.clientHeight, overflow:scroll.scrollHeight-scroll.clientHeight, pageOverflow:document.documentElement.scrollHeight-innerHeight};
    });
    assert.ok(metrics.pageOverflow<=1,JSON.stringify({viewport,number,metrics}));
    assert.ok(metrics.overflow<=1,JSON.stringify({viewport,number,metrics}));
    assert.ok(metrics.frameHeight>viewport.height*.60,JSON.stringify({viewport,number,metrics}));
    const instruction=await page.locator('.visual-instruction p').boundingBox();
    const reference=await page.locator('.pencil-activity > details').boundingBox();
    assert.ok(instruction.y+instruction.height<=reference.y+1,'Sidebar overlap '+JSON.stringify({viewport,instruction,reference}));
    assert.equal(await button(page,'+').count(),0);
    if(number===9 && viewport.width===1280) {
     const frame=await page.locator('.pencil-frame').boundingBox();
     await page.mouse.move(frame.x+frame.width*.3,frame.y+frame.height*.4);
     await page.mouse.down();
     await page.mouse.move(frame.x+frame.width*.6,frame.y+frame.height*.5,{steps:5});
     await page.mouse.up();
     const state=await waitForSave(basic,p=>p.strokes['62']?.length===1);
     const first=state.strokes['62'][0][0];
     assert.ok(Math.abs(first[0]-(.09+.8*.3))<.01,JSON.stringify(first));
     assert.ok(Math.abs(first[1]-(.23+.56*.4))<.01,JSON.stringify(first));
    }
    await page.screenshot({path:path.join(output,'pattern-'+number+'-'+viewport.width+'.png'),fullPage:true});
   }
  }
  await page.setViewportSize({width:1280,height:800});
  await button(page,'हिंदी').click();
  await page.getByRole('heading',{name:'बुनियादी अभ्यास',exact:true}).waitFor();
  await page.screenshot({path:path.join(output,'pattern-hindi.png'),fullPage:true});
  await button(page,'English').click();
 });
 await run(profile,async page=>{
  await button(page,'Basic').click();
  await page.getByRole('heading',{name:'Page 14',exact:true}).waitFor();
  assert.equal((await saved(basic)).strokes['62'].length,1);
 });
 console.log('Passed: complete patterns 9–14 without scrolling at three viewport sizes, cropped canvas coordinate alignment, Hindi layout and restart recovery.');
})().catch(error=>{console.error(error);process.exitCode=1;});
