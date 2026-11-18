---
order: 3
---
# Search and hosting

Search indexes published page content and runs in the reader's browser.
Deploy HTML, scripts, and search data from the same build.

## Write for discoverability

Use descriptive titles and headings; matches in them rank higher. Include the
reader's vocabulary in section text, not only images or linked PDFs.

Pages are indexed by section. To make an attachment discoverable, add a Markdown
summary and download link. Attachments are not indexed separately; custom YAML
metadata such as `tags` does not add filters.

Drafts are excluded. Pages hidden with `navigation: false` remain searchable.
There is no supported per-page search-exclusion setting or configurable
ranking/keyword field.

## Query behavior

| Query | Meaning |
| --- | --- |
| `site configuration` | Sections matching both words |
| `"site configuration"` | The words together in that order |
| `config` | Prefix completion on the final unquoted term |

Results link to sections. Queries allow up to 200 characters and show up to
30 results; add words to narrow them. Wildcards, Boolean operators, fuzzy
matching, and language-stemming query syntax are unsupported.

The standalone search page supports shareable query URLs:

::code-block
```text
https://example.org/docs/search/?q=site%20configuration
```
::

Use your deployment URL. Queries stay local to the browser, but shareable URLs
can appear in hosting logs and browser history.

## Deploy a complete build

- Publish HTML, `_site/` assets, search data, and public files together, not just changed HTML.
- Use a host's atomic deployment feature when available.
- Ensure files removed from the new output are removed from the deployment.
- Keep the previous complete deployment if your host supports rollback.

Configure remote synchronization and caching through your host. Avoid immutable
caching for URLs reused across builds. To troubleshoot, invalidate stale
HTML/search assets or use a clean browser session.

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

Search errors are displayed in the interface.

## Offline and restricted networks

Build on a disconnected machine with the standalone executable; generation
needs no external assets or search service. For air-gapped networks, transfer
the complete output to an approved internal HTTP(S) server so search can load
its scripts and data. External links need access to their destinations.
The site uses ordinary browser caching, without a service worker.
