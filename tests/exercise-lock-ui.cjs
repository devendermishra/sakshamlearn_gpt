const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const path=require('node:path');
const net=require('node:net');
const {spawn}=require('node:child_process');
const {chromium}=require(path.join(path.dirname(process.execPath),'../node_modules/playwright'));
const root=path.resolve(__dirname,'..');
const button=(page,name)=>page.getByRole('button',{name,exact:true});
async function run(profile,check) {
 const server=net.createServer();await new Promise(r=>server.listen(0,'127.0.0.1',r));const port=server.address().port;await new Promise(r=>server.close(r));
 const webview=await fs.mkdtemp(path.join(profile,'webview-'));
 const child=spawn(path.join(root,'target/debug/saksham.exe'),[],{cwd:profile,windowsHide:true,stdio:'ignore',env:{...process.env,LOCALAPPDATA:profile,WEBVIEW2_USER_DATA_FOLDER:webview,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port='+port}});
 const closed=new Promise(r=>child.once('exit',r));let browser;
 try {
  for(let i=0;i<160;i++){
   if(child.exitCode!==null)throw Error('Preview exited early');
   if(await fetch('http://127.0.0.1:'+port+'/json/version').then(r=>r.ok,()=>false))break;
   await new Promise(r=>setTimeout(r,250));
  }
  browser=await chromium.connectOverCDP('http://127.0.0.1:'+port);const context=browser.contexts()[0];
  const page=context.pages()[0]||await context.waitForEvent('page');page.setDefaultTimeout(15000);
  await button(page,'Basic').waitFor();await check(page);
 } finally {await browser?.close().catch(()=>{});if(child.exitCode===null)child.kill();await closed;}
}
const lock=async page=>{await button(page,'Lock exercise').first().click();await button(page,'Unlock exercise').first().waitFor();};
const unlock=async page=>{await button(page,'Unlock exercise').first().click();await button(page,'Unlock exercise').first().waitFor({state:'detached'});};
async function disabled(locator) {await locator.waitFor();for(let i=0;i<40&&!await locator.isDisabled();i++)await new Promise(r=>setTimeout(r,50));assert.equal(await locator.isDisabled(),true);}
async function ink(page,selector) {
 await page.locator(selector).first().evaluate(c=>{const r=c.getBoundingClientRect();for(const [type,x] of [['pointerdown',.2],['pointermove',.7],['pointerup',.7]])c.dispatchEvent(new PointerEvent(type,{pointerType:'pen',pointerId:1,button:0,clientX:r.left+r.width*x,clientY:r.top+r.height*.3}));});
}
async function waitSave(file,predicate) {
 for(let i=0;i<60;i++) {const value=await fs.readFile(file,'utf8').then(JSON.parse,()=>null);if(value&&predicate(value))return value;await new Promise(r=>setTimeout(r,100));}
 throw Error('Saved draft did not reach expected state: '+file);
}
(async()=>{
 const profile=await fs.mkdtemp(path.join(root,'target/exercise-lock-ui-'));
 await fs.mkdir(path.join(profile,'Saksham'),{recursive:true});
 await fs.writeFile(path.join(profile,'Saksham/progress.json'),JSON.stringify({finished:['math/numbers-1-3'],reviews:{'math/numbers-1-3':{started:0,due:0,interval:0}}}));
 await run(profile,async page=>{
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await button(page,'Basic').click();await lock(page);
  for(const name of ['Previous','Next','All pages','Mark complete','‹ Subjects','हिंदी'])await disabled(button(page,name));
  await ink(page,'#pencil-workspace canvas');
  const basicFile=path.join(profile,'Saksham/learner-work/basic-pencil.json');
  await waitSave(basicFile,p=>p.strokes['0']?.length===1);
  await button(page,'Eraser').click();await ink(page,'#pencil-workspace canvas');await waitSave(basicFile,p=>p.strokes['0']?.length===0);
  await button(page,'Undo stroke').click();await waitSave(basicFile,p=>p.strokes['0']?.length===1);
  await button(page,'Break / Continue').click();await button(page,'Continue').click();await disabled(button(page,'Next'));
  await unlock(page);assert.equal(await button(page,'Next').isEnabled(),true);await button(page,'Next').click();await button(page,'‹ Subjects').click();
  await button(page,'ABA Learning').click();await button(page,'Exercises').click();await lock(page);
  await disabled(page.locator('.aba-practice select'));await disabled(button(page,'Flashcards'));await disabled(button(page,'Independent'));
  await button(page,'Take a break').click();await button(page,'Continue').click();await disabled(button(page,'Independent'));
  await unlock(page);await button(page,'Flashcards').click();await lock(page);await disabled(button(page,'Exercises'));await disabled(page.locator('.aba-practice select'));
  await unlock(page);await button(page,'‹ Subjects').click();
  await button(page,'गणित').click();await button(page,'English').click();await button(page,'Class 1').click();
  await page.getByRole('button',{name:/Rough Sheet/}).click();await lock(page);await disabled(page.locator('header > button').first());await ink(page,'#rough-ink canvas');await unlock(page);await page.locator('header > button').first().click();
  await page.getByRole('button',{name:/Tracing Practice/}).click();await button(page,'Numbers').click();await lock(page);
  await disabled(page.locator('.tracing-practice select'));await disabled(button(page,'Next'));await disabled(button(page,'‹ All tracing practice'));
  await unlock(page);await page.locator('header > button').first().click();
  await page.getByRole('button',{name:/Worksheets/}).click();await page.locator('.worksheet-list button').first().click();await lock(page);
  await disabled(button(page,'‹ All worksheets'));await disabled(page.locator('.worksheet-task button').last());
  await button(page,'Take a break').click();await button(page,'Continue').click();await disabled(button(page,'‹ All worksheets'));
  await unlock(page);await page.locator('header > button').first().click();
  // Both scheduled review and the ordinary lesson retain checked answers while locked.
  for(const review of [true,false]) {
   const topics=page.getByRole('button',{name:/^Count and write · 1–3/});
   await (review?topics.first():topics.last()).click();await lock(page);
   if(await button(page,'Show choices').count())await button(page,'Show choices').click();
   const before=await page.locator('.exercise-prompt').innerText();await page.locator('.answers button').first().click();
   await disabled(button(page,'Next'));assert.equal(await page.locator('.exercise-prompt').innerText(),before);
   await unlock(page);assert.equal(await page.locator('.exercise-prompt').innerText(),before);
   if(!review){await button(page,'Next').click();await page.locator('[data-done]').waitFor();await lock(page);await disabled(page.locator('[data-done]'));await unlock(page);}
   await page.locator('header > button').first().click();
  }
  await button(page,'Daily recall').click();await lock(page);
  if(await button(page,'Show choices').count())await button(page,'Show choices').click();
  await page.locator('.answers button').first().click();await disabled(button(page,'Next'));
  await unlock(page);await page.locator('header > button').first().click();await page.locator('header > button').first().click();
  await button(page,'Daily exercise plans · रोज़ के अभ्यास').click();
  await page.locator('[data-daily-subject="basic"]').getByRole('button',{name:'Start / Resume · शुरू / जारी',exact:true}).click();await lock(page);
  await disabled(button(page,'Back to day · दिन पर लौटें'));await disabled(button(page,'Mark complete · पूरा चिह्नित करें'));
  for(const label of ['Combined · 60 days','Class 1 · 30 days','Class 2 · 60 days']) await disabled(button(page,label));
  await button(page,'Take a break · थोड़ा आराम').click();await button(page,'Continue · जारी रखें').click();await disabled(button(page,'Mark complete · पूरा चिह्नित करें'));
  await unlock(page);await button(page,'Back to day · दिन पर लौटें').click();
  await page.locator('[data-daily-subject="math"]').getByRole('button',{name:'Start / Resume · शुरू / जारी',exact:true}).click();await lock(page);
  // Answer choices still give local feedback, while advancing controls stay disabled.
  const choices=page.locator('.answers button');if(await choices.count()) {
   await choices.first().click();
   await disabled(page.getByRole('button',{name:'Next',exact:true}));
  }
  assert.deepEqual(errors,[]);
  // End locked to verify this runtime preference is not persisted.
 });
 await run(profile,async page=>{
  assert.equal(await button(page,'Unlock exercise').count(),0);await button(page,'Basic').click();
  await button(page,'Next').waitFor();assert.equal(await button(page,'Next').isEnabled(),true);
  await button(page,'Previous').click();await page.getByRole('heading',{name:/Page 1 · Practice band 1/}).waitFor();
  await button(page,'Eraser').click();await ink(page,'#pencil-workspace canvas');
  await waitSave(path.join(profile,'Saksham/learner-work/basic-pencil.json'),p=>p.position===0&&p.strokes['0']?.length===0);
 });
 await run(profile,async page=>{
  await button(page,'Basic').click();
  const value=await waitSave(path.join(profile,'Saksham/learner-work/basic-pencil.json'),p=>p.position===0);
  assert.deepEqual(value.strokes['0'],[],'erased strokes stay removed across app restart');
 });
 console.log('Exercise lock: lessons, reviews, daily recall, Basic, ABA, flashcards, rough sheet, tracing, worksheets, daily tasks, breaks, and restart checks passed.');
})().catch(error=>{console.error(error);process.exitCode=1;});
