import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { AigDatabase } from "../db.js";
import path from "node:path";
import { existsSync } from "node:fs";

// Find the .aig directory (same logic as the server)
function findAigDir(): string {
  let current = path.resolve(__dirname, "../../..");
  while (true) {
    const candidate = path.join(current, ".aig");
    if (existsSync(path.join(candidate, "aig.db"))) {
      return candidate;
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  throw new Error("No .aig directory found");
}

describe("AigDatabase", () => {
  let db: AigDatabase;

  beforeAll(() => {
    db = new AigDatabase(findAigDir());
  });

  afterAll(() => {
    db.close();
  });

  describe("listIntents", () => {
    it("returns an array of intents", () => {
      const intents = db.listIntents();
      expect(Array.isArray(intents)).toBe(true);
      expect(intents.length).toBeGreaterThan(0);
    });

    it("each intent has required fields", () => {
      const intents = db.listIntents();
      for (const intent of intents) {
        expect(intent).toHaveProperty("id");
        expect(intent).toHaveProperty("description");
        expect(intent).toHaveProperty("created_at");
        expect(intent).toHaveProperty("checkpoint_count");
        expect(typeof intent.id).toBe("string");
        expect(typeof intent.description).toBe("string");
        expect(typeof intent.checkpoint_count).toBe("number");
      }
    });

    it("returns intents sorted by created_at DESC", () => {
      const intents = db.listIntents();
      for (let i = 1; i < intents.length; i++) {
        expect(new Date(intents[i - 1].created_at).getTime())
          .toBeGreaterThanOrEqual(new Date(intents[i].created_at).getTime());
      }
    });
  });

  describe("getIntent", () => {
    it("returns an intent by id", () => {
      const intents = db.listIntents();
      const intent = db.getIntent(intents[0].id);
      expect(intent).toBeDefined();
      expect(intent!.id).toBe(intents[0].id);
    });

    it("returns undefined for unknown id", () => {
      const intent = db.getIntent("nonexistent-id");
      expect(intent).toBeUndefined();
    });
  });

  describe("getCheckpoints", () => {
    it("returns checkpoints for an intent", () => {
      const intents = db.listIntents();
      const withCheckpoints = intents.find((i) => i.checkpoint_count > 0);
      expect(withCheckpoints).toBeDefined();

      const cps = db.getCheckpoints(withCheckpoints!.id);
      expect(cps.length).toBe(withCheckpoints!.checkpoint_count);
    });

    it("each checkpoint has required fields", () => {
      const intents = db.listIntents();
      const withCps = intents.find((i) => i.checkpoint_count > 0);
      const cps = db.getCheckpoints(withCps!.id);
      for (const cp of cps) {
        expect(cp).toHaveProperty("id");
        expect(cp).toHaveProperty("message");
        expect(cp).toHaveProperty("git_commit_sha");
        expect(cp).toHaveProperty("created_at");
      }
    });

    it("returns checkpoints sorted by created_at DESC", () => {
      const intents = db.listIntents();
      const withCps = intents.find((i) => i.checkpoint_count > 1);
      if (!withCps) return; // skip if no intent has 2+ checkpoints
      const cps = db.getCheckpoints(withCps.id);
      for (let i = 1; i < cps.length; i++) {
        expect(new Date(cps[i - 1].created_at).getTime())
          .toBeGreaterThanOrEqual(new Date(cps[i].created_at).getTime());
      }
    });

    it("returns empty array for unknown intent", () => {
      const cps = db.getCheckpoints("nonexistent");
      expect(cps).toEqual([]);
    });
  });

  describe("getTimeline", () => {
    it("returns timeline data points", () => {
      const timeline = db.getTimeline();
      expect(Array.isArray(timeline)).toBe(true);
      expect(timeline.length).toBeGreaterThan(0);
    });

    it("each point has required fields", () => {
      const timeline = db.getTimeline();
      for (const point of timeline) {
        expect(point).toHaveProperty("date");
        expect(point).toHaveProperty("checkpoints");
        expect(point).toHaveProperty("intents_opened");
        expect(point).toHaveProperty("intents_closed");
      }
    });
  });

  describe("getIntentGraph", () => {
    it("returns nodes and edges", () => {
      const graph = db.getIntentGraph();
      expect(graph).toHaveProperty("nodes");
      expect(graph).toHaveProperty("edges");
      expect(Array.isArray(graph.nodes)).toBe(true);
      expect(Array.isArray(graph.edges)).toBe(true);
      expect(graph.nodes.length).toBeGreaterThan(0);
    });

    it("each node has required fields", () => {
      const { nodes } = db.getIntentGraph();
      for (const node of nodes) {
        expect(node).toHaveProperty("id");
        expect(node).toHaveProperty("description");
        expect(node).toHaveProperty("status");
        expect(["active", "closed"]).toContain(node.status);
      }
    });
  });

  describe("getSemanticChangesByCheckpoint", () => {
    it("returns grouped changes", () => {
      const intents = db.listIntents();
      const withCps = intents.find((i) => i.checkpoint_count > 0);
      const cps = db.getCheckpoints(withCps!.id);
      const cpIds = cps.map((c) => c.id);
      const grouped = db.getSemanticChangesByCheckpoint(cpIds);
      expect(typeof grouped).toBe("object");
    });

    it("returns empty object for empty input", () => {
      const grouped = db.getSemanticChangesByCheckpoint([]);
      expect(grouped).toEqual({});
    });
  });

  describe("getFilesChanged", () => {
    it("returns a number", () => {
      const intents = db.listIntents();
      const count = db.getFilesChanged(intents[0].id);
      expect(typeof count).toBe("number");
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });
});
