import { useParams, useNavigate } from "react-router";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemText from "@mui/material/ListItemText";
import Chip from "@mui/material/Chip";
import LinearProgress from "@mui/material/LinearProgress";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Stack from "@mui/material/Stack";
import Breadcrumbs from "@mui/material/Breadcrumbs";
import Link from "@mui/material/Link";
import Collapse from "@mui/material/Collapse";
import TextField from "@mui/material/TextField";
import InputAdornment from "@mui/material/InputAdornment";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import SearchIcon from "@mui/icons-material/Search";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import ExpandLessIcon from "@mui/icons-material/ExpandLess";
import { useState } from "react";
import { StatusChip } from "../components/StatusChip";
import { IntentDiffViewer, CommitDiffViewer } from "../components/DiffViewer";
import { useApi } from "../hooks/useApi";

interface IntentDetail {
  intent: {
    id: string;
    description: string;
    created_at: string;
    closed_at: string | null;
    summary: string | null;
  };
  checkpoints: {
    id: string;
    message: string;
    git_commit_sha: string;
    created_at: string;
  }[];
  semanticChanges: { file_path: string; change_type: string; symbol_name: string }[];
  changesByCheckpoint: Record<string, unknown[]>;
  conversations: {
    id: string;
    message: string;
    created_at: string;
  }[];
  provenance: {
    file_path: string;
    origin: string;
    reviewed: number;
    start_line: number;
    end_line: number;
  }[];
  session: { started_at: string; ended_at: string | null } | null;
  filesChanged: number;
}


const GITHUB_REPO = "https://github.com/saschb2b/ai-git";

function formatDuration(startStr: string, endStr: string | null): string {
  const start = new Date(startStr).getTime();
  const end = endStr ? new Date(endStr).getTime() : Date.now();
  const ms = end - start;
  const mins = Math.floor(ms / 60000);
  const hours = Math.floor(mins / 60);
  const days = Math.floor(hours / 24);
  if (days > 0) return `${days}d ${hours % 24}h`;
  if (hours > 0) return `${hours}h ${mins % 60}m`;
  return `${mins}m`;
}

function TabPanel({
  children,
  value,
  index,
}: {
  children: React.ReactNode;
  value: number;
  index: number;
}) {
  if (value !== index) return null;
  return <Box sx={{ py: 2 }}>{children}</Box>;
}

export function IntentDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, loading, error } = useApi<IntentDetail>(`/api/intents/${id}`);
  const [tab, setTab] = useState(0);
  const [expandedCps, setExpandedCps] = useState<Set<string>>(new Set());
  const [convSearch, setConvSearch] = useState("");
  const navigate = useNavigate();

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", py: 8 }}>
        <CircularProgress />
      </Box>
    );
  }

  if (error || !data) {
    return <Alert severity="error">{error ?? "Intent not found"}</Alert>;
  }

  const {
    intent,
    checkpoints,
    semanticChanges,
    conversations,
    provenance,
    session,
    filesChanged,
  } = data;

  const toggleCp = (cpId: string) => {
    setExpandedCps((prev) => {
      const next = new Set(prev);
      if (next.has(cpId)) next.delete(cpId);
      else next.add(cpId);
      return next;
    });
  };

  // Provenance stats
  const humanCount = provenance.filter((p) => p.origin === "human").length;
  const aiCount = provenance.filter((p) => p.origin !== "human").length;
  const reviewedCount = provenance.filter((p) => p.reviewed).length;
  const reviewPct =
    provenance.length > 0 ? (reviewedCount / provenance.length) * 100 : 0;

  // Duration
  const duration = session
    ? formatDuration(session.started_at, session.ended_at)
    : intent.closed_at
      ? formatDuration(intent.created_at, intent.closed_at)
      : null;

  // Filter conversations
  const filteredConvs = convSearch
    ? conversations.filter((c) =>
        c.message.toLowerCase().includes(convSearch.toLowerCase()),
      )
    : conversations;

  return (
    <>
      {/* Breadcrumb */}
      <Breadcrumbs sx={{ mb: 2 }}>
        <Link
          component="button"
          underline="hover"
          color="text.secondary"
          onClick={() => navigate("/")}
          sx={{ display: "flex", alignItems: "center", gap: 0.5 }}
        >
          <ArrowBackIcon fontSize="small" />
          Intents
        </Link>
        <Typography color="text.primary" fontSize="0.875rem">
          {intent.description.length > 50
            ? intent.description.slice(0, 50) + "..."
            : intent.description}
        </Typography>
      </Breadcrumbs>

      {/* Header */}
      <Box sx={{ mb: 3 }}>
        <Stack direction="row" spacing={2} alignItems="center" sx={{ mb: 1 }}>
          <Typography variant="h5">{intent.description}</Typography>
          <StatusChip closed={intent.closed_at !== null} />
        </Stack>
        {intent.summary && (
          <Typography
            variant="body2"
            sx={{ mt: 1, color: "text.secondary", maxWidth: 800 }}
          >
            {intent.summary}
          </Typography>
        )}
      </Box>

      {/* Stat cards */}
      <Stack direction="row" spacing={2} sx={{ mb: 3 }} flexWrap="wrap" useFlexGap>
        <Card sx={{ minWidth: 120 }}>
          <CardContent sx={{ py: 1.5, "&:last-child": { pb: 1.5 } }}>
            <Typography color="text.secondary" variant="caption">
              Checkpoints
            </Typography>
            <Typography variant="h5">{checkpoints.length}</Typography>
          </CardContent>
        </Card>
        <Card sx={{ minWidth: 120 }}>
          <CardContent sx={{ py: 1.5, "&:last-child": { pb: 1.5 } }}>
            <Typography color="text.secondary" variant="caption">
              Files changed
            </Typography>
            <Typography variant="h5">{filesChanged}</Typography>
          </CardContent>
        </Card>
        <Card sx={{ minWidth: 120 }}>
          <CardContent sx={{ py: 1.5, "&:last-child": { pb: 1.5 } }}>
            <Typography color="text.secondary" variant="caption">
              Changes
            </Typography>
            <Typography variant="h5">{semanticChanges.length}</Typography>
          </CardContent>
        </Card>
        {duration && (
          <Card sx={{ minWidth: 120 }}>
            <CardContent sx={{ py: 1.5, "&:last-child": { pb: 1.5 } }}>
              <Typography color="text.secondary" variant="caption">
                Duration
              </Typography>
              <Typography variant="h5">{duration}</Typography>
            </CardContent>
          </Card>
        )}
        {conversations.length > 0 && (
          <Card sx={{ minWidth: 120 }}>
            <CardContent sx={{ py: 1.5, "&:last-child": { pb: 1.5 } }}>
              <Typography color="text.secondary" variant="caption">
                Conversations
              </Typography>
              <Typography variant="h5">{conversations.length}</Typography>
            </CardContent>
          </Card>
        )}
      </Stack>

      <Tabs value={tab} onChange={(_, v) => setTab(v)}>
        <Tab label="Checkpoints" />
        <Tab label="Diff" />
        <Tab label={`Conversations (${conversations.length})`} />
        <Tab label="Trust" />
      </Tabs>

      {/* Checkpoints tab — with inline git diff */}
      <TabPanel value={tab} index={0}>
        {checkpoints.map((cp) => {
          const isExpanded = expandedCps.has(cp.id);
          return (
            <Box
              key={cp.id}
              sx={{
                borderLeft: "2px solid",
                borderColor: "primary.main",
                pl: 2,
                mb: 3,
                py: 1,
              }}
            >
              <Stack direction="row" alignItems="center" spacing={1}>
                <Box sx={{ flex: 1 }}>
                  <Typography variant="body1" fontWeight={500}>
                    {cp.message}
                  </Typography>
                  <Typography variant="caption" color="text.secondary" fontFamily="monospace">
                    <Link
                      href={`${GITHUB_REPO}/commit/${cp.git_commit_sha}`}
                      target="_blank"
                      rel="noopener"
                      color="inherit"
                      underline="hover"
                    >
                      {cp.git_commit_sha.slice(0, 8)}
                    </Link>
                    {" · "}
                    {new Date(cp.created_at).toLocaleString()}
                  </Typography>
                </Box>
                <Chip
                  label={isExpanded ? "Hide diff" : "Show diff"}
                  size="small"
                  variant="outlined"
                  onClick={() => toggleCp(cp.id)}
                  onDelete={() => toggleCp(cp.id)}
                  deleteIcon={isExpanded ? <ExpandLessIcon /> : <ExpandMoreIcon />}
                  sx={{ cursor: "pointer" }}
                />
              </Stack>
              <Collapse in={isExpanded}>
                <Box sx={{ mt: 2 }}>
                  <CommitDiffViewer sha={cp.git_commit_sha} />
                </Box>
              </Collapse>
            </Box>
          );
        })}
        {checkpoints.length === 0 && (
          <Typography color="text.secondary" sx={{ py: 2 }}>
            No checkpoints yet.
          </Typography>
        )}
      </TabPanel>

      {/* Diff tab — combined diff across all checkpoints */}
      <TabPanel value={tab} index={1}>
        <IntentDiffViewer intentId={intent.id} />
      </TabPanel>

      {/* Conversations tab — searchable, collapsible */}
      <TabPanel value={tab} index={2}>
        {conversations.length > 10 && (
          <TextField
            size="small"
            placeholder="Search conversations..."
            value={convSearch}
            onChange={(e) => setConvSearch(e.target.value)}
            sx={{ mb: 2, width: 300 }}
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
        )}
        {convSearch && (
          <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: "block" }}>
            {filteredConvs.length} of {conversations.length} messages
          </Typography>
        )}
        <List sx={{ maxHeight: 600, overflow: "auto" }}>
          {filteredConvs.map((c) => (
            <ListItem key={c.id} divider sx={{ alignItems: "flex-start" }}>
              <ListItemText
                primary={
                  <Typography
                    fontSize="0.85rem"
                    sx={{
                      whiteSpace: "pre-wrap",
                      wordBreak: "break-word",
                      maxHeight: 200,
                      overflow: "hidden",
                    }}
                  >
                    {c.message}
                  </Typography>
                }
                secondary={new Date(c.created_at).toLocaleString()}
              />
            </ListItem>
          ))}
          {filteredConvs.length === 0 && (
            <Typography color="text.secondary" sx={{ py: 2 }}>
              {convSearch ? "No matching messages." : "No conversations recorded."}
            </Typography>
          )}
        </List>
      </TabPanel>

      {/* Trust tab */}
      <TabPanel value={tab} index={3}>
        {provenance.length > 0 ? (
          <Stack direction="row" spacing={2} sx={{ mb: 3 }}>
            <Card sx={{ flex: 1 }}>
              <CardContent>
                <Typography color="text.secondary" variant="body2">
                  Human
                </Typography>
                <Typography variant="h4">{humanCount}</Typography>
              </CardContent>
            </Card>
            <Card sx={{ flex: 1 }}>
              <CardContent>
                <Typography color="text.secondary" variant="body2">
                  AI-assisted
                </Typography>
                <Typography variant="h4">{aiCount}</Typography>
              </CardContent>
            </Card>
            <Card sx={{ flex: 1 }}>
              <CardContent>
                <Typography color="text.secondary" variant="body2">
                  Reviewed
                </Typography>
                <Typography variant="h4">
                  {Math.round(reviewPct)}%
                </Typography>
                <LinearProgress
                  variant="determinate"
                  value={reviewPct}
                  sx={{ mt: 1 }}
                />
              </CardContent>
            </Card>
          </Stack>
        ) : (
          <Typography color="text.secondary" sx={{ py: 2 }}>
            No provenance data recorded.
          </Typography>
        )}
      </TabPanel>
    </>
  );
}
