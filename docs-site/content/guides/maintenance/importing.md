---
order: 1
---
# Importing and troubleshooting

Use normal validation for new content. Compatibility mode lets you preview
imported Markdown while identifying migration work.

## Import an existing site

1. Run `sds-site-generator new my-docs`, then `cd my-docs`.
2. Replace its starter `content/` with the imported Markdown.
3. Copy images and downloads into `public/`.
4. Set the real name and publishing URL in `site.toml`.
5. If necessary, set `compatibility = true` while repairing the import.
6. Build and review the diagnostics; then run `check . --strict`.

Compatibility mode preserves route underscores and warns about missing links,
missing image descriptions, heading repairs, unsupported markup, and
sanitization changes. It supports YAML metadata, semantic HTML, anchors,
attachments, and documented wrappers, not arbitrary Nuxt/Vue widgets.
Use the [current callout syntax](../authoring/components.md#callouts);
legacy callouts require compatibility mode and produce migration warnings.

Invalid metadata, configuration, or menu targets; duplicate URLs; unsafe
filenames; and output collisions still fail validation.

The terminal shows up to 20 warnings; builds write every file, line, and message
to `dist/_site/diagnostics.json`. `check . --strict` fails if warnings remain.

Before disabling compatibility mode on a published site, plan the
[URL migration](url-migrations.md): underscores become hyphens.

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

Update every legacy callout before `check --strict` can pass. Never mix `tone`
with legacy properties. A bare `::callout` uses the neutral default in either mode.

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

If `url` contains `/docs/`, assets load from `/docs/`, not `/`. Use built-in
preview or configure your static server to match that prefix.

## Getting help

Report generator bugs in the
[issue tracker](https://github.com/cmu-sei/sds-site-generator/issues).
Contact your host or internal support for account, DNS, authentication, or
policy problems. Include:

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

For preview or hosting issues, add the deployment prefix, affected URL, and
browser network/console errors. Include relevant compatibility diagnostics.

Use a small synthetic example, not private documentation. Remove tokens,
passwords, private URLs, personal data, and confidential content from logs.
Report sensitive security issues through your approved private channel;
never post credentials or sensitive exploit details publicly.

## Review beyond validation

Validation checks local links, structure, and selected content constraints.
Manually test external links and review image descriptions and source
attachments. Follow [accessible authoring](../authoring/accessibility.md)
to test the deployed site. Pin the generator version and review upgrade warnings.
For structure and configuration, see the [user guide](../index.md).
