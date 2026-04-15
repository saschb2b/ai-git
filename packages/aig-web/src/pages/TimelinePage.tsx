import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import { BarChart } from "@mui/x-charts/BarChart";
import { useApi } from "../hooks/useApi";

interface TimelinePoint {
  date: string;
  checkpoints: number;
  intents_opened: number;
  intents_closed: number;
}

export function TimelinePage() {
  const { data, loading, error } = useApi<TimelinePoint[]>("/api/timeline");

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", py: 8 }}>
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return <Alert severity="error">{error}</Alert>;
  }

  if (!data || data.length === 0) {
    return (
      <Typography color="text.secondary" sx={{ py: 4, textAlign: "center" }}>
        No timeline data yet. Create some checkpoints to see activity.
      </Typography>
    );
  }

  const dates = data.map((d) => d.date);
  const checkpoints = data.map((d) => d.checkpoints);
  const opened = data.map((d) => d.intents_opened);
  const closed = data.map((d) => d.intents_closed);

  return (
    <>
      <Typography variant="h6" sx={{ mb: 3 }}>
        Activity
      </Typography>
      <Box sx={{ width: "100%", height: 400 }}>
        <BarChart
          xAxis={[
            {
              data: dates,
              scaleType: "band",
              label: "Date",
            },
          ]}
          series={[
            {
              data: checkpoints,
              label: "Checkpoints",
              color: "#646cff",
            },
            {
              data: opened,
              label: "Intents opened",
              color: "#7ee787",
            },
            {
              data: closed,
              label: "Intents closed",
              color: "#8b949e",
            },
          ]}
          height={400}
        />
      </Box>
    </>
  );
}
