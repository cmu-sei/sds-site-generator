# Publishing

The generator produces ordinary files. Any static website host can serve them.

## Set your address

Open site.toml and replace the example URL with your final website address.
Include a subdirectory if you use one, such as https://example.org/my-docs/.

## Check and build

```sh
sds-site-generator check .
sds-site-generator build .
```

Upload the contents of dist to your host. For GitHub Pages, use an Actions
deployment and upload dist as the Pages artifact.

## Before sharing

- Check your content for accuracy and clarity.
- Test keyboard navigation and a screen reader.
- Test your deployed pages on a phone.
- Run Lighthouse against the deployed URL.

The generator supplies accessible patterns and technical SEO defaults.
Accessibility conformance, search rankings, and perfect Lighthouse scores still
depend on your content, hosting, and testing.
