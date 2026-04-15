import { useState, useMemo } from "react";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import TextField from "@mui/material/TextField";
import InputAdornment from "@mui/material/InputAdornment";
import Stack from "@mui/material/Stack";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import SearchIcon from "@mui/icons-material/Search";
import PersonIcon from "@mui/icons-material/Person";
import SmartToyIcon from "@mui/icons-material/SmartToy";
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
}

type FilterMode = "all" | "human";

function parseMessage(entry: ConversationEntry): ParsedMessage {
  if (entry.message.startsWith("[User]")) {
    return { role: "human", text: entry.message.slice(6).trimStart(), id: entry.id };
  }
  if (entry.message.startsWith("[AI]")) {
    return { role: "assistant", text: entry.message.slice(4).trimStart(), id: entry.id };
  }
  return { role: "assistant", text: entry.message, id: entry.id };
}

function ChatMessage({ msg }: { msg: ParsedMessage }) {
  const [expanded, setExpanded] = useState(false);
  const isHuman = msg.role === "human";
  const isLong = msg.text.length > 600;
  const displayText = isLong && !expanded ? msg.text.slice(0, 500) + "\n\n..." : msg.text;

  return (
    <Box
      sx={{
        display: "flex",
        gap: 1.5,
        px: 2,
        py: 1.5,
        bgcolor: isHuman ? "rgba(100, 108, 255, 0.05)" : "transparent",
        borderLeft: isHuman ? "3px solid" : "3px solid transparent",
        borderColor: isHuman ? "primary.main" : "transparent",
        "&:hover": { bgcolor: isHuman ? "rgba(100, 108, 255, 0.07)" : "rgba(255,255,255,0.02)" },
      }}
    >
      {/* Avatar */}
      <Box
        sx={{
          width: 28,
          height: 28,
          borderRadius: "50%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          bgcolor: isHuman ? "rgba(100, 108, 255, 0.15)" : "rgba(255,255,255,0.06)",
          flexShrink: 0,
          mt: 0.2,
        }}
      >
        {isHuman ? (
          <PersonIcon sx={{ fontSize: 16, color: "#646cff" }} />
        ) : (
          <SmartToyIcon sx={{ fontSize: 16, color: "#8b949e" }} />
        )}
      </Box>

      {/* Content */}
      <Box sx={{ flex: 1, minWidth: 0 }}>
        <Typography
          fontSize="0.72rem"
          fontWeight={700}
          color={isHuman ? "primary.main" : "text.secondary"}
          sx={{ mb: 0.3 }}
        >
          {isHuman ? "You" : "Assistant"}
        </Typography>

        {isHuman ? (
          /* Human messages: plain text, slightly larger */
          <Typography
            fontSize="0.88rem"
            color="text.primary"
            fontWeight={500}
            sx={{ wordBreak: "break-word", whiteSpace: "pre-wrap" }}
          >
            {msg.text}
          </Typography>
        ) : (
          /* AI messages: markdown rendered */
          <Box
            sx={{
              "& p": { my: 0.4, fontSize: "0.82rem", color: "#c9d1d9", lineHeight: 1.6 },
              "& p:first-of-type": { mt: 0 },
              "& p:last-of-type": { mb: 0 },
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
                  fontSize: "0.76rem",
                  display: "block",
                  whiteSpace: "pre",
                },
              },
              "& ul, & ol": { pl: 2.5, my: 0.5, fontSize: "0.82rem", color: "#c9d1d9" },
              "& li": { my: 0.2 },
              "& h1, & h2, & h3, & h4": { color: "#e0e0e6", mt: 1, mb: 0.3, fontSize: "0.9rem", fontWeight: 600 },
              "& a": { color: "#646cff" },
              "& blockquote": {
                borderLeft: "3px solid #30363d",
                pl: 1.5,
                ml: 0,
                color: "#8b949e",
              },
              "& table": { borderCollapse: "collapse", my: 1, fontSize: "0.78rem", width: "100%" },
              "& th, & td": { border: "1px solid #30363d", px: 1, py: 0.4, textAlign: "left" },
              "& th": { bgcolor: "rgba(100,108,255,0.06)", color: "#e0e0e6" },
              "& strong": { color: "#e0e0e6" },
            }}
          >
            <Markdown>{displayText}</Markdown>
          </Box>
        )}

        {isLong && (
          <Typography
            component="span"
            fontSize="0.75rem"
            color="primary.main"
            sx={{ cursor: "pointer", mt: 0.5, display: "inline-block", "&:hover": { textDecoration: "underline" } }}
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? "Show less" : "Show more"}
          </Typography>
        )}
      </Box>
    </Box>
  );
}

export function ConversationView({ conversations }: { conversations: ConversationEntry[] }) {
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<FilterMode>("all");

  const parsed = useMemo(() => conversations.map(parseMessage), [conversations]);

  const filtered = useMemo(() => {
    let result = parsed;
    if (filter === "human") {
      result = result.filter((m) => m.role === "human");
    }
    if (search) {
      const q = search.toLowerCase();
      result = result.filter((m) => m.text.toLowerCase().includes(q));
    }
    return result;
  }, [parsed, filter, search]);

  const humanCount = parsed.filter((m) => m.role === "human").length;
  const aiCount = parsed.filter((m) => m.role === "assistant").length;

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
          <ToggleButton value="all">All ({parsed.length})</ToggleButton>
          <ToggleButton value="human">My messages ({humanCount})</ToggleButton>
        </ToggleButtonGroup>
        <Box sx={{ flex: 1 }} />
        <Typography fontSize="0.73rem" color="text.disabled">
          {search ? `${filtered.length} results` : `${humanCount} you, ${aiCount} assistant`}
        </Typography>
      </Stack>

      {/* Chat timeline */}
      <Box
        sx={{
          maxHeight: "70vh",
          overflow: "auto",
          borderRadius: "8px",
          border: "1px solid rgba(255,255,255,0.06)",
          bgcolor: "background.paper",
        }}
      >
        {filtered.map((msg) => (
          <ChatMessage key={msg.id} msg={msg} />
        ))}
        {filtered.length === 0 && (
          <Typography color="text.secondary" sx={{ py: 4, textAlign: "center" }}>
            {search ? "No matching messages." : "No messages to show."}
          </Typography>
        )}
      </Box>
    </>
  );
}
