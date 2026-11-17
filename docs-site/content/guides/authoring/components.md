---
order: 3
navigation:
  title: Callouts and code blocks
---
# Special syntax

Two component-style Markdown wrappers are supported: callouts and code blocks.
They generate static content, with optional browser enhancements, not Vue.
Put opening and closing delimiters on their own lines.

## Callouts

::code-block
```markdown
::callout{tone="warning" variant="outline" size="sm"}
**Before publishing:** Check every local link.
::
```
::

::callout{tone="warning" variant="outline" size="sm"}
**Before publishing:** Check every local link.
::

| Property | Allowed values | Default |
| --- | --- | --- |
| `tone` (meaning) | `neutral`, `accent`, `info`, `success`, `warning`, `danger` | `neutral` |
| `variant` (appearance) | `outline`, `bold`; omit for a subtle surface | Subtle surface |
| `size` | `xs`, `sm`, `md`, `lg` | `md` |

These options map directly to SDS Lite's `data-sds-tone`, `data-sds-variant`,
and `data-sds-size`. A bare `::callout` uses the neutral, subtle, medium recipe.
In normal mode, there is no `type` property, and color names such as `orange`
are not tones or variants. Compatibility mode accepts the
[old callout format with migration warnings](../maintenance/importing.md#legacy-callouts).
Do not mix legacy properties with `tone`. Unknown properties and invalid
values still fail in compatibility mode.
Nested Markdown, lists, and fenced code
are supported. Write a useful label such as **Warning:** rather than conveying
meaning by color alone.

## Code examples

::code-block
````markdown
::code-block{title="Return success" quality="good"}
```c
return 0;
```
::
````
::

::code-block{title="Return success" quality="good"}
```c
return 0;
```
::

| Property | Meaning |
| --- | --- |
| `title` | Optional title bar |
| `quality="good"` | Compliant code, blue treatment and badge |
| `quality="bad"` | Noncompliant code, red treatment and badge |
| `quality="average"` | Neutral example; the default |

Quality is an author-supplied label, not a code analyzer's verdict.
Use the neutral default for ordinary commands, configuration, and syntax
examples; reserve `good` and `bad` for actual compliance examples.
Use language-tagged fences inside the wrapper. Copy and fullscreen controls
are enhanced with JavaScript; Escape exits fullscreen and restores focus.
Clipboard failures are shown visibly. Code remains readable without scripts.

### Syntax highlighting

Put a lowercase language label immediately after the opening backticks,
inside the wrapper. The label belongs on the fence, not on `::code-block`:

::code-block
````markdown
::code-block{title="Site settings"}
```toml
name = "My documentation"
url = "https://example.org/docs/"
```
::
````
::

Highlighting uses locally bundled Shiki grammars and high-contrast light/dark
themes. It requires JavaScript, but no CDN, internet connection, or external
highlighting service. Both wrapped examples and plain Markdown fences are
highlighted when their language is supported.

| Language | Accepted fence labels |
| --- | --- |
| Markdown | `markdown`, `md` |
| TOML | `toml` |
| PowerShell | `powershell`, `ps`, `ps1`, `pwsh` |
| INI / properties | `ini`, `properties` |
| C | `c` |
| C++ | `cpp`, `c++` |
| Java | `java` |
| Perl | `perl` |
| Free-form Fortran | `fortran-free-form`, `fortran`, `f`, `f90`, `f95`, `f03`, `f08`, `f18` |
| JavaScript | `javascript`, `js`, `cjs`, `mjs` |
| TypeScript | `typescript`, `ts`, `cts`, `mts` |
| Python | `python`, `py` |
| Shell | `shellscript`, `sh`, `bash`, `shell`, `zsh` |
| HTML | `html` |
| XML | `xml` |
| JSON | `json` |
| YAML | `yaml`, `yml` |
| Diff | `diff` |

Dependency grammars also make CSS (`css`), GLSL (`glsl`), SQL (`sql`), regular
expressions (`regexp`, `regex`), and C++ macros (`cpp-macro`) available.
Fortran support is free-form, not a separate fixed-form grammar.

Use `text` or omit the label for intentional plain text. Unknown labels also
remain plain text; the generator does not auto-detect a language or download
grammars. Without JavaScript, all examples remain readable but unhighlighted.
These are the bundled languages, not every language supported by Shiki.
Adding a grammar requires rebuilding the generator's bundled assets; it is
not a `site.toml` option.

### Short and complete code

Use `#default` and `#complete` on separate lines inside a code block to offer a
short example and its complete version:

::code-block
````markdown
::code-block{title="A small C program" quality="average"}
#default
```c
return 0;
```
#complete
```c
int main(void) {
    return 0;
}
```
::
````
::

::code-block{title="A small C program" quality="average"}
#default
```c
return 0;
```
#complete
```c
int main(void) {
    return 0;
}
```
::

A toggle switches versions, and Copy uses the selected version. These slot
markers are not headings; write them exactly as shown, without a space after
`#`.

## Syntax rules

- Single or double quotes and any property order are supported.
- Close each wrapper with `::`, including nested wrappers.
- Only the listed properties and values are allowed.
- Vue bindings, executable attributes, and other MDC widgets are not supported.
- An unclosed wrapper or unsupported widget produces a diagnostic; fix it
  rather than enabling compatibility mode for new content.
