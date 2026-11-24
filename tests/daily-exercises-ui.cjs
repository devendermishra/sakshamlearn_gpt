const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {chromium} = require(path.join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright'));
const root = path.resolve(__dirname, '..');
const output = path.join(root, 'target/daily-exercises-review');
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
 const profile=await fs.mkdtemp(path.join(root,'target/daily-exercises-ui-'));
 const file=path.join(profile,'Saksham/learner-work/daily-exercises.json');
 let active;
 let worksheet;
 let lessonChoice;
 const sheets=JSON.parse(await fs.readFile(path.join(root,'src/content/worksheets.json'),'utf8'));
 await run(profile,async page=>{
  await open(page);
  assert.equal(await page.locator('[data-daily-subject]').count(),4);
  await button(page,'Class 2 · 60 days').click();
  await page.getByText('Choose a day · दिन चुनें',{exact:true}).click();
  assert.equal(await page.locator('[data-day]').count(),60);
  await page.locator('[data-day="3"]').click();
  assert.equal(await page.locator('[data-daily-subject="basic"]').getByRole('button',{name:'Start / Resume · शुरू / जारी',exact:true}).isDisabled(),true);
  await button(page,'Combined · 60 days').click();
  // Close the retained picker so the existing navigation check can reopen it.
  if(await page.locator('[data-day="1"]').isVisible()) await page.getByText('Choose a day · दिन चुनें',{exact:true}).click();
  assert.equal(await page.locator('.daily-pane').getByRole('button',{name:/ABA|therapy/i}).count(),0);
  await page.getByText('Choose a day · दिन चुनें',{exact:true}).click();
  assert.equal(await page.locator('[data-day]').count(),60);
  for(const day of [31,60,1]) {
   await page.locator(`[data-day="${day}"]`).click();
   await page.getByRole('heading',{name:new RegExp(`^Day ${day} ·`)}).waitFor();
  }
  for(const size of [{width:1280,height:800},{width:1024,height:768},{width:800,height:600}]) {
   await page.setViewportSize(size);
   assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));
   await page.screenshot({path:path.join(output,'day-1-'+size.width+'.png'),fullPage:true});
  }
  const basic=page.locator('[data-daily-subject="basic"]');
  await basic.getByRole('button',{name:'Start / Resume · शुरू / जारी',exact:true}).click();
  const canvas=page.locator('#pencil-workspace canvas');
  await canvas.scrollIntoViewIfNeeded();
  const box=await canvas.boundingBox();
  await page.mouse.move(box.x+box.width*.15,box.y+box.height*.4);
  await page.mouse.down();await page.mouse.move(box.x+box.width*.65,box.y+box.height*.4,{steps:10});await page.mouse.up();
  let saved=await waitSave(file,p=>Object.values(p.pencil).some(s=>s.length));
  active=saved.active;
  await button(page,'Take a break · थोड़ा आराम').click();
  await waitSave(file,p=>p.paused);
 });
 await run(profile,async page=>{
  await open(page);
  await page.getByRole('heading',{name:'Take a short break · थोड़ा आराम करें',exact:true}).waitFor();
  await button(page,'Continue · जारी रखें').click();
  await page.waitForFunction(()=>document.querySelector('#pencil-workspace canvas'));
  const saved=await waitSave(file,p=>!p.paused);
  assert.equal(saved.active,active);assert.ok(saved.pencil[active].length);
  await button(page,'Class 1 · 30 days').click();
  await page.getByText('Choose a day · दिन चुनें',{exact:true}).click();
  assert.equal(await page.locator('[data-day]').count(),30);
  await button(page,'Combined · 60 days').click();
  await page.waitForFunction(()=>document.querySelector('#pencil-workspace canvas'));
  await waitSave(file,p=>p.selected_plan==='combined'&&p.active===active);
  await button(page,'Mark complete · पूरा चिह्नित करें').click();
  await waitSave(file,p=>p.completed.includes(active));
  await button(page,'Class 1 · 30 days').click();
  await page.locator('[data-daily-subject="basic"] summary').click();
  assert.ok((await page.locator('[data-task-id="'+active+'"]').innerText()).includes('✓'));
  await button(page,'Combined · 60 days').click();
  // Return to the overview even when a second Basic activity follows.
  const combined=await waitSave(file,p=>p.selected_plan==='combined');
  const back=button(page,'Back to day · दिन पर लौटें');if(combined.active) await back.click();
  await button(page,'Continue next unfinished day · अगला अधूरा दिन').click();
  await page.getByRole('heading',{name:/^Day 1 ·/}).waitFor();
  await page.screenshot({path:path.join(output,'resumed.png'),fullPage:true});
  await page.getByText('Choose a day · दिन चुनें',{exact:true}).click();
  await page.locator('[data-day="3"]').click();
  await page.getByRole('heading',{name:/^Day 3 ·/}).waitFor();
  const english=page.locator('[data-daily-subject="en"]');
  await english.locator('summary').click();
  const recall=english.locator('[data-task-id^="recall/en/"]').first();
  lessonChoice=await recall.getAttribute('data-task-id');
  await button(page,'Class 1 · 30 days').click();
  if(!await page.locator('[data-day="3"]').isVisible()) await page.getByText('Choose a day · दिन चुनें',{exact:true}).click();
  await page.locator('[data-day="3"]').click();
  if(!await page.locator('[data-task-id="'+lessonChoice+'"]').isVisible()) await page.locator('[data-daily-subject="en"] summary').click();
  await page.locator('[data-task-id="'+lessonChoice+'"]').click();
  await button(page,'Show choices').click();
  await page.locator('.answers button').first().click();
  await waitSave(file,p=>p.answers[lessonChoice]?.selected===0);
  await button(page,'Take a break · थोड़ा आराम').click();
  await waitSave(file,p=>p.paused);
 });
 await run(profile,async page=>{
  await open(page);
  await button(page,'Continue · जारी रखें').click();
  await button(page,'Next').waitFor();
  await button(page,'Next').click();
  await waitSave(file,p=>p.completed.includes(lessonChoice));
  await button(page,'Combined · 60 days').click();
  const overview=button(page,'Back to day · दिन पर लौटें');if(await overview.count()) await overview.click();
  const enSummary=page.locator('[data-daily-subject="en"] summary');
  if(!await page.locator('[data-task-id="'+lessonChoice+'"]').isVisible()) await enSummary.click();
  assert.ok((await page.locator('[data-task-id="'+lessonChoice+'"]').innerText()).includes('✓'));
  const combined=await waitSave(file,p=>p.selected_plan==='combined');
  const back=button(page,'Back to day · दिन पर लौटें');if(combined.active) await back.click();
  await page.getByText('Choose a day · दिन चुनें',{exact:true}).click();
  for(let day=60;day>=31&&!worksheet;day--) {
   await page.locator(`[data-day="${day}"]`).click();
   await page.getByRole('heading',{name:new RegExp(`^Day ${day} ·`)}).waitFor();
   const ids=await page.locator('[data-task-id^="worksheet/"]').evaluateAll(nodes=>nodes.map(n=>n.dataset.taskId));
   worksheet=ids.map(id=>{
    const [,sheet,index]=id.split('/');const source=sheets.find(s=>s.id===sheet);
    return {id,source,task:source.tasks[Number(index)]};
   }).find(t=>t.task.kind==='choice');
  }
  assert.ok(worksheet,'Class 2 includes choice worksheets');
  const subject=page.locator(`[data-daily-subject="${worksheet.source.subject}"]`);
  await subject.locator('summary').click();
  await page.locator(`[data-task-id="${worksheet.id}"]`).click();
  await page.locator('.worksheet-choices button').nth(worksheet.task.correct).click();
  await waitSave(file,p=>p.worksheets[worksheet.id]?.taps[0]===worksheet.task.correct);
  // Simulate a failed write and verify that the learner can retry the same work.
  const folder=path.dirname(file),held=folder+'-held';
  await fs.rename(folder,held);await fs.writeFile(folder,'blocked');
  await button(page,'Take a break · थोड़ा आराम').click();
  await page.getByRole('alert').filter({hasText:'Work could not be saved.'}).waitFor();
  await fs.unlink(folder);await fs.rename(held,folder);
  await button(page,'Retry save · फिर सहेजें').click();
  await waitSave(file,p=>p.paused&&p.active===worksheet.id);
 });
 await run(profile,async page=>{
  await open(page);
  await button(page,'Continue · जारी रखें').click();
  const lang=worksheet.source.subject==='en'?'Continue':'जारी रखो';
  await button(page,lang).waitFor({state:'visible'});
  assert.equal(await button(page,lang).isEnabled(),true);
  await button(page,lang).click();
  await waitSave(file,p=>p.completed.includes(worksheet.id));
 });
 // Standalone subject work is untouched by daily-plan completion.
 const files=await fs.readdir(path.dirname(file));
 assert.deepEqual(files,['daily-exercises.json']);
 const normal=await fs.readFile(path.join(profile,'Saksham/progress.json'),'utf8').then(JSON.parse,()=>({finished:[]}));
 assert.equal(normal.finished.length,0);
 const corrupt=await fs.mkdtemp(path.join(root,'target/daily-exercises-corrupt-'));
 const corruptFile=path.join(corrupt,'Saksham/learner-work/daily-exercises.json');
 await fs.mkdir(path.dirname(corruptFile),{recursive:true});await fs.writeFile(corruptFile,'{broken');
 await run(corrupt,async page=>{
  await open(page);await page.getByRole('alert').filter({hasText:'Saved practice could not be opened.'}).waitFor();
  assert.equal(await page.locator('[data-daily-subject]').count(),0);
 });
 assert.equal(await fs.readFile(corruptFile,'utf8'),'{broken');
 console.log('Daily exercises offline, navigation, drawing, restart and corrupt-save checks passed. Screenshots: '+output);
})().catch(e=>{console.error(e);process.exitCode=1;});
