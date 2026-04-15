import { useState } from "react";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import IconButton from "@mui/material/IconButton";
import Collapse from "@mui/material/Collapse";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import ExpandLessIcon from "@mui/icons-material/ExpandLess";
import InsertDriveFileOutlinedIcon from "@mui/icons-material/InsertDriveFileOutlined";
import { useApi } from "../hooks/useApi";

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

const STATUS_COLORS: Record<string, string> = {
  added: "#7ee787",
  deleted: "#ff7b72",
  modified: "#e3b341",
  renamed: "#a5d6ff",
};

function StatBar({
  additions,
  deletions,
}: {
  additions: number;
  deletions: number;
}) {
  const total = additions + deletions;
  if (total === 0) return null;
  const blocks = 5;
  const addBlocks = Math.round((additions / total) * blocks);
  const delBlocks = blocks - addBlocks;

  return (
    <Box sx={{ display: "inline-flex", gap: "2px", ml: 1, alignItems: "center" }}>
      {Array.from({ length: addBlocks }).map((_, i) => (
        <Box
          key={`a${i}`}
          sx={{
            width: 8,
            height: 8,
            borderRadius: "2px",
            bgcolor: "#7ee787",
          }}
        />
      ))}
      {Array.from({ length: delBlocks }).map((_, i) => (
        <Box
          key={`d${i}`}
          sx={{
            width: 8,
            height: 8,
            borderRadius: "2px",
            bgcolor: "#ff7b72",
          }}
        />
      ))}
    </Box>
  );
}

function FileHeader({
  file,
  expanded,
  onToggle,
}: {
  file: FileDiff;
  expanded: boolean;
  onToggle: () => void;
}) {
  return (
    <Box
      onClick={onToggle}
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 1,
        px: 2,
        py: 1.2,
        bgcolor: "#161b22",
        borderBottom: expanded ? "1px solid #30363d" : "none",
        cursor: "pointer",
        userSelect: "none",
        "&:hover": { bgcolor: "#1c2128" },
      }}
    >
      <IconButton size="small" sx={{ p: 0 }}>
        {expanded ? (
          <ExpandLessIcon sx={{ fontSize: 18, color: "#8b949e" }} />
        ) : (
          <ExpandMoreIcon sx={{ fontSize: 18, color: "#8b949e" }} />
        )}
      </IconButton>
      <InsertDriveFileOutlinedIcon sx={{ fontSize: 16, color: "#8b949e" }} />
      <Typography
        fontSize="0.85rem"
        fontFamily="monospace"
        sx={{ flex: 1, color: "#e0e0e6" }}
      >
        {file.path}
      </Typography>
      {file.status !== "modified" && (
        <Typography
          fontSize="0.7rem"
          fontWeight={600}
          sx={{
            color: STATUS_COLORS[file.status],
            textTransform: "uppercase",
            letterSpacing: "0.05em",
          }}
        >
          {file.status}
        </Typography>
      )}
      {!file.isBinary && (file.additions > 0 || file.deletions > 0) && (
        <>
          <Typography
            fontSize="0.8rem"
            fontFamily="monospace"
            sx={{ color: "#7ee787" }}
          >
            +{file.additions}
          </Typography>
          <Typography
            fontSize="0.8rem"
            fontFamily="monospace"
            sx={{ color: "#ff7b72" }}
          >
            -{file.deletions}
          </Typography>
          <StatBar additions={file.additions} deletions={file.deletions} />
        </>
      )}
    </Box>
  );
}

function HunkHeader({ header }: { header: string }) {
  return (
    <Box
      sx={{
        px: 2,
        py: 0.5,
        bgcolor: "rgba(100, 108, 255, 0.06)",
        borderTop: "1px solid #21262d",
        borderBottom: "1px solid #21262d",
      }}
    >
      <Typography
        fontSize="0.75rem"
        fontFamily="monospace"
        color="#8b949e"
        fontStyle="italic"
      >
        {header}
      </Typography>
    </Box>
  );
}

function DiffLineRow({ line }: { line: DiffLine }) {
  const bgColors: Record<string, string> = {
    add: "rgba(46, 160, 67, 0.12)",
    delete: "rgba(248, 81, 73, 0.10)",
    context: "transparent",
  };

  const textColors: Record<string, string> = {
    add: "#aff5b4",
    delete: "#ffd7d5",
    context: "#c9d1d9",
  };

  const gutterBg: Record<string, string> = {
    add: "rgba(46, 160, 67, 0.25)",
    delete: "rgba(248, 81, 73, 0.20)",
    context: "transparent",
  };

  const prefix = line.type === "add" ? "+" : line.type === "delete" ? "-" : " ";

  return (
    <Box
      sx={{
        display: "flex",
        bgcolor: bgColors[line.type],
        "&:hover": {
          bgcolor:
            line.type === "context"
              ? "rgba(100, 108, 255, 0.04)"
              : bgColors[line.type],
        },
        minHeight: 20,
        lineHeight: "20px",
      }}
    >
      {/* Old line number */}
      <Box
        sx={{
          width: 50,
          minWidth: 50,
          textAlign: "right",
          pr: 1,
          color: "#484f58",
          fontSize: "0.75rem",
          fontFamily: "monospace",
          bgcolor: gutterBg[line.type],
          userSelect: "none",
          borderRight: "1px solid #21262d",
        }}
      >
        {line.oldLine ?? ""}
      </Box>
      {/* New line number */}
      <Box
        sx={{
          width: 50,
          minWidth: 50,
          textAlign: "right",
          pr: 1,
          color: "#484f58",
          fontSize: "0.75rem",
          fontFamily: "monospace",
          bgcolor: gutterBg[line.type],
          userSelect: "none",
          borderRight: "1px solid #21262d",
        }}
      >
        {line.newLine ?? ""}
      </Box>
      {/* Prefix (+/-/space) */}
      <Box
        sx={{
          width: 20,
          minWidth: 20,
          textAlign: "center",
          color: textColors[line.type],
          fontSize: "0.8rem",
          fontFamily: "monospace",
          fontWeight: line.type !== "context" ? 600 : 400,
          userSelect: "none",
        }}
      >
        {prefix}
      </Box>
      {/* Content */}
      <Box
        sx={{
          flex: 1,
          px: 1,
          color: textColors[line.type],
          fontSize: "0.8rem",
          fontFamily: "'SF Mono', 'Cascadia Code', 'Fira Code', monospace",
          whiteSpace: "pre",
          overflow: "hidden",
          textOverflow: "ellipsis",
        }}
      >
        {line.content}
      </Box>
    </Box>
  );
}

function FileDiffView({ file }: { file: FileDiff }) {
  const [expanded, setExpanded] = useState(true);

  return (
    <Box
      sx={{
        border: "1px solid #30363d",
        borderRadius: "8px",
        overflow: "hidden",
        mb: 2,
      }}
    >
      <FileHeader
        file={file}
        expanded={expanded}
        onToggle={() => setExpanded(!expanded)}
      />
      <Collapse in={expanded}>
        {file.isBinary ? (
          <Box sx={{ px: 2, py: 2 }}>
            <Typography fontSize="0.85rem" color="#8b949e" fontStyle="italic">
              Binary file not shown.
            </Typography>
          </Box>
        ) : (
          <Box sx={{ overflow: "auto" }}>
            {file.hunks.map((hunk, hi) => (
              <Box key={hi}>
                {hunk.header && <HunkHeader header={hunk.header} />}
                {hunk.lines.map((line, li) => (
                  <DiffLineRow key={li} line={line} />
                ))}
              </Box>
            ))}
            {file.hunks.length === 0 && (
              <Box sx={{ px: 2, py: 2 }}>
                <Typography fontSize="0.85rem" color="#8b949e" fontStyle="italic">
                  No changes to display.
                </Typography>
              </Box>
            )}
          </Box>
        )}
      </Collapse>
    </Box>
  );
}

/** Summary bar showing total files, additions, deletions */
function DiffSummary({ files }: { files: FileDiff[] }) {
  const totalAdditions = files.reduce((s, f) => s + f.additions, 0);
  const totalDeletions = files.reduce((s, f) => s + f.deletions, 0);
  const binaryCount = files.filter((f) => f.isBinary).length;

  return (
    <Box
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 2,
        mb: 2,
        py: 1,
        px: 2,
        bgcolor: "#161b22",
        borderRadius: "8px",
        border: "1px solid #21262d",
      }}
    >
      <Typography fontSize="0.85rem" color="#e0e0e6">
        Showing{" "}
        <strong>
          {files.length} changed file{files.length !== 1 ? "s" : ""}
        </strong>
      </Typography>
      <Typography fontSize="0.85rem" color="#7ee787" fontFamily="monospace">
        +{totalAdditions}
      </Typography>
      <Typography fontSize="0.85rem" color="#ff7b72" fontFamily="monospace">
        -{totalDeletions}
      </Typography>
      {binaryCount > 0 && (
        <Typography fontSize="0.85rem" color="#8b949e">
          ({binaryCount} binary)
        </Typography>
      )}
    </Box>
  );
}

/**
 * Diff viewer for an intent. Fetches the combined diff across all checkpoints.
 */
export function IntentDiffViewer({ intentId }: { intentId: string }) {
  const { data, loading, error } = useApi<FileDiff[]>(
    `/api/intents/${intentId}/diff`,
  );

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
        <CircularProgress size={28} />
      </Box>
    );
  }

  if (error) {
    return <Alert severity="error">{error}</Alert>;
  }

  if (!data || data.length === 0) {
    return (
      <Typography color="text.secondary" sx={{ py: 2 }}>
        No diff available.
      </Typography>
    );
  }

  return (
    <>
      <DiffSummary files={data} />
      {data.map((file) => (
        <FileDiffView key={file.path} file={file} />
      ))}
    </>
  );
}

/**
 * Diff viewer for a single commit.
 */
export function CommitDiffViewer({ sha }: { sha: string }) {
  const { data, loading, error } = useApi<FileDiff[]>(
    `/api/commits/${sha}/diff`,
  );

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", py: 2 }}>
        <CircularProgress size={20} />
      </Box>
    );
  }

  if (error) {
    return <Alert severity="error">{error}</Alert>;
  }

  if (!data || data.length === 0) {
    return (
      <Typography color="text.secondary" fontSize="0.85rem" sx={{ py: 1 }}>
        No diff available.
      </Typography>
    );
  }

  return (
    <>
      {data.map((file) => (
        <FileDiffView key={file.path} file={file} />
      ))}
    </>
  );
}
