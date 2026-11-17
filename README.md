# SDS Documentation Site

A Markdown-first documentation website generator written in Rust, with the
[SDS Lite documentation shell](https://github.com/cmu-sei/sds-lite).

**Authors need one executable.** No Node.js, npm, Rust, accounts, search service,
or internet connection is required to generate, preview, or search a website.
The output is ordinary HTML, CSS, JavaScript, and JSON. SDS Lite is bundled
locally at a pinned version.

## User guide

The [documentation example site](docs-site/README.md) is the GitHub Pages home
and a complete authoring reference:

- [Start here: installation and configuration](docs-site/content/guides/start/index.md)
- [Write content: pages, Markdown, components, and assets](docs-site/content/guides/authoring/index.md)
- [Publish and operate: deployment, CI, search, and hosting](docs-site/content/guides/publishing/index.md)
- [Maintain and migrate: importing, troubleshooting, and stable URLs](docs-site/content/guides/maintenance/index.md)

Preview it from this repository with `cargo run -- preview docs-site`, or use
`npm run preview:docs` after `npm run build`. The existing `example/` remains
the legacy compatibility demonstration; `new` still creates the small starter.

The generated interface uses SDS Lite's documentation shell, sidebar, table of
contents, prose, forms, lists, table containers, buttons, dropdowns, disclosures,
and dialogs. It does not add a breadcrumb, matching the original CERT site.
No generator-specific presentation stylesheet or interaction module is emitted.
The optional SDS Lite documentation CSS and runtime own the reference layout,
TOC tracking, display controls, code controls, and search-result presentation.
The generator owns content, routes, navigation data, search indexing/ranking,
keyboard result selection, and syntax highlighting. Safe inline styles already
present in imported content may be retained; they are not required by the shell.

The bundled `example/content/` and `example/public/` were imported from the
legacy playground in `/Users/jdshimkoski/work/sds/sds-site-generator/.playground`.
The 59-page import uses compatibility mode; callouts have been migrated to the
semantic SDS Lite API. Callouts, code blocks, plain code
fences, and Markdown tables use the shared SDS components. Ignored Nuxt navigation
metadata and authored link/content issues still produce warnings. It is not an accessibility-certified
or warning-free starter. The `new` command still creates the clean starter;
automated search and browser quality checks use that independent starter.

## Try it

A ready-to-run macOS executable is available locally in `bin/sds-site-generator`.
From this directory:

```sh
./bin/sds-site-generator preview example
```

Open the address printed in the terminal. Edit `example/content/index.md`, save,
and refresh. Press Ctrl+C to stop.

Create your own website in a **new** directory:

```sh
./bin/sds-site-generator new ~/work/my-docs \
  --name "My documentation" \
  --url https://example.org/docs/
./bin/sds-site-generator preview ~/work/my-docs
./bin/sds-site-generator check ~/work/my-docs
./bin/sds-site-generator build ~/work/my-docs
```

The CLI never overwrites an existing directory when creating a site. Commands
other than `new` default to the current directory. Preview listens on
`127.0.0.1:4173`; use `--port 8080` to choose another port.

For distribution, obtain the binary for your OS and put it on your PATH.
The release workflow builds macOS, Linux, and Windows binaries for both x64
and ARM64 when a matching version tag is pushed. It publishes standalone
archives to GitHub Releases and an optional npm installer to GitHub Packages.
This project has not been published to a remote repository; no public binary
or npm download is available yet.

### Optional npm installation

After a release is published, configure the `@cmu-sei` scope for GitHub Packages
and authenticate using GitHub's npm registry instructions. Even public GitHub
Packages downloads require authentication; standalone release archives do not.
Never put an authentication token into a committed configuration file.

```sh
npm config set @cmu-sei:registry https://npm.pkg.github.com
npm install --global @cmu-sei/sds-site-generator
sds-site-generator --version
sds-site-generator preview /path/to/my-docs
```

The npm wrapper requires Node.js 22.19+ and selects the bundled executable for
the current OS and architecture. There are no runtime npm dependencies,
postinstall hooks, binary downloads, or Rust compilation during installation.
All six binaries are included, trading package size for predictable offline
installation from a previously downloaded tarball. Standalone executables
still require neither Node.js nor npm.

### Use it in a documentation Git repository

After the npm package is published, install it as a pinned development
dependency in a separate documentation repository. Contributors need Node.js
22.19+ and npm, but not Rust or a global generator installation.

First configure the GitHub Packages scope and authenticate as described above.
Then scaffold a **new** directory before running `git init` or `npm init`, since
the generator refuses to overwrite an existing directory:

```sh
npm exec --yes --package=@cmu-sei/sds-site-generator@0.1.0 -- \
  sds-site-generator new my-docs \
  --name "My Docs" --url https://your-org.github.io/my-docs/
cd my-docs
git init
npm init -y
npm pkg set private=true --json
npm install --save-dev --save-exact @cmu-sei/sds-site-generator@0.1.0
npm pkg set "scripts.dev=sds-site-generator preview ." \
  "scripts.check=sds-site-generator check . --strict" \
  "scripts.build=sds-site-generator build ."
```

The version above is the initial release version; substitute a published version
when upgrading. Before publication, bootstrap using the local executable's
`new` command instead; the registry install commands will not work yet.

Add a token-free `.npmrc` containing
`@cmu-sei:registry=https://npm.pkg.github.com`. Keep authentication in each
developer's user configuration, never in Git. Add these entries to `.gitignore`:

```gitignore
node_modules/
dist/
```

Commit `content/`, `public/`, `site.toml`, `README.md`, `package.json`,
`package-lock.json`, `.npmrc`, and `.gitignore`, then push to the documentation
repository. Do not commit executables or generated output.

```sh
npm run dev
npm run check
npm run build
```

The npm scripts automatically use the repository-local executable. Preview
watches the content; save and refresh the browser. Publish the contents of
`dist/`, not the generator package. Other contributors clone the repo,
authenticate to GitHub Packages, run `npm ci --ignore-scripts`, then
`npm run dev`. To update the generator, explicitly install the new version with
`--save-dev --save-exact`, test it, and commit both npm manifest and lockfile.

The scaffold does not install GitHub workflows in the documentation repository.
Its own CI can authenticate using `GITHUB_TOKEN` with `packages: read`, run
`npm ci --ignore-scripts` and `npm run check && npm run build`, then upload
`dist/` to Pages. Grant that repository Actions read access to the generator
package; the generator repository's release workflow is not copied to it.

## Writing content

```text
my-docs/
  site.toml
  content/
    index.md
    introduction.md
    guides/
      index.md
      publishing.md
  public/
    images/
      diagram.png
```

- Every published Markdown file becomes a page. `index.md` is a directory's
  landing page. Directories become navigation groups.
- Start with one `# Main heading`, then use `## Sections` and `### Subsections`.
  Skipping heading levels is an error. When there is no H1, the generator adds
  one using the page title.
- Titles default to the main heading; descriptions default to the introductory
  paragraph. Every page needs a description or introductory paragraph.
- Folders and filenames become lowercase, hyphenated URLs. Numeric prefixes
  such as `01.introduction.md` control default ordering but are omitted from
  URLs. URLs must be unique.
- Relative Markdown links, like `[Guide](guides/index.md#headings)`, are
  rewritten into website links and checked. Site-root links start with `/`
  and automatically include the deployment subdirectory.
- Duplicate headings receive `-2`, `-3`, etc. IDs `site-main`, `site-sidebar`,
  `site-search`, `site-query`, `site-results`, `site-status`, `site-search-dialog`,
  `site-search-title`, `site-dialog-search`, `site-dialog-query`,
  `site-dialog-results`, and `site-dialog-status` are reserved.
- Use Markdown tables with a header row. Tables and code blocks scroll
  horizontally rather than forcing the whole page to overflow.
- Markdown and semantic HTML share heading, link, image, and search processing.
  Safe legacy HTML and conservative layout styles are preserved. Scripts,
  unsafe attributes, unsupported widgets, and unsafe styles are diagnosed and
  removed. Symbolic links are rejected. Arbitrary executable content is not
  supported.

### Optional metadata

Use YAML between `---` lines at the start of a file:

```yaml
---
title: Page title
description: A specific, useful summary of this page.
order: 2
draft: false
navigation:
  title: Short navigation label
tags: [documentation]
---
```

YAML accepts additional Nuxt/custom metadata such as `tags`; it is not
executed. `navigation: false` hides a page from navigation but leaves it
published and searchable. Fenced YAML code blocks in the body are supported
as ordinary code examples. TOML between `+++` lines is also supported:

```toml
+++
title = "Page title"
description = "A specific, useful summary of this page."
order = 2
draft = false
+++
```

All fields are optional. `order` is a signed integer; smaller numbers come
first among sibling pages. Unknown TOML fields fail with a diagnostic. Drafts are
excluded from output, navigation, sitemap, and search; links to drafts fail.
The home page must be published.

### Images and downloads

Place files in `public/`; reference them as `/images/diagram.png`:

```markdown
![Flowchart showing an author reviewing a draft](/images/diagram.png)
![](/images/decoration.png "decorative")
[Download the guide](/guide.pdf)
```

Images require a useful description unless explicitly marked decorative.
Dimensions, lazy loading, and asynchronous decoding are added automatically.
Supported image formats are PNG, JPEG, GIF, WebP, AVIF, and sanitized SVG; individual images
must not exceed 1 MiB. Resize and compress images before adding them. The
generator reads dimensions; it does not optimize general image content.
Legacy TIFF images receive browser-readable PNG aliases while the original
TIFF downloads remain available. Public downloads also support ICO, PDF, TXT,
CSV, ZIP, WOFF2, C/C++/Java source, XML, URL, JAR, TAR, and GZ attachments.
HTML attachments and SVG are sanitized and changes are diagnosed.
Asset names may contain spaces and Unicode, but not hidden path segments,
control characters, backslashes, or URL delimiters. Author-provided JavaScript
and CSS files are not published.

### Existing code-example components

Existing MDC examples work without Vue:

````markdown
::code-block{quality="good"}
```c
return 0;
```
::
````

Use `quality="bad"` for noncompliant examples. These become static
`<site-code-block>` elements with the original component's red/blue compliance
treatment, compact icon controls, and locally bundled Shiki GitHub high-contrast
syntax themes. Copying shows a two-second confirmation; clipboard failures are
visibly reported. Fullscreen supports Escape, keeps the page position, and
restores focus. Code remains readable without JavaScript.

The optional `title="Example title"` property adds a tinted title bar.
`quality="average"` selects the neutral treatment. Within a block, `#default`
and `#complete` introduce short and complete code slots; an icon toggles between
them, and copying uses the selected code. Property order and single or double
quotes are supported. Icon and highlighting licenses are bundled in
`dist/_site/code-controls-LICENSE.txt`.
Fence labels select the bundled language grammar. Markdown (`markdown`, `md`),
TOML (`toml`), PowerShell (`powershell`, `ps`, `ps1`, `pwsh`), and INI
(`ini`, `properties`) are supported alongside the programming, shell, and data
languages listed in the [code-block reference](docs-site/content/guides/authoring/components.md#syntax-highlighting).
Highlighting requires JavaScript, uses local assets with light/dark themes,
and does not auto-detect languages. Unlabeled fences, `text`, and unsupported
labels remain readable plain text.

Callouts use the shared SDS recipes and preserve nested Markdown and code blocks:

```markdown
::callout{tone='warning' variant='outline' size='sm'}
**Warning:** Validate input before processing it.
::
```

Supported semantic tones are `neutral` (default), `accent`, `info`, `success`,
`warning`, and `danger`. Variants are `outline` and `bold`; omit `variant` for
the default subtle surface. Sizes are `xs`, `sm`, `md` (default), and `lg`.
These map directly to SDS Lite's tone, variant, and size attributes.
The former `type` property and color-name variants are accepted only with
`compatibility = true`, with migration warnings. Normal mode rejects them;
`check --strict` also rejects the compatibility warnings.
Properties are allowlisted; executable Vue attributes are not supported.
Component syntax inside fenced code stays literal.

The rendered documentation and starter use the shared code-block wrapper for
all block examples. Plain fences remain supported as keyboard-focusable,
scrollable code without copy/fullscreen controls.
Markdown tables retain semantic, readable no-JavaScript markup;
the shared SDS runtime adds keyboard-accessible header sorting for simple
tables, with ascending/descending states and numeric comparison. Tables with
merged cells or existing interactive headers remain static.

Other Vue/MDC widgets are diagnosed, not executed.

### Importing the CERT content

Copy the original project's `content/` and `public/` directories into a new
site, replacing the starter content, and set `compatibility = true` in
`site.toml`. Existing YAML metadata, semantic HTML, anchors, attachments,
and code-example wrappers do not require rewriting. Compatibility mode preserves
underscores in page URLs so imported Nuxt routes remain available.
Compatibility mode accepts legacy callouts with warnings. To migrate them,
replace color-name `variant` with semantic `tone`, rename `type` to `variant`,
and omit the former `type="subtle"`.

Compatibility mode reports source problems such as missing targets, missing
image descriptions, heading gaps, and removed unsafe markup as warnings.
Review the terminal summary and complete `dist/_site/diagnostics.json`.
Run `sds-site-generator check --strict` to fail on any remaining warnings.
Invalid metadata, duplicate URLs, unsafe filenames, and output collisions
remain fatal in **every** mode; the last good build is preserved.

The current upstream corpus has **two filename collisions**. Fix these in
your imported copy before building; the generator deliberately does not
silently pick a page or rename URLs:

| Imported filename | Suggested replacement |
|---|---|
| `content/4.sei-cert-c-coding-standard/04.back-matter/7.ff-related-guidelines/10.2013.md` | `10.2013-alternate.md` |
| `content/6.sei-cert-oracle-coding-standard-for-java/6.admin/6.todo-list.md` | `6.todo-list-alternate.md` |

Both corrections preserve the page content. The original source repository
is untouched. The full-corpus regression builds all **1,265 Markdown pages**
after these explicit filename corrections. Remaining content warnings need
editorial review; compatibility mode cannot certify imported accessibility.

A prepared local import is available in `cert-example/` (ignored by Git).
Preview it with `./bin/sds-site-generator preview cert-example --port 4174`.
Open `http://127.0.0.1:4174/secure-coding-standards/`. Serving its `dist/`
directly at `/` with a generic static server does not mount the configured
deployment subdirectory, so stylesheet, script, image, and page URLs will fail.
The built-in preview command mounts that subdirectory automatically.
Its current report contains 2,259 warnings, including missing images, missing
image descriptions, broken page/anchor links, and heading/markup repairs.
These are not a claim that imported pages achieve perfect audit scores.

### Site configuration

```toml
name = "My documentation"
url = "https://example.org/docs/"
language = "en"
description = "Documentation for our team."
compatibility = false
repository = "https://github.com/example/docs"
```

`name` and `url` are required. The language defaults to `en`. The site
description is optional and is used as the search page's description; ordinary
pages have their own descriptions. The publishing URL determines canonical
URLs, sitemap URLs, and the deployment subdirectory. Preview keeps the same
subdirectory on localhost. Change the placeholder `https://example.com/`
before publishing. Credentials, URL queries, fragments, and unknown settings
are rejected.
`repository` is optional and adds a source link to the masthead. It must be an
absolute HTTP(S) URL without credentials; no CERT repository is hardcoded.

### Main navigation

By default, the main menu is generated from top-level content. To replace it,
add ordered `navigation` entries to `site.toml`:

```toml
[[navigation]]
title = "Home"
path = "/"

[[navigation]]
title = "Documentation"
path = "/guides/"

[[navigation.children]]
title = "Publishing"
path = "/guides/publishing/"
```

Entries with `children` become dropdown groups in the header and expandable
groups in the mobile sidebar. A group's optional `path` adds its landing-page
link inside the dropdown. Children can contain further groups. Ordering follows
the configuration; the full documentation tree is unchanged.

Local paths start at the site root, without the deployment subdirectory, which
is added automatically. Page links and heading fragments are checked; invalid
menu targets fail even in compatibility mode. Absolute HTTP(S) links are also
supported. Omit `navigation` to keep the automatic menu, or set `navigation = []`
before any TOML table declarations to hide it. Dropdowns work without JavaScript;
JavaScript adds outside-click and Escape dismissal.

## Search

Search is first-party JavaScript with no third-party search dependencies.
Rust generates a positional inverted index; a module Web Worker performs
queries away from the main thread.

- Page and heading matches receive additional ranking weight.
- Multiple words use AND matching.
- The final unquoted term supports prefix completion.
- Quoted phrases require adjacent tokens in order.
- A compact live-search palette follows the published CERT search layout:
  one input, unnumbered SDS Lite list rows, heading titles, and muted page context.
- Results link directly to sections; there are no long excerpts or submit button.
- Arrow keys navigate results, Enter follows a result, and Escape closes the
  dialog immediately and restores focus.
- Term buckets and document metadata chunks load only when needed.
- Fetches are cached in the worker. Errors appear explicitly on the page.
- Search computation stays in the browser; there is no telemetry or remote
  search service. The standalone search page puts the query in its bookmarkable
  URL, so opening or refreshing that URL may include it in normal hosting logs.

Search needs JavaScript and HTTP(S) serving. Opening generated HTML via
`file://` will not enable worker-based search. Content and navigation remain
usable without JavaScript in supported modern browsers.

## Build and publish

`check` runs the same content validation as `build`, without writing output.
`build` stages a complete website before replacing `dist/`. Validation failures
leave the last good build unchanged. An existing `dist/` without the generator's
ownership marker is never replaced. Keep hand-authored files outside `dist/`.
Do not run two builds against the same project simultaneously.

Preview watches `content/`, `public/`, and `site.toml` by polling and reports
rebuild errors in the terminal while serving the last good build. Refresh
manually after a successful rebuild. It is a local development server, not a
production web server.

Upload the **contents** of `dist/` to a static host. Output includes:

- Fully rendered pages, canonical URLs, descriptions, Open Graph/Twitter
  metadata, and WebPage structured data.
- Navigation, previous/next page links, heading anchors, and responsive TOCs.
- `sitemap.xml`, `robots.txt`, `.nojekyll`, and `404.html`.
- Local SDS Lite assets, license acknowledgments, and search data.
- A complete `_site/diagnostics.json` content-warning report.
- Precompressed `.gz` variants of text assets, pages, and search indexes.
  Preview negotiates these automatically. Production hosts can serve them
  with `Content-Encoding: gzip` and `Vary: Accept-Encoding`, or apply their own
  gzip/Brotli compression.

Configure your host to serve directory indexes and return `404.html` with an
actual HTTP 404 status for missing pages. Enable HTTPS and compression through
your host. For a subdirectory deployment, ensure the origin's `/robots.txt`
permits crawling and points to your sitemap; a project-level robots file
cannot control the origin. External link availability is not checked.

## Accessibility and performance

The fixed shell supplies semantic landmarks, skip navigation, labeled
controls, current-page state, native popover navigation, keyboard focus
indicators, reduced-motion support, and SDS Lite's light/dark system theme.
Content, navigation, and SEO metadata do not depend on hydration.

The generated layout follows the actual **Documentation Site** starter in
the SDS Lite playground: SEI wordmark masthead with GitHub/search actions,
shield-marked site title, horizontal section navigation, icon-bearing nested
sidebar, reading column, responsive table of contents, and branded SEI footer.
Search opens in a native modal dialog; without JavaScript its masthead link
opens the standalone search page instead. Heading levels remain H1/H2/etc.
without shifted typography classes. Shell spacing,
colors, responsive behavior, and typography come from SDS Lite's own CSS,
not a restyled approximation.

**Neither WCAG conformance nor a perfect Lighthouse score is guaranteed.**
Meaningful alt text, accurate descriptions, understandable links, content
structure, real-device testing, screen-reader testing, and production hosting
remain the publisher's responsibility. The generator cannot judge whether an
image description is accurate or whether a table needs additional context.
There is no automatic image optimizer; local Shiki supplies syntax highlighting.

Browser support follows SDS Lite v0.7.1: modern browsers supporting Custom
Elements, Popover, CSS `@scope`, and the documented modern CSS features.
Older browser support and polyfills are outside this release.

## Development

The optional root npm scripts provide a cross-platform contributor workflow.
Use the Node version in `.nvmrc` and npm 12.2.1, plus stable Rust:

```sh
npm ci --ignore-scripts
npm run build
npm test
npm run check
npm run build:site
npm run preview -- --port 4176
```

`npm run build` compiles and stages only the current platform in `native/`.
`npm test` then exercises the real CLI, offline tarball installation, and Pages
generation, as well as metadata and workflow checks. Native integration tests
are skipped before a binary is staged. `npm run check:package` and ordinary
`npm pack` deliberately require all six release binaries; a local host-only
build is not a publishable distribution. CI assembles the complete bundle.
Generated binaries, tarballs, and Pages output are ignored by Git.
The root `npm run check` honors the imported example's explicit compatibility
mode; run `sds-site-generator check example --strict` to reject import warnings.
Use `npm run check:docs`, `npm run build:docs`, and `npm run preview:docs` for
the clean documentation example. The Pages build validates that site strictly.

Install current stable Rust to build from source:

```sh
cargo build --release --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

`target/release/sds-site-generator` is self-contained with respect to the Rust
toolchain and website assets. OS system libraries still apply. Rust crates
are build-time dependencies compiled into that executable; `Cargo.lock`
records resolved versions.

### GitHub automation

The workflows follow SDS Native's pinned actions, direct pull-request checks,
locked npm installation, environment approvals, and separate build/deploy jobs.
They use this project's bundled SDS assets, not the local SDS Native workspace.

| Workflow | Trigger | Result |
|---|---|---|
| `.github/workflows/check.yml` | PRs, pushes to `main`/`hotfix/v*`, manual runs, reusable calls | Rust format/lint/tests, six native builds, real npm installation tests, full package and standalone archives, existing browser/Lighthouse checks |
| `.github/workflows/release.yml` | `v*` tags or manual runs | Reuses Check; matching version tags publish the tested npm tarball and GitHub Release archives |
| `.github/workflows/pages.yml` | Pushes to `main` or manual runs on the default branch | Generates and deploys `docs-site/` at the configured GitHub Pages URL |

Each OS/architecture builds and tests on its native hosted runner. Linux uses
statically linked musl binaries rather than requiring the build runner's glibc
version. macOS and Windows use their normal system toolchains. Hosted ARM
runner availability depends on repository visibility and GitHub's runner
offering; the default ARM labels target public repositories. For other runner
plans, adjust `scripts/platforms.mjs` without dropping native execution tests.
Unsupported OS/architecture combinations fail with an explicit npm diagnostic.
Standalone binaries are not code-signed or notarized; OS download/security
policies still apply. These builds do not claim support for every older OS.

Before enabling publication:

1. Create the GitHub repository. The package defaults to
  `@cmu-sei/sds-site-generator` and `cmu-sei/sds-site-generator`; change the npm scope
  and repository URL if the owner or repository differs. Refresh the lockfile
  with `npm install --package-lock-only --ignore-scripts` after metadata edits.
2. Review the [project license](LICENSE), which matches the SEI Design System
  license used by SDS Native. Preserve it in npm and standalone distributions,
  along with the independently included third-party notices.
3. Create the `github-packages` environment with the desired protection rules.
  Allow the release workflow's `GITHUB_TOKEN` to write packages and releases.
  For an existing package, grant this repository Actions access to it.
4. In repository Settings > Pages, select **GitHub Actions** as the source.
  Configure the `github-pages` environment and optional custom domain.
  The workflow uses Pages' actual base URL, including project prefixes,
  without changing `docs-site/site.toml`; neither legacy example is deployed.
5. Require the Check workflow's **Build and test** result for pull requests.
  Runner builds, browser checks, and complete-package validation must pass
  before a release can publish.

For a release, update `Cargo.toml` and `package.json` to the same version and
refresh `package-lock.json`. Commit and review those changes, then push the
matching tag, such as `v0.1.0`. A manual Release run on a branch builds artifacts
without publishing; publication only runs on version tags. Stable versions use
the npm `latest` tag; prereleases use `next` and a prerelease GitHub Release.
Downloaded release artifacts are checked against their SHA-256 manifest before
publication. Workflow publication uses `GITHUB_TOKEN`, not a new npm secret.
If npm publication succeeds but GitHub Release creation fails, the npm version
is immutable: complete the GitHub Release from that run's verified artifacts
rather than trying to overwrite the npm version.

For a local Pages build after `npm run build`:

```sh
node scripts/package.mjs pages https://your-org.github.io/your-repository/
```

Only the generated contents of `pages-dist/` are uploaded. Pages is a static
production host; the generator's preview process is never deployed.

The deep generation module exposes only `create`, `check`, and `build`;
preview builds on that interface. Content parsing and URL resolution are
shared by validation, rendering, navigation, and search rather than implemented
separately in each output path.

### Browser and Lighthouse checks (development only)

Node.js 22.19+ and Chrome are needed **only for contributors running audits**,
not website authors or website generation:

```sh
cargo run -- build example
# Optional regression against a local copy of the original project:
SITE_GENERATOR_COMPAT_SOURCE=/path/to/secure-coding-standards \
  cargo test --test compatibility -- --ignored
cd tools
npm ci
npm test
# Browser/Lighthouse scripts exercise the clean starter, not the legacy import.
# In a separate terminal, from the project root:
# cargo run -- new /tmp/sds-audit-site --url https://example.org/demo/
# cargo run -- preview /tmp/sds-audit-site
CHROME_PATH="/path/to/chrome" npm run audit
CHROME_PATH="/path/to/chrome" npm run test:search-ui
```

Without `CHROME_PATH`, install Playwright's browser with
`npx playwright install chromium` for browser tests; Lighthouse also needs a
Chrome installation or `CHROME_PATH`. `SITE_URL` overrides the default
`http://127.0.0.1:4173/demo/`.

The audit exercises desktop/mobile pages, axe, overflow, navigation, search,
and no-JavaScript content. It saves Lighthouse JSON reports under
`tools/reports/` and requires 100 accessibility, best-practices, and SEO scores
with a performance regression floor of 95. This is a regression budget, not a
claim that every generated website achieves those scores.

On 2026-10-28, the three bundled starter content pages each measured **100
performance, 100 accessibility, 100 best practices, and 100 SEO** with
Lighthouse 13.6.0's mobile profile against the gzip-enabled local preview.
The browser audit also passed axe at 1440, 390, and 320 CSS pixels, dark-mode
checks, modal focus/keyboard checks, search-failure handling, and no-JavaScript
navigation. These automated results do not replace manual accessibility or
production-host testing.

### SDS Lite updates

See [vendor/sds-lite/PROVENANCE.md](vendor/sds-lite/PROVENANCE.md) for the exact
upstream commit and asset checksums. Updates are explicit: replace the complete
asset set, preserve the license, update provenance, and rerun Rust/browser
checks. The generator never downloads assets during normal use.

## License

This project uses the [SEI Design System license](LICENSE), matching SDS Native
and the SEI Design System. The npm package and standalone release archives
include it. Bundled third-party software retains its own terms.
See [licensing and publication rights](docs-site/content/guides/publishing/licensing.md)
for the distinction between software terms and rights to authored content.

## Current scope

This supports the existing CERT Markdown corpus with the two explicit filename
corrections above and compatibility warnings. It does not execute arbitrary
Vue components or reproduce all Nuxt features, Tailwind utilities, custom
styling, or redirects. The observed `code-block` wrapper has a first-party
semantic wrapper enhanced by SDS Lite's documentation module; other widgets require an explicit implementation.
A browser-based editor, general image optimization, translations, and
automatic deployment are not implemented. No changes were made to the
original Nuxt repository.
