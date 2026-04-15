import type { Express } from "express";
import { existsSync } from "node:fs";
import path from "node:path";
import { AigDatabase } from "./db.js";

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
    const conversations = db.getConversations(intent.id);
    const provenance = db.getProvenance(intent.id);

    res.json({ intent, checkpoints, semanticChanges, conversations, provenance });
  });

  app.get("/api/timeline", (_req, res) => {
    res.json(db.getTimeline());
  });

  app.get("/api/graph", (_req, res) => {
    res.json(db.getIntentGraph());
  });
}
