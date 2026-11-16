import { build } from "esbuild";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const result = await build({
  absWorkingDir: root,
  entryPoints: ["highlight.mjs"],
  bundle: true,
  format: "esm",
  minify: true,
  legalComments: "inline",
  metafile: true,
  outfile: "../assets/highlight.js",
});
const packages = new Set();
for (const input of Object.keys(result.metafile.inputs)) {
  const marker = "node_modules/";
  const start = input.lastIndexOf(marker);
  if (start < 0) continue;
  const prefix = start + marker.length;
  const parts = input.slice(prefix).split("/");
  packages.add(input.slice(0, prefix) + parts.slice(0, parts[0].startsWith("@") ? 2 : 1).join("/"));
}
const notices = [];
for (const directory of [...packages].sort()) {
  const location = resolve(root, directory);
  const metadata = JSON.parse(await readFile(resolve(location, "package.json"), "utf8"));
  for (const filename of (await readdir(location)).filter((name) => /^licen[sc]e(?:\.|$)/i.test(name))) {
    notices.push(`${metadata.name} ${metadata.version}\n\n${await readFile(resolve(location, filename), "utf8")}`);
  }
}
await writeFile(resolve(root, "../assets/highlight-LICENSE.txt"), notices.join("\n\n---\n\n"));
console.log(`Bundled local syntax highlighting with notices for ${packages.size} dependencies.`);