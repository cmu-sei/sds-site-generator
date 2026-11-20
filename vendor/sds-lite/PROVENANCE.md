# Bundled SDS Lite

- Upstream: https://github.com/cmu-sei/sds-lite
- Release: [v0.8.0](https://github.com/cmu-sei/sds-lite/releases/tag/v0.8.0),
  published 2026-11-12.
- Source commit: `981b058e6a7d3c412bfe49b2225b6d73af09a950`.
- Verified against the latest stable release: 2026-11-20.
- Source archive:
  https://codeload.github.com/cmu-sei/sds-lite/tar.gz/981b058e6a7d3c412bfe49b2225b6d73af09a950
- Source archive SHA-256:
  `fca80cf4349014a723b468e611464aaebb0b67337116af0b39bbf2b880d1dc86`.
- License: see [LICENSE](LICENSE). Generated sites continue to bundle the
  required design-system license files.

All eight bundled files are byte-for-byte identical to the files in this
release's source archive. The existing development snapshot already matched
v0.8.0; verifying the released files replaced its local-build provenance
without changing asset bytes or behavior. There are no local patches.

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
