import { expect, test } from "bun:test";

// This file lives at crates/xtask/scripts/, three levels below the repository root.
const rootUrl = (path: string) => new URL(`../../../${path}`, import.meta.url);
const rootPackage = await Bun.file(rootUrl("package.json")).json();

test("the root package makes Cargo and docs visible to Turbo", () => {
  expect(rootPackage.workspaces).toEqual(["crates"]);
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

