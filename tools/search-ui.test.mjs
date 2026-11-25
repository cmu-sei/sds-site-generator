import { chromium } from "playwright";
import AxeBuilder from "@axe-core/playwright";
import assert from "node:assert/strict";

const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH });
try {
  const context = await browser.newContext();
  const page = await context.newPage({ viewport: { width: 1280, height: 900 } });
  await page.goto(process.env.SITE_URL || "http://127.0.0.1:4173/demo/");
  const dialog = page.getByRole("dialog");
  await page.keyboard.press("Meta+k");
  assert.equal(await dialog.isVisible(), true, "Cmd+K must open documentation search");
  assert.equal(await dialog.getByRole("searchbox").evaluate((element) => element === document.activeElement), true, "The shortcut must focus the search input");
  await page.keyboard.press("Escape");
  await dialog.waitFor({ state: "hidden" });
  await page.keyboard.press("Control+k");
  assert.equal(await dialog.isVisible(), true, "Ctrl+K must open documentation search");
  assert.equal(await dialog.getByRole("searchbox").evaluate((element) => element === document.activeElement), true, "Ctrl+K must focus the search input");
  await dialog.getByRole("button", { name: "Close documentation search" }).click();
  await dialog.waitFor({ state: "hidden" });
  assert.equal(await page.locator("[data-search-open]").evaluate((element) => element === document.activeElement), true, "Closing search must restore focus to the search button");
  for (const shortcut of ["k", "Meta+Shift+k", "Control+Shift+k", "Meta+Alt+k", "Control+Alt+k"]) {
    await page.keyboard.press(shortcut);
    assert.equal(await dialog.isVisible(), false, `${shortcut} must not open documentation search`);
  }
  await page.locator("[data-search-open]").click();
  assert.equal(await dialog.getByRole("heading").innerText(), "Search");
  assert.equal(await dialog.locator("#site-search-title").getAttribute("class"), "sds-text-title");
  assert.equal(await dialog.locator("#site-search-title").evaluate(element => getComputedStyle(element).fontSize), "20px");
  assert.equal(await dialog.getByRole("button", { name: "Search", exact: true }).count(), 0, "Live search must not show a submit button");
  const status = dialog.getByRole("status");
  const expectCallout = async () => {
    assert.equal(await status.evaluate((element) => element.matches('.sds-callout[data-sds-tone="info"]') && !element.classList.contains("sds-sr-only")), true, "The initial search prompt must use the visible informational callout recipe");
    assert.equal(await status.evaluate((element) => getComputedStyle(element).borderTopStyle), "solid", "The bundled callout border must render");
  };
  await expectCallout();
  await dialog.getByRole("searchbox").fill("zzzznomatchingsection");
  await page.waitForFunction(() => document.getElementById("site-dialog-status").textContent.startsWith("No results found."));
  assert.equal(await status.evaluate(element => element.matches(".sds-empty-state") && !element.matches(".sds-callout")), true, "No matches must use an empty state, not a callout");
  assert.equal(await status.getByRole("heading", { name: "No results found." }).isVisible(), true);
  assert.equal(await status.locator("h2.sds-text-h3").isVisible(), true, "Empty-state headings must use SDS's h3 visual style, not the dialog-title style");
  await dialog.getByRole("searchbox").fill("");
  await page.waitForFunction(() => document.getElementById("site-dialog-status").textContent.startsWith("Type a query"));
  await expectCallout();
  await dialog.getByRole("searchbox").fill("publis");
  await dialog.locator(".sds-search-results a").first().waitFor();
  assert.equal(await status.evaluate((element) => element.classList.contains("sds-sr-only") && !element.classList.contains("sds-callout")), true, "Successful results must hide the status without callout styling");
  assert.equal(await dialog.locator("ol").count(), 0, "Results must not be a numbered list");
  assert.equal(await dialog.locator(".sds-search-results a").first().locator("span").innerText(), "Publishing", "Title-prefix matches must outrank incidental code/body mentions");
  assert.equal(await dialog.locator(".sds-search-results p").count(), 0, "Palette results show context, not long excerpts");
  const row = await dialog.locator(".sds-search-results a").first().boundingBox();
  assert.ok(row.height <= 88, `Search row is too tall: ${row.height}`);
  await dialog.getByRole("searchbox").press("ArrowDown");
  assert.equal(await dialog.locator(".sds-search-results a").first().evaluate((element) => element === document.activeElement), true);
  await page.keyboard.press("Control+k");
  assert.equal(await dialog.getByRole("searchbox").evaluate((element) => element === document.activeElement), true, "Repeating the shortcut must refocus the open search input");
  assert.equal(await dialog.getByRole("searchbox").inputValue(), "publis", "Repeating the shortcut must preserve the query");
  await dialog.getByRole("searchbox").press("Meta+k");
  assert.equal(await dialog.getByRole("searchbox").evaluate((element) => element === document.activeElement), true, "The shortcut must keep focus in search while typing");
  await page.keyboard.press("Escape");
  await dialog.waitFor({ state: "hidden" });
  for (const mode of ["dialog", "page"]) {
    for (const colorScheme of ["light", "dark"]) {
      const stateContext = await browser.newContext({ colorScheme, viewport: { width: mode === "dialog" ? 390 : 1280, height: 900 } });
      await stateContext.addInitScript(() => {
        window.Worker = class {
          constructor() { window.searchWorker = this; }
          postMessage(request) { window.searchRequest = request; }
          terminate() { this.terminated = true; }
        };
      });
      const statePage = await stateContext.newPage();
      const origin = process.env.SITE_URL || "http://127.0.0.1:4173/demo/";
      await statePage.goto(mode === "page" ? new URL("search/", origin).href : origin);
      if (mode === "dialog") await statePage.locator("[data-search-open]").click();
      const region = mode === "dialog" ? statePage.getByRole("dialog") : statePage.locator("main");
      const input = region.getByRole("searchbox");
      const stateStatus = region.getByRole("status");
      const results = region.locator(".sds-search-results");
      const run = async query => {
        await input.fill(query);
        await statePage.waitForFunction(query => window.searchRequest?.query === query, query);
      };
      const reply = async data => statePage.evaluate(data => {
        window.searchWorker.onmessage({ data: { id: window.searchRequest.id, ...data } });
      }, data);
      await run("pending");
      const spinner = stateStatus.locator(".sds-spinner[aria-hidden='true']");
      assert.equal(await spinner.isVisible(), true);
      assert.deepEqual(await spinner.evaluate(element => {
        const style = getComputedStyle(element);
        return [style.width, style.height];
      }), ["48px", "48px"], "Search loading must use the large SDS section spinner");
      const spinnerBox = await spinner.boundingBox();
      const statusBox = await stateStatus.boundingBox();
      const labelBox = await stateStatus.locator("span").last().boundingBox();
      assert.ok(Math.abs(spinnerBox.x + spinnerBox.width / 2 - statusBox.x - statusBox.width / 2) <= 1, "The spinner must be horizontally centered in the search region");
      assert.ok(Math.abs(labelBox.x + labelBox.width / 2 - statusBox.x - statusBox.width / 2) <= 1, "The loading label must be centered beneath the spinner");
      assert.ok(labelBox.y >= spinnerBox.y + spinnerBox.height, "The loading label must appear below the spinner");
      assert.equal(await results.getAttribute("aria-busy"), "true");
      assert.equal(await stateStatus.evaluate(element => element.classList.contains("sds-callout")), false);
      await reply({ results: [], total: 0 });
      assert.equal(await stateStatus.evaluate(element => element.classList.contains("sds-empty-state")), true);
      assert.equal(await stateStatus.locator("h2.sds-text-h3").evaluate(element => getComputedStyle(element).fontSize),
        mode === "dialog" ? "20px" : "24px", "Empty-state headings must use SDS's responsive h3 scale");
      assert.equal(await stateStatus.locator(".sds-spinner").count(), 0);
      assert.equal(await results.getAttribute("aria-busy"), null);
      const emptyAccessibility = await new AxeBuilder({ page: statePage }).analyze();
      assert.deepEqual(emptyAccessibility.violations.map(({ id, nodes }) => ({ id, targets: nodes.map(({ target, failureSummary }) => ({ target, failureSummary })) })), [], `${mode} empty search accessibility in ${colorScheme}`);
      await run("failure");
      await reply({ error: "Could not load search data (503)." });
      assert.equal(await stateStatus.evaluate(element => element.matches('.sds-callout[data-sds-tone="danger"]') && !element.matches(".sds-empty-state, .sds-sr-only")), true);
      assert.match(await stateStatus.innerText(), /503.*Please try again/);
      const errorAccessibility = await new AxeBuilder({ page: statePage }).analyze();
      assert.deepEqual(errorAccessibility.violations.map(({ id, nodes }) => ({ id, targets: nodes.map(({ target, failureSummary }) => ({ target, failureSummary })) })), [], `${mode} search error accessibility in ${colorScheme}`);
      await run("worker-failure");
      await statePage.evaluate(() => window.searchWorker.onerror());
      assert.equal(await statePage.evaluate(() => window.searchWorker.terminated), true);
      assert.match(await stateStatus.innerText(), /search worker could not start/);
      await run("recovery");
      await reply({ results: [{ url: origin, title: "Recovered result", page: "Recovered result" }], total: 1 });
      await results.getByRole("link", { name: "Recovered result Documentation" }).waitFor();
      assert.equal(await stateStatus.evaluate(element => element.matches(".sds-sr-only") && !element.matches(".sds-callout, .sds-empty-state, .sds-stack, .sds-search-loading")), true);
      await input.press("ArrowDown");
      assert.equal(await results.locator("a").evaluate(element => element === document.activeElement), true);
      await input.fill("");
      await stateStatus.filter({ hasText: "Type a query" }).waitFor();
      assert.equal(await stateStatus.evaluate(element => element.matches('.sds-callout[data-sds-tone="info"]')), true);
      await stateContext.close();
    }
  }
  console.log("Search UI passed: shortcuts, dialog typography, loading spinner, empty/error recipes, recovery, accessible light/dark mobile/desktop states, ranking, keyboard navigation, and Escape.");
} finally {
  await browser.close();
}
