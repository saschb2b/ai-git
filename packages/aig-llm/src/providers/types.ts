export interface IntentInference {
  intent: string;
  summary: string;
}

export interface LineExplanation {
  intent: string;
  checkpoint: string;
  reasoning: string;
}

export interface ExplainLineContext {
  filePath: string;
  line: number;
  intentDescription: string;
  checkpointMessage: string;
  conversationNotes?: string[];
  semanticChanges?: string[];
  lineContent?: string;
}

export interface LLMProvider {
  inferIntent(
    commitMessages: string[],
    diffStats: string[],
  ): Promise<IntentInference>;

  generateSummary(changes: string[]): Promise<string>;

  explainLine(context: ExplainLineContext): Promise<string>;
}
