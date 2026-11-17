---
order: 1
---
# Build and deploy

Validate the source, build the site, and upload the contents of `dist/` to a
static host. The preview server is for local development, not production.

## Validate and build

::code-block
```sh
sds-site-generator check . --strict
sds-site-generator build .
```
::

`check` validates without changing output. A build stages a complete site
before replacing `dist/`; validation failures preserve the last good build.
An existing `dist/` without the generator's ownership marker is not replaced.
Keep hand-authored files elsewhere, and do not run simultaneous builds or a
separate build alongside preview for the same site.

Preview also keeps serving the last good build after an error. Read terminal
messages so an old page is not mistaken for a successful edit.

## What to upload

Upload everything **inside** `dist/`, including dotfiles. Do not upload
`content/`, `site.toml`, the executable, or the `dist` directory as an extra
URL segment.

Generated files include:

- Page HTML, navigation, descriptions, canonical and social metadata.
- A searchable `/search/` page and local search data.
- `sitemap.xml`, `robots.txt`, `404.html`, and `.nojekyll`.
- Bundled styles, scripts, licenses, and the content-warning report at
  `_site/diagnostics.json`.
- `.gz` variants of text files for hosts that support precompressed responses.

Configure directory indexes and a real HTTP 404 response using `404.html`.
Enable HTTPS. Let the host compress responses, or configure `.gz` responses
with `Content-Encoding: gzip` and `Vary: Accept-Encoding`.

For a subdirectory deployment, the origin's `/robots.txt` must allow crawling
and point to the sitemap; a project-level robots file cannot govern the origin.

## GitHub Pages: this repository

This repository's Pages workflow publishes the `docs-site/` guide you are
reading, not the imported `example/` or `cert-example/` sites.

1. Push the repository to GitHub.
2. In **Settings > Pages**, choose **GitHub Actions** as the source.
3. Configure the `github-pages` environment and any required approvals or
   custom domain.
4. Push to `main`, or manually run the Pages workflow on the default branch.
5. Use the deployment URL shown by the workflow.

The workflow builds the executable, tests it, and generates the guide using
Pages' actual base URL. It changes the URL in a temporary copy, leaving
`docs-site/site.toml` untouched. It uploads `pages-dist/`, then deploys in a
separate job.

Contributors can reproduce that output from the generator repository:

::code-block
```sh
npm ci --ignore-scripts
npm run build
node scripts/package.mjs pages https://your-org.github.io/your-repository/
```
::

This is a repository-development helper, not a command installed in an
end user's site.

## GitHub Pages: your own site

Set `url` to `https://OWNER.github.io/REPOSITORY/`, or to the root URL for an
account site or custom domain. The generator does not scaffold a workflow.

In your own repository's Actions workflow:

1. Check out the site and obtain a pinned generator executable.
2. Run `sds-site-generator check . --strict`.
3. Run `sds-site-generator build .`.
4. Upload `dist/` with GitHub's Pages artifact action.
5. Deploy with GitHub's Pages deployment action in a job with `pages: write`
   and `id-token: write`, using the `github-pages` environment.

Select **GitHub Actions** in Pages settings. If using the npm package, CI also
needs Node.js 22.19+, registry authentication, and `packages: read` access.
Grant the site repository access to the generator package, then use
`npm ci --ignore-scripts` and the pinned local CLI. Do not commit credentials.

For a custom domain, follow GitHub's DNS and Pages domain instructions and
make `url` match the final HTTPS domain. Rebuild whenever the public base URL
changes.

For a complete validation workflow, see [team workflows and CI](team-workflows.md).
For deployment caching, search diagnostics, and restricted networks, see
[search and hosting](search-and-hosting.md).

## Before sharing the URL

- Test a nested page directly, not just the home page.
- Check images, downloads, heading links, and a nonexistent URL.
- Test keyboard navigation, small screens, and light/dark appearance.
- Confirm descriptions, image alt text, and table headers are meaningful.
- Test search over HTTP(S), not by opening files with `file://`.

Search runs locally in the browser: multiple words use AND matching, quoted
phrases match consecutive tokens, and the final unquoted term supports prefix
completion. Arrow keys select results, Enter follows one, and Escape closes
the search dialog. No remote search service or telemetry is used. Standalone
search URLs contain the query, which may appear in normal hosting logs.

The shell supports modern browsers following SDS Lite's requirements,
including Custom Elements, Popover, and CSS `@scope`. No legacy-browser
polyfills or automatic accessibility certification are provided.
