import { execSync } from "node:child_process";
import path from "node:path";

export interface DiffLine {
  type: "add" | "delete" | "context";
  content: string;
  oldLine: number | null;
  newLine: number | null;
}

export interface DiffHunk {
  header: string;
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  lines: DiffLine[];
}

export interface FileDiff {
  path: string;
  status: "added" | "modified" | "deleted" | "renamed";
  additions: number;
  deletions: number;
  hunks: DiffHunk[];
  isBinary: boolean;
}

/**
 * Get the diff for a single git commit.
 */
export function getCommitDiff(
  repoDir: string,
  commitSha: string,
): FileDiff[] {
  try {
    const raw = execSync(
      `git show ${commitSha} --format="" --patch --unified=4 --no-color`,
      {
        cwd: repoDir,
        encoding: "utf-8",
        maxBuffer: 10 * 1024 * 1024,
        timeout: 10000,
      },
    );
    return parseUnifiedDiff(raw);
  } catch {
    return [];
  }
}

/**
 * Get a combined diff across multiple commits (first parent of earliest → latest).
 */
export function getCombinedDiff(
  repoDir: string,
  commitShas: string[],
): FileDiff[] {
  if (commitShas.length === 0) return [];

  const latest = commitShas[0]; // newest first
  const earliest = commitShas[commitShas.length - 1];

  try {
    // Get parent of earliest commit
    const parent = execSync(`git rev-parse ${earliest}^`, {
      cwd: repoDir,
      encoding: "utf-8",
      timeout: 5000,
    }).trim();

    const raw = execSync(
      `git diff ${parent}..${latest} --unified=4 --no-color`,
      {
        cwd: repoDir,
        encoding: "utf-8",
        maxBuffer: 10 * 1024 * 1024,
        timeout: 10000,
      },
    );
    return parseUnifiedDiff(raw);
  } catch {
    // Fallback: if earliest has no parent (root commit), diff against empty tree
    try {
      const raw = execSync(
        `git diff 4b825dc642cb6eb9a060e54bf899d15363d4e393..${latest} --unified=4 --no-color`,
        {
          cwd: repoDir,
          encoding: "utf-8",
          maxBuffer: 10 * 1024 * 1024,
          timeout: 10000,
        },
      );
      return parseUnifiedDiff(raw);
    } catch {
      return [];
    }
  }
}

/**
 * Parse unified diff output into structured FileDiff objects.
 */
function parseUnifiedDiff(raw: string): FileDiff[] {
  const files: FileDiff[] = [];
  const fileChunks = raw.split(/^diff --git /m).filter(Boolean);

  for (const chunk of fileChunks) {
    const lines = chunk.split("\n");

    // Extract file path from the first line: "a/path b/path"
    const headerMatch = lines[0]?.match(/a\/(.+?) b\/(.+)/);
    if (!headerMatch) continue;

    const filePath = headerMatch[2];

    // Determine status
    const isNew = lines.some((l) => l.startsWith("new file mode"));
    const isDeleted = lines.some((l) => l.startsWith("deleted file mode"));
    const isRenamed = lines.some((l) => l.startsWith("rename from"));
    const isBinary = lines.some((l) => l.includes("Binary files"));

    let status: FileDiff["status"] = "modified";
    if (isNew) status = "added";
    else if (isDeleted) status = "deleted";
    else if (isRenamed) status = "renamed";

    if (isBinary) {
      files.push({
        path: filePath,
        status,
        additions: 0,
        deletions: 0,
        hunks: [],
        isBinary: true,
      });
      continue;
    }

    // Parse hunks
    const hunks: DiffHunk[] = [];
    let currentHunk: DiffHunk | null = null;
    let oldLine = 0;
    let newLine = 0;

    for (const line of lines) {
      const hunkMatch = line.match(
        /^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(.*)/,
      );

      if (hunkMatch) {
        if (currentHunk) hunks.push(currentHunk);

        const oldStart = parseInt(hunkMatch[1], 10);
        const oldCount = parseInt(hunkMatch[2] ?? "1", 10);
        const newStart = parseInt(hunkMatch[3], 10);
        const newCount = parseInt(hunkMatch[4] ?? "1", 10);

        oldLine = oldStart;
        newLine = newStart;

        currentHunk = {
          header: hunkMatch[5]?.trim() || "",
          oldStart,
          oldCount,
          newStart,
          newCount,
          lines: [],
        };
        continue;
      }

      if (!currentHunk) continue;

      if (line.startsWith("+")) {
        currentHunk.lines.push({
          type: "add",
          content: line.slice(1),
          oldLine: null,
          newLine: newLine++,
        });
      } else if (line.startsWith("-")) {
        currentHunk.lines.push({
          type: "delete",
          content: line.slice(1),
          oldLine: oldLine++,
          newLine: null,
        });
      } else if (line.startsWith(" ")) {
        currentHunk.lines.push({
          type: "context",
          content: line.slice(1),
          oldLine: oldLine++,
          newLine: newLine++,
        });
      } else if (line === "\\ No newline at end of file") {
        // skip
      }
    }

    if (currentHunk) hunks.push(currentHunk);

    let additions = 0;
    let deletions = 0;
    for (const hunk of hunks) {
      for (const l of hunk.lines) {
        if (l.type === "add") additions++;
        else if (l.type === "delete") deletions++;
      }
    }

    files.push({
      path: filePath,
      status,
      additions,
      deletions,
      hunks,
      isBinary: false,
    });
  }

  return files;
}
