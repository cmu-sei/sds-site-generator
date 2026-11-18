---
order: 2
---
# Team workflows and CI

Version your source, pin the generator, and validate changes locally and in CI.

## What belongs in the repository

Commit `site.toml`, `content/`, `public/`, and instructions for contributors.
For npm-based projects, also commit `package.json`, `package-lock.json`, and
a token-free `.npmrc`:

::code-block
```ini
@cmu-sei:registry=https://npm.pkg.github.com
```
::

Ignore `dist/` and `node_modules/`. Store credentials in user configuration or
CI secrets, never in the repository or public assets.
[Set up npm](../start/installation.md#optional-npm-installation) with an exact
generator version and the scripts used below.

## Review a content change

1. Make a branch and edit the relevant Markdown or assets.
2. Run `npm run check` (configured to use `--strict`).
3. Use `npm run dev`, then review the changed pages in a browser.
4. Open a pull request describing URL changes, moved downloads, and new images.
5. Publish only after review and a successful build.

Review image descriptions, headings, examples, and link labels manually.
For compatibility imports, require strict validation before release and track
editorial repairs separately.

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

Commit the lockfile and grant the repository Actions read access to the package.
Adjust `main` for your branch; keep action revisions and Node maintained.
Add a [deployment workflow](deployment.md#github-pages-your-own-site) to publish.

Forked pull requests may lack package access. Use a trusted review/build path;
never expose privileged tokens to untrusted code.

Standalone CI needs no Node.js: obtain a pinned platform executable through
your approved process, then run `check . --strict` and `build .`.

## Drafts, hidden pages, and private documentation

`draft: true` excludes a page from the next build; links to it fail validation.
Replace the old deployment completely so previously published copies are removed.

`navigation: false` hides only the menu entry; the page stays accessible and
searchable. Unlinked pages, asset URLs, and robots rules are not access control.

For private documentation, enforce authentication at the host for **all**
pages, search data, downloads, and assets. The host must provide login,
authorization, encryption, and any per-page access policies.

## Upgrade deliberately

Install the selected release version with `--save-dev --save-exact`, review
the release notes, and commit both npm manifest and lockfile. Rebuild and check
representative pages, nested navigation, search, and downloads before release.
For standalone use, record the executable version in contributor instructions.
