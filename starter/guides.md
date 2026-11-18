# Writing guide

Each Markdown file in `content/` becomes a page; `index.md` is its directory's
landing page.

## Headings

Use one `#` heading, then `##` sections and `###` subsections. Do not skip levels.

## Links

Link to other Markdown files using their relative filenames:

::code-block
```markdown
[Publishing](publishing.md)
[Home](../index.md)
[This section](#links)
```
::

The generator checks page and heading links before publishing.

## Images

Put images in `public/` and describe their purpose:

::code-block
```markdown
![A diagram showing the review process](/images/review.png)
```
::

Use local PNG, JPEG, GIF, WebP, AVIF, or sanitized SVG; dimensions are added
automatically. Resize and compress each image to at most **1 MiB (1,048,576 bytes)**.
For decorative images, use an empty description and the title `"decorative"`.

## Optional metadata

Optional metadata goes first: YAML between `---` lines or TOML between `+++`:

::code-block
```yaml
---
title: Writing guide
description: A helpful summary for search engines.
navigation:
  title: Writing
tags: [documentation]
---
```
::

YAML accepts extra metadata such as tags. `navigation: false` hides only the
menu entry; the page stays published and searchable. For TOML:

::code-block
```toml
+++
title = "A shorter navigation title"
description = "A helpful summary for search engines."
order = 2
draft = false
+++
```
::

Unknown TOML settings fail validation. Fenced YAML in the body is code, not metadata.

## Code examples

Wrap a fenced example with `::code-block` and a closing `::` line, as above.
The default is neutral; `quality="good"` and `quality="bad"` label compliant
and noncompliant code. JavaScript adds copy/fullscreen controls; code stays
readable without it.

Choose highlighting with a lowercase fence label: `markdown`, `toml`,
`powershell`, or `ini`.
The bundled grammars also cover C, C++, Java, Perl, free-form Fortran,
JavaScript, TypeScript, Python, shell, HTML, XML, JSON, YAML, and diff, with
CSS, GLSL, SQL, regular-expression, and C++ macro dependency grammars.
Use `text` or no label for plain text; unknown labels are not highlighted.
Highlighting needs JavaScript and uses local assets.

## Callouts

Use semantic tones and visual variants matching SDS Lite:

::code-block
```markdown
::callout{tone="warning" variant="outline" size="sm"}
**Warning:** Check local links before publishing.
::
```
::

Tones are `neutral` (default), `accent`, `info`, `success`, `warning`, and
`danger`. Omit `variant` for a subtle surface, or use `outline` or `bold`.
Sizes are `xs`, `sm`, `md` (default), and `lg`.

## Importing existing content

Semantic HTML, tables, anchors, and legacy code-example wrappers are supported;
scripts and unsafe attributes are removed. Set `compatibility = true` in
`site.toml` to turn selected import errors into warnings. Review
`dist/_site/diagnostics.json`; `sds-site-generator check --strict` fails on
warnings. Duplicate URLs always stop the build.

## Next steps

[Publish your website](publishing.md).
