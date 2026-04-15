import { useParams } from "react-router";
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
import Accordion from "@mui/material/Accordion";
import AccordionSummary from "@mui/material/AccordionSummary";
import AccordionDetails from "@mui/material/AccordionDetails";
import LinearProgress from "@mui/material/LinearProgress";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Stack from "@mui/material/Stack";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import { useState } from "react";
import { StatusChip } from "../components/StatusChip";
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
  semanticChanges: {
    file_path: string;
    change_type: string;
    symbol_name: string;
  }[];
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
}

const CHANGE_COLORS: Record<string, string> = {
  added: "#7ee787",
  removed: "#ff7b72",
  modified: "#e3b341",
};

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

  const { intent, checkpoints, semanticChanges, conversations, provenance } =
    data;

  // Group semantic changes by file
  const changesByFile = semanticChanges.reduce(
    (acc, sc) => {
      if (!acc[sc.file_path]) acc[sc.file_path] = [];
      acc[sc.file_path].push(sc);
      return acc;
    },
    {} as Record<string, typeof semanticChanges>,
  );

  // Provenance stats
  const humanCount = provenance.filter((p) => p.origin === "human").length;
  const aiCount = provenance.filter((p) => p.origin !== "human").length;
  const reviewedCount = provenance.filter((p) => p.reviewed).length;
  const reviewPct =
    provenance.length > 0 ? (reviewedCount / provenance.length) * 100 : 0;

  return (
    <>
      <Box sx={{ mb: 3 }}>
        <Stack direction="row" spacing={2} alignItems="center" sx={{ mb: 1 }}>
          <Typography variant="h5">{intent.description}</Typography>
          <StatusChip closed={intent.closed_at !== null} />
        </Stack>
        <Typography variant="body2" color="text.secondary">
          {new Date(intent.created_at).toLocaleString()}
          {intent.closed_at &&
            ` — ${new Date(intent.closed_at).toLocaleString()}`}
        </Typography>
        {intent.summary && (
          <Typography variant="body1" sx={{ mt: 1, color: "text.secondary" }}>
            {intent.summary}
          </Typography>
        )}
      </Box>

      <Tabs value={tab} onChange={(_, v) => setTab(v)}>
        <Tab label={`Checkpoints (${checkpoints.length})`} />
        <Tab label={`Changes (${semanticChanges.length})`} />
        <Tab label={`Conversations (${conversations.length})`} />
        <Tab label="Trust" />
      </Tabs>

      <TabPanel value={tab} index={0}>
        <List>
          {checkpoints.map((cp) => (
            <ListItem key={cp.id} divider>
              <ListItemText
                primary={cp.message}
                secondary={
                  <Typography
                    variant="caption"
                    component="span"
                    fontFamily="monospace"
                    color="text.secondary"
                  >
                    {cp.git_commit_sha.slice(0, 8)} &middot;{" "}
                    {new Date(cp.created_at).toLocaleString()}
                  </Typography>
                }
              />
            </ListItem>
          ))}
          {checkpoints.length === 0 && (
            <Typography color="text.secondary" sx={{ py: 2 }}>
              No checkpoints yet.
            </Typography>
          )}
        </List>
      </TabPanel>

      <TabPanel value={tab} index={1}>
        {Object.entries(changesByFile).map(([file, changes]) => (
          <Accordion key={file} defaultExpanded>
            <AccordionSummary expandIcon={<ExpandMoreIcon />}>
              <Typography fontFamily="monospace" fontSize="0.875rem">
                {file}
              </Typography>
              <Typography sx={{ ml: "auto", mr: 2 }} color="text.secondary">
                {changes.length}
              </Typography>
            </AccordionSummary>
            <AccordionDetails>
              <List dense>
                {changes.map((sc, i) => (
                  <ListItem key={i}>
                    <Chip
                      label={sc.change_type}
                      size="small"
                      sx={{
                        mr: 1,
                        color: CHANGE_COLORS[sc.change_type] ?? "#9e9eab",
                        borderColor:
                          CHANGE_COLORS[sc.change_type] ?? "#9e9eab",
                        fontWeight: 500,
                        fontSize: "0.7rem",
                      }}
                      variant="outlined"
                    />
                    <Typography fontFamily="monospace" fontSize="0.85rem">
                      {sc.symbol_name}
                    </Typography>
                  </ListItem>
                ))}
              </List>
            </AccordionDetails>
          </Accordion>
        ))}
        {semanticChanges.length === 0 && (
          <Typography color="text.secondary" sx={{ py: 2 }}>
            No semantic changes recorded.
          </Typography>
        )}
      </TabPanel>

      <TabPanel value={tab} index={2}>
        <List>
          {conversations.map((c) => (
            <ListItem key={c.id} divider>
              <ListItemText
                primary={c.message}
                secondary={new Date(c.created_at).toLocaleString()}
                primaryTypographyProps={{ fontSize: "0.875rem" }}
              />
            </ListItem>
          ))}
          {conversations.length === 0 && (
            <Typography color="text.secondary" sx={{ py: 2 }}>
              No conversations recorded.
            </Typography>
          )}
        </List>
      </TabPanel>

      <TabPanel value={tab} index={3}>
        {provenance.length > 0 ? (
          <>
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
          </>
        ) : (
          <Typography color="text.secondary" sx={{ py: 2 }}>
            No provenance data recorded.
          </Typography>
        )}
      </TabPanel>
    </>
  );
}
