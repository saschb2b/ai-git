import Anthropic from "@anthropic-ai/sdk";
import type { IntentInference, LLMProvider } from "./types.js";

const DEFAULT_MODEL = "claude-sonnet-4-20250514";

export class AnthropicProvider implements LLMProvider {
  private client: Anthropic;
  private model: string;

  constructor(apiKey: string, model: string = DEFAULT_MODEL) {
    this.client = new Anthropic({ apiKey });
    this.model = model;
  }

  async inferIntent(
    commitMessages: string[],
    diffStats: string[],
  ): Promise<IntentInference> {
    const prompt = `You are analyzing a set of git commits to infer the developer's intent.

Commit messages:
${commitMessages.map((m, i) => `${i + 1}. ${m}`).join("\n")}

Diff stats:
${diffStats.map((s, i) => `${i + 1}. ${s}`).join("\n")}

Based on these commits and their diff stats, infer the developer's high-level intent and provide a concise summary.

Respond with a JSON object containing exactly two fields:
- "intent": A short label for the intent (e.g., "feature", "bugfix", "refactor", "docs", "test", "chore")
- "summary": A one-sentence summary of what the developer was trying to accomplish

Respond ONLY with valid JSON, no other text.`;

    const response = await this.client.messages.create({
      model: this.model,
      max_tokens: 256,
      messages: [{ role: "user", content: prompt }],
    });

    const text =
      response.content[0].type === "text" ? response.content[0].text : "";
    const parsed: unknown = JSON.parse(text);
    const result = parsed as IntentInference;

    return {
      intent: result.intent,
      summary: result.summary,
    };
  }

  async generateSummary(changes: string[]): Promise<string> {
    const prompt = `Summarize the following code changes in one concise paragraph:

${changes.join("\n\n")}

Provide only the summary, no preamble.`;

    const response = await this.client.messages.create({
      model: this.model,
      max_tokens: 512,
      messages: [{ role: "user", content: prompt }],
    });

    return response.content[0].type === "text" ? response.content[0].text : "";
  }

  async explainLine(context: {
    filePath: string;
    line: number;
    intentDescription: string;
    checkpointMessage: string;
  }): Promise<string> {
    const prompt = `Explain why this line was changed in the context of the developer's intent.

File: ${context.filePath}
Line number: ${context.line}
Developer intent: ${context.intentDescription}
Checkpoint message: ${context.checkpointMessage}

Provide a brief, clear explanation of the purpose of this change.`;

    const response = await this.client.messages.create({
      model: this.model,
      max_tokens: 256,
      messages: [{ role: "user", content: prompt }],
    });

    return response.content[0].type === "text" ? response.content[0].text : "";
  }
}
