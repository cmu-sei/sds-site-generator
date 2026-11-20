import { chromium } from "playwright";
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
  assert.equal(await dialog.getByRole("button", { name: "Search", exact: true }).count(), 0, "Live search must not show a submit button");
  const status = dialog.getByRole("status");
  const expectCallout = async () => {
    assert.equal(await status.evaluate((element) => element.matches('.sds-callout[data-sds-tone="info"]') && !element.classList.contains("sds-sr-only")), true, "Empty search states must use the visible informational callout recipe");
    assert.equal(await status.evaluate((element) => getComputedStyle(element).borderTopStyle), "solid", "The bundled callout border must render");
  };
  await expectCallout();
  await dialog.getByRole("searchbox").fill("zzzznomatchingsection");
  await page.waitForFunction(() => document.getElementById("site-dialog-status").textContent.startsWith("No results found."));
  await expectCallout();
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
  console.log("Published-search parity passed: Cmd+K/Ctrl+K, modifier filtering, search focus, informational empty-state callouts, compact live results, title ranking, keyboard navigation, Escape.");
} finally {
  await browser.close();
}
