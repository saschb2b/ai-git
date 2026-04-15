#!/usr/bin/env node
import React, { useState, useMemo } from "react";
import { render, Box, Text, useInput, useApp } from "ink";
import { AigDatabase } from "./db.js";
import type { Intent, Checkpoint, SemanticChange } from "./db.js";

// ── Data types ──────────────────────────────────────────────────────────

interface IntentDetail {
  intent: Intent;
  checkpoints: Checkpoint[];
  semanticChanges: Map<string, { symbol: string; changeType: string }[]>;
  conversations: string[];
  provenance: { human: number; ai: number; reviewed: number; total: number };
}

// ── Data loading ────────────────────────────────────────────────────────

function loadIntentDetail(db: AigDatabase, intent: Intent): IntentDetail {
  const checkpoints = db.getCheckpoints(intent.id);
  const cpIds = checkpoints.map((c) => c.id);
  const rawChanges = db.getSemanticChanges(cpIds);

  const semanticChanges = new Map<
    string,
    { symbol: string; changeType: string }[]
  >();
  for (const sc of rawChanges) {
    const existing = semanticChanges.get(sc.file_path) ?? [];
    const found = existing.find((e) => e.symbol === sc.symbol_name);
    if (found) {
      found.changeType = sc.change_type;
    } else {
      existing.push({ symbol: sc.symbol_name, changeType: sc.change_type });
    }
    semanticChanges.set(sc.file_path, existing);
  }

  const conversations = db.getConversations(intent.id);
  const prov = db.getProvenance(intent.id);

  let human = 0;
  let ai = 0;
  let reviewed = 0;
  for (const p of prov) {
    if (p.origin === "ai-assisted") ai++;
    else human++;
    if (p.reviewed) reviewed++;
  }

  return {
    intent,
    checkpoints,
    semanticChanges,
    conversations,
    provenance: { human, ai, reviewed, total: prov.length },
  };
}

// ── Components ──────────────────────────────────────────────────────────

function changeIcon(type: string): string {
  if (type === "added") return "+";
  if (type === "removed") return "-";
  return "~";
}

function IntentList({
  intents,
  selected,
}: {
  intents: Intent[];
  selected: number;
}) {
  return (
    <Box flexDirection="column" width={40} borderStyle="single" paddingX={1}>
      <Text bold>Intents ({intents.length})</Text>
      <Text dimColor>{"─".repeat(36)}</Text>
      {intents.map((intent, i) => {
        const isSelected = i === selected;
        const status = intent.closed_at ? "done" : "active";
        const prefix = isSelected ? ">" : " ";
        const shortId = intent.id.slice(0, 8);
        return (
          <Text key={intent.id} wrap="truncate">
            <Text color={isSelected ? "cyan" : undefined} bold={isSelected}>
              {prefix} [{shortId}] {intent.description}
            </Text>
            <Text dimColor> ({status})</Text>
          </Text>
        );
      })}
    </Box>
  );
}

function DetailPanel({ detail }: { detail: IntentDetail }) {
  const { intent, checkpoints, semanticChanges, conversations, provenance } =
    detail;
  const status = intent.closed_at ? "done" : "active";

  return (
    <Box
      flexDirection="column"
      flexGrow={1}
      borderStyle="single"
      paddingX={1}
    >
      <Text bold>{intent.description}</Text>
      <Text dimColor>
        Status: {status} | Checkpoints: {checkpoints.length}
      </Text>
      <Text dimColor>{"─".repeat(50)}</Text>

      {/* Checkpoints */}
      <Box flexDirection="column" marginTop={1}>
        <Text bold underline>
          Checkpoints
        </Text>
        {checkpoints.map((cp, i) => {
          const shortSha = cp.git_commit_sha.slice(0, 8);
          return (
            <Text key={cp.id}>
              {"  "}
              {i + 1}. ({shortSha}) {cp.message}
            </Text>
          );
        })}
      </Box>

      {/* Semantic changes */}
      {semanticChanges.size > 0 && (
        <Box flexDirection="column" marginTop={1}>
          <Text bold underline>
            Semantic Changes
          </Text>
          {Array.from(semanticChanges.entries()).map(([file, symbols]) => (
            <Box key={file} flexDirection="column">
              <Text>{"  "}{file}</Text>
              {symbols.map((s) => (
                <Text key={s.symbol} color={s.changeType === "added" ? "green" : s.changeType === "removed" ? "red" : "yellow"}>
                  {"    "}
                  {changeIcon(s.changeType)} {s.changeType} `{s.symbol}`
                </Text>
              ))}
            </Box>
          ))}
        </Box>
      )}

      {/* Provenance */}
      {provenance.total > 0 && (
        <Box flexDirection="column" marginTop={1}>
          <Text bold underline>
            Trust
          </Text>
          <Text>
            {"  "}
            {provenance.human} human, {provenance.ai} AI-assisted,{" "}
            {provenance.reviewed}/{provenance.total} reviewed
          </Text>
        </Box>
      )}

      {/* Conversations */}
      {conversations.length > 0 && (
        <Box flexDirection="column" marginTop={1}>
          <Text bold underline>
            Conversation ({conversations.length})
          </Text>
          {conversations.map((msg, i) => (
            <Text key={i} wrap="truncate">
              {"  "}- {msg}
            </Text>
          ))}
        </Box>
      )}
    </Box>
  );
}

function App({ db }: { db: AigDatabase }) {
  const { exit } = useApp();
  const intents = useMemo(() => db.listIntents(), [db]);
  const [selected, setSelected] = useState(0);

  const detail = useMemo(() => {
    if (intents.length === 0) return null;
    return loadIntentDetail(db, intents[selected]!);
  }, [db, intents, selected]);

  useInput((input, key) => {
    if (input === "q" || (key.ctrl && input === "c")) {
      db.close();
      exit();
      return;
    }
    if (key.upArrow || input === "k") {
      setSelected((s) => Math.max(0, s - 1));
    }
    if (key.downArrow || input === "j") {
      setSelected((s) => Math.min(intents.length - 1, s + 1));
    }
  });

  if (intents.length === 0) {
    return <Text>No intents recorded yet. Start with: aig session start &quot;your intent&quot;</Text>;
  }

  return (
    <Box flexDirection="column">
      <Box>
        <Text bold color="cyan">
          {" "}
          aig review{" "}
        </Text>
        <Text dimColor> | j/k or arrows to navigate | q to quit</Text>
      </Box>
      <Box flexDirection="row">
        <IntentList intents={intents} selected={selected} />
        {detail && <DetailPanel detail={detail} />}
      </Box>
    </Box>
  );
}

// ── Entry point ─────────────────────────────────────────────────────────

const aigDir = process.argv[2] ?? ".aig";
let db: AigDatabase;
try {
  db = new AigDatabase(aigDir);
} catch {
  console.error(
    "Could not open .aig database. Run from an aig-initialized repository.",
  );
  process.exit(1);
}

render(<App db={db} />);
