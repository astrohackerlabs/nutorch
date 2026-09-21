/** Local static preview, adapted from tscom/server.ts; never publishes. */
import { resolve } from "node:path";
import { stat } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("./build/client/", import.meta.url));
const args = process.argv.slice(2);
function option(name: string, fallback: string): string {
  const index = args.indexOf(name);
  if (index < 0) return fallback;
  const value = args[index + 1];
  if (!value) throw new Error(`Missing value for ${name}`);
  return value;
}
const hostname = option("--host", "127.0.0.1");
if (!["localhost", "127.0.0.1", "::1"].includes(hostname)) {
  throw new Error("Experiment 8 preview is loopback-only");
}
const port = Number(option("--port", "4398"));
async function file(path: string): Promise<Bun.BunFile | null> {
  const absolute = resolve(root, path.replace(/^\/+/, ""));
  if (absolute !== resolve(root) && !absolute.startsWith(resolve(root) + "/"))
    return null;
  if (!(await stat(absolute).catch(() => null))?.isFile()) return null;
  return Bun.file(absolute);
}
const server = Bun.serve({
  hostname,
  port,
  async fetch(request) {
    if (!["GET", "HEAD"].includes(request.method))
      return new Response(null, { status: 405 });
    let path: string;
    try {
      path = decodeURIComponent(new URL(request.url).pathname);
    } catch {
      return new Response(null, { status: 400 });
    }
    for (const candidate of [path, `${path.replace(/\/$/, "")}/index.html`]) {
      const asset = await file(candidate);
      if (asset)
        return new Response(request.method === "HEAD" ? null : asset, {
          headers: { "Content-Type": asset.type, "Cache-Control": "no-store" },
        });
    }
    const missing = await file("404/index.html");
    return new Response(request.method === "HEAD" ? null : missing, {
      status: 404,
      headers: { "Content-Type": "text/html", "Cache-Control": "no-store" },
    });
  },
});
console.log(`nutorch-com preview: http://${hostname}:${String(server.port)}`);
