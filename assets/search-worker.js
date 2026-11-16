const cache = new Map();
const root = new URL("./search/", import.meta.url);

async function load(name) {
  if (!cache.has(name)) {
    const request = fetch(new URL(name, root)).then(async (response) => {
      if (!response.ok) throw new Error(`Could not load search data (${response.status}).`);
      return response.json();
    }).catch((error) => {
      cache.delete(name);
      throw error;
    });
    cache.set(name, request);
  }
  return cache.get(name);
}

export function tokenize(value) {
  return (value.match(/[\p{L}\p{N}]+/gu) || []).map((term) => term.toLowerCase());
}

export function parseQuery(query) {
  const groups = [];
  for (const match of query.matchAll(/"([^"]+)"|([^"\s]+)/g)) {
    const terms = tokenize(match[1] || match[2]);
    if (terms.length) groups.push({ terms, phrase: Boolean(match[1]) });
  }
  return groups;
}

export async function search(query) {
  const groups = parseQuery(query);
  if (!groups.length) return { results: [], total: 0 };
  const manifest = await load("manifest.json");
  if (manifest.version !== 1) throw new Error("Search data is out of date. Rebuild this website.");
  const allowed = new Set(manifest.buckets);
  const terms = groups.flatMap((group) => group.terms);
  const buckets = await Promise.all([...new Set(terms.map((term) => term.codePointAt(0).toString(16)))]
    .map(async (key) => [key, allowed.has(key) ? await load(`terms-${key}.json`) : {}]));
  const dictionary = Object.assign(Object.create(null), ...buckets.map(([, bucket]) => bucket));
  let candidates = null;
  for (const [groupIndex, group] of groups.entries()) {
    let matches = null;
    for (const [termIndex, term] of group.terms.entries()) {
      const prefix = !group.phrase && groupIndex === groups.length - 1 && termIndex === group.terms.length - 1;
      const postings = new Map();
      const matchingTerms = prefix ? Object.keys(dictionary).filter((key) => key.startsWith(term)) : [term];
      for (const key of matchingTerms) {
        for (const [id, positions] of Object.entries(dictionary[key] || {})) {
          postings.set(id, [...(postings.get(id) || []), ...positions]);
        }
      }
      if (matches === null) {
        matches = new Map([...postings].map(([id, positions]) => [id, { starts: positions, score: positions.length }]));
      } else {
        for (const [id, match] of matches) {
          const positions = postings.get(id);
          if (!positions) { matches.delete(id); continue; }
          if (group.phrase) {
            const valid = new Set(positions);
            match.starts = match.starts.filter((position) => valid.has(position + termIndex));
            if (!match.starts.length) { matches.delete(id); continue; }
          }
          match.score += positions.length;
        }
      }
    }
    if (candidates === null) {
      candidates = matches;
    } else {
      for (const [id, match] of candidates) {
        if (!matches.has(id)) candidates.delete(id);
        else match.score += matches.get(id).score;
      }
    }
  }
  const scored = [...(candidates || [])];
  const chunks = new Map(await Promise.all([...new Set(scored.map(([id]) => Math.floor(Number(id) / manifest.chunkSize)))]
    .map(async (chunk) => [chunk, await load(`docs-${chunk}.json`)])));
  const results = scored.map(([id, match]) => {
    const numeric = Number(id);
    const doc = chunks.get(Math.floor(numeric / manifest.chunkSize))[numeric % manifest.chunkSize];
    const title = new Set(tokenize(doc.title));
    const page = new Set(tokenize(doc.page));
    const score = Math.min(match.score, 20) + terms.reduce((sum, term, index) => {
      const prefix = index === terms.length - 1 && !groups.at(-1).phrase;
      const matches = (words) => prefix ? [...words].some((word) => word.startsWith(term)) : words.has(term);
      return sum + (matches(title) ? 120 : 0) + (matches(page) ? 40 : 0);
    }, 0);
    return { ...doc, score };
  }).sort((a, b) => b.score - a.score || a.url.localeCompare(b.url));
  return { results: results.slice(0, 30), total: results.length };
}

if (typeof self !== "undefined" && typeof self.postMessage === "function") {
  self.onmessage = async ({ data: { id, query } }) => {
    try {
      self.postMessage({ id, ...await search(query) });
    } catch (error) {
      self.postMessage({ id, error: error instanceof Error ? error.message : "Search failed." });
    }
  };
}
