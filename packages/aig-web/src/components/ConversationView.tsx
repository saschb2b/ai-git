import { useState, useMemo } from "react";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import TextField from "@mui/material/TextField";
import InputAdornment from "@mui/material/InputAdornment";
import Collapse from "@mui/material/Collapse";
import Stack from "@mui/material/Stack";
import Chip from "@mui/material/Chip";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import IconButton from "@mui/material/IconButton";
import Tooltip from "@mui/material/Tooltip";
import SearchIcon from "@mui/icons-material/Search";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import ExpandLessIcon from "@mui/icons-material/ExpandLess";
import PersonIcon from "@mui/icons-material/Person";
import SmartToyIcon from "@mui/icons-material/SmartToy";
import UnfoldMoreIcon from "@mui/icons-material/UnfoldMore";
import UnfoldLessIcon from "@mui/icons-material/UnfoldLess";
import Markdown from "react-markdown";

interface ConversationEntry {
  id: string;
  message: string;
  created_at: string;
}

interface ParsedMessage {
  role: "human" | "assistant";
  text: string;
  id: string;
  created_at: string;
}

interface Turn {
  human: ParsedMessage | null;
  assistant: ParsedMessage[];
}

type FilterMode = "all" | "human" | "assistant";

function parseMessage(entry: ConversationEntry): ParsedMessage {
  if (entry.message.startsWith("[User]")) {
    return { role: "human", text: entry.message.slice(6).trimStart(), id: entry.id, created_at: entry.created_at };
  }
  if (entry.message.startsWith("[AI]")) {
    return { role: "assistant", text: entry.message.slice(4).trimStart(), id: entry.id, created_at: entry.created_at };
  }
  return { role: "assistant", text: entry.message, id: entry.id, created_at: entry.created_at };
}

function groupIntoTurns(messages: ParsedMessage[]): Turn[] {
  const turns: Turn[] = [];
  let current: Turn = { human: null, assistant: [] };

  for (const msg of messages) {
    if (msg.role === "human") {
      // Start a new turn if the current one already has content
      if (current.human || current.assistant.length > 0) {
        turns.push(current);
        current = { human: null, assistant: [] };
      }
      current.human = msg;
    } else {
      current.assistant.push(msg);
    }
  }

  // Flush last turn
  if (current.human || current.assistant.length > 0) {
    turns.push(current);
  }

  return turns;
}

function TurnView({ turn, index, defaultExpanded }: { turn: Turn; index: number; defaultExpanded: boolean }) {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const hasAssistant = turn.assistant.length > 0;
  const totalAssistantLength = turn.assistant.reduce((s, m) => s + m.text.length, 0);

  return (
    <Box
      sx={{
        border: "1px solid",
        borderColor: expanded ? "rgba(100, 108, 255, 0.15)" : "rgba(255,255,255,0.06)",
        borderRadius: "8px",
        overflow: "hidden",
        mb: 1,
        transition: "border-color 0.2s",
      }}
    >
      {/* Human message header — always visible */}
      <Box
        onClick={() => hasAssistant && setExpanded(!expanded)}
        sx={{
          display: "flex",
          alignItems: "flex-start",
          gap: 1.5,
          px: 2,
          py: 1.5,
          bgcolor: "rgba(100, 108, 255, 0.04)",
          cursor: hasAssistant ? "pointer" : "default",
          "&:hover": hasAssistant ? { bgcolor: "rgba(100, 108, 255, 0.07)" } : {},
        }}
      >
        <PersonIcon sx={{ fontSize: 18, color: "#646cff", mt: 0.3, flexShrink: 0 }} />
        <Box sx={{ flex: 1, minWidth: 0 }}>
          <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 0.3 }}>
            <Typography fontSize="0.72rem" fontWeight={700} color="primary.main">
              You
            </Typography>
            <Typography fontSize="0.65rem" color="text.disabled">
              Turn {index + 1}
            </Typography>
          </Stack>
          <Typography fontSize="0.85rem" color="text.primary" sx={{ wordBreak: "break-word" }}>
            {turn.human?.text ?? "(no user message)"}
          </Typography>
        </Box>
        {hasAssistant && (
          <Box sx={{ display: "flex", alignItems: "center", gap: 0.5, flexShrink: 0, mt: 0.3 }}>
            <Chip
              size="small"
              label={`${turn.assistant.length} response${turn.assistant.length > 1 ? "s" : ""}`}
              sx={{ fontSize: "0.68rem", height: 22, color: "#8b949e" }}
              variant="outlined"
            />
            {expanded ? (
              <ExpandLessIcon sx={{ fontSize: 18, color: "#8b949e" }} />
            ) : (
              <ExpandMoreIcon sx={{ fontSize: 18, color: "#8b949e" }} />
            )}
          </Box>
        )}
      </Box>

      {/* Assistant responses — collapsible */}
      {hasAssistant && (
        <Collapse in={expanded}>
          <Box sx={{ borderTop: "1px solid rgba(255,255,255,0.04)" }}>
            {turn.assistant.map((msg) => (
              <Box
                key={msg.id}
                sx={{
                  display: "flex",
                  gap: 1.5,
                  px: 2,
                  py: 1.5,
                  borderBottom: "1px solid rgba(255,255,255,0.02)",
                  "&:last-child": { borderBottom: "none" },
                }}
              >
                <SmartToyIcon sx={{ fontSize: 18, color: "#8b949e", mt: 0.3, flexShrink: 0 }} />
                <Box
                  sx={{
                    flex: 1,
                    minWidth: 0,
                    "& p": { my: 0.5, fontSize: "0.82rem", color: "#c9d1d9", lineHeight: 1.6 },
                    "& code": {
                      bgcolor: "rgba(100,108,255,0.08)",
                      px: 0.5,
                      borderRadius: "3px",
                      fontSize: "0.78rem",
                      fontFamily: "'SF Mono','Cascadia Code','Fira Code',monospace",
                    },
                    "& pre": {
                      bgcolor: "#0d1117",
                      border: "1px solid #21262d",
                      borderRadius: "6px",
                      p: 1.5,
                      overflow: "auto",
                      my: 1,
                      "& code": {
                        bgcolor: "transparent",
                        px: 0,
                        fontSize: "0.78rem",
                        display: "block",
                        whiteSpace: "pre",
                      },
                    },
                    "& ul, & ol": { pl: 2.5, my: 0.5, fontSize: "0.82rem", color: "#c9d1d9" },
                    "& h1, & h2, & h3, & h4": { color: "#e0e0e6", mt: 1.5, mb: 0.5, fontSize: "0.95rem" },
                    "& a": { color: "#646cff" },
                    "& blockquote": {
                      borderLeft: "3px solid #30363d",
                      pl: 1.5,
                      ml: 0,
                      color: "#8b949e",
                    },
                    "& table": { borderCollapse: "collapse", my: 1, fontSize: "0.8rem" },
                    "& th, & td": { border: "1px solid #30363d", px: 1, py: 0.5 },
                  }}
                >
                  <Markdown>{msg.text}</Markdown>
                </Box>
              </Box>
            ))}
          </Box>
        </Collapse>
      )}
    </Box>
  );
}

export function ConversationView({ conversations }: { conversations: ConversationEntry[] }) {
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<FilterMode>("all");
  const [allExpanded, setAllExpanded] = useState(false);

  const parsed = useMemo(() => conversations.map(parseMessage), [conversations]);
  const turns = useMemo(() => groupIntoTurns(parsed), [parsed]);

  // Filter turns
  const filteredTurns = useMemo(() => {
    let result = turns;

    if (search) {
      const q = search.toLowerCase();
      result = result.filter((t) => {
        const humanMatch = t.human?.text.toLowerCase().includes(q);
        const assistantMatch = t.assistant.some((m) => m.text.toLowerCase().includes(q));
        return humanMatch || assistantMatch;
      });
    }

    return result;
  }, [turns, search]);

  // For "human only" mode, show just the human messages as a flat list
  const humanMessages = useMemo(
    () => parsed.filter((m) => m.role === "human"),
    [parsed],
  );

  if (conversations.length === 0) {
    return (
      <Typography color="text.secondary" sx={{ py: 4, textAlign: "center" }}>
        No conversations recorded.
      </Typography>
    );
  }

  return (
    <>
      {/* Toolbar */}
      <Stack
        direction={{ xs: "column", sm: "row" }}
        spacing={1.5}
        alignItems={{ sm: "center" }}
        sx={{ mb: 2 }}
      >
        <TextField
          size="small"
          placeholder="Search conversations..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          sx={{ width: { xs: "100%", sm: 280 } }}
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon fontSize="small" />
                </InputAdornment>
              ),
            },
          }}
        />
        <ToggleButtonGroup
          size="small"
          value={filter}
          exclusive
          onChange={(_, v) => v && setFilter(v)}
          sx={{
            "& .MuiToggleButton-root": {
              px: 1.5,
              py: 0.3,
              fontSize: "0.75rem",
              textTransform: "none",
              color: "#8b949e",
              borderColor: "#30363d",
              "&.Mui-selected": { color: "#e0e0e6", bgcolor: "rgba(100,108,255,0.12)" },
            },
          }}
        >
          <ToggleButton value="all">All turns</ToggleButton>
          <ToggleButton value="human">My messages</ToggleButton>
        </ToggleButtonGroup>
        <Box sx={{ flex: 1 }} />
        <Stack direction="row" spacing={0.5} alignItems="center">
          <Typography fontSize="0.75rem" color="text.disabled">
            {filter === "human" ? `${humanMessages.length} messages` : `${filteredTurns.length} turns`}
          </Typography>
          {filter === "all" && (
            <Tooltip title={allExpanded ? "Collapse all" : "Expand all"}>
              <IconButton size="small" onClick={() => setAllExpanded(!allExpanded)} sx={{ color: "#8b949e" }}>
                {allExpanded ? <UnfoldLessIcon fontSize="small" /> : <UnfoldMoreIcon fontSize="small" />}
              </IconButton>
            </Tooltip>
          )}
        </Stack>
      </Stack>

      {/* Content */}
      <Box sx={{ maxHeight: "70vh", overflow: "auto" }}>
        {filter === "human" ? (
          // Flat list of just human messages
          humanMessages
            .filter((m) => !search || m.text.toLowerCase().includes(search.toLowerCase()))
            .map((msg, i) => (
              <Box
                key={msg.id}
                sx={{
                  display: "flex",
                  gap: 1.5,
                  px: 2,
                  py: 1.2,
                  borderBottom: "1px solid rgba(255,255,255,0.04)",
                  bgcolor: "rgba(100, 108, 255, 0.03)",
                }}
              >
                <PersonIcon sx={{ fontSize: 18, color: "#646cff", mt: 0.2, flexShrink: 0 }} />
                <Box>
                  <Typography fontSize="0.68rem" color="text.disabled" sx={{ mb: 0.3 }}>
                    Message {i + 1}
                  </Typography>
                  <Typography fontSize="0.85rem" color="text.primary" sx={{ wordBreak: "break-word" }}>
                    {msg.text}
                  </Typography>
                </Box>
              </Box>
            ))
        ) : (
          // Grouped turn view
          filteredTurns.map((turn, i) => (
            <TurnView key={turn.human?.id ?? turn.assistant[0]?.id ?? i} turn={turn} index={i} defaultExpanded={allExpanded} />
          ))
        )}

        {filter === "human" && humanMessages.length === 0 && (
          <Typography color="text.secondary" sx={{ py: 4, textAlign: "center" }}>
            No user messages found.
          </Typography>
        )}
        {filter === "all" && filteredTurns.length === 0 && (
          <Typography color="text.secondary" sx={{ py: 4, textAlign: "center" }}>
            {search ? "No matching conversations." : "No conversations recorded."}
          </Typography>
        )}
      </Box>
    </>
  );
}
