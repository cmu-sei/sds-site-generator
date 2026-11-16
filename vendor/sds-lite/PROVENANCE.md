# Bundled SDS Lite

- Upstream: https://github.com/cmu-sei/sds-lite
- Base package version: v0.7.1; these assets are an uncommitted local development build,
  not the published v0.7.1 release.
- Source workspace: `/Users/jdshimkoski/work/playground/sds-native`.
- Base source commit: `f45068d5faa850e7f7685a462eb27002a17f58c8`, plus
  uncommitted production-readiness, documentation presentation, and shared
  standalone/application sidebar styling changes, including documentation-tree
  reference spacing, disclosure controls, marker-free root items, and unclipped
  nested current-item markers. The rebuilt `brand.css` also includes consistent
  documentation content alignment without a sidebar and primary-tab styling
  for header dropdown triggers, without leaking tab styling into menu items.
  The rebuilt `documentation.css` hides primary-navigation-only drawers on
  desktop while retaining their mobile popover behavior. Mobile primary dropdown
  triggers use full-width navigation rows with matching typography, neutral
  text, square corners, and active navigation markers.
  Mobile configured navigation uses sliding child panels instead of floating
  dropdowns, with a Go Back control, reference-sized rows and drawer width,
  whole-drawer Escape dismissal, link dismissal, and reset-on-close behavior.
  These behaviors are provided by the rebuilt `documentation.css` and
  `documentation.js`.
  The latest shared documentation recipes also provide token-driven panel
  entry, exit, and return animations, centered navigation chevrons, themed
  heading typography for Go Back, and aligned mobile drawer titles with
  44px close controls. All five CSS/runtime assets were refreshed from the
  current source build; no generator-specific styling was added.
- Last copied: 2026-11-12. No commit, staging, or publication was performed.
- Documentation icon bundle added: 2026-11-05. The shared starter no longer
  includes fullscreen. Its GitHub, Material Symbols theme/search, and mobile
  menu artwork matches the live CERT reference. The native build extracts these
  definitions from `index.html` into `dist/documentation-icons.json`, which the
  generator consumes directly instead of substituting Lucide or hand-drawn
  header icons. Artwork attribution is included in the JSON and generated
  `_site/code-controls-LICENSE.txt`.
  The shared theme-toggle visibility rule displays the active scheme: sun in
  light mode and moon in dark mode. Its accessible label still describes the
  action that switches to the other scheme.
- Documentation stylesheet refreshed: 2026-11-05. Search results retain a
  high-contrast 2px keyboard-focus outline, drawn inset to avoid clipping in
  the scrollable result list, with a system Highlight outline in forced colors.
- Component recipes refreshed: 2026-11-05. Shared `sds.css` supplies all
  callout colors, types, sizes, and prose spacing, including indigo and teal
  recipes derived from semantic theme tokens. Shared `documentation.js` adds
  keyboard-accessible text and numeric table sorting with `aria-sort` states;
  merged-cell tables and interactive headers remain static. The table recipe
  supplies sort-icon sizing and orientation. All assets are copied unchanged
  from the source build; no generator-specific CSS or interaction module was added.
- Sortable documentation headers refreshed: 2026-11-05. The shared table
  recipe makes sort buttons inherit their header cell's color and typography
  instead of the prose link tone or generic button font. Inactive indicators
  are hidden until hover or keyboard focus; active sort indicators remain
  visible. Keyboard focus uses an inset current-color outline.
- Code examples refreshed: 2026-11-06. The shared documentation recipe matches
  the CERT reference's compact monospace code typography, Forge semantic tones
  and radius tokens, icon-only quality badges with SDS Lite tooltips, and
  tooltips for code actions. Copy and fullscreen tooltip labels track their
  current state. Documentation also moves
  removes the design-system acknowledgment from the generated footer, uses the plain list
  TOC icon, and limits TOC/scrollspy headings to H2. Table sort controls
  inherit their headers' typography on initial render; active sort indicators
  are visible immediately and don't shrink in narrow tables. The assets are copied
  unchanged from the SDS Native source build.
- Interaction polish refreshed: 2026-11-06. Code-copy feedback keeps the SDS
  tooltip aligned, replaces the copy icon with a semantic success check, and
  suppresses action tooltips in fullscreen. Mobile Go Back controls use a
  full-width aligned row with Forge/Plaid typography tokens.
- Configured primary overview links refreshed: 2026-11-06. Parent links that
  also have children use the native SDS Lite dropdown CTA menu-item variant
  and dropdown divider in desktop menus; they retain menu-item typography rather
  than adopting the link component's font sizing.
- Production-readiness assets refreshed: 2026-11-12. All vendored assets were
  copied unchanged from the audited source build. This refresh supersedes the
  earlier theme-icon and H2-only TOC notes: theme artwork describes the target
  scheme (moon in light mode, sun in dark mode), and TOC targets support H2-H6
  within their own application shell. It also corrects code-block colors and
  typography, mobile dropdown row sizing, authored tooltip initialization, and
  restoration of authored fullscreen control labels.
- Source paths: `dist/sds.css`, `dist/brand.css`, `dist/auto.js`,
  `dist/documentation.css`, `dist/documentation.js`, `dist/documentation-icons.json`,
  `dist/package/assets/sei-wordmark.svg`, and the root `LICENSE`.
- License: see [LICENSE](LICENSE). Generated sites continue to bundle the
  required design-system license files.

The files are unmodified. `brand.css` references the wordmark by its original
relative path; that directory layout is preserved in generated websites.
Shell markup and decorative icons in `src/render.rs` are adapted from the
upstream Documentation Site starter and its optional documentation recipes.

Exact SHA-256 checksums for every copied asset, including the wordmark and
license, are recorded in [SHA256SUMS](SHA256SUMS). Verify them with
`shasum -a 256 -c SHA256SUMS` from this directory.
