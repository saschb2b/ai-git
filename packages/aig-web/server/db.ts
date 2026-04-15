// Database types and queries for the aig web UI.
// Derived from packages/aig-tui/src/db.ts — keep in sync.

import Database from "better-sqlite3";
import path from "node:path";

export interface Intent {
  id: string;
  description: string;
  parent_id: string | null;
  created_at: string;
  closed_at: string | null;
  summary: string | null;
}

export interface Checkpoint {
  id: string;
  message: string;
  git_commit_sha: string;
  created_at: string;
}

export interface SemanticChange {
  file_path: string;
  change_type: string;
  symbol_name: string;
}

export interface ProvenanceEntry {
  file_path: string;
  origin: string;
  reviewed: number;
  start_line: number;
  end_line: number;
}

export interface Session {
  id: string;
  intent_id: string;
  started_at: string;
  ended_at: string | null;
}

export interface Conversation {
  id: string;
  session_id: string;
  message: string;
  created_at: string;
}

export interface IntentListItem extends Intent {
  checkpoint_count: number;
}

export interface TimelinePoint {
  date: string;
  checkpoints: number;
  intents_opened: number;
  intents_closed: number;
}

export interface IntentGraphNode {
  id: string;
  description: string;
  status: "active" | "closed";
}

export interface IntentGraphEdge {
  source: string;
  target: string;
}

export class AigDatabase {
  private db: Database.Database;

  constructor(aigDir?: string) {
    const dir = aigDir ?? path.resolve(process.cwd(), ".aig");
    this.db = new Database(path.join(dir, "aig.db"), { readonly: true });
  }

  listIntents(): IntentListItem[] {
    return this.db
      .prepare(
        `SELECT i.id, i.description, i.parent_id, i.created_at, i.closed_at, i.summary,
                COUNT(c.id) as checkpoint_count
         FROM intents i
         LEFT JOIN checkpoints c ON c.intent_id = i.id
         GROUP BY i.id
         ORDER BY i.created_at DESC`,
      )
      .all() as IntentListItem[];
  }

  getIntent(id: string): Intent | undefined {
    return this.db
      .prepare(
        "SELECT id, description, parent_id, created_at, closed_at, summary FROM intents WHERE id = ?",
      )
      .get(id) as Intent | undefined;
  }

  getCheckpoints(intentId: string): Checkpoint[] {
    return this.db
      .prepare(
        "SELECT id, message, git_commit_sha, created_at FROM checkpoints WHERE intent_id = ? ORDER BY created_at DESC",
      )
      .all(intentId) as Checkpoint[];
  }

  getSemanticChanges(checkpointIds: string[]): SemanticChange[] {
    if (checkpointIds.length === 0) return [];
    const placeholders = checkpointIds.map(() => "?").join(",");
    return this.db
      .prepare(
        `SELECT sc.file_path, sc.change_type, sc.symbol_name
         FROM semantic_changes sc
         JOIN checkpoints c ON sc.checkpoint_id = c.id
         WHERE sc.checkpoint_id IN (${placeholders})
         ORDER BY c.created_at DESC, sc.id`,
      )
      .all(checkpointIds) as SemanticChange[];
  }

  getSemanticChangesByCheckpoint(
    checkpointIds: string[],
  ): Record<string, SemanticChange[]> {
    if (checkpointIds.length === 0) return {};
    const placeholders = checkpointIds.map(() => "?").join(",");
    const rows = this.db
      .prepare(
        `SELECT sc.checkpoint_id, sc.file_path, sc.change_type, sc.symbol_name
         FROM semantic_changes sc
         WHERE sc.checkpoint_id IN (${placeholders})
         ORDER BY sc.id`,
      )
      .all(checkpointIds) as (SemanticChange & { checkpoint_id: string })[];

    const grouped: Record<string, SemanticChange[]> = {};
    for (const row of rows) {
      if (!grouped[row.checkpoint_id]) grouped[row.checkpoint_id] = [];
      grouped[row.checkpoint_id].push({
        file_path: row.file_path,
        change_type: row.change_type,
        symbol_name: row.symbol_name,
      });
    }
    return grouped;
  }

  getSessionDuration(intentId: string): { started_at: string; ended_at: string | null } | null {
    return this.db
      .prepare(
        "SELECT started_at, ended_at FROM sessions WHERE intent_id = ? ORDER BY started_at LIMIT 1",
      )
      .get(intentId) as { started_at: string; ended_at: string | null } | null;
  }

  getFilesChanged(intentId: string): number {
    const row = this.db
      .prepare(
        `SELECT COUNT(DISTINCT sc.file_path) as count
         FROM semantic_changes sc
         JOIN checkpoints c ON sc.checkpoint_id = c.id
         WHERE c.intent_id = ?`,
      )
      .get(intentId) as { count: number };
    return row?.count ?? 0;
  }

  getConversations(intentId: string): Conversation[] {
    return this.db
      .prepare(
        `SELECT c.id, c.session_id, c.message, c.created_at
         FROM conversations c
         JOIN sessions s ON c.session_id = s.id
         WHERE s.intent_id = ?
         ORDER BY c.created_at DESC`,
      )
      .all(intentId) as Conversation[];
  }

  getProvenance(intentId: string): ProvenanceEntry[] {
    return this.db
      .prepare(
        `SELECT p.file_path, p.origin, p.reviewed, p.start_line, p.end_line
         FROM provenance p
         JOIN checkpoints c ON p.checkpoint_id = c.id
         WHERE c.intent_id = ?
         ORDER BY p.file_path, p.start_line`,
      )
      .all(intentId) as ProvenanceEntry[];
  }

  getTimeline(): TimelinePoint[] {
    const rows = this.db
      .prepare(
        `SELECT date, SUM(checkpoints) as checkpoints, SUM(intents_opened) as intents_opened, SUM(intents_closed) as intents_closed
         FROM (
           SELECT date(created_at) as date, COUNT(*) as checkpoints, 0 as intents_opened, 0 as intents_closed
           FROM checkpoints GROUP BY date(created_at)
           UNION ALL
           SELECT date(created_at), 0, COUNT(*), 0 FROM intents GROUP BY date(created_at)
           UNION ALL
           SELECT date(closed_at), 0, 0, COUNT(*) FROM intents WHERE closed_at IS NOT NULL GROUP BY date(closed_at)
         )
         GROUP BY date
         ORDER BY date`,
      )
      .all() as TimelinePoint[];
    return rows;
  }

  getIntentGraph(): { nodes: IntentGraphNode[]; edges: IntentGraphEdge[] } {
    const nodes = this.db
      .prepare(
        `SELECT id, description,
                CASE WHEN closed_at IS NULL THEN 'active' ELSE 'closed' END as status
         FROM intents`,
      )
      .all() as IntentGraphNode[];

    const edges = this.db
      .prepare(
        "SELECT parent_id as source, id as target FROM intents WHERE parent_id IS NOT NULL",
      )
      .all() as IntentGraphEdge[];

    return { nodes, edges };
  }

  close(): void {
    this.db.close();
  }
}
