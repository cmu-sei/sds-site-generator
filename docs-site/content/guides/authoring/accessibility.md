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

Explain complex charts in nearby prose or a table, not long alt text.
Use the [decorative marker](assets.md#add-an-image) only for decorative images.

## Make the reading order clear

- Use [heading levels](markdown.md#a-complete-page), not bold text as headings.
- Write descriptive headings that make sense in a table of contents.
- Use numbered lists for ordered tasks and bullets for unordered choices.
- Introduce code examples with their purpose and explain important results.
- Label warnings and noncompliant examples; do not rely on position, color, or icons.

Write "Choose **Publish**," not "Choose the green option on the right."
Do not put essential instructions only in screenshots.

## Keep tables simple

Use a header row; avoid merged cells and complex headers. Prefer lists for
instructions or unrelated facts.

## Review with a keyboard and assistive technology

1. Use Tab and Shift+Tab to reach links and controls in a sensible order.
2. Try the skip link, sidebar disclosure controls, search, and code controls.
3. Check that long code and tables remain readable on a narrow screen and
   with browser zoom increased.
4. Review light and dark appearance; do not use color as the only distinction.
5. Use a screen reader's heading and link lists to check labels and descriptions
   without visual context.

Combine automated checks with manual review of the complete deployed site,
including host-added authentication or UI.
