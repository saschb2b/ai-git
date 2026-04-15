import Database from "better-sqlite3";
import path from "node:path";

export interface Intent {
  id: string;
  description: string;
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
  reviewed: boolean;
  start_line: number;
  end_line: number;
}

export class AigDatabase {
  private db: Database.Database;

  constructor(aigDir: string = ".aig") {
    const dbPath = path.join(aigDir, "aig.db");
    this.db = new Database(dbPath, { readonly: true });
  }

  listIntents(): Intent[] {
    return this.db
      .prepare(
        "SELECT id, description, created_at, closed_at, summary FROM intents ORDER BY created_at DESC",
      )
      .all() as Intent[];
  }

  getCheckpoints(intentId: string): Checkpoint[] {
    return this.db
      .prepare(
        "SELECT id, message, git_commit_sha, created_at FROM checkpoints WHERE intent_id = ? ORDER BY created_at",
      )
      .all(intentId) as Checkpoint[];
  }

  getSemanticChanges(checkpointIds: string[]): SemanticChange[] {
    if (checkpointIds.length === 0) return [];
    const placeholders = checkpointIds.map(() => "?").join(",");
    return this.db
      .prepare(
        `SELECT file_path, change_type, symbol_name FROM semantic_changes WHERE checkpoint_id IN (${placeholders})`,
      )
      .all(...checkpointIds) as SemanticChange[];
  }

  getConversations(intentId: string): string[] {
    const rows = this.db
      .prepare(
        `SELECT c.message FROM conversations c
         JOIN sessions s ON c.session_id = s.id
         WHERE s.intent_id = ?
         ORDER BY c.created_at`,
      )
      .all(intentId) as { message: string }[];
    return rows.map((r) => r.message);
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

  close(): void {
    this.db.close();
  }
}
