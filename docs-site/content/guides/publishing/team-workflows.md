---
order: 2
---
# Team workflows and CI

Keep the source under version control, pin the generator version, and validate
changes before publishing. The same source can be built locally or in CI.

## What belongs in the repository

Commit `site.toml`, `content/`, `public/`, and instructions for contributors.
For npm-based projects, also commit `package.json`, `package-lock.json`, and
a token-free `.npmrc`:

::code-block
```ini
@cmu-sei:registry=https://npm.pkg.github.com
```
::

Ignore `dist/` and `node_modules/`. Keep credentials in each author's user
configuration or the CI secret store, never in the repository or public assets.
Use [the setup commands](../start/getting-started.md#optional-npm-installation) to
install an exact published generator version and define npm scripts.

## Review a content change

1. Make a branch and edit the relevant Markdown or assets.
2. Run `npm run check` (configured to use `--strict`).
3. Use `npm run dev`, then review the changed pages in a browser.
4. Open a pull request describing URL changes, moved downloads, and new images.
5. Publish only after review and a successful build.

Review meaning, not just validation: accurate image descriptions, meaningful
headings, useful examples, and understandable link labels require a person.
For a compatibility import, use strict validation as a release gate while
tracking the remaining editorial repairs separately.

## A minimal npm-based CI check

In a separate documentation repository using the npm scripts above, create
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

This requires a published package, a committed lockfile, and Actions read
access to that package for the documentation repository. It validates and
builds, but does not deploy. Adjust `main` if your publishing branch differs.
GitHub Packages access may be restricted for forked pull requests; arrange a
trusted review/build path rather than exposing a privileged token to untrusted
code. Keep action revisions and the Node version maintained.

Standalone-binary CI can run the equivalent `check . --strict` and `build .`
commands without Node.js. Obtain the exact platform release through your
organization's approved process.

## Drafts, hidden pages, and private documentation

Use `draft: true` for work that must not appear in the next generated site.
Draft pages are omitted, so links from published pages to drafts fail checks.
Build before deploying, and replace the old deployment: omitting a page from
new output does not delete a previously uploaded copy on every host.

Use `navigation: false` only to remove a page from navigation. It remains
accessible and searchable. Unlinked pages, asset URLs, and robots rules do not
provide confidentiality.

For private documentation, enforce authentication at the host for **all**
generated pages, search data, downloads, and assets. The generator does not
implement login, authorization, encryption, or per-page access policies.

## Upgrade deliberately

Install the selected published version with `--save-dev --save-exact`, review
the release notes, and commit both npm manifest and lockfile. Rebuild and check
representative pages, nested navigation, search, and downloads before release.
For standalone use, record the executable version in contributor instructions.
