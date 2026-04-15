import { useState, useRef, useEffect, useCallback } from "react";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import IconButton from "@mui/material/IconButton";
import Collapse from "@mui/material/Collapse";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Button from "@mui/material/Button";
import Tooltip from "@mui/material/Tooltip";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import ExpandLessIcon from "@mui/icons-material/ExpandLess";
import UnfoldMoreIcon from "@mui/icons-material/UnfoldMore";
import UnfoldLessIcon from "@mui/icons-material/UnfoldLess";
import InsertDriveFileOutlinedIcon from "@mui/icons-material/InsertDriveFileOutlined";
import ViewStreamIcon from "@mui/icons-material/ViewStream";
import ViewColumnIcon from "@mui/icons-material/ViewColumn";

interface DiffLine {
  type: "add" | "delete" | "context";
  content: string;
  oldLine: number | null;
  newLine: number | null;
}

interface DiffHunk {
  header: string;
  lines: DiffLine[];
}

interface FileDiff {
  path: string;
  status: "added" | "modified" | "deleted" | "renamed";
  additions: number;
  deletions: number;
  hunks: DiffHunk[];
  isBinary: boolean;
}

type ViewMode = "unified" | "split";

const STATUS_LABELS: Record<string, { label: string; color: string }> = {
  added: { label: "Added", color: "#7ee787" },
  deleted: { label: "Deleted", color: "#ff7b72" },
  modified: { label: "Modified", color: "#e3b341" },
  renamed: { label: "Renamed", color: "#a5d6ff" },
};

const LINE_BG: Record<string, string> = {
  add: "rgba(46, 160, 67, 0.10)",
  delete: "rgba(248, 81, 73, 0.08)",
  context: "transparent",
};

const LINE_TEXT: Record<string, string> = {
  add: "#aff5b4",
  delete: "#ffd7d5",
  context: "#c9d1d9",
};

const GUTTER_BG: Record<string, string> = {
  add: "rgba(46, 160, 67, 0.20)",
  delete: "rgba(248, 81, 73, 0.15)",
  context: "transparent",
};

// ─── Stat bar (green/red blocks like GitHub) ────────────────────────

function StatBar({ additions, deletions }: { additions: number; deletions: number }) {
  const total = additions + deletions;
  if (total === 0) return null;
  const blocks = 5;
  const addBlocks = Math.round((additions / total) * blocks);
  const delBlocks = blocks - addBlocks;

  return (
    <Box sx={{ display: "inline-flex", gap: "2px", ml: 1, alignItems: "center" }}>
      {Array.from({ length: addBlocks }, (_, i) => (
        <Box key={`a${i}`} sx={{ width: 8, height: 8, borderRadius: "2px", bgcolor: "#7ee787" }} />
      ))}
      {Array.from({ length: delBlocks }, (_, i) => (
        <Box key={`d${i}`} sx={{ width: 8, height: 8, borderRadius: "2px", bgcolor: "#ff7b72" }} />
      ))}
    </Box>
  );
}

// ─── File header ────────────────────────────────────────────────────

function FileHeader({
  file,
  expanded,
  onToggle,
  id,
}: {
  file: FileDiff;
  expanded: boolean;
  onToggle: () => void;
  id: string;
}) {
  const st = STATUS_LABELS[file.status];
  return (
    <Box
      id={id}
      onClick={onToggle}
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 1,
        px: 2,
        py: 1,
        bgcolor: "#161b22",
        borderBottom: expanded ? "1px solid #30363d" : "none",
        cursor: "pointer",
        userSelect: "none",
        position: "sticky",
        top: 0,
        zIndex: 1,
        "&:hover": { bgcolor: "#1c2128" },
      }}
    >
      <IconButton size="small" sx={{ p: 0 }}>
        {expanded ? (
          <ExpandLessIcon sx={{ fontSize: 16, color: "#8b949e" }} />
        ) : (
          <ExpandMoreIcon sx={{ fontSize: 16, color: "#8b949e" }} />
        )}
      </IconButton>
      <InsertDriveFileOutlinedIcon sx={{ fontSize: 15, color: "#8b949e" }} />
      <Typography fontSize="0.82rem" fontFamily="monospace" sx={{ flex: 1, color: "#e0e0e6" }}>
        {file.path}
      </Typography>
      {file.status !== "modified" && st && (
        <Typography fontSize="0.65rem" fontWeight={700} sx={{ color: st.color, textTransform: "uppercase", letterSpacing: "0.06em" }}>
          {st.label}
        </Typography>
      )}
      {!file.isBinary && (file.additions > 0 || file.deletions > 0) && (
        <>
          <Typography fontSize="0.78rem" fontFamily="monospace" sx={{ color: "#7ee787" }}>+{file.additions}</Typography>
          <Typography fontSize="0.78rem" fontFamily="monospace" sx={{ color: "#ff7b72" }}>-{file.deletions}</Typography>
          <StatBar additions={file.additions} deletions={file.deletions} />
        </>
      )}
    </Box>
  );
}

// ─── Hunk separator ─────────────────────────────────────────────────

function HunkSeparator({ header }: { header: string }) {
  return (
    <Box sx={{ px: 2, py: 0.4, bgcolor: "rgba(100, 108, 255, 0.05)", borderTop: "1px solid #21262d", borderBottom: "1px solid #21262d" }}>
      <Typography fontSize="0.73rem" fontFamily="monospace" color="#6e7681" fontStyle="italic">
        {header}
      </Typography>
    </Box>
  );
}

// ─── Unified diff line ──────────────────────────────────────────────

function UnifiedLine({ line }: { line: DiffLine }) {
  const prefix = line.type === "add" ? "+" : line.type === "delete" ? "-" : " ";
  return (
    <Box sx={{ display: "flex", bgcolor: LINE_BG[line.type], minHeight: 20, lineHeight: "20px", "&:hover": { filter: "brightness(1.15)" } }}>
      <Box sx={{ width: 48, minWidth: 48, textAlign: "right", pr: 0.8, color: "#484f58", fontSize: "0.72rem", fontFamily: "monospace", bgcolor: GUTTER_BG[line.type], userSelect: "none", borderRight: "1px solid #21262d" }}>
        {line.oldLine ?? ""}
      </Box>
      <Box sx={{ width: 48, minWidth: 48, textAlign: "right", pr: 0.8, color: "#484f58", fontSize: "0.72rem", fontFamily: "monospace", bgcolor: GUTTER_BG[line.type], userSelect: "none", borderRight: "1px solid #21262d" }}>
        {line.newLine ?? ""}
      </Box>
      <Box sx={{ width: 18, minWidth: 18, textAlign: "center", color: LINE_TEXT[line.type], fontSize: "0.78rem", fontFamily: "monospace", fontWeight: line.type !== "context" ? 600 : 400, userSelect: "none" }}>
        {prefix}
      </Box>
      <Box sx={{ flex: 1, px: 0.8, color: LINE_TEXT[line.type], fontSize: "0.78rem", fontFamily: "'SF Mono','Cascadia Code','Fira Code',monospace", whiteSpace: "pre", overflow: "hidden" }}>
        {line.content || " "}
      </Box>
    </Box>
  );
}

// ─── Split diff line pair ───────────────────────────────────────────

function SplitHalf({ line, side }: { line: DiffLine | null; side: "left" | "right" }) {
  if (!line) {
    return (
      <Box sx={{ flex: 1, display: "flex", bgcolor: "#0d1117", minHeight: 20, borderRight: side === "left" ? "1px solid #21262d" : "none" }}>
        <Box sx={{ width: 48, minWidth: 48, bgcolor: "#0d1117", borderRight: "1px solid #21262d" }} />
        <Box sx={{ flex: 1 }} />
      </Box>
    );
  }

  const bg = line.type === "context" ? "transparent" : LINE_BG[line.type];
  const color = LINE_TEXT[line.type];
  const lineNo = side === "left" ? line.oldLine : line.newLine;

  return (
    <Box sx={{ flex: 1, display: "flex", bgcolor: bg, minHeight: 20, borderRight: side === "left" ? "1px solid #21262d" : "none", "&:hover": { filter: "brightness(1.15)" } }}>
      <Box sx={{ width: 48, minWidth: 48, textAlign: "right", pr: 0.8, color: "#484f58", fontSize: "0.72rem", fontFamily: "monospace", bgcolor: GUTTER_BG[line.type], userSelect: "none", borderRight: "1px solid #21262d" }}>
        {lineNo ?? ""}
      </Box>
      <Box sx={{ flex: 1, px: 0.8, color, fontSize: "0.78rem", fontFamily: "'SF Mono','Cascadia Code','Fira Code',monospace", whiteSpace: "pre", overflow: "hidden" }}>
        {line.content || " "}
      </Box>
    </Box>
  );
}

function SplitHunk({ hunk }: { hunk: DiffHunk }) {
  // Pair up delete/add lines side by side; context lines go on both sides
  const pairs: { left: DiffLine | null; right: DiffLine | null }[] = [];
  let i = 0;
  const lines = hunk.lines;

  while (i < lines.length) {
    if (lines[i].type === "context") {
      pairs.push({ left: lines[i], right: lines[i] });
      i++;
    } else {
      // Collect consecutive deletes then adds
      const deletes: DiffLine[] = [];
      const adds: DiffLine[] = [];
      while (i < lines.length && lines[i].type === "delete") deletes.push(lines[i++]);
      while (i < lines.length && lines[i].type === "add") adds.push(lines[i++]);
      const max = Math.max(deletes.length, adds.length);
      for (let j = 0; j < max; j++) {
        pairs.push({ left: deletes[j] ?? null, right: adds[j] ?? null });
      }
    }
  }

  return (
    <>
      {pairs.map((pair, idx) => (
        <Box key={idx} sx={{ display: "flex", lineHeight: "20px" }}>
          <SplitHalf line={pair.left} side="left" />
          <SplitHalf line={pair.right} side="right" />
        </Box>
      ))}
    </>
  );
}

// ─── Single file diff ───────────────────────────────────────────────

const LARGE_DIFF_THRESHOLD = 300;

function FileDiffView({
  file,
  viewMode,
  defaultExpanded,
}: {
  file: FileDiff;
  viewMode: ViewMode;
  defaultExpanded: boolean;
}) {
  const totalLines = file.hunks.reduce((s, h) => s + h.lines.length, 0);
  const isLarge = totalLines > LARGE_DIFF_THRESHOLD;
  const [expanded, setExpanded] = useState(defaultExpanded && !isLarge);
  const [showLarge, setShowLarge] = useState(false);
  const fileId = `diff-file-${file.path.replace(/[^a-zA-Z0-9]/g, "-")}`;

  // Update expanded when defaultExpanded changes (expand/collapse all)
  useEffect(() => {
    setExpanded(defaultExpanded && !isLarge);
  }, [defaultExpanded, isLarge]);

  return (
    <Box sx={{ border: "1px solid #30363d", borderRadius: "6px", overflow: "hidden", mb: 1.5 }}>
      <FileHeader file={file} expanded={expanded} onToggle={() => setExpanded(!expanded)} id={fileId} />
      <Collapse in={expanded}>
        {file.isBinary ? (
          <Box sx={{ px: 2, py: 2 }}>
            <Typography fontSize="0.82rem" color="#8b949e" fontStyle="italic">Binary file not shown.</Typography>
          </Box>
        ) : isLarge && !showLarge ? (
          <Box sx={{ px: 2, py: 3, textAlign: "center" }}>
            <Typography fontSize="0.85rem" color="#8b949e" sx={{ mb: 1 }}>
              Large diff ({totalLines} lines) — collapsed for performance
            </Typography>
            <Button size="small" variant="outlined" onClick={() => setShowLarge(true)}>
              Load diff
            </Button>
          </Box>
        ) : (
          <Box sx={{ overflow: "auto" }}>
            {file.hunks.map((hunk, hi) => (
              <Box key={hi}>
                {hunk.header && <HunkSeparator header={hunk.header} />}
                {viewMode === "unified"
                  ? hunk.lines.map((line, li) => <UnifiedLine key={li} line={line} />)
                  : <SplitHunk hunk={hunk} />
                }
              </Box>
            ))}
          </Box>
        )}
      </Collapse>
    </Box>
  );
}

// ─── Summary bar ────────────────────────────────────────────────────

function DiffSummary({ files }: { files: FileDiff[] }) {
  const totalAdd = files.reduce((s, f) => s + f.additions, 0);
  const totalDel = files.reduce((s, f) => s + f.deletions, 0);

  return (
    <Box sx={{ display: "flex", alignItems: "center", gap: 2, py: 1, px: 2, bgcolor: "#161b22", borderRadius: "6px", border: "1px solid #21262d" }}>
      <Typography fontSize="0.82rem" color="#e0e0e6">
        <strong>{files.length}</strong> file{files.length !== 1 ? "s" : ""} changed
      </Typography>
      <Typography fontSize="0.82rem" color="#7ee787" fontFamily="monospace" fontWeight={600}>+{totalAdd}</Typography>
      <Typography fontSize="0.82rem" color="#ff7b72" fontFamily="monospace" fontWeight={600}>-{totalDel}</Typography>
    </Box>
  );
}

// ─── File tree navigation ───────────────────────────────────────────

function FileTree({ files }: { files: FileDiff[] }) {
  return (
    <Box sx={{ py: 1, px: 1, maxHeight: 300, overflow: "auto", bgcolor: "#0d1117", borderRadius: "6px", border: "1px solid #21262d" }}>
      {files.map((f) => {
        const st = STATUS_LABELS[f.status];
        return (
          <Box
            key={f.path}
            component="a"
            href={`#diff-file-${f.path.replace(/[^a-zA-Z0-9]/g, "-")}`}
            sx={{
              display: "flex",
              alignItems: "center",
              gap: 1,
              px: 1,
              py: 0.4,
              borderRadius: "4px",
              textDecoration: "none",
              color: "#c9d1d9",
              fontSize: "0.78rem",
              fontFamily: "monospace",
              "&:hover": { bgcolor: "rgba(100, 108, 255, 0.08)" },
            }}
          >
            <Box sx={{ width: 6, height: 6, borderRadius: "50%", bgcolor: st?.color ?? "#8b949e", flexShrink: 0 }} />
            <Typography fontSize="0.78rem" fontFamily="monospace" sx={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {f.path}
            </Typography>
            {f.additions > 0 && <Typography fontSize="0.7rem" color="#7ee787">+{f.additions}</Typography>}
            {f.deletions > 0 && <Typography fontSize="0.7rem" color="#ff7b72">-{f.deletions}</Typography>}
          </Box>
        );
      })}
    </Box>
  );
}

// ─── Toolbar ────────────────────────────────────────────────────────

function DiffToolbar({
  viewMode,
  onViewModeChange,
  allExpanded,
  onToggleAll,
  fileCount,
}: {
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
  allExpanded: boolean;
  onToggleAll: () => void;
  fileCount: number;
}) {
  return (
    <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 1.5 }}>
      <ToggleButtonGroup
        size="small"
        value={viewMode}
        exclusive
        onChange={(_, v) => v && onViewModeChange(v)}
        sx={{ "& .MuiToggleButton-root": { px: 1.2, py: 0.4, fontSize: "0.75rem", textTransform: "none", color: "#8b949e", borderColor: "#30363d", "&.Mui-selected": { color: "#e0e0e6", bgcolor: "rgba(100,108,255,0.12)" } } }}
      >
        <ToggleButton value="unified">
          <Tooltip title="Unified view"><ViewStreamIcon sx={{ fontSize: 16, mr: 0.5 }} /></Tooltip>
          Unified
        </ToggleButton>
        <ToggleButton value="split">
          <Tooltip title="Split view"><ViewColumnIcon sx={{ fontSize: 16, mr: 0.5 }} /></Tooltip>
          Split
        </ToggleButton>
      </ToggleButtonGroup>
      <Box sx={{ flex: 1 }} />
      <Tooltip title={allExpanded ? "Collapse all files" : "Expand all files"}>
        <IconButton size="small" onClick={onToggleAll} sx={{ color: "#8b949e" }}>
          {allExpanded ? <UnfoldLessIcon fontSize="small" /> : <UnfoldMoreIcon fontSize="small" />}
        </IconButton>
      </Tooltip>
    </Box>
  );
}

// ─── Main: intent-level diff ────────────────────────────────────────

export function IntentDiffViewer({ intentId, active }: { intentId: string; active: boolean }) {
  const [loaded, setLoaded] = useState(false);
  const [data, setData] = useState<FileDiff[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("unified");
  const [allExpanded, setAllExpanded] = useState(true);

  // Lazy load: only fetch when tab becomes active
  useEffect(() => {
    if (active && !loaded) {
      setLoading(true);
      fetch(`/api/intents/${intentId}/diff`)
        .then((res) => {
          if (!res.ok) throw new Error(`${res.status}`);
          return res.json();
        })
        .then((json) => { setData(json); setLoaded(true); setLoading(false); })
        .catch((err) => { setError(err.message); setLoading(false); });
    }
  }, [active, intentId, loaded]);

  if (!active && !loaded) return null;
  if (loading) return <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}><CircularProgress size={28} /></Box>;
  if (error) return <Alert severity="error">{error}</Alert>;
  if (!data || data.length === 0) return <Typography color="text.secondary" sx={{ py: 2 }}>No diff available.</Typography>;

  return (
    <>
      <DiffSummary files={data} />
      <Box sx={{ mt: 1.5 }} />
      <DiffToolbar viewMode={viewMode} onViewModeChange={setViewMode} allExpanded={allExpanded} onToggleAll={() => setAllExpanded(!allExpanded)} fileCount={data.length} />
      {data.length > 3 && <FileTree files={data} />}
      <Box sx={{ mt: 1.5 }} />
      {data.map((file) => (
        <FileDiffView key={file.path} file={file} viewMode={viewMode} defaultExpanded={allExpanded} />
      ))}
    </>
  );
}

// ─── Main: single commit diff ───────────────────────────────────────

export function CommitDiffViewer({ sha, active }: { sha: string; active: boolean }) {
  const [loaded, setLoaded] = useState(false);
  const [data, setData] = useState<FileDiff[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>("unified");
  const [allExpanded, setAllExpanded] = useState(true);

  useEffect(() => {
    if (active && !loaded) {
      setLoading(true);
      fetch(`/api/commits/${sha}/diff`)
        .then((res) => {
          if (!res.ok) throw new Error(`${res.status}`);
          return res.json();
        })
        .then((json) => { setData(json); setLoaded(true); setLoading(false); })
        .catch((err) => { setError(err.message); setLoading(false); });
    }
  }, [active, sha, loaded]);

  if (!active && !loaded) return null;
  if (loading) return <Box sx={{ display: "flex", justifyContent: "center", py: 2 }}><CircularProgress size={20} /></Box>;
  if (error) return <Alert severity="error">{error}</Alert>;
  if (!data || data.length === 0) return <Typography color="text.secondary" fontSize="0.85rem" sx={{ py: 1 }}>No diff available.</Typography>;

  return (
    <>
      <DiffToolbar viewMode={viewMode} onViewModeChange={setViewMode} allExpanded={allExpanded} onToggleAll={() => setAllExpanded(!allExpanded)} fileCount={data.length} />
      {data.map((file) => (
        <FileDiffView key={file.path} file={file} viewMode={viewMode} defaultExpanded={allExpanded} />
      ))}
    </>
  );
}
