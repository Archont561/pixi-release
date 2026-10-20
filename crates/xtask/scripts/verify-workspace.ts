import { join } from "node:path";

const root = process.cwd();
const requiredFiles = [
  "Cargo.toml",
  "pixi.toml",
  "crates/package.json",
  "crates/xtask/Cargo.toml",
  "turbo.json",
  "lefthook.yml",
  ".gitignore",
  ".versionrc",
  "deny.toml",
] as const;

for (const relativePath of requiredFiles) {
  const path = join(root, relativePath);
  if (!(await Bun.file(path).exists())) {
    throw new Error(`workspace contract is missing ${relativePath}`);
  }
}

const packageJson = await Bun.file(join(root, "package.json")).json();
const workspaces = packageJson.workspaces as string[];
for (const workspace of ["crates"]) {
  if (!workspaces.includes(workspace)) {
    throw new Error(`root package.json must declare ${workspace} as a workspace`);
  }
}

// crates/xtask is a Cargo member but deliberately has no package.json: the Turbo facade at
// crates/package.json already drives `cargo --workspace`, which includes it. A second facade
// would be a second command path (decision-1).
const cargoToml = await Bun.file(join(root, "Cargo.toml")).text();
if (!cargoToml.includes('"crates/xtask"')) {
  throw new Error("Cargo.toml must declare crates/xtask as a workspace member");
}

console.log(`workspace contract verified (${process.argv.slice(2).join(" ") || "check"})`);
