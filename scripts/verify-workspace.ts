import { join } from "node:path";

const root = process.cwd();
const requiredFiles = [
  "Cargo.toml",
  "pixi.toml",
  "crates/package.json",
  "docs/package.json",
  "turbo.json",
  "lefthook.yml",
] as const;

for (const relativePath of requiredFiles) {
  const path = join(root, relativePath);
  if (!(await Bun.file(path).exists())) {
    throw new Error(`workspace contract is missing ${relativePath}`);
  }
}

const packageJson = await Bun.file(join(root, "package.json")).json();
const workspaces = packageJson.workspaces as string[];
for (const workspace of ["crates", "docs"]) {
  if (!workspaces.includes(workspace)) {
    throw new Error(`root package.json must declare ${workspace} as a workspace`);
  }
}

console.log(`workspace contract verified (${process.argv.slice(2).join(" ") || "check"})`);
