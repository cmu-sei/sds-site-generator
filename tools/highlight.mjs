import { createHighlighterCore } from "shiki/core";
import { createJavaScriptRegexEngine } from "shiki/engine/javascript";
import c from "shiki/langs/c.mjs";
import cpp from "shiki/langs/cpp.mjs";
import java from "shiki/langs/java.mjs";
import perl from "shiki/langs/perl.mjs";
import fortran from "shiki/langs/fortran-free-form.mjs";
import javascript from "shiki/langs/javascript.mjs";
import typescript from "shiki/langs/typescript.mjs";
import python from "shiki/langs/python.mjs";
import shell from "shiki/langs/shellscript.mjs";
import html from "shiki/langs/html.mjs";
import xml from "shiki/langs/xml.mjs";
import json from "shiki/langs/json.mjs";
import yaml from "shiki/langs/yaml.mjs";
import diff from "shiki/langs/diff.mjs";
import light from "shiki/themes/github-light-high-contrast.mjs";
import dark from "shiki/themes/github-dark-high-contrast.mjs";

const highlighter = await createHighlighterCore({
  themes: [light, dark],
  langs: [c, cpp, java, perl, fortran, javascript, typescript, python, shell, html, xml, json, yaml, diff],
  engine: createJavaScriptRegexEngine({ forgiving: true }),
});

const aliases = { fortran: "fortran-free-form", f: "fortran-free-form", "c++": "cpp", sh: "shellscript", bash: "shellscript", js: "javascript", ts: "typescript" };
for (const code of document.querySelectorAll("pre > code")) {
  const language = [...code.classList].find((name) => name.startsWith("language-"))?.slice(9);
  const lang = aliases[language] || language;
  if (!lang || !highlighter.getLoadedLanguages().includes(lang)) continue;
  const rendered = highlighter.codeToHtml(code.textContent, {
    lang,
    themes: { light: "github-light-high-contrast", dark: "github-dark-high-contrast" },
    defaultColor: false,
  });
  const parsed = new DOMParser().parseFromString(rendered, "text/html");
  code.replaceChildren(...parsed.querySelector("code").childNodes);
  code.dataset.highlighted = "true";
}