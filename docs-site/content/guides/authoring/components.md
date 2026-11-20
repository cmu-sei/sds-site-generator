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

A bare `::callout` uses the defaults above. Callouts support nested Markdown,
lists, and fenced code.

For imported callouts, see
[the legacy callout migration guide](../start/configuration.md#legacy-callouts).

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

Choose quality labels yourself; use `average` for ordinary examples.

JavaScript adds copy and fullscreen controls. Escape exits fullscreen;
code remains readable without JavaScript. Plain fences scroll and are
keyboard-focusable but have no copy or fullscreen controls.

### Syntax highlighting

Put a lowercase language label after the fence's opening backticks, as in the
`c` example above, not on `::code-block`. Highlighting works with wrapped or
plain fences and needs JavaScript, but no internet access.

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
| CSS | `css` |
| GLSL | `glsl` |
| SQL | `sql` |
| Regular expressions | `regexp`, `regex` |
| C++ macros | `cpp-macro` |

Use `text` or omit the label for plain text. Unknown labels stay unhighlighted.

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
