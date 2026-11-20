---
order: 2
---
# Team workflows and CI

Version your source, pin the generator, and validate changes in CI.

## What belongs in the repository

Commit `site.toml`, `content/`, `public/`, and instructions for contributors.
For npm-based projects, also commit `package.json`, `package-lock.json`, and
a token-free `.npmrc`:

::code-block
```ini
@cmu-sei:registry=https://npm.pkg.github.com
```
::

Ignore `dist/` and `node_modules/`. Keep credentials in user configuration or CI secrets.
[Set up npm](../start/installation.md#optional-npm-installation) with an exact
generator version and the scripts used below.

## Review a content change

1. Make a branch and edit the relevant Markdown or assets.
2. Run `npm run check` (configured to use `--strict`).
3. Use `npm run dev`, then review the changed pages in a browser.
4. Open a pull request describing URL changes, moved downloads, and new images.
5. Publish only after review and a successful build.

Review content using [Accessible authoring](../authoring/accessibility.md).

## A minimal npm-based CI check

In your documentation repository, create
`.github/workflows/check.yml`:

::code-block
```yaml
name: Check documentation
on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read
  packages: read

jobs:
  check:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@08c6903cd8c0fde910a37f88322edcfb5dd907a8
        with:
          persist-credentials: false
      - uses: actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020
        with:
          node-version: "24"
          registry-url: https://npm.pkg.github.com
          scope: "@cmu-sei"
          cache: npm
      - run: npm ci --ignore-scripts
        env:
          NODE_AUTH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      - run: npm run check
      - run: npm run build
```
::

Grant the repository Actions read access to the package. Adjust `main` for your
branch and keep action revisions and Node current.
Add a [deployment workflow](deployment.md#github-pages-your-own-site) to publish.

Forked pull requests may lack package access. Use a trusted review/build path;
never expose privileged tokens to untrusted code.

Standalone CI needs no Node.js: obtain a pinned platform executable through
your approved process, then run `check . --strict` and `build .`.

## Drafts, hidden pages, and private documentation

Use [front matter](../authoring/markdown.md#front-matter) for drafts and hidden
pages, and [deploy complete builds](deployment.md#what-to-upload) to remove old copies.
Hidden pages, unlinked assets, and robots rules are **not access control**.

For private sites, enforce authentication and authorization at the host for
**all** pages, search data, downloads, and assets, including any per-page policies.

## Upgrade deliberately

Review release notes, install with `--save-dev --save-exact`, and commit the
manifest and lockfile. For standalone use, record the version in contributor
instructions. Rebuild and test pages, navigation, search, and downloads.
