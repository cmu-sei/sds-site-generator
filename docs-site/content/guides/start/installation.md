---
order: 1
---
# Installation

Install the standalone executable on the computer where you write and build
documentation. It needs no Node.js, npm, Rust, account, or search service.

## Choose your download

From [GitHub Releases](https://github.com/cmu-sei/sds-site-generator/releases),
download `SHA256SUMS` and the archive for your build computer's operating
system and processor.

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

Before extracting, replace `ARCHIVE` below with the downloaded filename and
calculate its SHA-256 checksum:

- macOS: `shasum -a 256 ARCHIVE`
- Linux: `sha256sum ARCHIVE`
- PowerShell: `Get-FileHash ARCHIVE -Algorithm SHA256`

Compare it with the archive's entry in `SHA256SUMS`. Do not run the executable
if the checksums differ.

## macOS and Linux

Extract the archive, open a terminal there, and verify the executable:

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

Keep the PATH setting by adding `export PATH="$HOME/.local/bin:$PATH"` to your
shell's startup file: for example, `.zprofile` for login zsh or `.bashrc` for
interactive bash. Open a new terminal and run `sds-site-generator --version`.

Follow your organization's download-approval process; do not disable OS
security controls. To avoid changing PATH, use the executable's full path.

## Windows PowerShell

Use **Extract All** to unpack the ZIP into a user-owned folder such as
`$HOME\Tools\sds-site-generator`. With the executable directly inside it, run:

::code-block
```powershell
cd "$HOME\Tools\sds-site-generator"
.\sds-site-generator.exe --version
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

With the executable on PATH, verify it in your shell:

::code-block
```sh
sds-site-generator --version
sds-site-generator --help
```
::

Continue to [Create your first site](getting-started.md).
Multiline `sh` examples elsewhere in this guide use POSIX backslash
continuations; in PowerShell, put each command on one line.

## Optional npm installation

To manage the generator with npm, install Node.js 22.19+ and authenticate to
GitHub Packages, which requires authentication even for public packages.

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

The package includes the executable; no compilation or postinstall download
is needed. For a shared project, run these commands in the site directory:

::code-block
```sh
npm init -y
npm pkg set private=true --json
npm install --save-dev --save-exact @cmu-sei/sds-site-generator@0.1.0
npm pkg set "scripts.dev=sds-site-generator preview ." "scripts.check=sds-site-generator check . --strict" "scripts.build=sds-site-generator build ."
```
::

Replace `0.1.0` with your chosen version. Commit the manifest and lockfile;
ignore `node_modules/` and `dist/`. Contributors authenticate, run
`npm ci --ignore-scripts`, then use `npm run dev`, `npm run check`, and
`npm run build`.

## Build from source

With stable Rust installed, run this from a checkout of the
[source repository](https://github.com/cmu-sei/sds-site-generator):

::code-block
```sh
cargo build --release --locked
```
::

The executable is `target/release/sds-site-generator` on macOS/Linux or
`target\release\sds-site-generator.exe` on Windows. Use its full path or
install it using the platform instructions above.

## Installation problems

| Symptom | Check |
| --- | --- |
| Download or package not found | Use the linked GitHub Releases page for archives; for npm, configure the `@cmu-sei` registry and authenticate to GitHub Packages |
| Command not found / not recognized | Try the executable's full path; add its folder, not the executable itself, to PATH and open a new terminal |
| Permission denied on macOS/Linux | Check file permissions and that the filesystem permits execution |
| Wrong executable format / incompatible application | Download the correct operating-system and architecture archive |
| OS blocks an unsigned download | Verify the source and follow your organization's approval process |
| npm returns an authentication or permission error | Check the scope registry, GitHub Packages authentication, and package access |

If installation still fails, collect the details listed in
[Getting help](../maintenance/importing.md#getting-help).
