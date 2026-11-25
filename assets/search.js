function setupSearch(form) {
  const prefix = form.dataset.searchPrefix;
  const input = form.querySelector('[name="q"]');
  const status = document.getElementById(`${prefix}-status`);
  const results = document.getElementById(`${prefix}-results`);
  const syncUrl = form.dataset.syncUrl === "true";
  let worker;
  let requestId = 0;
  let timer;
  let active = -1;

  function message(text, state = "info") {
    status.replaceChildren();
    status.classList.toggle("sds-callout", state === "info" || state === "error");
    status.classList.toggle("sds-empty-state", state === "empty");
    status.classList.toggle("sds-stack", state === "loading");
    status.classList.toggle("sds-search-loading", state === "loading");
    status.classList.toggle("sds-sr-only", state === "results");
    status.dataset.sdsTone = state === "error" ? "danger" : "info";
    if (state === "loading") {
      const spinner = document.createElement("span");
      spinner.className = "sds-spinner";
      spinner.dataset.sdsSize = "lg";
      spinner.setAttribute("aria-hidden", "true");
      const label = document.createElement("span");
      label.textContent = text;
      status.append(spinner, label);
    } else if (state === "empty") {
      const heading = document.createElement("h2");
      heading.className = "sds-text-h3";
      heading.textContent = text;
      const description = document.createElement("p");
      description.textContent = "Try a different query.";
      status.append(heading, description);
    } else {
      status.textContent = text;
    }
  }

  function select(index, focus = false) {
    const links = [...results.querySelectorAll("a")];
    active = Math.max(-1, Math.min(index, links.length - 1));
    links.forEach((link, index) => {
      if (index === active) link.setAttribute("aria-current", "true");
      else link.removeAttribute("aria-current");
    });
    if (focus && active >= 0) {
      links[active].focus();
      links[active].scrollIntoView({ block: "nearest" });
    } else if (focus) input.focus();
  }

  function failure(text) {
    results.replaceChildren();
    results.removeAttribute("aria-busy");
    select(-1);
    message(`${text} Please try again or browse using the navigation.`, "error");
  }

  function render(data) {
    if (data.id !== requestId) return;
    results.removeAttribute("aria-busy");
    if (data.error) { failure(data.error); return; }
    if (!data.total) {
      results.replaceChildren();
      select(-1);
      message("No results found.", "empty");
      return;
    }
    const list = document.createElement("ul");
    list.className = "sds-list";
    list.append(...data.results.map((result, index) => {
      const item = document.createElement("li");
      item.className = "sds-list-item";
      const link = document.createElement("a");
      link.href = result.url;
      const title = document.createElement("span");
      title.textContent = result.title;
      const context = document.createElement("small");
      context.textContent = result.title === result.page ? "Documentation" : result.page;
      link.append(title, context);
      link.addEventListener("focus", () => select(index));
      item.append(link);
      return item;
    }));
    results.replaceChildren(list);
    select(data.results.length ? 0 : -1);
    message(`${data.total} matching sections.${data.total > 30 ? " Showing the first 30. Add more words to narrow your search." : ""} Use the arrow keys to navigate and Enter to open a result.`, "results");
  }

  function run() {
    clearTimeout(timer);
    const query = input.value.trim().slice(0, 200);
    requestId += 1;
    if (syncUrl) {
      const address = new URL(location.href);
      if (query) address.searchParams.set("q", query);
      else address.searchParams.delete("q");
      history.replaceState(null, "", address);
    }
    if (!query) {
      results.replaceChildren();
      results.removeAttribute("aria-busy");
      select(-1);
      message("Type a query in the search box above to display a list of results.");
      return;
    }
    message("Searching…", "loading");
    results.setAttribute("aria-busy", "true");
    try {
      if (!worker) {
        worker = new Worker(new URL("./search-worker.js", import.meta.url), { type: "module" });
        worker.onmessage = ({ data }) => render(data);
        worker.onerror = () => {
          worker.terminate();
          worker = undefined;
          failure("The search worker could not start.");
        };
      }
      worker.postMessage({ id: requestId, query });
    } catch (error) {
      failure(error instanceof Error ? error.message : "Search could not start.");
    }
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    const selected = results.querySelectorAll("a")[active];
    if (selected && !results.hasAttribute("aria-busy")) selected.click();
    else run();
  });
  input.addEventListener("input", () => {
    requestId += 1;
    clearTimeout(timer);
    results.replaceChildren();
    select(-1);
    timer = setTimeout(run, 150);
  });
  input.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" && results.querySelector("a")) {
      event.preventDefault();
      select(0, true);
    }
  });
  results.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      select(active + (event.key === "ArrowDown" ? 1 : -1), true);
    }
  });
  message(status.textContent);
  if (syncUrl) {
    input.value = new URL(location.href).searchParams.get("q")?.slice(0, 200) || "";
    if (input.value) run();
  }
}

document.querySelectorAll("[data-site-search]").forEach(setupSearch);

for (const tooltip of document.querySelectorAll(".sds-app-masthead sds-tooltip")) {
  const trigger = tooltip.firstElementChild;
  const content = tooltip.lastElementChild;
  const update = () => {
    const label = trigger.getAttribute("aria-label");
    if (content.textContent !== label) content.textContent = label;
    trigger.removeAttribute("title");
  };
  // The SDS theme action rewrites native titles as well as accessible labels.
  new MutationObserver(update).observe(trigger, {
    attributes: true,
    attributeFilter: ["aria-label", "title"],
  });
  update();
}

document.addEventListener("keydown", (event) => {
  if (event.altKey || event.shiftKey || !(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== "k") return;
  const opener = document.querySelector("[data-search-open]");
  if (!opener) return;
  event.preventDefault();
  opener.click();
  document.querySelector('#site-search-dialog [name="q"]')?.focus();
});
