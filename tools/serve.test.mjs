import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { gzipSync } from "node:zlib";
import { serveStatic } from "./serve.mjs";

test("the browser fixture serves concurrent pages and module scripts at a project URL", async () => {
  const root = await mkdtemp(join(tmpdir(), "static-fixture-"));
  let server;
  try {
    await mkdir(join(root, "components"));
    await mkdir(join(root, "_site"));
    await writeFile(join(root, "components/index.html"), "<h1>Components</h1>");
    await writeFile(join(root, "_site/auto.js"), "export const ready = true;");
    await writeFile(join(root, "_site/auto.js.gz"), gzipSync("export const ready = true;"));
    await writeFile(join(root, "_site/site.css"), "body { color: black; }");
    const fixture = await serveStatic(root, "/test/");
    server = fixture.server;
    const resources = [
      ["components/", "text/html", "<h1>Components</h1>"],
      ["_site/auto.js", "text/javascript", "export const ready = true;"],
      ["_site/site.css", "text/css", "body { color: black; }"],
    ];
    await Promise.all(Array.from({ length: 32 }, async (_, index) => {
      const [path, type, body] = resources[index % resources.length];
      const response = await fetch(new URL(path, fixture.url), { signal: AbortSignal.timeout(3000) });
      assert.equal(response.status, 200);
      assert.equal(response.headers.get("content-type"), type);
      assert.equal(response.headers.get("content-encoding"), path.endsWith(".js") ? "gzip" : null);
      assert.equal(await response.text(), body);
    }));
    for (const encoding of ["identity", "gzip;q=0"]) {
      const response = await fetch(new URL("_site/auto.js", fixture.url), { headers: { "accept-encoding": encoding } });
      assert.equal(response.headers.get("content-encoding"), null);
      assert.equal(await response.text(), "export const ready = true;");
    }
    const redirect = await fetch(new URL("components?mode=test", fixture.url), { redirect: "manual" });
    assert.equal(redirect.status, 301);
    assert.equal(redirect.headers.get("location"), "/test/components/?mode=test");
    assert.equal((await fetch(new URL("missing/", fixture.url))).status, 404);
  } finally {
    if (server) await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
    await rm(root, { recursive: true, force: true });
  }
});
