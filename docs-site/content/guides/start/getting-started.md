---
order: 2
navigation:
  title: Create your first site
---
# Getting started

Create a site in a new directory, edit Markdown, and preview locally before
publishing. First, follow [Installation](installation.md) to get the generator
for macOS, Linux, or Windows.

## Get the generator

The [installation guide](installation.md) covers downloads, platform selection,
PATH setup, and verification. The commands below assume the executable is on
PATH; otherwise use its full path.

::code-block
```sh
sds-site-generator --version
sds-site-generator --help
```
::

## Platform setup

### macOS and Linux

Follow [macOS and Linux installation](installation.md#macos-and-linux)
to install the executable and configure your shell's PATH.

### Windows PowerShell

Follow [Windows PowerShell installation](installation.md#windows-powershell)
to extract the executable and configure your user Path.

The multiline `sh` examples in this guide use POSIX backslash continuations,
not PowerShell syntax. In PowerShell, put each command on one line as above.

### Installation problems

See [Installation problems](installation.md#installation-problems) for
download, permissions, PATH, and package authentication checks.

## Create your site

::code-block
```sh
sds-site-generator new my-docs \
  --name "My documentation" \
  --url https://example.org/docs/
cd my-docs
sds-site-generator preview .
```
::

Choose a directory that does not exist: `new` never overwrites an existing
directory. It creates `site.toml`, starter pages in `content/`, an empty
`public/`, a README, and ignore rules for generated output.

Open the address printed in the terminal. Preview defaults to
`127.0.0.1:4173` and includes the publishing URL's subdirectory, such as
`http://127.0.0.1:4173/docs/`. To use another port:

::code-block
```sh
sds-site-generator preview . --port 8080
```
::

Edit `content/index.md`, save, wait for a successful rebuild, and refresh the
browser. There is no automatic browser reload. Preview watches `content/`,
`public/`, and `site.toml`. Stop it with Ctrl+C.

## Daily commands

| Command | Result |
| --- | --- |
| `sds-site-generator new DIRECTORY --name NAME --url URL` | Create a site in a new directory |
| `sds-site-generator preview [DIRECTORY] --port 4173` | Build, serve locally, and watch for edits |
| `sds-site-generator check [DIRECTORY]` | Validate without writing output |
| `sds-site-generator check [DIRECTORY] --strict` | Also fail on compatibility warnings |
| `sds-site-generator build [DIRECTORY]` | Generate `dist/` inside the site directory |

Square brackets above mean optional arguments; do not type the brackets.
Omitted directories default to the current directory except for `new`.
Quote paths and names containing spaces. Errors produce a nonzero exit status.

## Optional npm installation

See [Optional npm installation](installation.md#optional-npm-installation)
for GitHub Packages authentication, global installation, and shared-project
setup. Node.js and npm are not required for the standalone executable.
