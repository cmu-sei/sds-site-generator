import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";

// Use the real Rust-generated index, not a second implementation of indexing.
const temporary = await fs.mkdtemp(join(tmpdir(), "sds-search-fixture-"));
const site = join(temporary, "site");
const index = pathToFileURL(join(site, "dist/_site/search/"));
globalThis.fetch = async (url) => {
  const name = new URL(url).pathname.split("/").pop();
  try {
    return { ok: true, json: async () => JSON.parse(await fs.readFile(new URL(name, index), "utf8")) };
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    return { ok: false, status: 404 };
  }
};
const { search, tokenize, parseQuery } = await import("../assets/search-worker.js");

before(async () => {
  const binary = process.env.SITE_GENERATOR_BINARY || fileURLToPath(new URL(`../target/debug/sds-site-generator${process.platform === "win32" ? ".exe" : ""}`, import.meta.url));
  execFileSync(binary, ["new", site, "--url", "https://example.org/demo/"], { stdio: "pipe" });
  await fs.writeFile(join(site, "content/search-fixture.md"), "# Search fixture\n\nMarkdown files are searchable.\n");
  execFileSync(binary, ["build", site], { stdio: "pipe" });
  await fs.access(fileURLToPath(new URL("manifest.json", index)));
});

after(async () => {
  await fs.rm(temporary, { recursive: true, force: true });
});

test("Unicode tokenization and phrase parsing", () => {
  assert.deepEqual(tokenize("Café — Résumé 123"), ["café", "résumé", "123"]);
  assert.deepEqual(parseQuery('"exact phrase" prefix'), [
    { terms: ["exact", "phrase"], phrase: true },
    { terms: ["prefix"], phrase: false },
  ]);
});

test("title matches, prefix completion and subdirectory URLs", async () => {
  const result = await search("publis");
  assert.ok(result.total > 0);
  assert.equal(result.results[0].title, "Publishing");
  assert.ok(result.results.every((item) => item.url.startsWith("/demo/")));
});

test("phrases require adjacent terms in the correct order", async () => {
  assert.ok((await search('"plain text"')).total > 0);
  assert.equal((await search('"text plain"')).total, 0);
  assert.equal((await search('"plain headings text"')).total, 0);
  assert.equal((await search('"welcome welcome"')).total, 0);
});

test("AND queries and no-results cases", async () => {
  assert.equal((await search("Markdown files")).total, 1);
  assert.equal((await search("Markdown zzznotaword")).total, 0);
  assert.deepEqual(await search(""), { results: [], total: 0 });
  assert.deepEqual(await search("!!!"), { results: [], total: 0 });
  assert.equal((await search("constructor")).total, 0);
  assert.equal((await search("__proto__")).total, 0);
});
