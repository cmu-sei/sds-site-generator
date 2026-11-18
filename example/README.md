# Your documentation website

This example preserves imported Nuxt content and public assets to demonstrate
legacy compatibility. To create a new site, run `sds-site-generator new my-docs`.
Compatibility mode reports unrecognized MDC components, ignored `.navigation.yml` files,
and source link/content problems in `dist/_site/diagnostics.json`.
Callouts, code blocks, plain fences, and sortable Markdown tables use shared
SDS components without rewriting the copied content.
Use `sds-site-generator check . --strict` to reject these warnings.

Edit Markdown files in `content/`. Put images and downloads in `public/`.
Set your website name and publishing URL in `site.toml`.

```sh
sds-site-generator preview .
sds-site-generator check .
sds-site-generator build .
```

Preview prints a local address and rebuilds when you save a file. Refresh your
browser after a successful rebuild. Press Ctrl+C to stop the server.

Publish the contents of `dist/` to a static website host. Keep that directory
for generated output only; the generator replaces it on a successful build.

This website was created using the SEI Design System. Its license and
acknowledgments are included in every generated website.
