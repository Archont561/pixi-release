import { expect, test } from "bun:test";

// This file lives at crates/xtask/scripts/, three levels below the repository root.
const rootUrl = (path: string) => new URL(`../../../${path}`, import.meta.url);
const rootPackage = await Bun.file(rootUrl("package.json")).json();

test("the root package makes Cargo and docs visible to Turbo", () => {
  expect(rootPackage.workspaces).toEqual(["crates", "apps/docs"]);
  expect(rootPackage.scripts["all:test"]).toBe("bun x turbo run test");
});

test("the bun lockfile agrees with the workspace layout", async () => {
  const lock = await Bun.file(rootUrl("bun.lock")).text();
  for (const workspace of rootPackage.workspaces as string[]) {
    expect(lock).toContain(`"${workspace}": {`);
  }
  expect(lock).not.toContain("workspace:docs");
});

test("xtask is a Cargo member and carries no npm facade of its own", async () => {
  const cargoToml = await Bun.file(rootUrl("Cargo.toml")).text();
  expect(cargoToml).toContain('"crates/xtask"');
  expect(await Bun.file(rootUrl("crates/xtask/package.json")).exists()).toBe(false);
});

test("the authored release policy is schema 1 and names verbs, never tools", async () => {
  const policy = await Bun.file(rootUrl(".pixi-release.toml")).text();
  expect(policy).toContain("schema = 1");
  // doc-3 §3: .pixi-release.toml names verbs and targets; it never names a build tool.
  for (const tool of ["turbo", "pytest", "bun x"]) {
    expect(policy.includes(tool)).toBe(false);
  }
});
