import { chromium } from "playwright";
import AxeBuilder from "@axe-core/playwright";
import lighthouse from "lighthouse";
import { launch } from "chrome-launcher";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import { assertLighthouseBudget } from "./lighthouse-budget.mjs";

const origin = process.env.SITE_URL || "http://127.0.0.1:4173/demo/";
const executablePath = process.env.CHROME_PATH;
const browser = await chromium.launch({ executablePath });
const reports = new URL("./reports/", import.meta.url);
await fs.mkdir(reports, { recursive: true });

try {
  const context = await browser.newContext();
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => {
    if (response.status() >= 400) errors.push(`${response.status()} ${response.url()}`);
  });
  for (const width of [1440, 390, 320]) {
    await page.setViewportSize({ width, height: 900 });
    for (const route of ["", "guides/", "guides/publishing/", "search/"]) {
      await page.goto(new URL(route, origin).href);
      const accessibility = await new AxeBuilder({ page }).analyze();
      assert.deepEqual(accessibility.violations.map(({ id, nodes }) => ({ id, targets: nodes.map((node) => node.target) })), [], `axe violations at ${width}px on ${route}`);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false, `Horizontal overflow at ${width}px on ${route}`);
      assert.equal(await page.locator("h1").count(), 1);
      await page.locator(".sds-app-masthead").waitFor({ state: "visible" });
      assert.equal(await page.locator(".sds-sei-wordmark").count(), 2);
      assert.equal(await page.locator(".sds-app-footer-content").count(), 1);
      if (width === 1440) {
        await page.locator("#site-sidebar").waitFor({ state: route.startsWith("guides/") ? "visible" : "hidden" });
        await page.locator(".sds-app-header > .sds-app-navigation").waitFor({ state: "visible" });
        await page.getByRole("button", { name: "Open site navigation" }).waitFor({ state: "hidden" });
      }
      if (width <= 390) {
        await page.getByRole("button", { name: "Open site navigation" }).click();
        await page.getByRole("button", { name: "Close site navigation" }).waitFor({ state: "visible" });
        await page.keyboard.press("Escape");
        await page.getByRole("button", { name: "Close site navigation" }).waitFor({ state: "hidden" });
      }
    }
  }
  await page.goto(origin);
  const searchOpener = page.locator("[data-search-open]");
  await searchOpener.click();
  const dialog = page.getByRole("dialog", { name: "Search", exact: true });
  await dialog.waitFor({ state: "visible" });
  assert.equal(await dialog.getByRole("searchbox").evaluate((element) => element === document.activeElement), true);
  await dialog.getByRole("searchbox").fill("publis");
  await dialog.locator(".sds-search-results a").first().waitFor();
  assert.match(await dialog.locator(".sds-search-results").innerText(), /Publishing/);
  const modalAccessibility = await new AxeBuilder({ page }).analyze();
  assert.deepEqual(modalAccessibility.violations.map(({ id }) => id), [], "Search dialog accessibility");
  await page.keyboard.press("Escape");
  await dialog.waitFor({ state: "hidden" });
  assert.equal(await searchOpener.evaluate((element) => element === document.activeElement), true, "Search focus returns to opener");
  assert.equal(new URL(page.url()).search, "", "Modal searches do not change the article URL");
  await searchOpener.click();
  await dialog.getByRole("button", { name: "Close documentation search" }).click();
  await dialog.waitFor({ state: "hidden" });
  await page.goto(new URL("search/?q=publis", origin).href);
  await page.locator("#site-results a").first().waitFor();
  assert.match(await page.locator("#site-results").innerText(), /Publishing/);
  await page.getByRole("searchbox").fill('"plain text"');
  await page.getByRole("searchbox").press("Enter");
  await page.waitForFunction(() => document.querySelector("#site-status").textContent.includes("matching section"));
  await page.getByRole("searchbox").fill("zzznotaword");
  await page.getByRole("searchbox").press("Enter");
  await page.waitForFunction(() => document.querySelector("#site-status").textContent.includes("No results"));
  assert.deepEqual(errors, [], "Browser errors");
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
  await page.goto(origin);
  const dark = await new AxeBuilder({ page }).analyze();
  assert.deepEqual(dark.violations.map(({ id }) => id), [], "Dark theme accessibility");
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.goto(new URL("guides/", origin).href);
  const example = page.locator("site-code-block").first();
  await example.getByRole("button", { name: "Copy to clipboard" }).click();
  await page.waitForFunction(() => document.querySelector(".sds-code-block > output").textContent === "Code copied.");
  assert.equal(await page.evaluate(() => navigator.clipboard.readText()), await example.locator("pre code").textContent(), "Copied text matches the code exactly");
  await page.evaluate(() => Object.defineProperty(navigator, "clipboard", {
    value: { writeText: async () => { throw new Error("Clipboard permission denied."); } },
    configurable: true,
  }));
  await example.locator("[data-sds-action='copy-code']").click();
  await page.waitForFunction(() => document.querySelector(".sds-code-block > output").textContent.includes("Copy failed."));
  assert.equal(await example.locator("output").evaluate((element) => element.classList.contains("sds-sr-only")), false, "Copy failure is visibly reported");
  const failureContext = await browser.newContext();
  await failureContext.route("**/_site/search/manifest.json", (route) => route.fulfill({ status: 503, body: "Unavailable" }));
  const failedSearch = await failureContext.newPage();
  await failedSearch.goto(new URL("search/?q=test", origin).href);
  await failedSearch.waitForFunction(() => document.querySelector("#site-status").textContent.includes("Could not load search data"));
  await failureContext.close();
  const nojs = await browser.newContext({ javaScriptEnabled: false });
  const plain = await nojs.newPage();
  await plain.goto(origin);
  assert.equal(await plain.getByRole("heading", { name: "Welcome", exact: true }).count(), 1);
  await plain.getByRole("link", { name: "Read the writing guide" }).click();
  assert.match(plain.url(), /\/guides\/$/);
  await plain.locator("[data-search-open]").click();
  assert.match(plain.url(), /\/search\/$/);
  await plain.goto(new URL("guides/", origin).href);
  assert.ok((await plain.locator("site-code-block pre code").textContent()).includes("[Publishing]"));
  assert.equal(await plain.locator("site-code-block [data-sds-action='copy-code']").isVisible(), false, "No nonfunctional copy button without JavaScript");
  await nojs.close();
  console.log("Browser checks passed: desktop/mobile axe, overflow, navigation, search, no-JS content.");
} finally {
  await browser.close();
}

const chrome = await launch({ chromePath: executablePath, chromeFlags: ["--headless", "--no-sandbox"] });
try {
  for (const route of ["", "guides/", "guides/publishing/"]) {
    const name = route.replaceAll("/", "-") || "home";
    const measurements = [];
    for (let sample = 1; sample <= 3; sample++) {
      const result = await lighthouse(new URL(route, origin).href, {
        port: chrome.port,
        output: "json",
        onlyCategories: ["performance", "accessibility", "best-practices", "seo"],
        formFactor: "mobile",
        logLevel: "error",
      });
      const scores = Object.fromEntries(Object.entries(result.lhr.categories).map(([name, value]) => [name, Math.round(value.score * 100)]));
      await fs.writeFile(new URL(`${name}-run-${sample}.json`, reports), result.report);
      console.log(`Lighthouse ${route || "home"} measurement ${sample}:`, scores);
      measurements.push({ scores, report: result.report });
    }
    const median = [...measurements].sort((a, b) => a.scores.performance - b.scores.performance)[1];
    await fs.writeFile(new URL(`${name}.json`, reports), median.report);
    const performance = assertLighthouseBudget(measurements.map(measurement => measurement.scores));
    console.log(`Lighthouse ${route || "home"} median performance: ${performance}`);
  }
} finally {
  await chrome.kill();
}
