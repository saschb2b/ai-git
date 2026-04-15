import * as readline from "node:readline";
import type { LLMProvider } from "./providers/types.js";

interface IpcRequest {
  id: string;
  command: "infer_intent" | "generate_summary" | "explain_line";
  params: Record<string, unknown>;
}

interface InferIntentParams {
  commit_messages: string[];
  diff_stats: string[];
}

interface GenerateSummaryParams {
  changes: string[];
}

interface ExplainLineParams {
  file_path: string;
  line: number;
  intent_description: string;
  checkpoint_message: string;
  conversation_notes?: string[];
  semantic_changes?: string[];
  line_content?: string;
}

interface IpcResponse {
  id: string;
  result?: unknown;
  error?: string;
}

function writeResponse(response: IpcResponse): void {
  process.stdout.write(JSON.stringify(response) + "\n");
}

async function handleCommand(
  request: IpcRequest,
  provider: LLMProvider,
): Promise<unknown> {
  switch (request.command) {
    case "infer_intent": {
      const params = request.params as unknown as InferIntentParams;
      return provider.inferIntent(params.commit_messages, params.diff_stats);
    }
    case "generate_summary": {
      const params = request.params as unknown as GenerateSummaryParams;
      return provider.generateSummary(params.changes);
    }
    case "explain_line": {
      const params = request.params as unknown as ExplainLineParams;
      return provider.explainLine({
        filePath: params.file_path,
        line: params.line,
        intentDescription: params.intent_description,
        checkpointMessage: params.checkpoint_message,
        conversationNotes: params.conversation_notes,
        semanticChanges: params.semantic_changes,
        lineContent: params.line_content,
      });
    }
    default:
      throw new Error(`Unknown command: ${String(request.command)}`);
  }
}

export function startIpcServer(provider: LLMProvider): void {
  const rl = readline.createInterface({
    input: process.stdin,
    terminal: false,
  });

  rl.on("line", (line: string) => {
    const trimmed = line.trim();
    if (!trimmed) return;

    let request: IpcRequest;
    try {
      request = JSON.parse(trimmed) as IpcRequest;
    } catch {
      writeResponse({ id: "unknown", error: "Invalid JSON" });
      return;
    }

    handleCommand(request, provider)
      .then((result) => {
        writeResponse({ id: request.id, result });
      })
      .catch((err: unknown) => {
        const message = err instanceof Error ? err.message : String(err);
        writeResponse({ id: request.id, error: message });
      });
  });

  rl.on("close", () => {
    process.exit(0);
  });
}
