import type { LLMProvider } from "./providers/types.js";

export interface GitCommit {
  sha: string;
  message: string;
  author: string;
  timestamp: string;
  filesChanged: string[];
}

export interface CommitCluster {
  commits: GitCommit[];
  inferredIntent?: string;
  summary?: string;
}

const TWO_HOURS_MS = 2 * 60 * 60 * 1000;

export function clusterCommits(commits: GitCommit[]): CommitCluster[] {
  if (commits.length === 0) return [];

  const sorted = [...commits].sort(
    (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
  );

  const clusters: CommitCluster[] = [];
  let currentCluster: GitCommit[] = [sorted[0]];

  for (let i = 1; i < sorted.length; i++) {
    const prev = sorted[i - 1];
    const curr = sorted[i];

    const prevTime = new Date(prev.timestamp).getTime();
    const currTime = new Date(curr.timestamp).getTime();
    const timeDiff = currTime - prevTime;

    const sameAuthor = prev.author === curr.author;

    if (timeDiff <= TWO_HOURS_MS && sameAuthor) {
      currentCluster.push(curr);
    } else {
      clusters.push({ commits: currentCluster });
      currentCluster = [curr];
    }
  }

  clusters.push({ commits: currentCluster });

  return clusters;
}

export async function inferIntentsForClusters(
  clusters: CommitCluster[],
  provider: LLMProvider,
): Promise<CommitCluster[]> {
  const results: CommitCluster[] = [];

  for (const cluster of clusters) {
    const commitMessages = cluster.commits.map((c) => c.message);
    const diffStats = cluster.commits.map(
      (c) => `${c.sha.slice(0, 7)}: ${c.filesChanged.length} file(s) changed`,
    );

    try {
      const inference = await provider.inferIntent(commitMessages, diffStats);
      results.push({
        ...cluster,
        inferredIntent: inference.intent,
        summary: inference.summary,
      });
    } catch {
      results.push(cluster);
    }
  }

  return results;
}
