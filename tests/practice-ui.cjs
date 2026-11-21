const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {chromium} = require(path.join(path.dirname(process.execPath), '../node_modules/playwright'));
const root = path.resolve(__dirname, '..');
const output = path.join(root, 'target', 'practice-review');
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
 const profile=await fs.mkdtemp(path.join(root,'target/practice-ui-'));
 const folder=path.join(profile,'Saksham/learner-work');
 const basic=path.join(folder,'basic-pencil.json');
 const aba=path.join(folder,'aba-exercises.json');
 await run(profile,async page=>{
  await button(page,'Basic').click();
  await page.getByRole('heading',{name:'Basic',exact:true}).waitFor();
  await button(page,'हिंदी').click();
  await page.getByRole('heading',{name:'बुनियादी अभ्यास',exact:true}).waitFor();
  await button(page,'English').click();
  await page.getByRole('heading',{name:'Basic',exact:true}).waitFor();
  await page.locator('.pencil-surface img').evaluate(i=>i.decode());
  await stroke(page);
  await waitForSave(basic,p=>p.strokes['0']?.length===1);
  await button(page,'Undo stroke').click();
  await waitForSave(basic,p=>p.strokes['0']?.length===0);
  await stroke(page);
  const before=await waitForSave(basic,p=>p.strokes['0']?.length===1);
  await button(page,'+').click();
  await button(page,'Move page').click();
  await page.waitForFunction(()=>getComputedStyle(document.querySelector('#pencil-workspace canvas')).pointerEvents==='none');
  assert.equal(await page.locator('#pencil-workspace canvas').evaluate(c=>getComputedStyle(c).pointerEvents),'none');
  await button(page,'Draw with pen').click();
  assert.deepEqual((await saved(basic)).strokes,before.strokes);
  await button(page,'Mark complete').click();
  await waitForSave(basic,p=>p.completed.includes(0));
  await button(page,'Next').click();
  await waitForSave(basic,p=>p.position===1);
  await button(page,'Break / Continue').click();
  await waitForSave(basic,p=>p.paused);
  await button(page,'Continue').click();
  await button(page,'Previous').click();
  await page.screenshot({path:path.join(output,'basic-ui.png'),fullPage:true});
  await button(page,'All pages').click();
  await page.waitForFunction(()=>document.querySelectorAll('.practice-page-grid button').length===14);
  assert.equal(await page.locator('.practice-page-grid button').count(),14);
  await button(page,'Page 14').click();
  await page.locator('.pencil-surface img').evaluate(i=>i.decode());
  await page.screenshot({path:path.join(output,'spiral-ui.png'),fullPage:true});
  await button(page,'‹ Subjects').click();
  await button(page,'ABA Learning').click();
  await button(page,'Home plan checklist').click();
  await page.waitForFunction(()=>document.querySelectorAll('.aba-checklist li>button').length===30);
  assert.equal(await button(page,'Open practice').count(),30);
  await button(page,'Open practice').nth(2).click();
  await page.getByRole('heading',{name:'Tap: Apple'}).waitFor();
  await page.locator('.picture-choices button').filter({has:page.getByRole('img',{name:'Apple',exact:true})}).click();
  await button(page,'Next').click();
  await page.getByRole('heading',{name:'Find the same picture.'}).waitFor();
  await button(page,'With help').click();
  await button(page,'Independent').click();
  await page.getByRole('heading',{name:'Three tasks practised'}).waitFor();
  await waitForSave(aba,p=>p.activities.fruit.position===3 && p.activities.fruit.boundary);
  await button(page,'Repeat').click();
  await waitForSave(aba,p=>p.activities.fruit.position===0 && !p.activities.fruit.boundary);
  await page.screenshot({path:path.join(output,'aba-ui.png'),fullPage:true});
  await page.locator('select').selectOption('transport-birds');
  await page.getByRole('heading',{name:'Tap: Car',exact:true}).waitFor();
  for(let i=0;i<2;i++) await button(page,'Independent').click();
  await page.getByRole('heading',{name:'Which group does this belong to?'}).waitFor();
  await button(page,'Transport').click();
  await button(page,'Next').click();
  await page.locator('select').selectOption('pattern-box');
  await page.locator('.pattern-strip img').first().waitFor();
  assert.equal(await page.locator('.pattern-strip img').count(),4);
  await page.locator('select').selectOption('yes-no');
  await button(page,'Yes').click();
  await button(page,'Next').click();
  await button(page,'No').click();
  await button(page,'Next').click();
  await button(page,'Flashcards').click();
  await page.locator('select').selectOption('speech');
  await page.waitForFunction(()=>document.querySelectorAll('.flashcard').length===24);
  assert.equal(await page.locator('.flashcard').count(),24);
  await page.locator('.flashcard img').evaluateAll(imgs=>Promise.all(imgs.map(i=>i.decode())));
  assert.equal(await page.locator('.flashcard img').evaluateAll(imgs=>imgs.every(i=>i.src.startsWith('data:image/'))),true);
  await page.screenshot({path:path.join(output,'speech-ui.png'),fullPage:true});
  await button(page,'Home plan checklist').click();
  assert.equal(await page.locator('.aba-checklist input:checked').count(),0);
 });
 await run(profile,async page=>{
  await button(page,'Basic').click();
  await page.getByRole('heading',{name:'Page 14',exact:true}).waitFor();
  assert.equal((await saved(basic)).strokes['0'].length,1);
  assert.equal((await saved(basic)).completed.includes(0),true);
  await button(page,'‹ Subjects').click();
  await button(page,'ABA Learning').click();
  await button(page,'Flashcards').click();
  await page.locator('select').waitFor();
  assert.equal(await page.locator('select').inputValue(),'speech');
 });
 const corrupt=await fs.mkdtemp(path.join(root,'target/practice-corrupt-'));
 const corruptFolder=path.join(corrupt,'Saksham/learner-work');
 await fs.mkdir(corruptFolder,{recursive:true});
 for(const name of ['basic-pencil.json','aba-exercises.json']) await fs.writeFile(path.join(corruptFolder,name),'{broken');
 await run(corrupt,async page=>{
  await button(page,'Basic').click();
  await page.getByRole('alert').getByText('Saved practice could not be opened. Go back and try again.').waitFor();
  await button(page,'‹ Subjects').click();
  await button(page,'ABA Learning').click();
  await button(page,'Flashcards').click();
  await page.getByRole('alert').getByText('Saved practice could not be opened. Go back and try again.').waitFor();
  // Give mount effects a chance to run, then ensure unreadable saves remain untouched.
  await page.waitForTimeout(200);
  for(const name of ['basic-pencil.json','aba-exercises.json']) assert.equal(await fs.readFile(path.join(corruptFolder,name),'utf8'),'{broken');
 });
 console.log('Basic/ABA UI and restart checks passed. Screenshots: '+output);
})().catch(error=>{console.error(error);process.exitCode=1;});
