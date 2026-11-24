# Bundled SDS Lite

- Upstream: https://github.com/cmu-sei/sds-lite
- Release: [v0.9.0](https://github.com/cmu-sei/sds-lite/releases/tag/v0.9.0),
  published 2026-11-24.
- Source commit: `ab7f0cecd096708acd52bcb6da789a3ac67b915e`.
- Verified against the latest stable release: 2026-11-24.
- Source archive:
  https://codeload.github.com/cmu-sei/sds-lite/tar.gz/ab7f0cecd096708acd52bcb6da789a3ac67b915e
- Source archive SHA-256:
  `85b745ff8eca42fb47d6e17ed3f209e4a928d61fb57f37f787ee8707af6201e3`.
- License: see [LICENSE](LICENSE). Generated sites continue to bundle the
  required design-system license files.

All eight bundled files are byte-for-byte identical to this release's source
archive. The previous local testing snapshot already matched v0.9.0; replacing
it with the verified release files changed provenance without changing asset
bytes or behavior. No local patches are applied.

## Source files

| Bundled file | Path in the release source archive |
| --- | --- |
| `sds.css` | `dist/sds.css` |
| `brand.css` | `dist/brand.css` |
| `auto.js` | `dist/auto.js` |
| `documentation.css` | `dist/documentation.css` |
| `documentation.js` | `dist/documentation.js` |
| `documentation-icons.json` | `dist/documentation-icons.json` |
| `package/assets/sei-wordmark.svg` | `dist/package/assets/sei-wordmark.svg` |
| `LICENSE` | `LICENSE` |

These are the standalone browser bundles distributed in the release source,
not the different module-tree entry points in the NPM package. Rust embeds
them in the executable, so building and using the generator does not require
installing SDS Lite through NPM.

`brand.css` references the wordmark by its original relative path; that directory
layout is preserved in generated websites. The documentation icon bundle includes
artwork attribution, which is also written to `_site/code-controls-LICENSE.txt`.
Shell markup and decorative icons in `src/render.rs` are adapted from the
upstream Documentation Site starter and its optional documentation recipes.

## Verification and updates

Exact SHA-256 checksums for every bundled asset, including the wordmark and
license, are recorded in [SHA256SUMS](SHA256SUMS). Verify them from this directory:

```sh
shasum -a 256 -c SHA256SUMS
```

For an update, resolve the latest stable upstream release to its exact commit,
download that commit's source archive, and copy only the files listed above.
Do not mix assets from different releases or substitute NPM module entry points
for the standalone bundles. Record the release, commit, archive checksum, and
verification date here, then regenerate `SHA256SUMS` for the same eight files.
Run the generator tests and browser checks documented in the root README before
accepting changed assets. Archive and asset checksums detect changes; upstream
release and commit verification establishes which source is being bundled.
