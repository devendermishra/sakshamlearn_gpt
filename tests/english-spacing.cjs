const assert=require('node:assert/strict');
const test=require('node:test');
const vm=require('node:vm');
const fs=require('node:fs');
const source=fs.readFileSync(require('node:path').join(__dirname,'../src/ink.js'),'utf8');
const mapping=source.slice(source.indexOf('const paperRows ='),source.indexOf('const cells ='));
for(const fullSheet of [false,true])test('English legacy coordinates round trip: '+(fullSheet?'rough':'multiline'),()=>{
 const rows=fullSheet?12:3,oldPitch=fullSheet?20:16;
 const oldExtent=16*rows+(fullSheet?4*(rows-1):0),newExtent=16*rows+1.6*(rows-1);
 const ctx={englishPaper:true,fullSheet,lines:rows,sentence:fullSheet?[]:Array(rows).fill('a'),count:1,initial:[[]]};
 vm.createContext(ctx);vm.runInContext(mapping+';this.convert=mapInk;',ctx);
 for(let row=0;row<rows;row++)for(const guide of [2,6,10,14]){
  const y=(row*oldPitch+guide)/oldExtent;
  const converted=ctx.convert([[[[.3,y]]]])[0][0][0][1];
  assert.ok(Math.abs(converted-(row*17.6+guide)/newExtent)<1e-10);
 }
 for(let i=0;i<=1000;i++){
  const value=[[[[.3,i/1000]]]];
  assert.ok(Math.abs(ctx.convert(ctx.convert(value),true)[0][0][0][1]-i/1000)<1e-10);
 }
});
