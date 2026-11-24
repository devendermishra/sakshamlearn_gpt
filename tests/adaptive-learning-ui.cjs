const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {chromium} = require(path.join(process.env.USERPROFILE, '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright'));
const root = path.resolve(__dirname, '..');
const output = path.join(root, 'target/adaptive-learning-review');
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
 const profile=await fs.mkdtemp(path.join(root,'target/adaptive-learning-ui-'));
 const file=path.join(profile,'Saksham/progress.json');
 await fs.mkdir(path.dirname(file),{recursive:true});
 const day=Math.floor((Date.now()+19800000)/86400000);
 const task=(index)=>({id:'math/numbers-1-3/true/'+index,subject:'math',topic:'numbers-1-3',indices:[index],recall:true,competency:'math/numbers-1-3/recognition/'+index,label:'Number recognition',minutes:1,reason:'Review'});
 await fs.writeFile(file,JSON.stringify({math_language:'English',adaptive:{day,grade:1,tasks:[task(0),task(1)],position:0,part:0}}));
 let submission;
 await run(profile,async page=>{
  await button(page,"Today's learning · आज की पढ़ाई").click();
  await button(page,'Show choices').click();
  await page.getByText('Record adult help (optional)',{exact:true}).click();
  await page.getByLabel('Adult assistance',{exact:true}).selectOption('VerbalPrompt');
  const saved=await waitSave(file,p=>p.adaptive?.drafts?.['0/0']?.assessment?.assistance==='VerbalPrompt');
  submission=saved.adaptive.drafts['0/0'].assessment.submission_id;
  await button(page,'2').click(); // Deliberately wrong before correcting.
  await waitSave(file,p=>p.adaptive.drafts['0/0'].missed===true&&p.adaptive.drafts['0/0'].assessment.prior_errors===true);
 });
 await run(profile,async page=>{
  await button(page,"Today's learning · आज की पढ़ाई").click();
  await page.getByText('Record adult help (optional)',{exact:true}).click();
  assert.equal(await page.getByLabel('Adult assistance',{exact:true}).inputValue(),'VerbalPrompt');
  await button(page,'1').click();
  await button(page,'Next').click();
  let saved=await waitSave(file,p=>p.adaptive.position===1&&Object.keys(p.evidence.attempts).length===1);
  const attempt=Object.values(saved.evidence.attempts)[0];
  assert.equal(attempt.submission_id,submission);assert.equal(attempt.correct,false);
  assert.equal(attempt.context.assistance,'VerbalPrompt');assert.equal(attempt.context.prior_errors,true);
  assert.equal(saved.finished.length,0);
  await button(page,'Show choices').click();await button(page,'2').click();
  // Force a canonical save failure; the activity must not advance until retry succeeds.
  const held=file+'.held';await fs.rename(file,held);await fs.mkdir(file);
  await button(page,'Next').click();
  await page.getByRole('alert').filter({hasText:'Work could not be saved.'}).waitFor();
  assert.ok((await page.locator('.adaptive-pane').innerText()).includes('Activity 2 / 2'));
  await fs.rmdir(file);await fs.rename(held,file);
  await button(page,'Next').click();
  await page.getByText('Session complete. Your work is saved.',{exact:true}).waitFor();
  saved=await waitSave(file,p=>p.adaptive.position===2&&Object.keys(p.evidence.attempts).length===2);
  assert.equal(saved.finished.length,0);
  await page.getByRole('button',{name:/बड़ों के लिए|For parents/}).click();
  assert.equal(await page.getByRole('heading',{name:'Learning evidence',exact:true}).count(),0);
  await page.getByLabel('Set a parent PIN',{exact:true}).fill('2468');await button(page,'Continue').click();
  await page.getByRole('heading',{name:'Learning evidence',exact:true}).waitFor();
  await page.getByLabel('Daily estimated minutes',{exact:true}).selectOption('20');
  await waitSave(file,p=>p.daily_minutes===20);
  await page.getByLabel('Evidence subject',{exact:true}).selectOption('math');
  await page.getByLabel('Evidence class',{exact:true}).selectOption('1');
  await page.locator('.mastery-dashboard summary').filter({hasText:'Practising'}).first().waitFor();
  await page.screenshot({path:path.join(output,'parent-dashboard.png'),fullPage:true});
 });
 await run(profile,async page=>{
  await button(page,"Today's learning · आज की पढ़ाई").click();
  await page.getByText('Session complete. Your work is saved.',{exact:true}).waitFor();
  await button(page,"Start today's session").click();
  const saved=await waitSave(file,p=>p.adaptive.position===0&&p.adaptive.tasks.length>0);
  assert.ok(saved.adaptive.tasks.reduce((sum,t)=>sum+t.minutes,0)<=20);
  assert.ok(saved.adaptive.tasks.filter(t=>t.reason==='New').length<=2);
  assert.equal(Object.keys(saved.evidence.attempts).length,2);
  await page.screenshot({path:path.join(output,'daily-session.png'),fullPage:true});
 });
 console.log('Adaptive learning UI: assistance/error restart, canonical-save retry, deduplication, nonblocking completion, parent PIN, budget persistence and new session passed.');
})().catch(error=>{console.error(error);process.exitCode=1;});
