---
order: 5
---
# Accessible authoring

The site shell supplies accessible foundations, but authors determine whether
the content is understandable. Validation cannot judge the quality of a
description, a link label, or an explanation.

## Describe purpose, not just appearance

| Avoid | Prefer |
| --- | --- |
| Image description: "Screenshot" | "The Settings panel with Publish selected" |
| Diagram description: "Three boxes" | "Write content, validate it, then publish the generated site" |
| Link label: "Click here" | "Download the installation checklist (PDF)" |
| Heading: "More" | "Troubleshoot installation errors" |

Use the image description that helps the reader complete the task. Do not
repeat a nearby caption unnecessarily. A complex chart needs its conclusions
and important data in nearby prose or a table, not a very long alt attribute.
Only purely decorative images should use the
[explicit decorative marker](assets.md#add-an-image).

::code-block
```markdown
![The Settings panel with Publish selected](/images/publish-settings.png)
[Download the installation checklist (PDF)](/downloads/installation.pdf)
```
::

These are illustrative asset paths; add the files before using them.

## Make the reading order clear

- Use one main heading and consecutive section levels, not bold text posing
  as headings.
- Write descriptive headings that make sense in a table of contents.
- Use numbered lists for ordered tasks and bullets for unordered choices.
- Introduce code examples with their purpose and explain important results.
- Do not rely on position, color, or an icon alone: write "Warning" or
  "Noncompliant example" as well as choosing a visual treatment.

For example, replace "Choose the green option on the right" with
"Choose **Publish**." Use ordinary text for essential instructions, not text
embedded only in a screenshot.

## Keep tables simple

Use a header row and a short introduction explaining what the reader should
compare. Prefer a list for a sequence of instructions or unrelated facts.
Avoid merged cells and complicated header relationships when a simpler table
will work. Markdown tables cannot express every accessible table structure;
complex data may need carefully authored semantic HTML and manual review.

## Review with a keyboard and assistive technology

1. Use Tab and Shift+Tab to reach links and controls in a sensible order.
2. Try the skip link, sidebar disclosure controls, search, and code controls.
3. Check that long code and tables remain readable on a narrow screen and
   with browser zoom increased.
4. Review light and dark appearance; do not use color as the only distinction.
5. If possible, use a screen reader's heading and link lists to check that
   headings, labels, and image descriptions make sense without visual context.

Automated accessibility tools can help identify issues, but do not replace
manual review or establish WCAG conformance. Test the deployed site as well as
local preview, especially when your host adds authentication or other UI.
