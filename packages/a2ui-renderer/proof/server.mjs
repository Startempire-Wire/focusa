import { createReadStream, statSync } from "node:fs";
import { createServer, request } from "node:http";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../proof-dist/", import.meta.url));
const api = new URL(process.env.FOCUSA_API_ORIGIN ?? "http://127.0.0.1:8789");
const port = Number(process.env.PORT ?? 4173);
const mime = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".json": "application/json" };

createServer((incoming, outgoing) => {
  const url = new URL(incoming.url ?? "/", `http://${incoming.headers.host ?? "localhost"}`);
  if (url.pathname.startsWith("/v1/")) {
    if (incoming.method === "OPTIONS" && url.pathname === "/v1/events/stream") {
      outgoing.writeHead(204, {
        "access-control-allow-origin": "*",
        "access-control-allow-methods": "GET, OPTIONS",
        "access-control-allow-headers": "bypass-tunnel-reminder",
        "access-control-max-age": "86400",
      });
      outgoing.end();
      return;
    }
    const upstream = request(new URL(url.pathname + url.search, api), {
      method: incoming.method,
      headers: { ...incoming.headers, host: api.host },
    }, (response) => {
      const headers = { ...response.headers };
      if (String(headers["content-type"] ?? "").startsWith("text/event-stream")) {
        headers["cache-control"] = "no-cache, no-transform";
        headers["x-accel-buffering"] = "no";
        headers["content-encoding"] = "identity";
        headers["access-control-allow-origin"] = "*";
        delete headers["content-length"];
      }
      outgoing.writeHead(response.statusCode ?? 502, headers);
      outgoing.flushHeaders();
      response.pipe(outgoing);
    });
    upstream.on("error", (error) => {
      outgoing.writeHead(502, { "content-type": "application/json" });
      outgoing.end(JSON.stringify({ error: "focusa_api_unavailable", message: error.message }));
    });
    incoming.pipe(upstream);
    return;
  }

  const pathname = url.pathname === "/" ? "/index.html" : url.pathname;
  const safe = normalize(pathname).replace(/^(\.\.[/\\])+/, "").replace(/^[/\\]+/, "");
  const path = join(root, safe);
  try {
    if (!statSync(path).isFile()) throw new Error("not a file");
    outgoing.writeHead(200, { "content-type": mime[extname(path)] ?? "application/octet-stream" });
    createReadStream(path).pipe(outgoing);
  } catch {
    outgoing.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    outgoing.end("not found");
  }
}).listen(port, "127.0.0.1", () => {
  console.log(`Focusa proof server http://127.0.0.1:${port} -> ${api.origin}`);
});
