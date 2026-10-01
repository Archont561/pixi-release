import { expect, test } from "bun:test";

const rootPackage = await Bun.file(new URL("../package.json", import.meta.url)).json();

test("the root package makes Cargo and docs visible to Turbo", () => {
  expect(rootPackage.workspaces).toEqual(["crates", "docs"]);
  expect(rootPackage.scripts["all:test"]).toBe("bun x turbo run test");
});
