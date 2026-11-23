import { test } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { build } from "esbuild";

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
      { languages: ["html"], text: '<style>body { color: red; }</style>\n<script>const count = 1;</script>\n<p title="Example">Hello</p>' },
      { languages: ["javascript", "js"], text: 'const name = "docs";\nconsole.log(name);' },
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

test("the highlighter initializes only requested grammar groups and skips plain-text pages", async () => {
  const core = fileURLToPath(new URL("./node_modules/shiki/dist/core.mjs", import.meta.url));
  const result = await build({
    entryPoints: [fileURLToPath(new URL("./highlight.mjs", import.meta.url))],
    bundle: true,
    format: "esm",
    write: false,
    plugins: [{
      name: "observe-highlighter-initialization",
      setup(build) {
        build.onResolve({ filter: /^shiki\/core$/ }, () => ({ path: "core", namespace: "observe" }));
        build.onLoad({ filter: /^core$/, namespace: "observe" }, () => ({
          resolveDir: fileURLToPath(new URL("./", import.meta.url)),
          contents: `import { createHighlighterCore as create } from ${JSON.stringify(core)};
export function createHighlighterCore(options) {
  globalThis.loadedGrammarGroups = options.langs.map(grammar => grammar.name);
  return create(options);
}`,
        }));
      },
    }],
  });
  const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH });
  try {
    for (const language of ["md", "text", "not-a-language"]) {
      const page = await browser.newPage();
      await page.setContent(`<main><pre><code class="language-${language}"># Hello **world**</code></pre><pre><code class="language-${language}"># Another example</code></pre></main>`);
      await page.addScriptTag({ content: `${result.outputFiles[0].text}\nglobalThis.highlightFinished = true;`, type: "module" });
      await page.waitForFunction(() => globalThis.highlightFinished);
      const loaded = await page.evaluate(() => globalThis.loadedGrammarGroups);
      assert.deepEqual(loaded, language === "md" ? ["markdown"] : undefined);
      assert.equal(await page.locator("code").first().textContent(), "# Hello **world**");
      assert.equal(await page.locator("code").first().getAttribute("data-highlighted"), language === "md" ? "true" : null);
      await page.close();
    }
  } finally {
    await browser.close();
  }
});
