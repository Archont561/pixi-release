import { join } from "node:path";

const source = await Bun.file(join(process.cwd(), "index.md")).text();
const title = source.match(/^#\s+(.+)$/m)?.[1] ?? "pixi-release";
const body = source
  .split("\n")
  .filter((line) => !line.startsWith("#"))
  .map((line) => `<p>${escapeHtml(line)}</p>`)
  .join("\n");

const html = `<!doctype html>
<html lang="en">
  <head><meta charset="utf-8"><title>${escapeHtml(title)}</title></head>
  <body><main><h1>${escapeHtml(title)}</h1>${body}</main></body>
</html>\n`;

await Bun.write(join(process.cwd(), "dist/index.html"), html);
console.log("built apps/docs/dist/index.html");

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
