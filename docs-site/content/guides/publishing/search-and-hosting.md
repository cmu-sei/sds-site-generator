---
order: 3
---
# Search and hosting

Search is generated from published page content and runs in the reader's
browser. Reliable deployment requires the HTML, scripts, and search data from
the same build.

## Write for discoverability

Use descriptive page titles and section headings: page and heading matches
receive extra ranking weight. Put the reader's vocabulary in the relevant
section text, not just an image or linked PDF.

The generator indexes page text by section. Public attachments are not
separate searchable pages; add a Markdown summary and a download link if a
document needs to be discoverable. Custom YAML metadata such as `tags` does
not create search filters or a tagging system.

Drafts are excluded. Pages hidden with `navigation: false` remain searchable.
There is no supported per-page search-exclusion setting or configurable
ranking/keyword field.

## Query behavior

| Query | Meaning |
| --- | --- |
| `site configuration` | Sections matching both words |
| `"site configuration"` | The words together in that order |
| `config` | Prefix completion on the final unquoted term |

Results point to sections, not only page tops. Up to 30 results are displayed;
add words to narrow a broad query. The query input is limited to 200 characters.
Do not assume wildcard, Boolean-operator, fuzzy, or language-stemming syntax.

The standalone search page supports shareable query URLs:

::code-block
```text
https://example.org/docs/search/?q=site%20configuration
```
::

Use your deployment URL. Search computations do not go to a remote search
service, but URLs with queries can appear in hosting logs or browser history.

## Deploy a complete build

- Publish HTML, `_site/` assets, search data, and public files together.
- Use a host's atomic deployment feature when available.
- Do not overlay only changed HTML onto old search data.
- Ensure files removed from the new output are removed from the deployment.
- Keep the previous complete deployment if your host supports rollback.

The generator replaces local `dist/` safely, but it does not synchronize a
remote host or control its cache. During troubleshooting, invalidate stale
HTML/search assets or use a clean browser session. Avoid long-lived immutable
cache rules for files whose URLs remain the same across builds.

## Diagnose search failures

1. Open the site over HTTP(S), not `file://`.
2. Confirm JavaScript is enabled and the browser meets the supported modern
   browser requirements.
3. Check that the configured URL prefix matches the deployed location.
4. In browser developer tools, look for failed script, worker, or JSON requests.
5. Confirm the host serves JavaScript and JSON with appropriate content types,
   not an HTML error page with a successful status.
6. If the host imposes a Content Security Policy, ensure it permits the site's
   local scripts, module worker, and same-origin search-data requests.

There is no single CSP snippet guaranteed for every hosting environment;
check the host's policy and browser-reported failures. Search errors appear
in the interface rather than silently returning no results.

## Offline and restricted networks

Generation does not fetch website assets or call an external search service.
After the executable is available, a site can be built on a disconnected
machine and served on an internal HTTP(S) host. External links still require
access to their destinations.

This is not an installable offline app: there is no service worker or
guaranteed offline cache. Downloading HTML alone does not make browser search
work. For an air-gapped network, transfer the complete output and serve it
through an approved local web server.
