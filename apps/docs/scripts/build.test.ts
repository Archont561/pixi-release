import { expect, test } from "bun:test";

const documentation = await Bun.file(new URL("../index.md", import.meta.url)).text();

test("documents the safe release ordering", () => {
  expect(documentation).toContain("Verify source and artifacts");
  expect(documentation).toContain("resume safely");
});
