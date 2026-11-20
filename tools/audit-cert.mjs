import assert from "node:assert/strict";
import { createServer } from "node:http";
import fs from "node:fs/promises";
import path from "node:path";
import { parseArgs } from "node:util";
import { chromium } from "playwright";
import AxeBuilder from "@axe-core/playwright";

const { values } = parseArgs({ options: {
  candidate: { type: "string" }, reference: { type: "string" }, output: { type: "string" },
} });
assert.ok(values.candidate && values.reference && values.output, "Use --candidate DIST --reference DIST --output REPORT.json");
const candidate = path.resolve(values.candidate);
const reference = path.resolve(values.reference);
const home = await fs.readFile(path.join(candidate, "index.html"), "utf8");
const canonical = home.match(/<link rel="canonical" href="([^"]+)"/)?.[1];
assert.ok(canonical, "Candidate home must have a canonical URL");
const prefix = new URL(canonical).pathname;
const mime = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json",
  ".css": "text/css", ".svg": "image/svg+xml", ".png": "image/png", ".jpg": "image/jpeg",
  ".gif": "image/gif", ".ico": "image/x-icon", ".wasm": "application/wasm" };
const servers = [];
async function serve(root) {
  const server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url, "http://localhost");
      if (!url.pathname.startsWith(prefix)) {
        response.writeHead(404).end("Outside site mount");
        return;
      }
      let filename = path.resolve(root, decodeURIComponent(url.pathname.slice(prefix.length)));
      if (filename !== root && !filename.startsWith(root + path.sep)) {
        response.writeHead(403).end("Outside site directory");
        return;
      }
      const stat = await fs.stat(filename);
      if (stat.isDirectory()) {
        if (!url.pathname.endsWith("/")) {
          response.writeHead(301, { location: url.pathname + "/" + url.search }).end();
          return;
        }
        filename = path.join(filename, "index.html");
      }
      const data = await fs.readFile(filename);
      response.writeHead(200, { "content-type": mime[path.extname(filename)] || "application/octet-stream" }).end(data);
    } catch (error) {
      if (error.code === "ENOENT" || error.code === "ENOTDIR") {
        response.writeHead(404).end("Not found");
      } else {
        console.error(error);
        response.writeHead(500).end(String(error));
      }
    }
  });
  await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
  servers.push(server);
  return `http://127.0.0.1:${server.address().port}${prefix}`;
}

const routes = ["", "sei-cert-c-coding-standard/",
  "sei-cert-c-coding-standard/rules/memory-management-mem/mem30-c/",
  "sei-cert-c-coding-standard/back-matter/ee-analyzers/helix-qac/",
  "sei-cert-cpp-coding-standard/", "sei-cert-oracle-coding-standard-for-java/",
  "sei-cert-perl-coding-standard/", "sei-cert-fortran-coding-standard/",
  "android-secure-coding-standard/"];
const report = { auditedAt: new Date().toISOString(), candidate, reference,
  pages: [], search: [], interactions: [], nojs: [], dark: [], failures: [] };
let browser;
try {
  const origins = { candidate: await serve(candidate), reference: await serve(reference) };
  for (const origin of Object.values(origins)) {
    assert.equal((await fetch(origin)).status, 200, `Server must be responsive: ${origin}`);
  }
  browser = await chromium.launch({ executablePath: process.env.CHROME_PATH });
  for (const [label, origin] of Object.entries(origins)) {
    const context = await browser.newContext();
    await context.route("**/*", route => new URL(route.request().url()).hostname === "127.0.0.1" ? route.continue() : route.abort());
    const page = await context.newPage();
    let errors = [], badResponses = [];
    page.on("pageerror", error => errors.push(error.message));
    page.on("response", response => {
      if (response.status() >= 400) badResponses.push({ status: response.status(), path: new URL(response.url()).pathname });
    });
    for (const width of [1440, 390, 320]) {
      await page.setViewportSize({ width, height: 900 });
      for (const route of routes) {
        errors = []; badResponses = [];
        await page.goto(origin + route, { waitUntil: "networkidle" });
        await page.evaluate(() => document.fonts.ready);
        const row = { label, width, route, errors: [...errors], badResponses: [...badResponses] };
        Object.assign(row, await page.evaluate(() => ({
          title: document.title, h1: document.querySelectorAll("main h1").length,
          overflow: document.documentElement.scrollWidth > innerWidth,
          scrollWidth: document.documentElement.scrollWidth,
          overflowDetails: [...document.querySelectorAll("main *")].filter(e => e.clientWidth > 0 && e.scrollWidth > e.clientWidth && getComputedStyle(e).overflowX === "visible").slice(0, 8).map(e => ({
            tag: e.tagName, text: e.textContent.slice(0, 100), width: e.clientWidth, scrollWidth: e.scrollWidth,
          })),
          brokenImages: [...document.querySelectorAll("main img")].filter(e => !e.complete || !e.naturalWidth).map(e => e.getAttribute("src")),
          codeBlocks: document.querySelectorAll("main pre").length,
          tables: document.querySelectorAll("main table").length,
          highlighterRequested: performance.getEntriesByType("resource").some(e => e.name.endsWith("/_site/highlight.js")),
        })));
        if (width !== 320) {
          row.axe = (await new AxeBuilder({ page }).analyze()).violations.map(v => ({
            id: v.id, impact: v.impact, count: v.nodes.length, targets: v.nodes.map(n => n.target),
          }));
        }
        report.pages.push(row);
        console.log(`${label} ${width}px ${route || "/"}: ${row.errors.length} JS errors, ${row.axe?.length ?? "-"} axe rules, overflow=${row.overflow}`);
      }
    }
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto(origin, { waitUntil: "networkidle" });
    if (label === "candidate") {
      assert.equal(await page.evaluate(() => performance.getEntriesByType("resource").some(e => e.name.endsWith("/_site/highlight.js"))), false);
      await page.locator("[data-search-open]").click();
      const dialog = page.getByRole("dialog", { name: "Search", exact: true });
      const input = dialog.getByRole("searchbox");
      assert.equal(await input.evaluate(e => e === document.activeElement), true);
      for (const query of ["MEM30-C", "STR31-C", "FIO", "\"freed memory\"", "concurrency", "zzzznomatches"]) {
        await input.fill("");
        await page.waitForFunction(() => document.querySelector("#site-dialog-status").textContent.startsWith("Type a query"));
        const started = Date.now();
        await input.fill(query);
        await page.waitForFunction(() => /matching section|No results|Could not/.test(document.querySelector("#site-dialog-status").textContent));
        const results = await dialog.locator(".sds-search-results a").evaluateAll(es => es.map(e => ({ text: e.textContent, href: e.getAttribute("href") })));
        const status = await dialog.getByRole("status").textContent();
        assert.ok(!status.includes("Could not"), status);
        assert.equal(results.length > 0, query !== "zzzznomatches", `Search: ${query}`);
        if (query === "MEM30-C") assert.match(results[0].href, /\/mem30-c\/#/);
        report.search.push({ query, ms: Date.now() - started, status, results });
      }
      assert.deepEqual((await new AxeBuilder({ page }).analyze()).violations.map(v => v.id), []);
      await page.keyboard.press("Escape");
      await dialog.waitFor({ state: "hidden" });
      assert.equal(await page.locator("[data-search-open]").evaluate(e => e === document.activeElement), true);
      await page.goto(origin + "search/?q=MEM30-C", { waitUntil: "networkidle" });
      await page.locator("#site-results a").first().waitFor();
      assert.match(await page.locator("#site-results a").first().getAttribute("href"), /\/mem30-c\//);
      await page.goto(origin + routes[2], { waitUntil: "networkidle" });
      await context.grantPermissions(["clipboard-read", "clipboard-write"]);
      const block = page.locator("site-code-block").first();
      const code = await block.locator("pre code").textContent();
      await block.getByRole("button", { name: "Copy to clipboard", exact: true }).click();
      await page.waitForFunction(() => document.querySelector("site-code-block output").textContent === "Code copied.");
      assert.equal(await page.evaluate(() => navigator.clipboard.readText()), code);
      await block.getByRole("button", { name: "Toggle fullscreen", exact: true }).click();
      const fullscreen = page.locator("dialog:has(> .sds-code-block)");
      await fullscreen.waitFor({ state: "visible" });
      await page.keyboard.press("Escape");
      await fullscreen.waitFor({ state: "hidden" });
      const table = page.locator("main table").nth(1);
      const before = await table.locator("tbody tr td:first-child").allTextContents();
      await table.locator("th").first().getByRole("button").click();
      await table.locator("th").first().getByRole("button").click();
      assert.deepEqual(await table.locator("tbody tr td:first-child").allTextContents(), before.toReversed());
      await page.setViewportSize({ width: 390, height: 900 });
      await page.getByRole("button", { name: "Open site navigation", exact: true }).click();
      await page.getByRole("button", { name: "Close site navigation", exact: true }).waitFor({ state: "visible" });
      await page.keyboard.press("Escape");
      await page.getByRole("button", { name: "Close site navigation", exact: true }).waitFor({ state: "hidden" });
      await page.setViewportSize({ width: 1440, height: 900 });
      for (const route of ["", routes[2], routes[5]]) {
        await page.emulateMedia({ colorScheme: "dark" });
        await page.goto(origin + route, { waitUntil: "networkidle" });
        const toggle = page.getByRole("button", { name: "Switch to dark mode", exact: true });
        if (await toggle.count()) await toggle.click();
        await page.getByRole("button", { name: "Switch to light mode", exact: true }).waitFor({ state: "visible" });
        report.dark.push({ route, axe: (await new AxeBuilder({ page }).analyze()).violations.map(v => ({ id: v.id, count: v.nodes.length })) });
      }
      const failedContext = await browser.newContext();
      await failedContext.route("**/_site/search/manifest.json", route => route.fulfill({ status: 503, body: "Unavailable" }));
      const failedPage = await failedContext.newPage();
      await failedPage.goto(origin + "search/?q=MEM30-C");
      await failedPage.waitForFunction(() => document.querySelector("#site-status").textContent.includes("Could not load search data"));
      await failedContext.close();
      assert.equal((await fetch(origin + "not-a-real-route/")).status, 404);
      report.interactions.push({ searchFocusAndEscape: true, dedicatedSearch: true, exactClipboard: true,
        fullscreenAndEscape: true, tableSorting: true, mobileNavigationAndEscape: true,
        explicitSearchFailure: true, missingRoute404: true, noHighlighterOnHome: true });
    } else {
      await page.getByRole("button", { name: "Search", exact: true }).click();
      const input = page.locator("input:visible").first();
      await input.fill("MEM30-C");
      await page.waitForTimeout(1000);
      report.referenceSearch = await page.getByRole("dialog").allTextContents();
    }
    await context.close();
    const nojs = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 900 } });
    const plain = await nojs.newPage();
    await plain.goto(origin + routes[2]);
    assert.equal(await plain.locator("main h1").textContent(), "MEM30-C. Do not access freed memory");
    assert.equal(await plain.locator("main pre").count(), 8);
    report.nojs.push({ label, codeBlocks: 8, article: true });
    await nojs.close();
  }
  for (const row of report.pages.filter(p => p.label === "candidate")) {
    const old = report.pages.find(p => p.label === "reference" && p.route === row.route && p.width === row.width);
    if (row.errors.length || row.h1 !== 1) report.failures.push({ route: row.route, width: row.width, errors: row.errors, h1: row.h1 });
    if (row.overflow) report.failures.push({ route: row.route, width: row.width, kind: "horizontal_overflow" });
    for (const violation of row.axe || []) {
      const previous = old.axe.find(v => v.id === violation.id);
      if (!previous || violation.count > previous.count) report.failures.push({ route: row.route, width: row.width, kind: "introduced_axe", violation });
    }
  }
  assert.deepEqual(report.failures, [], "No newly introduced browser/accessibility failures");
} catch (error) {
  report.failures.push({ error: error.stack });
  throw error;
} finally {
  await browser?.close();
  await Promise.all(servers.map(server => new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()))));
  await fs.mkdir(path.dirname(path.resolve(values.output)), { recursive: true });
  await fs.writeFile(values.output, JSON.stringify(report, null, 2) + "\n");
  console.log(`Browser evidence: ${values.output}; pages=${report.pages.length}; failures=${report.failures.length}`);
}
