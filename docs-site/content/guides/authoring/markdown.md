---
order: 2
navigation:
  title: Markdown and metadata
---
# Writing Markdown

Write pages in Markdown, with tables, strikethrough, smart punctuation,
and [component wrappers](components.md).

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

Use one `#` heading, then `##` and `###` without skipping levels. A missing H1
is generated from the title; title metadata does not replace an existing H1.
All headings have anchors; only `##` sections appear in "On this page."

## Front matter

Put optional metadata first: YAML between `---` lines (above), or TOML between
`+++` lines:

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
| `title` | First H1, otherwise the filename without its order prefix, with hyphens/underscores replaced by spaces |
| `description` | First nonempty paragraph, truncated to 160 characters |
| `order` | `0`; signed 32-bit integer used to sort sibling pages |
| `draft` | `false`; `true` excludes the page from output, navigation, sitemap, and search |
| `navigation` | Visible; `false` hides generated page entries, or an object sets the navigation `title` |

Every published page needs a description or introductory paragraph.
`navigation: false` is **not privacy**: the page remains published and searchable.
Explicit [main-menu entries](../start/configuration.md#main-navigation) are configured
separately. Draft source files are unresolved link targets; the home page must be published.

YAML accepts extra metadata such as `tags`; only the fields above affect output.
Unknown top-level TOML fields fail validation. Metadata is limited to 64 KiB.

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

Separate blocks with blank lines. Smart punctuation affects prose, not code.
Escape Markdown with backslashes: `\*literal\*`.

## Tables

::code-block
```markdown
| Setting | Meaning |
| --- | --- |
| `name` | Website title |
| `url` | Public deployment URL |
```
::

Wide tables scroll; JavaScript adds keyboard-accessible sorting unless headers
are interactive or cells are merged. Follow [accessible table guidance](accessibility.md#keep-tables-simple).

## Code

Use inline backticks for short literals and fences for longer examples.
See [Code examples](components.md#code-examples) for wrapper syntax, copy/fullscreen
controls, titles, quality labels, and highlighting languages.

Enclose triple-backtick examples in four-backtick fences. Component syntax
inside a fence stays literal.

## HTML and unsupported extensions

Add semantic HTML such as `<sup>`, `<sub>`, or explicit heading IDs. HTML uses
the same heading, link, image, and search processing as Markdown.

Sanitization removes scripts, forms, iframes, event handlers, and unsafe or
unsupported markup/styles with diagnostics. A limited set of inline layout
and text styles is retained.

Use explicit links and [supported components](components.md).
Footnotes, task-list checkboxes, math rendering, heading attribute shorthand,
automatic bare-URL linking, includes, templates, and arbitrary Vue/MDC widgets
are unsupported.
