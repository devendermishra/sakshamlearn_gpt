const root = document.querySelector('[data-aba-learning]');
if (!root) return;
const listeners = new AbortController();
let timer = null;
let paused = false;
let end;
const lifetime = new Promise(resolve => { end = resolve; });
function cancel() { clearTimeout(timer); timer = null; }
function refresh() {
    cancel();
    if (!root.isConnected || document.hidden || paused) return;
    dioxus.send(true);
    const day = 86400000;
    const indiaOffset = 19800000;
    const now = Date.now();
    const midnight = (Math.floor((now + indiaOffset) / day) + 1) * day - indiaOffset;
    timer = setTimeout(refresh, midnight - now + 50);
}
document.addEventListener('visibilitychange', refresh, {signal: listeners.signal});
window.addEventListener('saksham:pause', () => { paused = true; cancel(); }, {signal: listeners.signal});
window.addEventListener('saksham:resume', () => { paused = false; refresh(); }, {signal: listeners.signal});
window.addEventListener('focus', refresh, {signal: listeners.signal});
const observer = new MutationObserver(() => { if (!root.isConnected) { cancel(); end(); } });
observer.observe(document.body, {childList: true, subtree: true});
refresh();
if (!root.isConnected) end();
await lifetime;
cancel();
listeners.abort();
observer.disconnect();
