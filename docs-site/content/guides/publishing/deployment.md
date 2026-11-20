---
order: 1
---
# Build and deploy

Validate, build, and upload `dist/` contents to a static host.

## Validate and build

::code-block
```sh
sds-site-generator check . --strict
sds-site-generator build .
```
::

`check` leaves files unchanged. Builds replace generator-owned `dist/` only after
successful validation, preserving the last good build on failure.
Keep hand-authored files elsewhere. Stop preview before building; do not run
simultaneous builds of one site.

## What to upload

Upload everything **inside `dist/`**, including dotfiles.
Replace the previous deployment completely so removed files disappear. Use
atomic deployment when available and retain the previous build for rollback.

Generated files include:

- Page HTML, navigation, and metadata.
- A searchable `/search/` page and local search data.
- `sitemap.xml`, `robots.txt`, `404.html`, and `.nojekyll`.
- Bundled styles, scripts, licenses, and the content-warning report at
  `_site/diagnostics.json`.
- `.gz` variants of text files for hosts that support precompressed responses.

Enable HTTPS, directory indexes, and HTTP 404 responses using `404.html`.
Use host compression, or serve `.gz` files with `Content-Encoding: gzip` and
`Vary: Accept-Encoding`.

For subdirectory deployments, configure crawling and the sitemap in the
origin's `/robots.txt`, not just the project-level file.

## GitHub Pages: this repository

The Pages workflow publishes `docs-site/`. In **Settings > Pages**, select
**GitHub Actions** and configure the `github-pages` environment, approvals,
and optional custom domain. Push to `main` or run the workflow on the default
branch; use the deployment URL it reports.

The workflow builds with Pages' URL in a temporary copy and deploys
`pages-dist/`, leaving source configuration unchanged. See the
[documentation README](https://github.com/cmu-sei/sds-site-generator/tree/main/docs-site#readme)
to reproduce the output locally.

## GitHub Pages: your own site

[Set the publishing URL](../start/configuration.md#choose-the-publishing-url)
to your Pages address, including the repository subdirectory for a project site.
In your site's Actions workflow:

1. Check out the site and obtain a pinned generator executable.
2. Validate and build using the commands above.
3. Upload `dist/` with GitHub's Pages artifact action.
4. Deploy with GitHub's Pages deployment action in a job with `pages: write`
   and `id-token: write`, using the `github-pages` environment.

Select **GitHub Actions** in Pages settings. For npm-based CI, use the
[team workflow](team-workflows.md#a-minimal-npm-based-ci-check) for Node.js,
package authentication, and pinned CLI setup.

For a custom domain, configure GitHub Pages and DNS, set `url` to the final
HTTPS address, and rebuild.

See [Search and hosting](search-and-hosting.md) for caching and search diagnostics.

## Replace a Nuxt deployment

Keep the publishing URL and prefix unchanged. Replace `nuxt generate` with
the generator's check/build commands and `.output/public` with `dist/` in
the upload step. Replace `_nuxt` checks with checks for `_site/` assets,
sitemap, search manifest, `404.html`, and `.nojekyll`. Publish only the new output.

If the import cannot pass strict validation, use a reviewed
[output audit](../maintenance/importing.md#audit-the-cert-import) that records
inherited defects and rejects regressions.

Before production cutover, run the [pre-publication checks](#before-sharing-the-url)
on staging and test old heading fragments.

## Before sharing the URL

- Test a nested page directly, not just the home page.
- Check images, downloads, heading links, and a nonexistent URL.
- Test external links and review descriptions and attachments.
- Follow the [accessible authoring checklist](../authoring/accessibility.md#review-with-a-keyboard-and-assistive-technology).
- Test search over HTTP(S), not by opening files with `file://`.

Use a browser supporting Custom Elements, Popover, and CSS `@scope`.
