---
order: 5
---
# Accessible authoring

Review descriptions, link labels, and explanations with every content change.

## Describe purpose, not just appearance

| Avoid | Prefer |
| --- | --- |
| Image description: "Screenshot" | "The Settings panel with Publish selected" |
| Diagram description: "Three boxes" | "Write content, validate it, then publish the generated site" |
| Link label: "Click here" | "Download the installation checklist (PDF)" |
| Heading: "More" | "Troubleshoot installation errors" |

Describe images to help the reader complete the task; avoid repeating captions.
Explain complex charts in nearby prose or a table rather than long alt text.
Use the
[explicit decorative marker](assets.md#add-an-image)
only for purely decorative images.

::code-block
```markdown
![The Settings panel with Publish selected](/images/publish-settings.png)
[Download the installation checklist (PDF)](/downloads/installation.pdf)
```
::

These are illustrative asset paths; add the files before using them.

## Make the reading order clear

- Use one main heading and consecutive heading levels, not bold text as headings.
- Write descriptive headings that make sense in a table of contents.
- Use numbered lists for ordered tasks and bullets for unordered choices.
- Introduce code examples with their purpose and explain important results.
- Label warnings and noncompliant examples; do not rely on position, color, or icons.

For example, replace "Choose the green option on the right" with
"Choose **Publish**." Use ordinary text for essential instructions, not text
embedded only in a screenshot.

## Keep tables simple

Use a header row and explain what to compare. Prefer lists for instructions or
unrelated facts. Avoid merged cells and complex headers; data beyond Markdown
tables may need semantic HTML and manual accessibility review.

## Review with a keyboard and assistive technology

1. Use Tab and Shift+Tab to reach links and controls in a sensible order.
2. Try the skip link, sidebar disclosure controls, search, and code controls.
3. Check that long code and tables remain readable on a narrow screen and
   with browser zoom increased.
4. Review light and dark appearance; do not use color as the only distinction.
5. Use a screen reader's heading and link lists to check labels and descriptions
   without visual context.

Combine automated checks with manual review. Assess WCAG conformance on the
complete deployed site, including content and host-added authentication or UI.
