# Documentation example

This is the generator's end-user guide and GitHub Pages home. It is also a
working example: its Markdown, configuration, callouts, code blocks, images,
and downloads use the same features described in the guide.
The sidebar groups the guides into Start here, Write content, Publish and
operate, and Maintain and migrate. Each group has a landing page; visit a child
to see its group expand and mark the active page.

From the repository root:

```sh
cargo run -- check docs-site --strict
cargo run -- preview docs-site
cargo run -- build docs-site
```

Open the address printed by preview; save and refresh to see changes.
The checked-in URL uses `/sds-site-generator/`, so preview uses that prefix too.
To reuse this site, copy it without `dist/` and change `site.toml`.
The `new` command still creates the smaller starter, not this guide.

The Pages workflow builds this directory with the actual deployment URL in
a temporary copy. It does not modify the checked-in configuration.
After staging the native executable with `npm run build`, reproduce it with:

```sh
node scripts/package.mjs pages https://your-org.github.io/your-repository/
```

Only the contents of `pages-dist/` are deployed. See
[the publishing guide](content/guides/publishing/deployment.md) for author-facing instructions.
