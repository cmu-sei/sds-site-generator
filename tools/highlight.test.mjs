import { test } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

test("the bundled highlighter colors documentation languages and preserves literal examples", async () => {
  const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH });
  try {
    const page = await browser.newPage();
    const examples = [
      { languages: ["markdown", "md"], text: '---\ntitle: Install\n---\n# Install\n\nUse **bold** and `code`.' },
      { languages: ["toml"], text: 'name = "My docs"\n[server]\nport = 4173' },
      { languages: ["powershell", "ps", "ps1", "pwsh"], text: '$name = "docs"\nWrite-Output $name' },
      { languages: ["ini", "properties"], text: "[server]\nport=4173\nenabled=true" },
      { languages: ["c"], text: "int main(void) {\n  return 0;\n}" },
    ].flatMap(({ languages, text }) => languages.map(language => ({ language, text })));
    examples.push({ language: "text", text: "::callout{tone=\"info\"}\nLiteral <example> & text.\n::" });
    await page.setContent("<main></main>");
    await page.evaluate(examples => {
      for (const { language, text } of examples) {
        const pre = document.createElement("pre");
        const code = document.createElement("code");
        code.className = `language-${language}`;
        code.textContent = text;
        pre.append(code);
        document.querySelector("main").append(pre);
      }
    }, examples);
    await page.addScriptTag({
      path: fileURLToPath(new URL("../assets/highlight.js", import.meta.url)),
      type: "module",
    });
    await page.waitForFunction(() => document.querySelector("code.language-c").dataset.highlighted === "true");
    for (const { language, text } of examples) {
      const code = page.locator(`code.language-${language}`);
      assert.equal(await code.textContent(), text, `${language} must preserve the example text`);
      if (language === "text") {
        assert.equal(await code.getAttribute("data-highlighted"), null);
        continue;
      }
      assert.equal(await code.getAttribute("data-highlighted"), "true", `${language} must be highlighted`);
      const styles = await code.locator("span[style]").evaluateAll(nodes => nodes.map(node => node.getAttribute("style")));
      assert.ok(new Set(styles).size > 1, `${language} must have multiple token styles`);
      assert.ok(styles.every(style => style.includes("--shiki-light:") && style.includes("--shiki-dark:")));
    }
  } finally {
    await browser.close();
  }
});
