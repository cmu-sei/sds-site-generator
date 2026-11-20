import { chromium } from "playwright";
import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, writeFile, rm } from "node:fs/promises";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = await mkdtemp(join(tmpdir(), "code-components-"));
const binary = process.env.SITE_GENERATOR_BINARY || fileURLToPath(new URL(`../target/debug/sds-site-generator${process.platform === "win32" ? ".exe" : ""}`, import.meta.url));
let preview;
let browser;
try {
  await mkdir(join(root, "content"));
  await writeFile(join(root, "site.toml"), "name = 'Code tests'\nurl = 'https://example.org/test/'\nnavigation = [{ title = 'Examples', children = [{ title = 'Code examples', path = '/' }, { title = 'Parameter validation', path = '/#code-examples' }] }, { title = 'Reference', children = [{ title = 'Examples home', path = '/' }] }, { title = 'More', children = [{ title = 'Nested examples', children = [{ title = 'Nested home', path = '/' }] }] }]\n");
  const siteTomlPath = join(root, "site.toml");
  const siteConfig = await readFile(siteTomlPath, "utf8");
  assert.ok(siteConfig.includes("{ title = 'Examples', children = ["));
  const withHomeNavigation = siteConfig.replace("navigation = [", "navigation = [{ title = 'Home', path = '/' }, ");
  assert.notEqual(withHomeNavigation, siteConfig);
  const childRouteConfig = withHomeNavigation.replace("title = 'Code examples', path = '/'", "title = 'Code examples', path = '/components/'");
  assert.notEqual(childRouteConfig, siteConfig);
  await writeFile(siteTomlPath, childRouteConfig.replace("{ title = 'Examples', children = [", "{ title = 'Examples', path = '/', children = ["));
  await writeFile(join(root, "content/index.md"), `# Code examples

Interactive example regression tests.

## Examples

[Code reference](https://example.org/reference)

| Control | State |
| --- | --- |
| Copy | Ready |

## Parameter checks

::code-block{title='Parameter validation' quality='good'}
#default
\`\`\`c
return 0;
\`\`\`
#complete
\`\`\`c
int main(void) {
  return 0;
}
\`\`\`
::

### Implementation details

This lower-level heading should remain in the page but not in the on-page navigation.

## Additional examples

::code-block{quality='bad'}
\`\`\`c
extern int setfile(FILE *file);
// ${"Long code line. ".repeat(24)}
\`\`\`
::

${"Supporting reference information for parameter validation and secure coding.\n\n".repeat(16)}

## Final notes

Review the examples before publishing.
`);
  const tones = ["neutral", "accent", "info", "success", "warning", "danger"];
  const callouts = tones.flatMap(tone => ["", "outline", "bold"].map(variant =>
    `::callout{tone='${tone}'${variant ? ` variant='${variant}'` : ""} size='sm'}\n**${tone} ${variant || "subtle"}:** Shared component content.\n::\n`
  )).join("\n");
  await writeFile(join(root, "content/components.md"), `# Shared components

Reference content component regression tests.

## Callouts

${callouts}
## Plain code and tables

\`\`\`text
Plain fenced code.
\`\`\`

| Name | Status | Score |
| :--- | :---: | ---: |
| Beta | Active | 9.12 |
| Alpha | Active | 9.5 |
| Gamma | Beta | -1 |
`);
  preview = spawn(binary, ["preview", root, "--port", "0"], { stdio: ["ignore", "pipe", "pipe"] });
  const url = await new Promise((resolveReady, reject) => {
    let output = "";
    preview.stdout.on("data", (chunk) => {
      output += chunk;
      const match = output.match(/Preview: (http:\/\/[^\s]+)/);
      if (match) resolveReady(match[1]);
    });
    preview.stderr.on("data", (chunk) => { output += chunk; });
    preview.on("error", reject);
    preview.on("exit", (code) => reject(new Error(`Preview exited ${code}: ${output}`)));
  });
  browser = await chromium.launch({ executablePath: process.env.CHROME_PATH });
  const context = await browser.newContext();
  await context.addInitScript(() => {
    window.copiedCode = [];
    Object.defineProperty(navigator, "clipboard", { configurable: true, value: {
      writeText: async (text) => {
        if (window.failClipboard) throw new Error("Clipboard denied");
        window.copiedCode.push(text);
      },
    } });
  });
  const components = await context.newPage();
  const componentSchemes = process.env.CODE_BLOCK_ONLY === "1" ? [] : ["light", "dark"];
  for (const scheme of componentSchemes) {
    await components.emulateMedia({ colorScheme: scheme });
    for (const width of [1280, 390]) {
      await components.setViewportSize({ width, height: 900 });
      await components.goto(`${url}components/`);
      const table = components.locator("article table");
      const score = table.getByRole("button", { name: "Sort by Score", exact: true });
      await score.waitFor({ state: "visible" });
      assert.equal(await table.getAttribute("class"), "sds-table");
      assert.equal(await table.locator("th button svg[data-sds-sort-chevron][aria-hidden='true']").count(), 3);
      const headerStyles = await table.locator("th").evaluateAll(headers => headers.map(header => {
        const cell = getComputedStyle(header);
        const control = getComputedStyle(header.querySelector("button"));
        const properties = ["color", "fontSize", "fontWeight", "lineHeight"];
        return { alignment: cell.textAlign, cell: properties.map(property => cell[property]), control: properties.map(property => control[property]) };
      }));
      assert.deepEqual(headerStyles.map(style => style.alignment), ["left", "center", "right"]);
      assert.deepEqual(await table.locator("th").evaluateAll(headers => headers.map(header => header.getAttribute("aria-sort"))), ["none", "none", "none"]);
      for (const style of headerStyles) assert.deepEqual(style.control, style.cell);
      assert.equal(await score.locator("svg").evaluate(element => getComputedStyle(element).flexShrink), "0");
      await score.press("Enter");
      assert.deepEqual(await table.locator("tbody td:last-child").allTextContents(), ["-1", "9.12", "9.5"]);
      assert.equal(await table.locator("th").last().getAttribute("aria-sort"), "ascending");
      assert.equal(await score.locator("svg").evaluate(element => getComputedStyle(element).opacity), "1");
      assert.equal(await score.locator("svg").evaluate(element => getComputedStyle(element).transform), "matrix(0, -1, 1, 0, 0, 0)");
      await score.press("Space");
      assert.deepEqual(await table.locator("tbody td:last-child").allTextContents(), ["9.5", "9.12", "-1"]);
      assert.equal(await table.locator("th").last().getAttribute("aria-sort"), "descending");
      assert.equal(await score.locator("svg").evaluate(element => getComputedStyle(element).opacity), "1");
      await components.waitForFunction(element => getComputedStyle(element).transform === "matrix(0, 1, -1, 0, 0, 0)", await score.locator("svg").elementHandle());
      assert.equal(await score.locator("svg").evaluate(element => getComputedStyle(element).transform), "matrix(0, 1, -1, 0, 0, 0)");
      await table.getByRole("button", { name: "Sort by Name", exact: true }).click();
      assert.deepEqual(await table.locator("tbody td:first-child").allTextContents(), ["Alpha", "Beta", "Gamma"]);
      assert.equal(await table.locator("th").last().getAttribute("aria-sort"), "none");
      assert.equal(await components.locator("aside.sds-callout[role='note']").count(), 18);
      const recipes = await components.locator("aside.sds-callout").evaluateAll(elements => elements.map(element => {
        const style = getComputedStyle(element);
        const paragraph = getComputedStyle(element.querySelector("p"));
        return { tone: element.dataset.sdsTone, variant: element.dataset.sdsVariant || "subtle",
          background: style.backgroundColor, text: style.color, paragraphText: paragraph.color,
          top: paragraph.marginTop, bottom: paragraph.marginBottom, padding: style.paddingTop };
      }));
      for (const recipe of recipes) {
        assert.equal(recipe.text, recipe.paragraphText);
        assert.equal(recipe.top, "0px");
        assert.equal(recipe.bottom, "0px");
        assert.notEqual(recipe.padding, "0px");
        if (recipe.variant !== "outline") assert.notEqual(recipe.background, "rgba(0, 0, 0, 0)");
      }
      const background = tone => recipes.find(recipe => recipe.tone === tone && recipe.variant === "subtle").background;
      assert.notEqual(background("info"), background("warning"));
      assert.notEqual(background("neutral"), background("accent"));
      assert.equal(await components.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
      assert.equal(await components.locator("article pre[tabindex='0'][role='group']").count(), 1);
    }
  }
  await components.close();
  const page = await context.newPage({ viewport: { width: 1280, height: 900 } });
  await page.goto(url);
  assert.equal(await page.locator(".sds-app-footer > p[data-sds-margin='none']").count(), 0);
  assert.equal(await page.getByRole("link", { name: "Acknowledgments", exact: true }).count(), 0);
  assert.equal(await page.getByRole("dialog", { name: "Design system acknowledgments" }).count(), 0);
  const toc = page.locator("#site-main .sds-app-toc");
  const assertCurrentSection = async (id) => {
    await page.waitForFunction((id) => {
      const current = [...document.querySelectorAll(".sds-app-toc a[aria-current='location']")];
      return current.length === 2 && current.every((link) => link.getAttribute("href") === `#${id}`);
    }, id);
  };
  await assertCurrentSection("examples");
  assert.equal(await toc.locator("> strong > svg[data-sds-toc-icon] path").count(), 6);
  assert.equal(await toc.getByRole("link", { name: "Implementation details", exact: true }).count(), 0);
  assert.equal(await page.locator(".sds-app-toc > strong > svg[aria-hidden='true']").count(), 2);
  await page.locator("#parameter-checks").evaluate((heading) => scrollTo({ top: heading.getBoundingClientRect().top + scrollY - document.querySelector(".sds-app-header").offsetHeight - 29, behavior: "instant" }));
  await assertCurrentSection("parameter-checks");
  await page.locator("#implementation-details").evaluate((heading) => scrollTo({ top: heading.getBoundingClientRect().top + scrollY - document.querySelector(".sds-app-header").offsetHeight - 29, behavior: "instant" }));
  await assertCurrentSection("parameter-checks");
  assert.deepEqual(await toc.locator("a[aria-current='location']").evaluate((link) => {
    const style = getComputedStyle(link);
    return { weight: style.fontWeight, border: style.borderInlineStartWidth,
      matchingColor: style.color === style.borderInlineStartColor,
      aligned: Math.abs(link.getBoundingClientRect().left - link.closest("ul").getBoundingClientRect().left) < 1 };
  }), { weight: "600", border: "2px", matchingColor: true, aligned: true });
  await page.locator("#additional-examples").evaluate((heading) => scrollTo({ top: heading.getBoundingClientRect().top + scrollY - document.querySelector(".sds-app-header").offsetHeight - 29, behavior: "instant" }));
  await assertCurrentSection("additional-examples");
  await page.evaluate(() => scrollTo({ top: document.documentElement.scrollHeight, behavior: "instant" }));
  await assertCurrentSection("final-notes");
  await toc.locator("a[href='#examples']").click();
  await assertCurrentSection("examples");
  assert.equal(new URL(page.url()).hash, "#examples");
  await page.goto(`${url}#additional-examples`);
  await assertCurrentSection("additional-examples");
  await page.evaluate(() => scrollTo({ top: 0, behavior: "instant" }));
  await assertCurrentSection("examples");
  assert.equal(await page.locator(".sds-table-container[tabindex='0'] table").count(), 1);
  assert.equal(await page.locator("nav[aria-label='Breadcrumb']").count(), 0);
  assert.equal(await page.locator("#site-main article.sds-prose").getAttribute("data-sds-tone"), "info");
  assert.equal(await page.getByRole("link", { name: "Code reference", exact: true }).evaluate((element) => getComputedStyle(element).color), "rgb(2, 102, 161)");
  const dropdown = page.locator(".sds-app-header sds-dropdown").first();
  const trigger = dropdown.locator(":scope > button");
  const menu = dropdown.locator("menu.sds-dropdown-menu");
  await page.waitForFunction(() => customElements.get("sds-dropdown"));
  await trigger.press("ArrowDown");
  await menu.waitFor({ state: "visible" });
  const overview = menu.locator("li.site-primary-overview");
  const overviewLink = overview.getByRole("menuitem", { name: "Examples", exact: true });
  assert.equal(await overviewLink.getAttribute("href"), "/test/");
  assert.equal(await overviewLink.getAttribute("data-sds-variant"), "cta");
  assert.equal(await overviewLink.evaluate(link => link.classList.contains("sds-link")), false);
  const overviewDivider = overview.locator(":scope > hr.sds-dropdown-divider");
  assert.equal(await overviewDivider.count(), 1);
  const overviewBounds = await overview.boundingBox();
  const overviewDividerBounds = await overviewDivider.boundingBox();
  assert.ok(overviewDividerBounds.width >= overviewBounds.width - 2);
  assert.equal(await overviewLink.evaluate(link => getComputedStyle(link, "::after").borderTopWidth), "2px");
  assert.match(await overviewLink.evaluate(link => getComputedStyle(link, "::after").transform), /, 0, 0\)$/);
  assert.equal(await overviewLink.evaluate(link => getComputedStyle(link, "::after").width), "8px");
  const overviewTypography = await overviewLink.evaluate(element => {
    const style = getComputedStyle(element);
    return { fontSize: style.fontSize, fontWeight: style.fontWeight };
  });
  const menuItemTypography = await menu.getByRole("menuitem", { name: "Code examples", exact: true }).evaluate(element => {
    const style = getComputedStyle(element);
    return { fontSize: style.fontSize, fontWeight: style.fontWeight };
  });
  assert.equal(overviewTypography.fontSize, menuItemTypography.fontSize);
  assert.equal(overviewTypography.fontWeight, "600");
  const idleArrowTransform = await overviewLink.evaluate(link => getComputedStyle(link, "::after").transform);
  assert.equal(await overviewLink.evaluate(link => getComputedStyle(link, "::after").borderTopWidth), "2px");
  await overviewLink.hover();
  await page.waitForFunction((link, transform) => getComputedStyle(link, "::after").transform !== transform, await overviewLink.elementHandle(), idleArrowTransform);
  await page.mouse.move(0, 0);
  assert.equal(await menu.locator("a").first().evaluate((element) => element === document.activeElement), true);
  await page.keyboard.press("End");
  assert.equal(await menu.locator("a").last().evaluate((element) => element === document.activeElement), true);
  await page.keyboard.press("Escape");
  await menu.waitFor({ state: "hidden" });
  assert.equal(await trigger.evaluate((element) => element === document.activeElement), true);
  await trigger.click();
  await page.locator(".sds-app-header sds-dropdown").nth(1).locator("button").click();
  await menu.waitFor({ state: "hidden" });
  await page.getByRole("heading", { name: "Code examples", exact: true }).click();
  assert.equal(await page.locator("menu:popover-open").count(), 0);
  const hierarchy = page.locator(".sds-app-header details.sds-disclosure");
  await hierarchy.locator("summary").click();
  await hierarchy.locator("sds-dropdown > button").click();
  await hierarchy.getByRole("menuitem", { name: "Nested home", exact: true }).waitFor({ state: "visible" });
  await page.getByRole("heading", { name: "Code examples", exact: true }).click();
  assert.equal(await hierarchy.getAttribute("open"), null);
  const block = page.locator(".sds-code-block:has(> header > strong)");
  await block.locator("[data-highlighted='true']").first().waitFor();
  assert.equal(await block.locator("header > nav button:not(.sds-button)").count(), 0);
  assert.equal(await block.locator(".sds-button[data-sds-shape='icon'][data-sds-density='compact'][data-sds-variant='text']").count(), 3);
  assert.equal(await block.locator("header > nav > sds-tooltip[size='auto']").count(), 3);
  const badgeTooltip = block.locator("header > sds-tooltip:has(> .sds-badge)");
  const badge = badgeTooltip.locator(":scope > .sds-badge");
  assert.equal(await badge.getAttribute("aria-label"), "Compliant code");
  assert.equal(await badge.innerText(), "");
  assert.equal(await badge.getAttribute("title"), null);
  await badge.hover();
  const badgeTooltipContent = badgeTooltip.locator("[role='tooltip']");
  await badgeTooltipContent.waitFor({ state: "visible" });
  assert.equal(await badgeTooltipContent.innerText(), "Compliant code");
  await block.locator("pre").first().hover();
  assert.deepEqual(await block.evaluate((element) => {
    const pre = getComputedStyle(element.querySelector("pre"));
    const code = getComputedStyle(element.querySelector("pre code"));
    return { preFont: pre.fontSize, preLine: pre.lineHeight, padding: pre.padding,
      margin: pre.margin, codeFont: code.fontSize, codeLine: code.lineHeight,
      border: getComputedStyle(element).borderWidth, radius: getComputedStyle(element).borderRadius };
  }), { preFont: "16px", preLine: "24px", padding: "16px", margin: "16px 0px",
    codeFont: "16px", codeLine: "24px", border: "1px", radius: "8px" });
  assert.deepEqual(await block.evaluate(element => {
    const body = document.body;
    const app = document.querySelector(".sds-app");
    const radius = () => getComputedStyle(element).borderRadius;
    const forge = radius();
    body.dataset.sdsTheme = "plaid";
    app.dataset.sdsTheme = "plaid";
    const plaid = radius();
    body.dataset.sdsTheme = "forge";
    app.dataset.sdsTheme = "forge";
    return { forge, plaid };
  }), { forge: "8px", plaid: "0px" });
  const bad = page.locator("site-code-block[quality='bad']");
  assert.notEqual(await bad.evaluate((element) => getComputedStyle(element).borderColor), await block.evaluate((element) => getComputedStyle(element).borderColor));
  assert.equal(await bad.locator("header > sds-tooltip > .sds-badge").getAttribute("aria-label"), "Non-compliant code");
  assert.equal(await bad.locator("header > sds-tooltip > .sds-badge").innerText(), "");
  assert.equal(await block.locator("[data-sds-code-slot='complete']").isVisible(), false);
  const copy = block.getByRole("button", { name: "Copy to clipboard", exact: true });
  const copyButton = block.locator("button[data-sds-action='copy-code']");
  const copyTooltip = block.locator("header > nav > sds-tooltip:has(> button[data-sds-action='copy-code'])");
  await copy.hover();
  const copyTooltipContent = copyTooltip.locator("[role='tooltip']");
  await copyTooltipContent.waitFor({ state: "visible" });
  assert.equal(await copyTooltipContent.innerText(), "Copy to clipboard");
  const copyTooltipWidth = await copyTooltipContent.evaluate(element => element.getBoundingClientRect().width);
  assert.equal(await copy.getAttribute("title"), null);
  await block.locator("pre").first().hover();
  await copyTooltipContent.waitFor({ state: "hidden" });
  await page.clock.install();
  await copy.click();
  await copyTooltipContent.waitFor({ state: "visible" });
  await block.getByRole("button", { name: "Copied", exact: true }).waitFor();
  assert.equal(await copyTooltipContent.innerText(), "Copied");
  assert.ok(Math.abs(await copyTooltipContent.evaluate(element => element.getBoundingClientRect().width) - copyTooltipWidth) < 1);
  assert.equal(await copyButton.locator("[data-sds-copy-check]").count(), 1);
  assert.equal(await copyButton.locator("[data-sds-copy-icon]").evaluate(icon => icon.hasAttribute("hidden")), true);
  assert.equal(await copyButton.locator("[data-sds-copy-icon]").evaluate(icon => getComputedStyle(icon).display), "none");
  assert.notEqual(await copyButton.locator("[data-sds-copy-check]").evaluate(icon => getComputedStyle(icon).color), await copyButton.evaluate(button => getComputedStyle(button).color));
  assert.deepEqual(await page.evaluate(() => window.copiedCode), ["return 0;\n"]);
  await page.clock.runFor(2100);
  await copy.waitFor({ state: "visible" });
  assert.equal(await copyTooltipContent.innerText(), "Copy to clipboard");
  assert.equal(await copyButton.locator("[data-sds-copy-check]").count(), 0);
  assert.equal(await copyButton.locator("[data-sds-copy-icon]").evaluate(icon => icon.hasAttribute("hidden")), false);
  assert.notEqual(await copyButton.locator("[data-sds-copy-icon]").evaluate(icon => getComputedStyle(icon).display), "none");
  await block.getByRole("button", { name: "Toggle complete code" }).click();
  assert.equal(await block.locator("[data-sds-code-slot='default']").isVisible(), false);
  assert.equal(await block.locator("[data-sds-code-slot='complete']").isVisible(), true);
  await copy.click();
  assert.match((await page.evaluate(() => window.copiedCode)).at(-1), /int main\(void\)/);
  assert.doesNotMatch((await page.evaluate(() => window.copiedCode)).at(-1), /Copied!|Compliant/);
  const fullscreen = block.locator("button[data-sds-action='toggle-code-fullscreen']");
  const fullscreenTooltip = block.locator("header > nav > sds-tooltip:has(> button[data-sds-action='toggle-code-fullscreen']) [role='tooltip']");
  const originalHeight = (await block.boundingBox()).height;
  await fullscreen.click();
  const dialog = page.locator("dialog.sds-dialog:has(> .sds-code-block)");
  await dialog.waitFor({ state: "visible" });
  assert.equal(await fullscreenTooltip.innerText(), "Exit fullscreen");
  await fullscreenTooltip.waitFor({ state: "hidden" });
  assert.equal(await fullscreen.getAttribute("title"), null);
  assert.equal(await dialog.evaluate((element) => element.classList.contains("sds-dialog")), true);
  assert.equal(await fullscreen.getAttribute("aria-pressed"), "true");
  assert.equal(await page.evaluate(() => document.documentElement.style.overflow), "hidden");
  assert.ok(Math.abs((await page.locator("._sds-code-placeholder").boundingBox()).height - originalHeight) <= 1);
  await page.keyboard.press("Escape");
  await page.emulateMedia({ colorScheme: "dark" });
  assert.equal(await block.evaluate((element) => getComputedStyle(element).backgroundColor), "rgb(0, 0, 0)");
  await page.emulateMedia({ colorScheme: "light" });
  await dialog.waitFor({ state: "detached" });
  assert.equal(await fullscreenTooltip.innerText(), "Enter fullscreen");
  assert.equal(await fullscreen.evaluate((element) => element === document.activeElement), true);
  assert.equal(await page.evaluate(() => document.documentElement.style.overflow), "");
  await fullscreen.click();
  await fullscreen.click();
  await dialog.waitFor({ state: "detached" });
  await bad.getByRole("button", { name: "Toggle fullscreen" }).click();
  const untitledDialog = page.locator("dialog.sds-dialog:has(> .sds-code-block)");
  await untitledDialog.waitFor({ state: "visible" });
  const fullscreenHeaderSpacing = await bad.locator("header").evaluate((header) => {
    const headerBounds = header.getBoundingClientRect();
    const nav = header.querySelector("nav").getBoundingClientRect();
    return { padding: getComputedStyle(header).padding, rightInset: Math.round(headerBounds.right - nav.right), topInset: Math.round(nav.top - headerBounds.top) };
  });
  assert.equal(fullscreenHeaderSpacing.padding, "8px 16px");
  assert.ok(fullscreenHeaderSpacing.rightInset >= 12);
  assert.ok(fullscreenHeaderSpacing.topInset >= 8);
  await page.keyboard.press("Escape");
  await untitledDialog.waitFor({ state: "detached" });
  await page.evaluate(() => { window.failClipboard = true; });
  await copy.click();
  await block.locator("output:not(.sds-sr-only)").waitFor();
  assert.match(await block.locator("output").innerText(), /Copy failed/);
  await page.setViewportSize({ width: 390, height: 844 });
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
  await page.evaluate(() => scrollTo({ top: document.documentElement.scrollHeight, behavior: "instant" }));
  await page.clock.runFor(30);
  await assertCurrentSection("final-notes");
  await page.getByRole("button", { name: "Open site navigation", exact: true }).click();
  const mobilePanel = page.locator("#site-sidebar details.sds-navigation-panel").first();
  const mobileSummary = mobilePanel.locator(":scope > summary");
  assert.equal(await mobileSummary.getAttribute("data-sds-active"), "");
  assert.equal(await mobileSummary.evaluate(element => getComputedStyle(element).borderInlineStartWidth), "4px");
  const rootHomeLink = page.locator("#site-sidebar .sds-app-navigation > ul > li > a").first();
  const rootTextStart = element => element.evaluate(node => {
    const range = document.createRange();
    range.selectNodeContents(node);
    return range.getBoundingClientRect().left;
  });
  assert.ok(Math.abs(await rootTextStart(rootHomeLink) - await rootTextStart(mobileSummary)) < 1);
  const activeOverview = mobilePanel.locator("a[data-sds-navigation-overview]");
  assert.equal(await activeOverview.getAttribute("aria-current"), "page");
  assert.equal(await activeOverview.evaluate(element => getComputedStyle(element).borderInlineStartColor), "rgb(196, 18, 48)");
  await mobileSummary.click();
  await page.clock.runFor(300);
  const mobileMenu = mobilePanel.locator(":scope > div");
  await mobileMenu.waitFor({ state: "visible" });
  await page.waitForFunction(() => {
    const menu = document.querySelector("#site-sidebar details.sds-navigation-panel[open] > div");
    if (!menu) return false;
    const bounds = menu.getBoundingClientRect();
    return bounds.left >= 0 && bounds.right <= innerWidth;
  });
  const menuBounds = await mobileMenu.boundingBox();
  assert.ok(menuBounds.x >= 0 && menuBounds.x + menuBounds.width <= 390);
  const back = mobilePanel.locator("button[data-sds-action='navigation-back']");
  const backLayout = await back.evaluate(button => {
    const panel = button.closest("details").getBoundingClientRect();
    const bounds = button.getBoundingClientRect();
    const icon = button.querySelector("svg").getBoundingClientRect();
    const label = button.querySelector("[data-sds-navigation-back-label]").getBoundingClientRect();
    return { panelX: panel.x, panelWidth: panel.width, buttonX: bounds.x, buttonWidth: bounds.width,
      iconX: icon.x, labelGap: label.x - icon.right };
  });
  assert.ok(Math.abs(backLayout.buttonX - backLayout.panelX) < 1);
  assert.ok(Math.abs(backLayout.buttonWidth - backLayout.panelWidth) < 1);
  assert.ok(Math.abs(backLayout.iconX - backLayout.panelX - 16) < 1);
  assert.equal(backLayout.labelGap, 12);
  assert.deepEqual(await back.evaluate(button => {
    const body = document.body;
    const app = document.querySelector(".sds-app");
    const style = () => {
      const current = getComputedStyle(button);
      return { family: current.fontFamily, weight: current.fontWeight };
    };
    const forge = style();
    body.dataset.sdsTheme = "plaid";
    app.dataset.sdsTheme = "plaid";
    const plaid = style();
    body.dataset.sdsTheme = "forge";
    app.dataset.sdsTheme = "forge";
    return { forge, plaid };
  }), { forge: { family: '"Open Sans", ui-sans-serif, system-ui, -apple-system, "system-ui", "Segoe UI", sans-serif', weight: "600" },
    plaid: { family: '"Source Serif", ui-serif, Georgia, Cambria, "Times New Roman", serif', weight: "400" } });
  await page.keyboard.press("Escape");
  await page.clock.runFor(300);
  await mobileMenu.waitFor({ state: "hidden" });
  assert.equal(await bad.locator("pre").evaluate((element) => element.scrollWidth > element.clientWidth), true);
  await fullscreen.click();
  const bounds = await dialog.boundingBox();
  assert.ok(bounds.x >= 0 && bounds.x + bounds.width <= 390);
  await page.keyboard.press("Escape");
  const noScript = await browser.newPage({ javaScriptEnabled: false });
  await noScript.goto(url);
  assert.equal(await noScript.locator(".sds-app-toc a[href^='#']").count(), 8);
  assert.equal(await noScript.getByRole("link", { name: "Implementation details", exact: true }).count(), 0);
  assert.equal(await noScript.getByRole("heading", { name: "Implementation details", exact: true }).isVisible(), true);
  const nativeDropdown = noScript.locator(".sds-app-header sds-dropdown").first();
  await nativeDropdown.locator("button").click();
  assert.equal(await nativeDropdown.locator("menu").isVisible(), true);
  assert.equal(await noScript.locator("site-code-block [data-sds-action='copy-code']").first().isVisible(), false);
  assert.equal(await noScript.locator("site-code-block [data-sds-code-slot='default']").first().isVisible(), true);
  assert.match(await noScript.locator("site-code-block pre code").first().textContent(), /return 0/);
  await noScript.goto(`${url}components/`);
  assert.equal(await noScript.locator("aside.sds-callout[role='note']").count(), 18);
  assert.equal(await noScript.locator("article table button").count(), 0);
  assert.deepEqual(await noScript.locator("article table tbody td:first-child").allTextContents(), ["Beta", "Alpha", "Gamma"]);
  assert.equal(await noScript.locator("article pre[tabindex='0']").count(), 1);
  const childPage = await context.newPage({ viewport: { width: 1280, height: 900 } });
  await childPage.goto(`${url}components/`);
  const parentDropdown = childPage.locator(".sds-app-header sds-dropdown").first();
  const parentTrigger = parentDropdown.locator(":scope > button");
  assert.equal(await parentDropdown.locator("menu a[aria-current='page']").count(), 1);
  assert.equal(await parentDropdown.locator("li.site-primary-overview a").getAttribute("aria-current"), null);
  assert.equal(await parentTrigger.getAttribute("aria-expanded"), "false");
  assert.match(await parentTrigger.evaluate(button => getComputedStyle(button).boxShadow), /inset/);
  const parentPage = await context.newPage({ viewport: { width: 1280, height: 900 } });
  await parentPage.goto(url);
  const activeParent = parentPage.locator(".sds-app-header sds-dropdown").first();
  const activeParentTrigger = activeParent.locator(":scope > button");
  assert.equal(await activeParent.locator("li.site-primary-overview a").getAttribute("aria-current"), "page");
  assert.match(await activeParentTrigger.evaluate(button => getComputedStyle(button).boxShadow), /inset/);
  await childPage.setViewportSize({ width: 390, height: 844 });
  await childPage.getByRole("button", { name: "Open site navigation" }).click();
  const childPanel = childPage.locator("#site-sidebar details.sds-navigation-panel").first();
  const parentSummary = childPanel.locator(":scope > summary");
  await parentSummary.click();
  const activeChild = childPanel.locator("a[aria-current='page']");
  await activeChild.waitFor({ state: "visible" });
  const childPanelSummary = childPanel.locator(":scope > summary");
  assert.equal(await childPanelSummary.evaluate(element => getComputedStyle(element).borderInlineStartColor), await activeChild.evaluate(element => getComputedStyle(element).borderInlineStartColor));
  assert.equal(await childPanelSummary.getAttribute("data-sds-active"), null);
  assert.equal(await activeChild.evaluate(element => getComputedStyle(element).borderInlineStartWidth), "4px");
  assert.equal(await activeChild.evaluate(element => getComputedStyle(element, "::before").content), "none");
  const mobileOverview = childPanel.locator("a[data-sds-navigation-overview]");
  assert.equal(await mobileOverview.evaluate(link => new URL(link.href).pathname), "/test/");
  assert.equal(await mobileOverview.getAttribute("aria-current"), null);
  assert.equal(await mobileOverview.evaluate(link => getComputedStyle(link).borderInlineStartWidth), "4px");
  assert.notEqual(await mobileOverview.evaluate(link => getComputedStyle(link).borderInlineStartColor), await activeChild.evaluate(link => getComputedStyle(link).borderInlineStartColor));
  assert.equal(await mobileOverview.evaluate(link => getComputedStyle(link, "::before").content), "none");
  await mobileOverview.click();
  await childPage.waitForURL(url);
  console.log("SDS recipes passed: callout tones/variants/light/dark/mobile/no-JS, semantic tables and keyboard/numeric sorting with bundled icons, plain fences, scrollspy, dropdown navigation, code controls, highlighting, copying, fullscreen, and visual parity.");
} finally {
  await browser?.close();
  if (preview && preview.exitCode === null) {
    preview.kill();
    await once(preview, "exit");
  }
  await rm(root, { recursive: true, force: true });
}