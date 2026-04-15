#!/usr/bin/env node
import React, { useState, useMemo, useEffect } from "react";
import { render, Box, Text, useInput, useApp, useStdout } from "ink";
import { AigDatabase } from "./db.js";
import type { Intent } from "./db.js";

// ── Data types ──────────────────────────────────────────────────────────

interface IntentDetail {
  intent: Intent;
  checkpoints: { id: string; message: string; sha: string }[];
  semanticChanges: Map<string, { symbol: string; changeType: string }[]>;
  conversations: string[];
  provenance: { human: number; ai: number; reviewed: number; total: number };
  duration: string;
}

// ── Helpers ─────────────────────────────────────────────────────────────

function formatDuration(startIso: string, endIso: string | null): string {
  const start = new Date(startIso).getTime();
  const end = endIso ? new Date(endIso).getTime() : Date.now();
  const secs = Math.max(0, Math.floor((end - start) / 1000));

  const days = Math.floor(secs / 86400);
  const hours = Math.floor((secs % 86400) / 3600);
  const mins = Math.floor((secs % 3600) / 60);

  if (days > 0) return days === 1 ? "1 day" : `${days} days`;
  if (hours > 0) return mins === 0 ? `${hours} h` : `${hours} h ${mins} min`;
  if (mins > 0) return `${mins} min`;
  return `${secs} sec`;
}

function changeIcon(type: string): string {
  if (type === "added") return "+";
  if (type === "removed") return "-";
  return "~";
}

// ── Data loading ────────────────────────────────────────────────────────

function loadIntentDetail(db: AigDatabase, intent: Intent): IntentDetail {
  const rawCheckpoints = db.getCheckpoints(intent.id);
  const cpIds = rawCheckpoints.map((c) => c.id);
  const rawChanges = db.getSemanticChanges(cpIds);

  const checkpoints = rawCheckpoints.map((c) => ({
    id: c.id,
    message: c.message,
    sha: c.git_commit_sha.slice(0, 8),
  }));

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
    duration: formatDuration(intent.created_at, intent.closed_at),
  };
}

// ── Components ──────────────────────────────────────────────────────────

const MAX_VISIBLE = 20;

function IntentList({
  intents,
  selected,
  height,
}: {
  intents: Intent[];
  selected: number;
  height: number;
}) {
  const visible = Math.min(intents.length, Math.max(5, height - 4));
  const half = Math.floor(visible / 2);
  let start = Math.max(0, selected - half);
  if (start + visible > intents.length) {
    start = Math.max(0, intents.length - visible);
  }
  const slice = intents.slice(start, start + visible);

  return (
    <Box flexDirection="column" width={40} borderStyle="single" paddingX={1}>
      <Text bold>Intents ({intents.length})</Text>
      <Box flexDirection="column">
        {slice.map((intent, i) => {
          const realIndex = start + i;
          const isSelected = realIndex === selected;
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
      {intents.length > visible && (
        <Text dimColor>
          [{start + 1}-{start + slice.length} of {intents.length}]
        </Text>
      )}
    </Box>
  );
}

function DetailPanel({ detail, width }: { detail: IntentDetail; width: number }) {
  const { intent, checkpoints, semanticChanges, conversations, provenance, duration } =
    detail;
  const status = intent.closed_at ? "done" : "active";
  const separatorWidth = Math.max(10, width - 46);

  return (
    <Box
      flexDirection="column"
      flexGrow={1}
      borderStyle="single"
      paddingX={1}
    >
      <Text bold>{intent.description}</Text>
      <Text dimColor>
        Status: {status} | Duration: {duration} | Checkpoints: {checkpoints.length}
      </Text>
      {intent.summary && <Text dimColor>Summary: {intent.summary}</Text>}
      <Text dimColor>{"─".repeat(separatorWidth)}</Text>

      {/* Checkpoints */}
      <Box flexDirection="column" marginTop={1}>
        <Text bold underline>
          Checkpoints
        </Text>
        {checkpoints.length === 0 ? (
          <Text dimColor>{"  "}(none)</Text>
        ) : (
          checkpoints.map((cp, i) => (
            <Text key={cp.id}>
              {"  "}
              {i + 1}. ({cp.sha}) {cp.message}
            </Text>
          ))
        )}
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
              {symbols.map((s, i) => (
                <Text
                  key={`${file}-${s.symbol}-${i}`}
                  color={
                    s.changeType === "added"
                      ? "green"
                      : s.changeType === "removed"
                        ? "red"
                        : "yellow"
                  }
                >
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
            <Text key={`conv-${i}`} wrap="truncate">
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
  const { stdout } = useStdout();
  const intents = useMemo(() => db.listIntents(), [db]);
  const [selected, setSelected] = useState(0);

  const termHeight = stdout?.rows ?? MAX_VISIBLE;
  const termWidth = stdout?.columns ?? 80;

  // Clean up database on unmount
  useEffect(() => {
    return () => db.close();
  }, [db]);

  const detail = useMemo(() => {
    if (intents.length === 0) return null;
    const intent = intents[selected];
    if (!intent) return null;
    return loadIntentDetail(db, intent);
  }, [db, intents, selected]);

  useInput((input, key) => {
    if (input === "q") {
      exit();
      return;
    }
    if (key.upArrow || input === "k") {
      setSelected((s) => Math.max(0, s - 1));
    }
    if (key.downArrow || input === "j") {
      setSelected((s) => Math.min(intents.length - 1, s + 1));
    }
    // Jump to top/bottom
    if (input === "g") {
      setSelected(0);
    }
    if (input === "G") {
      setSelected(intents.length - 1);
    }
  });

  if (intents.length === 0) {
    return (
      <Text>
        No intents recorded yet. Start with: aig session start &quot;your intent&quot;
      </Text>
    );
  }

  return (
    <Box flexDirection="column">
      <Box>
        <Text bold color="cyan">
          {" "}
          aig review{" "}
        </Text>
        <Text dimColor>
          {" "}
          | j/k navigate | g/G top/bottom | q quit
        </Text>
      </Box>
      <Box flexDirection="row">
        <IntentList intents={intents} selected={selected} height={termHeight} />
        {detail && <DetailPanel detail={detail} width={termWidth} />}
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

// Ensure cleanup on unexpected exit
process.on("exit", () => {
  try {
    db.close();
  } catch {
    // already closed
  }
});

render(<App db={db} />);
