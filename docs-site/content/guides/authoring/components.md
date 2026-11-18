---
order: 3
navigation:
  title: Callouts and code blocks
---
# Special syntax

Add notes and code examples with `::callout` and `::code-block`.
Put opening and closing delimiters on separate lines.

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

Properties map to SDS Lite's `data-sds-tone`, `data-sds-variant`, and
`data-sds-size`. A bare `::callout` uses the defaults above.

Use nested Markdown, lists, and fenced code. Include a label such as
**Warning:** so meaning does not depend on color.

Legacy `type` and color variants such as `orange` require
[compatibility mode](../maintenance/importing.md#legacy-callouts) and produce
migration warnings. Never mix them with `tone`; unknown properties and invalid
values fail in either mode.

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

Quality labels are yours, not automated analysis. Use neutral for ordinary
examples; reserve `good` and `bad` for compliance examples.

JavaScript adds copy and fullscreen controls. Escape exits fullscreen and
restores focus; clipboard failures are displayed. Code stays readable without
JavaScript.

### Syntax highlighting

Put a lowercase language label after the fence's opening backticks,
not on `::code-block`:

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

Highlighting uses bundled Shiki grammars and high-contrast light/dark themes.
It needs JavaScript, not internet access, and works with wrapped or plain fences.

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

Use `text` or omit the label for plain text. Unknown labels stay unhighlighted;
languages are not auto-detected or downloaded. Without JavaScript, code stays
readable and unhighlighted. Adding a grammar requires rebuilding the bundled
highlighting assets.

### Short and complete code

Offer short and complete versions with `#default` and `#complete`:

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

A toggle switches versions; Copy uses the selection. Put each marker on its own
line, without a space after `#`.

## Syntax rules

- Single or double quotes and any property order are supported.
- Close each wrapper with `::`, including nested wrappers.
- Only the listed properties and values are allowed.
- Vue bindings, executable attributes, and other MDC widgets are not supported.
- An unclosed wrapper or unsupported widget produces a diagnostic; fix it
  rather than enabling compatibility mode for new content.
