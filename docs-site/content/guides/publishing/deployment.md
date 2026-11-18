---
order: 1
---
# Build and deploy

Check your content, build, and upload `dist/` contents to a static host.
Use preview only for local development.

## Validate and build

::code-block
```sh
sds-site-generator check . --strict
sds-site-generator build .
```
::

`check` leaves output unchanged. A build stages the complete site, then replaces
`dist/`; validation failures preserve the last good build. An existing `dist/`
without the generator's ownership marker is not replaced.

Keep hand-authored files outside `dist/`. Stop preview before building, and
never run simultaneous builds of one site. After a preview error, read the
terminal: the browser still shows the last good build.

## What to upload

Upload everything **inside `dist/`**, including dotfiles, not the source,
executable, or enclosing `dist` directory.

Generated files include:

- Page HTML, navigation, descriptions, canonical and social metadata.
- A searchable `/search/` page and local search data.
- `sitemap.xml`, `robots.txt`, `404.html`, and `.nojekyll`.
- Bundled styles, scripts, licenses, and the content-warning report at
  `_site/diagnostics.json`.
- `.gz` variants of text files for hosts that support precompressed responses.

Enable HTTPS, directory indexes, and HTTP 404 responses using `404.html`.
Use host compression, or serve `.gz` files with `Content-Encoding: gzip` and
`Vary: Accept-Encoding`.

For a subdirectory deployment, the origin's `/robots.txt` must allow crawling
and point to the sitemap; a project-level robots file cannot govern the origin.

## GitHub Pages: this repository

This repository's Pages workflow publishes `docs-site/`, not `example/` or
`cert-example/`.

1. Push the repository to GitHub.
2. In **Settings > Pages**, choose **GitHub Actions** as the source.
3. Configure the `github-pages` environment and any required approvals or
   custom domain.
4. Push to `main`, or manually run the Pages workflow on the default branch.
5. Use the deployment URL shown by the workflow.

The workflow builds and tests the executable, generates the guide with Pages'
URL in a temporary copy, then deploys `pages-dist/` in a separate job.
The checked-in `docs-site/site.toml` stays unchanged.

Contributors can reproduce that output from the generator repository:

::code-block
```sh
npm ci --ignore-scripts
npm run build
node scripts/package.mjs pages https://your-org.github.io/your-repository/
```
::

## GitHub Pages: your own site

Set `url` to `https://OWNER.github.io/REPOSITORY/`, or to the root URL for an
account site or custom domain. In your site's Actions workflow:

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

See [team workflows and CI](team-workflows.md) for a validation workflow, or
[search and hosting](search-and-hosting.md) for caching and search diagnostics.

## Before sharing the URL

- Test a nested page directly, not just the home page.
- Check images, downloads, heading links, and a nonexistent URL.
- Test keyboard navigation, small screens, and light/dark appearance.
- Confirm descriptions, image alt text, and table headers are meaningful.
- Test search over HTTP(S), not by opening files with `file://`.

Search runs locally without telemetry. Arrow keys select results, Enter opens
one, and Escape closes the dialog. See [query behavior](search-and-hosting.md#query-behavior)
for matching rules and shareable search URLs.

Use a modern browser supporting Custom Elements, Popover, and CSS `@scope`;
legacy polyfills are not bundled. Follow the
[accessible authoring guide](../authoring/accessibility.md) for content review.
