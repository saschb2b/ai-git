export interface IntentInference {
  intent: string;
  summary: string;
}

export interface LineExplanation {
  intent: string;
  checkpoint: string;
  reasoning: string;
}

export interface LLMProvider {
  inferIntent(
    commitMessages: string[],
    diffStats: string[],
  ): Promise<IntentInference>;

  generateSummary(changes: string[]): Promise<string>;

  explainLine(context: {
    filePath: string;
    line: number;
    intentDescription: string;
    checkpointMessage: string;
  }): Promise<string>;
}
