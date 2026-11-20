---
order: 3
---
# Search and hosting

Search indexes published page sections and runs locally in the reader's browser
without telemetry. It requires JavaScript and HTTP(S).

## Write for discoverability

Use descriptive titles and headings, which rank higher, and include the
reader's vocabulary in the text.

Attachments are not indexed; add a Markdown summary and download link.
Custom metadata such as `tags` does not add search filters.

Search follows the [page publication settings](../authoring/markdown.md#front-matter).
Per-page exclusion and custom ranking/keyword fields are unsupported.

## Query behavior

Press Cmd+K (macOS) or Ctrl+K to open search and focus its input, including
while typing. Pressing the shortcut again refocuses the input without clearing
the query. Arrow keys select results, Enter opens one, and Escape closes the dialog.

| Query | Meaning |
| --- | --- |
| `site configuration` | Sections matching both words |
| `"site configuration"` | The words together in that order |
| `config` | Prefix completion on the final unquoted term |

Queries allow 200 characters and show up to 30 section results; add words to
narrow them. Wildcards, Boolean operators, fuzzy matching, and stemming
syntax are unsupported.

The standalone search page supports shareable query URLs:

::code-block
```text
https://example.org/docs/search/?q=site%20configuration
```
::

Use your deployment URL. Shared query URLs can appear in hosting logs and
browser history.

## Deploy a complete build

Follow the [upload instructions](deployment.md#what-to-upload) to keep pages and
search data consistent. Avoid immutable caching for URLs reused across builds.
Clear stale HTML/search assets or try a clean browser session when troubleshooting.

## Diagnose search failures

1. Open the site over HTTP(S), not `file://`.
2. Enable JavaScript and use a [supported browser](deployment.md#before-sharing-the-url).
3. Check that the configured URL prefix matches the deployed location.
4. In browser developer tools, look for failed script, worker, or JSON requests.
5. Check JavaScript/JSON responses have the right content types, not HTML error pages.
6. Ensure any Content Security Policy permits local scripts, the module worker,
   and same-origin search-data requests.

Search errors are displayed in the interface.

## Offline and restricted networks

The standalone executable builds offline. Transfer the complete output to an
internal HTTP(S) server for disconnected networks; external links still need
network access. There is no service worker.
