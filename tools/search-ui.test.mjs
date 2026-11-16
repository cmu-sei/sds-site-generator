import { chromium } from "playwright";
import assert from "node:assert/strict";

const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH });
try {
  const context = await browser.newContext();
  const page = await context.newPage({ viewport: { width: 1280, height: 900 } });
  await page.goto(process.env.SITE_URL || "http://127.0.0.1:4173/demo/");
  await page.locator("[data-search-open]").click();
  const dialog = page.getByRole("dialog");
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
  await page.keyboard.press("Escape");
  await dialog.waitFor({ state: "hidden" });
  console.log("Published-search parity passed: informational empty-state callouts, compact live results, title ranking, keyboard navigation, Escape.");
} finally {
  await browser.close();
}
