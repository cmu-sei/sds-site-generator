---
order: 1
navigation:
  title: Create your first site
---
# Getting started

Create a site in a new directory, edit Markdown, and preview locally before
publishing. The standalone executable is the simplest installation.

## Get the generator

When releases are available, download the standalone archive for your operating
system and architecture from the source repository's GitHub Releases. Releases
target macOS, Linux, and Windows on x64 and ARM64. Extract the archive and put
`sds-site-generator` (or `sds-site-generator.exe`) on your PATH.
Follow your organization's rules for unsigned downloaded executables.

::code-block
```sh
sds-site-generator --version
sds-site-generator --help
```
::

If no release has been published yet, a contributor can build from the source
repository using stable Rust:

::code-block
```sh
cargo build --release --locked
./target/release/sds-site-generator --version
```
::

The repository may also have a locally staged executable at
`bin/sds-site-generator`. The commands below assume the executable is on PATH;
otherwise use its full path.

## Platform setup

Choose the archive matching the machine that will run the generator, not the
machine that will read the generated website. On macOS/Linux, `uname -m`
commonly reports `arm64`/`aarch64` for ARM64 or `x86_64` for x64. On Windows,
check **Settings > System > About > System type**.

### macOS and Linux

After extracting a trusted release, run it directly from the extracted
directory:

::code-block
```sh
chmod u+x ./sds-site-generator
./sds-site-generator --version
```
::

For a user-local installation, move that executable into a directory on your
PATH, such as `$HOME/.local/bin`. If necessary, add that directory for the
current terminal session:

::code-block
```sh
export PATH="$HOME/.local/bin:$PATH"
```
::

To keep the setting, add it to the startup file appropriate for your shell
and terminal, such as `.zprofile` for a login zsh or `.bashrc` for an interactive
bash. Open a new terminal and confirm `sds-site-generator --version`.
Follow your organization's macOS download-approval process; do not disable OS
security controls to run an untrusted executable.

### Windows PowerShell

Extract the Windows archive into a user-owned folder, such as
`$HOME\Tools\sds-site-generator`. Run the executable with a relative path:

::code-block
```powershell
cd "$HOME\Tools\sds-site-generator"
.\sds-site-generator.exe --version
.\sds-site-generator.exe new "$HOME\my-docs" --name "My documentation" --url "https://example.org/docs/"
.\sds-site-generator.exe preview "$HOME\my-docs"
```
::

Add the executable's containing folder to your **user Path** through Windows
environment-variable settings, then open a new PowerShell window to use
`sds-site-generator` without a path. For a quoted absolute executable path,
use PowerShell's call operator, for example
`& "$HOME\Tools\sds-site-generator\sds-site-generator.exe" --version`.
Follow your organization's download approval policy if Windows blocks it.

The multiline `sh` examples in this guide use POSIX backslash continuations,
not PowerShell syntax. In PowerShell, put each command on one line as above.
If a published release supplies `SHA256SUMS`, compare the archive's checksum
with it before extraction using `shasum -a 256` on macOS, `sha256sum` on Linux,
or `Get-FileHash -Algorithm SHA256` in PowerShell.

### Installation problems

| Symptom | Check |
| --- | --- |
| Command not found / not recognized | Try the executable's full path; add its folder, not the executable itself, to PATH |
| Permission denied on macOS/Linux | Check file permissions and that the filesystem permits execution |
| Wrong executable format / incompatible application | Download the correct operating-system and architecture archive |
| OS blocks an unsigned download | Verify the source and follow your organization's approval process |
| npm returns an authentication or permission error | Check the scope registry, GitHub Packages authentication, and package access |

If installation still fails, collect the details listed in
[Getting help](../maintenance/importing.md#getting-help).

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

The npm wrapper requires Node.js 22.19+ and authentication to GitHub Packages,
even for public packages. First configure the registry and authenticate using
GitHub's npm registry instructions. Keep tokens out of committed files.
These commands only work after the package has been published:

::code-block
```sh
npm config set @cmu-sei:registry https://npm.pkg.github.com
npm install --global @cmu-sei/sds-site-generator
sds-site-generator --version
```
::

No Rust compilation or postinstall download is needed. For a shared project,
install a published version as an exact development dependency in the site
directory, and commit the manifest and lockfile:

::code-block
```sh
npm init -y
npm pkg set private=true --json
npm install --save-dev --save-exact @cmu-sei/sds-site-generator@0.1.0
npm pkg set "scripts.dev=sds-site-generator preview ." \
  "scripts.check=sds-site-generator check . --strict" \
  "scripts.build=sds-site-generator build ."
```
::

Replace `0.1.0` with the published version you intend to use. Contributors then
authenticate, run `npm ci --ignore-scripts`, and use `npm run dev`,
`npm run check`, and `npm run build`. Ignore `node_modules/` and `dist/`.
