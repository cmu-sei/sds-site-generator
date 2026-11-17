---
order: 1
---
# User guide

Everything an author needs to create, maintain, and publish a documentation site.
Follow the four sections in order for a first site, or expand the relevant
sidebar group to jump straight to a task.

| Section | Covers |
| --- | --- |
| [Start here](start/index.md) | Install, create a site, and configure it |
| [Write content](authoring/index.md) | Organize pages, write Markdown, and add components and assets |
| [Publish and operate](publishing/index.md) | Build, deploy, collaborate, and run search reliably |
| [Maintain and migrate](maintenance/index.md) | Import content, troubleshoot, and preserve URLs during changes |

Looking for a specific reference? Open [site configuration](start/configuration.md),
[front matter](authoring/markdown.md#front-matter), or
[allowed asset types](authoring/assets.md#allowed-files).

## Authoring checklist

- Give every page one main heading and an introductory paragraph.
- Use meaningful link labels and accurate image descriptions.
- Keep Markdown in `content/` and assets in `public/`.
- Set the real publishing URL before deployment.
- Run `sds-site-generator check . --strict` before publishing.
- Review the built site on a small screen and with a keyboard.

For practical review examples, see [accessible authoring](authoring/accessibility.md).
Before sharing content, review [licensing and publication rights](publishing/licensing.md).
