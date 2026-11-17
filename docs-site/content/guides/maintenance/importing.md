---
order: 1
---
# Importing and troubleshooting

Use normal validation for new content. Compatibility mode helps review an
existing Markdown corpus without pretending its content problems are fixed.

## Import an existing site

1. Create a fresh site with `sds-site-generator new`.
2. Replace its starter `content/` with the imported Markdown.
3. Copy images and downloads into `public/`.
4. Set the real name and publishing URL in `site.toml`.
5. If necessary, set `compatibility = true` while repairing the import.
6. Build and review the diagnostics; then run `check . --strict`.

Compatibility mode preserves underscores in routes and turns selected content
problems into warnings: missing links, missing image descriptions, heading
repairs, unsupported markup, and sanitization changes. Existing YAML metadata,
semantic HTML, anchors, attachments, callouts, and code-block wrappers are
supported; arbitrary Nuxt/Vue widgets are not executed.
Callouts should use the current [semantic API](../authoring/components.md#callouts).
Compatibility mode also accepts the old callout format with migration warnings;
normal mode rejects it.

Invalid metadata/configuration, duplicate URLs, unsafe filenames, and output
collisions remain fatal. Invalid main-menu targets are fatal too.

The terminal shows up to 20 warnings. Build to write the full report to
`dist/_site/diagnostics.json`, which records each file, line, and message.
`check . --strict` returns failure if any warnings remain.

Turning compatibility mode off changes underscore routes, so plan URL
migration before switching an existing published import to normal mode.
Compatibility is not an accessibility or security certification.

## Legacy callouts

With `compatibility = true`, existing callouts such as this still render:

::code-block
```markdown
::callout{variant="orange" type="outline" size="sm"}
**Warning:** Check local links.
::
```
::

For normal mode, convert that example to:

::code-block
```markdown
::callout{tone="warning" variant="outline" size="sm"}
**Warning:** Check local links.
::
```
::

Map old color variants to tones: `blue`/`indigo` to `info`, `green`/`teal` to
`success`, `orange` to `warning`, `red` to `danger`, `gray` to `neutral`, and
`purple` to `accent`. Rename `type` to `variant`, omitting it when the old
type was `subtle`. Keep `size` unchanged.

Each legacy callout produces a migration warning, so `check --strict` fails
until those callouts are updated. Do not mix a semantic `tone` with legacy
properties. A bare `::callout` uses the new neutral default in either mode.

## Common problems

| Message or symptom | What to do |
| --- | --- |
| Missing `site.toml` | Run from the site root, or pass its directory explicitly |
| Duplicate URL | Check numeric prefixes, normalized names, and `page.md` versus `page/index.md`; rename deliberately |
| Missing home page | Add a published `content/index.md` (a numeric prefix is also allowed) |
| Missing description | Add an introductory paragraph or nonempty metadata `description` |
| Heading skips a level | Follow H1 with H2, H2 with H3, and so on |
| Missing page or fragment | Check source-relative `.md` paths, numeric prefixes, drafts, and generated heading IDs |
| Image validation fails | Check the local path, alt text, dimensions, format, and 1 MiB limit |
| Unsupported component/property | Use only the documented callout and code-block syntax |
| Refusing to replace `dist/` | Move hand-authored output elsewhere; do not fabricate an ownership marker |
| Preview shows old content | Read the terminal for a failed rebuild; refresh after a successful one |
| Port already in use | Choose a different `--port`; do not stop unrelated processes |
| Pages load without styles | Check `url`, the deployment prefix, and which directory was uploaded |
| Search fails from a local file | Use preview or an HTTP(S) host instead of `file://` |

If a generic static server serves `dist/` at `/` but the configured publishing
URL contains `/docs/`, assets will be requested under `/docs/`. Use the built-in
preview, which mounts the correct prefix, or configure your server to match.

## Getting help

For generator bugs, use the source repository's
[issue tracker](https://github.com/cmu-sei/sds-site-generator/issues) when it is
available. For deployment-account, DNS, authentication, or organizational
policy problems, contact your hosting provider or internal support team.

Include enough information to reproduce the problem:

::code-block
```text
Generator version: (output of sds-site-generator --version)
Operating system and architecture:
Installation method: standalone archive / npm / source build
Exact command and working directory:
Expected behavior:
Actual behavior and full terminal error:
Smallest site.toml and Markdown example that reproduces it:
```
::

For preview or hosting issues, include the configured deployment prefix,
the affected page URL, and relevant browser network/console errors.
If compatibility mode is enabled, include the relevant diagnostic entries.

Remove tokens, passwords, private URLs, personal data, and confidential
content before sharing logs or examples. Do not post an entire private
documentation corpus; reproduce the problem with a small synthetic site.
Report suspected sensitive security issues through your organization's
approved private channel, not a public issue containing exploit details or
credentials.

## Scope and responsibility

The generator checks local links, structure, and selected content constraints.
It does not check external link availability, judge the accuracy of image
descriptions, scan source attachments for correctness, or guarantee WCAG
conformance or a Lighthouse score.

Keep the executable version pinned for repeatable builds, review warnings
after upgrades, and test the actual deployed website. For new site structure
or configuration questions, return to the [user guide](../index.md).
