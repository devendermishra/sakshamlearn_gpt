const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const net = require("node:net");
const { spawn } = require("node:child_process");
const { chromium } = require(path.join(process.env.USERPROFILE, ".cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright"));
const preview = path.join(__dirname, "target", "debug", "saksham.exe");

async function port() {
    const server = net.createServer();
    await new Promise((resolve, reject) => server.listen(0, "127.0.0.1", resolve).once("error", reject));
    const result = server.address().port;
    await new Promise(resolve => server.close(resolve));
    return result;
}

async function run(profile, exercise) {
    const endpoint = "http://127.0.0.1:" + await port();
    const browserProfile = await fs.mkdtemp(path.join(profile, "webview-"));
    const child = spawn(preview, [], {
        cwd: profile,
        env: {
            ...process.env,
            LOCALAPPDATA: profile,
            WEBVIEW2_USER_DATA_FOLDER: browserProfile,
            WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: "--remote-debugging-port=" + new URL(endpoint).port,
        },
        windowsHide: true,
        stdio: "ignore",
    });
    const closed = new Promise(resolve => child.once("exit", resolve));
    let browser;
    try {
        for (let index = 0; index < 80; index++) {
            if (child.exitCode !== null) throw Error("Preview exited before its window was ready");
            const ready = await fetch(endpoint + "/json/version").then(response => response.ok, () => false);
            if (ready) break;
            await new Promise(resolve => setTimeout(resolve, 250));
        }
        browser = await chromium.connectOverCDP(endpoint, {timeout: 10000});
        const context = browser.contexts()[0];
        const page = context.pages()[0] ?? await context.waitForEvent("page", {timeout: 10000});
        page.setDefaultTimeout(7000);
        await exercise(page);
    } finally {
        await browser?.close().catch(() => {});
        if (child.exitCode === null) child.kill();
        await closed;
    }
}

function button(page, name) {
    return page.getByRole("button", {name, exact: true});
}

async function fillQuestion(scope, prompt, first, second) {
    await scope.getByLabel("Question", {exact: true}).fill(prompt);
    await scope.getByLabel("Choice 1", {exact: true}).fill(first);
    await scope.getByLabel("Choice 2", {exact: true}).fill(second);
}


function near(actual, expected) { assert.ok(Math.abs(actual - expected) < .12, actual + " != " + expected); }

(async () => {
    const profile = await fs.mkdtemp(path.join(__dirname, "target", "calibration-ui-"));
    const save = path.join(profile, "Saksham", "progress.json");
    const horizontal = 96 / 25.4 * 80 / 100;
    const vertical = 96 / 25.4 * 80 / 64;
    await run(profile, async page => {
        await button(page, "बड़ों के लिए").click();
        const pin = page.getByLabel("Set a parent PIN");
        await pin.focus();
        const keys = page.getByRole("group", {name: "Numeric keypad"});
        for (const digit of ["1", "2", "3", "9"]) await button(keys, digit).click();
        await button(keys, "Backspace").click();
        await button(keys, "4").click();
        assert.equal(await pin.getAttribute("type"), "password");
        assert.equal(await pin.inputValue(), "1234");
        await button(page, "Continue").click();
        const square = page.locator(".calibration-square");
        const before = await square.boundingBox();
        near(before.width, 80 * 96 / 25.4);
        near(before.height, before.width);
        await page.getByLabel("Measured width (mm)").fill("100");
        await page.getByLabel("Measured height (mm)").fill("0");
        await button(page, "Adjust guides").click();
        await page.getByRole("alert").waitFor();
        assert.equal(await button(page, "Save calibration").isEnabled(), false);
        await page.getByLabel("Measured height (mm)").fill("64");
        await button(page, "Adjust guides").click();
        await page.getByText("Check both guides below with your ruler, then save.").waitFor();
        const hindi = await page.locator("#check-hindi .ink-grid").boundingBox();
        const math = await page.locator("#check-math .ink-grid").boundingBox();
        near(hindi.height, 48 * vertical);
        near(hindi.width, 180 * horizontal);
        near(math.width, 110 * horizontal);
        near(math.height, 11 * vertical);
        await button(page, "Save calibration").click();
        await page.getByText("Ruler calibration saved.").waitFor();
        const persisted = JSON.parse(await fs.readFile(save, "utf8"));
        near(persisted.scale, horizontal);
        near(persisted.vertical_scale, vertical);
    });
    await run(profile, async page => {
        await button(page, "गणित").click();
        await button(page, "कक्षा 1").click();
        await button(page, "गिनो और लिखो · 1–3").click();
        await button(page, "1").click();
        await button(page, "आगे").click();
        const cells = page.locator(".ink-square canvas:visible");
        await cells.nth(9).waitFor();
        const math = await cells.first().boundingBox();
        near(math.width, 11 * horizontal);
        near(math.height, 11 * vertical);
        await button(page, "⌂ गणित").click();
        await button(page, "‹ विषय").click();
        await button(page, "हिंदी").click();
        await button(page, "कक्षा 1").click();
        await button(page, "स्वर").click();
        await button(page, "स्वर · अ आ इ").click();
        const guides = page.locator(".ink-hindi-trace .ink-grid");
        await guides.locator("canvas").nth(11).waitFor();
        const hindi = await guides.boundingBox();
        near(hindi.height, 48 * vertical);
        near(hindi.width, 180 * horizontal);
    });
    console.log("Verified ruler correction on both axes, invalid input, saved calibration after restart, and the practice guides in both subjects.");
})().catch(error => { console.error(error); process.exitCode = 1; });

