---
order: 1
---
# Installation

Install SDS Site Generator on the computer where you will write and build your
documentation. The standalone executable is recommended for end users: it does
not require Node.js, npm, Rust, an account, or a search service.

::callout{tone="info"}
**Public downloads are not available yet.** The instructions below describe
the release archives and optional npm package once published. Until then,
request an executable for your platform from your project maintainer, or ask
a contributor to [build from source](#build-from-source).
::

## Choose your download

When a release is published, open
[GitHub Releases](https://github.com/cmu-sei/sds-site-generator/releases)
and download the archive for your operating system and processor, along with
`SHA256SUMS`. Choose the computer that will run the generator, not the
computers that will read the generated website.

| Operating system | Processor | Archive name |
| --- | --- | --- |
| macOS | Apple silicon (ARM64) | `sds-site-generator-darwin-arm64-vVERSION.tar.gz` |
| macOS | Intel (x64) | `sds-site-generator-darwin-x64-vVERSION.tar.gz` |
| Linux | ARM64 | `sds-site-generator-linux-arm64-vVERSION.tar.gz` |
| Linux | x64 | `sds-site-generator-linux-x64-vVERSION.tar.gz` |
| Windows | ARM64 | `sds-site-generator-win32-arm64-vVERSION.zip` |
| Windows | x64 | `sds-site-generator-win32-x64-vVERSION.zip` |

`VERSION` stands for the release number, such as `0.1.0`. On macOS/Linux,
`uname -m` commonly reports `arm64`/`aarch64` for ARM64 or `x86_64` for x64.
On Windows, check **Settings > System > About > System type**.

Before extracting, calculate the archive's SHA-256 checksum with
`shasum -a 256 ARCHIVE` on macOS, `sha256sum ARCHIVE` on Linux, or
`Get-FileHash ARCHIVE -Algorithm SHA256` in PowerShell. Replace `ARCHIVE` with
the downloaded filename and compare the result with its entry in `SHA256SUMS`.
Do not run the executable if the checksums differ.

## macOS and Linux

Extract the archive and open a terminal in the extracted directory. Verify
that the executable runs:

::code-block
```sh
chmod u+x ./sds-site-generator
./sds-site-generator --version
```
::

To install for your user account without administrator privileges:

::code-block
```sh
mkdir -p "$HOME/.local/bin"
cp ./sds-site-generator "$HOME/.local/bin/sds-site-generator"
export PATH="$HOME/.local/bin:$PATH"
sds-site-generator --version
```
::

To keep the PATH setting, add `export PATH="$HOME/.local/bin:$PATH"` to the
startup file appropriate for your shell and terminal, such as `.zprofile`
for a login zsh or `.bashrc` for an interactive bash. Open a new terminal
and confirm `sds-site-generator --version`.

Follow your organization's macOS download-approval process. Do not disable
OS security controls to run an untrusted executable. You can also use the
executable's full path instead of changing PATH.

## Windows PowerShell

Use **Extract All** on the downloaded ZIP archive. Put its contents in a
user-owned folder, such as `$HOME\Tools\sds-site-generator`, with the
executable directly inside that folder. In PowerShell, run:

::code-block
```powershell
cd "$HOME\Tools\sds-site-generator"
.\sds-site-generator.exe --version
.\sds-site-generator.exe --help
```
::

To run it from any directory, search Windows Settings for
**Edit environment variables for your account**. Edit your user **Path**,
add the executable's containing folder, and open a new PowerShell window.
Confirm `sds-site-generator --version`.

You can instead use a full path with PowerShell's call operator:
`& "$HOME\Tools\sds-site-generator\sds-site-generator.exe" --version`.
Follow your organization's download approval policy if Windows blocks it.

## Verify and create a site

With the executable on PATH, these commands work in both a POSIX shell and
PowerShell:

::code-block
```sh
sds-site-generator --version
sds-site-generator --help
sds-site-generator new my-docs --name "My documentation" --url "https://example.org/docs/"
sds-site-generator preview my-docs
```
::

Choose a directory that does not already exist. Open the address printed by
preview; Ctrl+C stops it. Continue with
[Create your first site](getting-started.md) for editing, validation, and builds.
Multiline `sh` examples elsewhere in this guide use POSIX backslash
continuations; in PowerShell, put each command on one line.

## Optional npm installation

The optional npm wrapper requires Node.js 22.19+ and authentication to GitHub
Packages, even for public packages. It is not required for standalone use.
These commands only work after the package has been published.

Follow [GitHub's npm registry authentication instructions](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-npm-registry#authenticating-to-github-packages)
using a personal access token (classic) with `read:packages` and access to
the package. Keep tokens out of committed files.

::code-block
```sh
npm config set @cmu-sei:registry https://npm.pkg.github.com
npm login --scope=@cmu-sei --auth-type=legacy --registry=https://npm.pkg.github.com
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
npm pkg set "scripts.dev=sds-site-generator preview ." "scripts.check=sds-site-generator check . --strict" "scripts.build=sds-site-generator build ."
```
::

Replace `0.1.0` with the published version you intend to use. Contributors then
authenticate, run `npm ci --ignore-scripts`, and use `npm run dev`,
`npm run check`, and `npm run build`. Ignore `node_modules/` and `dist/`.

## Build from source

This is a contributor alternative, not a prerequisite for end users.
With stable Rust installed, run these commands from a checkout of the
[source repository](https://github.com/cmu-sei/sds-site-generator):

::code-block
```sh
cargo build --release --locked
```
::

The executable is `target/release/sds-site-generator` on macOS/Linux or
`target\release\sds-site-generator.exe` on Windows. Use its full path or
install it using the platform instructions above. A locally staged
`bin/sds-site-generator` may also be available in a maintainer's checkout;
it is not a cross-platform download.

## Installation problems

| Symptom | Check |
| --- | --- |
| No release assets or npm package found | Public downloads are not available yet; request an executable from your maintainer or build from source |
| Command not found / not recognized | Try the executable's full path; add its folder, not the executable itself, to PATH and open a new terminal |
| Permission denied on macOS/Linux | Check file permissions and that the filesystem permits execution |
| Wrong executable format / incompatible application | Download the correct operating-system and architecture archive |
| OS blocks an unsigned download | Verify the source and follow your organization's approval process |
| npm returns an authentication or permission error | Check the scope registry, GitHub Packages authentication, and package access |

If installation still fails, collect the details listed in
[Getting help](../maintenance/importing.md#getting-help).
