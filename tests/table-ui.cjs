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
(async()=>{
 await fs.mkdir(output,{recursive:true});
 const profile=await fs.mkdtemp(path.join(root,'target/table-ui-'));
 await run(profile,async page=>{
  await button(page,'गणित').click();
  await button(page,'English').click();
  await button(page,'Class 2').click();
  await button(page,'Multiplication tables 1 to 10').click();
  await button(page,'3 times table').click();
  await page.getByRole('heading',{name:'The complete 3 times table: ×1 to ×10',exact:true}).waitFor();
  assert.equal(await page.locator('.table-practice form').count(),10);
  await page.locator('.table-concept summary').click();
  assert.equal(await page.locator('.table-concept figure').count(),11);
  assert.equal(await page.locator('.table-concept figure .table-dot').count(),165);
  assert.equal(await page.locator('.table-concept figure').nth(0).locator('.table-dot').count(),0);
  assert.equal(await page.locator('.table-concept figure').nth(10).locator('.table-dot').count(),30);
  const example=page.locator('.table-concept figure').nth(3);
  assert.equal(await example.locator('.table-concept-meaning p').innerText(),'3 groups of 3');
  assert.equal(await example.locator('.table-from-previous').innerText(),'Previous total 6 + 3 = 9');
  assert.equal(await example.locator('.table-count-on').innerText(),'Count on from the previous total: 6 → 7 → 8 → 9');
  const meaning=await example.locator('.table-concept-meaning').boundingBox();
  const pattern=await example.locator('.table-concept-pattern').boundingBox();
  assert.ok(pattern.x>meaning.x && Math.abs(pattern.y-meaning.y)<2,'Concept uses two columns');
  await page.screenshot({path:path.join(output,'table-concept-full.png'),fullPage:true});
  await page.locator('.table-concept summary').click();
  assert.equal(await page.locator('.table-sheet .table-practice').count(),1);
  assert.equal(await button(page,'Next').count(),0);
  for(let groups=1;groups<=10;groups++) {
   const row=page.locator('.table-practice form').nth(groups-1);
   await row.getByRole('textbox').fill(String(3*groups));
   await row.getByRole('button',{name:'Check',exact:true}).click();
   if(groups<10) assert.equal(await button(page,'Next').count(),0);
  }
  await button(page,'Next').waitFor();
  await page.screenshot({path:path.join(output,'table-practice.png'),fullPage:true});
  await button(page,'Next').click();
  await button(page,'⌂ Mathematics').click();
  await button(page,'Solve in writing').click();
  await button(page,'Write the 10 times table').click();
  await page.locator('#solve-math-ink canvas').nth(99).waitFor();
  assert.equal(await page.locator('#solve-math-ink canvas').count(),100);
  assert.equal(await page.locator('.table-writing-labels>div').count(),10);
  assert.equal(await page.locator('.table-writing-labels>div').last().innerText(),'10 × 10 =');
  const labels=await page.locator('.table-writing-labels>div').all();
  for(let row=0;row<10;row++) {
   const label=await labels[row].boundingBox();
   const grid=await page.locator('#solve-math-ink canvas').nth(row*10).boundingBox();
   assert.ok(Math.abs(label.y-grid.y)<2,'Equation is beside its writing row');
  }
  await page.screenshot({path:path.join(output,'table-writing.png'),fullPage:true});
  const cell=page.locator('#solve-math-ink canvas').first();
  await cell.scrollIntoViewIfNeeded();
  const box=await cell.boundingBox();
  await page.mouse.move(box.x+box.width*.3,box.y+box.height*.3);
  await page.mouse.down();
  await page.mouse.move(box.x+box.width*.6,box.y+box.height*.7,{steps:5});
  await page.mouse.up();
  await button(page,'Enter answer').click();
  const answers=page.getByLabel('Enter the numbers you wrote, in order.',{exact:true});
  await answers.fill('10, 20, 30, 40, 50, 60, 70, 80, 90, 100');
  await button(page,'Check').click();
  await page.getByRole('heading',{name:'Worked solution',exact:true}).waitFor();
  assert.equal((await page.locator('.passage').innerText()).split('\n').length,10);
  await button(page,'Next').click();
  await button(page,'⌂ Mathematics').click();
  await button(page,'Tracing Practice').click();
  await button(page,'3 times table').click();
  const tracing=page.locator('#table-trace-ink canvas');
  await tracing.nth(99).waitFor();
  assert.equal(await tracing.count(),100);
  assert.equal(await tracing.nth(92).getAttribute('data-model'),'10');
  assert.equal(await tracing.nth(94).getAttribute('data-model'),'30');
  assert.equal(await page.getByLabel('Choose a model').count(),0);
  assert.equal(await button(page,'Undo last stroke').count(),1);
  assert.equal(await button(page,'Clear writing').count(),1);
  const hasInk=canvas=>canvas.evaluate(node=>node.getContext('2d').getImageData(0,0,node.width,node.height).data.some((v,i)=>i%4===3 && v>0));
  for(let row=0;row<10;row++) {
   const left=await tracing.nth(row*10).boundingBox();
   const right=await tracing.nth(row*10+5).boundingBox();
   assert.ok(right.x>left.x && Math.abs(right.y-left.y)<2,'Trace is left and blank is right');
   assert.equal(await hasInk(tracing.nth(row*10)),true);
   assert.equal(await hasInk(tracing.nth(row*10+5)),false);
  }
  await page.screenshot({path:path.join(output,'table-tracing-sheet.png'),fullPage:true});
  const saveFile=path.join(profile,'Saksham/learner-work/trace-math-2-table-3-sheet.json');
  async function strokeAt(index) {
   const canvas=tracing.nth(index);
   await canvas.scrollIntoViewIfNeeded();
   const bounds=await canvas.boundingBox();
   await page.mouse.move(bounds.x+bounds.width*.25,bounds.y+bounds.height*.3);
   await page.mouse.down();
   await page.mouse.move(bounds.x+bounds.width*.6,bounds.y+bounds.height*.8,{steps:5});
   await page.mouse.up();
  }
  async function savedCount(expected) {
   for(let i=0;i<50;i++) {
    const ink=await fs.readFile(saveFile,'utf8').then(JSON.parse).catch(()=>[]);
    if(ink.reduce((sum,cell)=>sum+cell.length,0)===expected) return ink;
    await new Promise(resolve=>setTimeout(resolve,100));
   }
   throw Error('Expected saved stroke count '+expected);
  }
  await strokeAt(5);
  await savedCount(1);
  await strokeAt(95);
  await savedCount(2);
  await button(page,'Undo last stroke').click();
  const undone=await savedCount(1);
  assert.equal(undone[5].length,1);
  assert.equal(undone[95].length,0);
  await strokeAt(95);
  await savedCount(2);
  await button(page,'Clear writing').click();
  await savedCount(0);
  assert.equal(await hasInk(tracing.nth(5)),false);
  assert.equal(await hasInk(tracing.nth(0)),true,'Clear preserves trace guides');
  await button(page,'‹ All tracing practice').click();
  await button(page,'10 times table').click();
  await tracing.nth(99).waitFor();
  assert.equal(await tracing.count(),100);
  assert.equal(await tracing.nth(90).getAttribute('data-model'),'10');
  assert.equal(await tracing.nth(92).getAttribute('data-model'),'10');
  assert.equal(await tracing.nth(94).getAttribute('data-model'),'100');
  await page.screenshot({path:path.join(output,'table-whole-number-boxes.png'),fullPage:true});
 });
 console.log('Verified left trace/right blank rows, global undo/clear, saved strokes, and two-column count-on diagrams through ×10.');
})().catch(error=>{console.error(error);process.exitCode=1;});
