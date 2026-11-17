---
order: 2
navigation:
  title: Markdown and metadata
---
# Writing Markdown

Write ordinary Markdown with a clear heading structure. Tables, strikethrough,
smart punctuation, and the documented component wrappers are also supported.

## A complete page

::code-block
```markdown
---
title: Install the toolkit
description: Set up the toolkit and verify your first installation.
order: 2
draft: false
navigation:
  title: Installation
---
# Install the toolkit

Set up the toolkit before following the tutorials.

## Requirements

You need an account on your team's documentation host.

### Supported systems

Ask your team which operating systems it supports.
```
::

Start with one `#` main heading, then `##` sections and `###` subsections.
Do not skip levels or add another H1. If there is no H1, one is generated from
the page title. Metadata titles do not replace an explicitly written H1.
The "On this page" list includes only `##` sections, not `###` or deeper
subsections. All headings still receive linkable anchors.

## Front matter

Optional metadata must be the first block in the file. Use YAML between `---`
lines (as above), or TOML between `+++` lines:

::code-block
```toml
+++
title = "Install the toolkit"
description = "Set up the toolkit and verify your first installation."
order = 2
draft = false
navigation = { title = "Installation" }
+++
```
::

| Field | Default and effect |
| --- | --- |
| `title` | First H1, otherwise the filename with its order prefix removed |
| `description` | First nonempty paragraph, truncated to 160 characters |
| `order` | `0`; signed 32-bit integer used to sort sibling pages |
| `draft` | `false`; `true` excludes the page from output, navigation, sitemap, and search |
| `navigation` | Visible; `false` hides the page from navigation, or an object sets `title` |

Every published page needs a nonempty description or an introductory paragraph.
A hidden page is **still published and searchable**; this is not access control.
Links to drafts fail validation, and the home page cannot be a draft.

YAML accepts additional metadata such as `tags` for import compatibility, but
it does not enable tagging or execute anything. Unknown top-level TOML metadata
fields fail validation. Metadata blocks are limited to 64 KiB.

## Text, lists, and quotations

::code-block
```markdown
Use **bold** for emphasis, *italics* for a term, and `code` for literals.
Use ~~strikethrough~~ sparingly.

- One unordered item
- Another item
  - A nested item

1. First step
2. Second step

> A quotation or a short note.
```
::

Separate paragraphs and blocks with blank lines. Smart punctuation converts
straight quotes and punctuation sequences in prose; code remains literal.
Use backslash escapes to show Markdown punctuation, for example `\*literal\*`.

## Tables

::code-block
```markdown
| Setting | Meaning |
| --- | --- |
| `name` | Website title |
| `url` | Public deployment URL |
```
::

Include a header row and an explanatory sentence when needed. Wide tables
scroll horizontally. With JavaScript, simple tables gain keyboard-accessible
header sorting; complex tables with merged cells or interactive headers stay
static. Prefer lists when the content is not genuinely tabular.

## Code

Use inline backticks for short literals. For block examples, put a
language-tagged fence inside the shared code-block wrapper:

::code-block
````markdown
::code-block
```sh
sds-site-generator check . --strict
```
::
````
::

The wrapper supplies copy/fullscreen controls and supports a title, compliance
badge, or short/complete examples. See the
[code-block reference](components.md#code-examples).
This documentation uses the wrapper for all block examples.
Plain Markdown fences are still supported as keyboard-focusable, horizontally
scrollable code, but do not receive those controls.
Syntax highlighting is bundled locally and selected by the fence's language
label, not by the wrapper. See [supported languages and aliases](components.md#syntax-highlighting),
including Markdown, TOML, PowerShell, and INI. Use `text` for plain output;
unknown or missing labels are not highlighted. Code remains readable without
JavaScript.

To show a triple-backtick example without rendering it, enclose it in a
four-backtick fence. Component syntax inside a code fence also stays literal.

## HTML and unsupported extensions

Semantic HTML can supplement Markdown, for example `<sup>`, `<sub>`, or an
explicit heading ID. HTML headings, links, images, and text go through the same
validation and search processing as Markdown.

HTML is sanitized, not executed. Scripts, forms, iframes, event handlers,
unsafe attributes, and unsupported markup/styles are removed and diagnosed.
Only conservative inline layout styles are retained; arbitrary styling is
not a supported theme mechanism.

Do not rely on footnotes, task-list checkboxes, math rendering, heading
attribute shorthand, automatic bare-URL linking, includes, templates, or
arbitrary Vue/MDC widgets. These extensions are not enabled. Use ordinary
links and the [supported special syntax](components.md) instead.
