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

(async () => {
    const profile = await fs.mkdtemp(path.join(__dirname, "target", "ui-check-"));
    const save = path.join(profile, "Saksham", "progress.json");
    await run(profile, async page => {
        await button(page, "बड़ों के लिए").click();
        await page.getByLabel("Set a parent PIN").fill("1234");
        await button(page, "Continue").click();
        await button(page, "Manage activities").click();
        await page.getByLabel("Subject name", {exact: true}).fill("UI Practice");
        await button(page, "Create subject").click();
        await page.getByText("Subject created.").waitFor();
        await button(page, "New activity").click();
        await page.getByLabel("Activity title").fill("Counting practice");
        await button(page, "Publish").click();
        await page.getByRole("alert").getByText("Add at least one exercise.").waitFor();
        await button(page, "Add exercise").click();
        await fillQuestion(page.getByRole("group", {name:"Exercise 1"}), "How many?", "One", "Two");
        await button(page, "Add recall question").click();
        await fillQuestion(page.getByRole("group", {name:"Recall 1"}), "Remember the answer", "One", "Two");
        const before = await fs.readFile(save, "utf8");
        await button(page, "Preview").click();
        await page.getByRole("heading", {name:"How many?"}).waitFor();
        await button(page, "One").click();
        await page.getByText("सही!", {exact: true}).waitFor();
        await button(page, "आगे").click();
        await page.getByRole("heading", {name:"Remember the answer"}).waitFor();
        await button(page, "विकल्प देखो").click();
        await button(page, "One").click();
        await button(page, "आगे").click();
        await page.getByText("Preview complete.").waitFor();
        assert.equal(await fs.readFile(save, "utf8"), before);
        await button(page, "Back to draft").click();
        await button(page, "Publish").click();
        await page.getByText("Activity published.").waitFor();
        await page.locator("select").first().selectOption("hi");
        await page.getByText("Copy a bundled activity", {exact: true}).click();
        await button(page, "नीमा की दादी").click();
        await page.getByLabel("Activity title").fill("Hindi copied activity");
        await button(page, "Preview").click();
        await button(page, "पढ़ लिया").click();
        await page.getByRole("heading", {name:"घर पर कौन थीं?"}).waitFor();
        await button(page, "Back to draft").click();
        await button(page, "Publish").click();
        await page.getByText("Activity published.").waitFor();
        console.log("Created, previewed and published an activity and a Hindi copy. Preview left the save untouched.");
    });
    await run(profile, async page => {
        await button(page, "UI Practice").click();
        await button(page, "Counting practice").click();
        await button(page, "One").click();
        await button(page, "आगे").click();
        await button(page, "विकल्प देखो").click();
        await button(page, "One").click();
        await button(page, "आगे").click();
        await page.getByRole("heading", {name:"अभ्यास पूरा"}).waitFor();
        await button(page, "UI Practice पर लौटो").click();
        await button(page, "‹ विषय").click();
        await button(page, "हिंदी").click();
        await button(page, "अतिरिक्त अभ्यास").click();
        await button(page, "Hindi copied activity").click();
        await button(page, "पढ़ लिया").click();
        await page.getByRole("heading", {name:"घर पर कौन थीं?"}).waitFor();
        await button(page, "⌂ हिंदी").click();
        await button(page, "‹ विषय").click();
        await button(page, "गणित").click();
        await button(page, "कक्षा 1").click();
        await button(page, "गिनो और लिखो · 1–3").click();
        assert.equal(await page.locator(".count-objects span").count(), 1);
        await button(page, "2").click();
        await page.getByText("फिर कोशिश करो। सवाल और विकल्प फिर देखो।").waitFor();
        await button(page, "आगे").click();
        await page.locator("canvas").nth(11).waitFor();
        const canvases = page.locator("canvas");
        const box = await canvases.nth(1).boundingBox();
        assert.ok(Math.abs(box.width - box.height) < .1);
        for (const index of [1, 7]) {
            const cell = await canvases.nth(index).boundingBox();
            await page.mouse.move(cell.x + cell.width / 2, cell.y + cell.height / 4);
            await page.mouse.down();
            await page.mouse.move(cell.x + cell.width / 2, cell.y + cell.height * .75, {steps: 6});
            await page.mouse.up();
        }
        await button(page, "हो गया").click();
        await page.getByText("अभ्यास 3 / 6").waitFor();
        const data = JSON.parse(await fs.readFile(save, "utf8"));
        assert.equal(data.history["math/numbers-1-3/write-1"][0].paper, "Digit");
        console.log("Reopened both published activities, completed the new topic, and submitted Mathematics writing.");
    });
})().catch(error => { console.error(error); process.exitCode = 1; });
