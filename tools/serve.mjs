import { createServer } from "node:http";
import { once } from "node:events";
import { readFile, stat } from "node:fs/promises";
import { extname, join, resolve, sep } from "node:path";

const mime = {
  ".html": "text/html", ".js": "text/javascript", ".json": "application/json",
  ".css": "text/css", ".svg": "image/svg+xml", ".png": "image/png", ".jpg": "image/jpeg",
  ".gif": "image/gif", ".ico": "image/x-icon", ".wasm": "application/wasm", ".woff2": "font/woff2",
};

export async function serveStatic(directory, prefix, port = 0) {
  const root = resolve(directory);
  const server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url, "http://localhost");
      if (!url.pathname.startsWith(prefix)) {
        response.writeHead(404).end("Outside site mount");
        return;
      }
      let filename = resolve(root, decodeURIComponent(url.pathname.slice(prefix.length)));
      if (filename !== root && !filename.startsWith(root + sep)) {
        response.writeHead(403).end("Outside site directory");
        return;
      }
      if ((await stat(filename)).isDirectory()) {
        if (!url.pathname.endsWith("/")) {
          response.writeHead(301, { location: url.pathname + "/" + url.search }).end();
          return;
        }
        filename = join(filename, "index.html");
      }
      const acceptsGzip = request.headers["accept-encoding"]?.split(",").some(encoding => {
        const [name, ...parameters] = encoding.trim().split(";");
        return name.trim().toLowerCase() === "gzip" && parameters.every(parameter => {
          const value = parameter.trim();
          return !value.startsWith("q=") || (Number(value.slice(2)) > 0 && Number(value.slice(2)) <= 1);
        });
      });
      let data;
      let contentEncoding;
      if (acceptsGzip) {
        try {
          data = await readFile(`${filename}.gz`);
          contentEncoding = "gzip";
        } catch (error) {
          if (error.code !== "ENOENT") throw error;
        }
      }
      data ??= await readFile(filename);
      response.writeHead(200, {
        "content-type": mime[extname(filename)] || "application/octet-stream",
        "cache-control": "no-store",
        "vary": "Accept-Encoding",
        ...(contentEncoding ? { "content-encoding": contentEncoding } : {}),
      }).end(data);
    } catch (error) {
      if (error.code === "ENOENT" || error.code === "ENOTDIR") {
        response.writeHead(404).end("Not found");
      } else {
        console.error(error);
        response.writeHead(500).end(String(error));
      }
    }
  });
  server.listen(port, "127.0.0.1");
  await once(server, "listening");
  return { server, url: `http://127.0.0.1:${server.address().port}${prefix}` };
}
