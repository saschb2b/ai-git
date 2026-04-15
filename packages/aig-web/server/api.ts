import type { Express } from "express";
import { existsSync } from "node:fs";
import path from "node:path";
import { AigDatabase } from "./db.js";
import { getCombinedDiff, getCommitDiff } from "./diff.js";

function findAigDir(): string {
  let current = process.cwd();
  while (true) {
    const candidate = path.join(current, ".aig");
    if (existsSync(path.join(candidate, "aig.db"))) {
      return candidate;
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return path.resolve(process.cwd(), ".aig");
}

export function registerApiRoutes(app: Express) {
  // Walk up from cwd to find .aig directory
  const aigDir = process.env.AIG_DIR ?? findAigDir();
  const db = new AigDatabase(aigDir);

  app.get("/api/intents", (_req, res) => {
    res.json(db.listIntents());
  });

  app.get("/api/intents/:id", (req, res) => {
    const intent = db.getIntent(req.params.id);
    if (!intent) {
      res.status(404).json({ error: "Intent not found" });
      return;
    }

    const checkpoints = db.getCheckpoints(intent.id);
    const cpIds = checkpoints.map((c) => c.id);
    const semanticChanges = db.getSemanticChanges(cpIds);
    const changesByCheckpoint = db.getSemanticChangesByCheckpoint(cpIds);
    const conversations = db.getConversations(intent.id);
    const provenance = db.getProvenance(intent.id);
    const session = db.getSessionDuration(intent.id);
    const filesChanged = db.getFilesChanged(intent.id);

    res.json({
      intent,
      checkpoints,
      semanticChanges,
      changesByCheckpoint,
      conversations,
      provenance,
      session,
      filesChanged,
    });
  });

  // Git repo root: walk up from .aig dir
  const repoDir = path.resolve(aigDir, "..");

  app.get("/api/intents/:id/diff", (req, res) => {
    const intent = db.getIntent(req.params.id);
    if (!intent) {
      res.status(404).json({ error: "Intent not found" });
      return;
    }

    const checkpoints = db.getCheckpoints(intent.id);
    if (checkpoints.length === 0) {
      res.json([]);
      return;
    }

    const shas = checkpoints.map((c) => c.git_commit_sha);
    const files = getCombinedDiff(repoDir, shas);
    res.json(files);
  });

  app.get("/api/commits/:sha/diff", (req, res) => {
    const files = getCommitDiff(repoDir, req.params.sha);
    res.json(files);
  });

  app.get("/api/timeline", (_req, res) => {
    res.json(db.getTimeline());
  });

  app.get("/api/graph", (_req, res) => {
    res.json(db.getIntentGraph());
  });
}
