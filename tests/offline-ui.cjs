const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const crypto = require('node:crypto');
const {chromium} = require(path.join(path.dirname(process.execPath), '../node_modules/playwright'));
const policy = fs.readFileSync(path.join(__dirname, '../android/OfflinePolicy.kt'), 'utf8');
// Exercise the actual CSP shipped in the Android response header.
const csp = [...policy.match(/const val CSP = ([\s\S]*?)\n\n/)[1].matchAll(/"([^"]*)"/g)].map(m=>m[1]).join('');
(async()=>{
 const sockets=new Set();
 const server=http.createServer();
 server.on('connection',s=>{sockets.add(s);s.on('close',()=>sockets.delete(s));});
 server.on('upgrade',(request,socket)=>{
  const accept=crypto.createHash('sha1').update(request.headers['sec-websocket-key']+'258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest('base64');
  socket.write('HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: '+accept+'\r\n\r\n');
 });
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 let browser;
 try {
  browser=await chromium.launch({channel:'msedge',headless:true});
  const page=await browser.newPage();const external=[];
  await page.context().grantPermissions(['local-network-access'], {origin:'https://dioxus.index.html'});
  await page.route('**/*',route=>{
   const url=new URL(route.request().url());
   if(url.hostname==='dioxus.index.html')return route.fulfill({status:200,headers:{'Content-Security-Policy':csp},contentType:'text/html',body:'<html><body><script>window.inlineWorks=true</script><h1>Offline</h1></body></html>'});
   external.push(url.href);return route.abort();
  });
  await page.goto('https://dioxus.index.html/');
  const allowed=await page.evaluate(async port=>{
   const channel=new WebSocket(`ws://127.0.0.1:${port}/ui`);
   const connected=await new Promise(resolve=>{channel.onopen=()=>resolve(true);channel.onerror=()=>resolve(false);});
   const result={connected,inline:window.inlineWorks,eval:new Function('return 42')(),localAsset:(await fetch('/__events')).ok};
   channel.close();return result;
  },server.address().port);
  assert.deepEqual(allowed,{connected:true,inline:true,eval:42,localAsset:true});
  await page.evaluate(async()=>{
   window.violations=[];document.addEventListener('securitypolicyviolation',e=>violations.push(e.effectiveDirective));
   await fetch('https://example.com/data').catch(()=>{});
   await fetch('http://192.168.1.1/data').catch(()=>{});
   new WebSocket('wss://example.com/socket');new WebSocket('ws://192.168.1.1/socket');
   navigator.sendBeacon('https://example.com/beacon','blocked');
   const image=new Image();image.src='https://example.com/image.png';document.body.append(image);
   const style=document.createElement('link');style.rel='stylesheet';style.href='https://example.com/style.css';document.head.append(style);
   const frame=document.createElement('iframe');frame.src='https://example.com/frame';document.body.append(frame);
   const form=document.createElement('form');form.action='https://example.com/submit';form.target='blocked';document.body.append(form);form.submit();
   await new Promise(resolve=>setTimeout(resolve,150));
  });
  const violations=await page.evaluate(()=>violations);
  for(const directive of ['connect-src','img-src','style-src-elem','frame-src','form-action'])assert(violations.includes(directive),directive+' must block external requests');
  assert.deepEqual(external,[],'external requests must be blocked before reaching the network');
  console.log('Offline CSP: local assets, inline/eval bootstrap and localhost WebSocket work; external fetch, beacon, socket, images, styles, frames and forms blocked.');
 }finally{await browser?.close();for(const socket of sockets)socket.destroy();await new Promise(resolve=>server.close(resolve));}
})().catch(error=>{console.error(error);process.exitCode=1;});
