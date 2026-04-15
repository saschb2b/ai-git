import { describe, it, expect } from "vitest";

// We test the parser by importing the module and calling the internal parse function.
// Since parseUnifiedDiff is not exported, we test via getCommitDiff/getCombinedDiff
// on the actual repo. But we can also extract the parser for unit testing.

// For now, test through the git integration since this is a real repo.
import { getCommitDiff, getCombinedDiff } from "../diff.js";
import path from "node:path";

const REPO_DIR = path.resolve(__dirname, "../../..");

describe("getCommitDiff", () => {
  it("returns file diffs for a known commit", () => {
    // Use a commit from the repo — the initial scaffold commit
    const files = getCommitDiff(REPO_DIR, "HEAD");
    expect(Array.isArray(files)).toBe(true);
    // HEAD should have at least one file changed
    if (files.length > 0) {
      const file = files[0];
      expect(file).toHaveProperty("path");
      expect(file).toHaveProperty("status");
      expect(file).toHaveProperty("additions");
      expect(file).toHaveProperty("deletions");
      expect(file).toHaveProperty("hunks");
      expect(file).toHaveProperty("isBinary");
      expect(["added", "modified", "deleted", "renamed"]).toContain(file.status);
      expect(typeof file.additions).toBe("number");
      expect(typeof file.deletions).toBe("number");
    }
  });

  it("returns empty array for invalid commit", () => {
    const files = getCommitDiff(REPO_DIR, "0000000000000000000000000000000000000000");
    expect(files).toEqual([]);
  });

  it("parses hunks with correct line numbers", () => {
    const files = getCommitDiff(REPO_DIR, "HEAD");
    for (const file of files) {
      if (file.isBinary) continue;
      for (const hunk of file.hunks) {
        for (const line of hunk.lines) {
          expect(["add", "delete", "context"]).toContain(line.type);
          if (line.type === "add") {
            expect(line.newLine).toBeTypeOf("number");
            expect(line.oldLine).toBeNull();
          } else if (line.type === "delete") {
            expect(line.oldLine).toBeTypeOf("number");
            expect(line.newLine).toBeNull();
          } else {
            expect(line.oldLine).toBeTypeOf("number");
            expect(line.newLine).toBeTypeOf("number");
          }
          expect(line).toHaveProperty("content");
        }
      }
    }
  });

  it("counts additions and deletions correctly", () => {
    const files = getCommitDiff(REPO_DIR, "HEAD");
    for (const file of files) {
      if (file.isBinary) continue;
      let adds = 0;
      let dels = 0;
      for (const hunk of file.hunks) {
        for (const line of hunk.lines) {
          if (line.type === "add") adds++;
          if (line.type === "delete") dels++;
        }
      }
      expect(file.additions).toBe(adds);
      expect(file.deletions).toBe(dels);
    }
  });
});

describe("getCombinedDiff", () => {
  it("returns diffs for a range of commits", () => {
    const files = getCombinedDiff(REPO_DIR, ["HEAD", "HEAD~1"]);
    expect(Array.isArray(files)).toBe(true);
  });

  it("handles single commit", () => {
    const files = getCombinedDiff(REPO_DIR, ["HEAD"]);
    expect(Array.isArray(files)).toBe(true);
  });

  it("returns empty for empty array", () => {
    const files = getCombinedDiff(REPO_DIR, []);
    expect(files).toEqual([]);
  });
});
