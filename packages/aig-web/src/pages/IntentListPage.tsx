import { useNavigate } from "react-router";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import { DataGrid, type GridColDef } from "@mui/x-data-grid";
import { StatusChip } from "../components/StatusChip";
import { useApi } from "../hooks/useApi";

interface IntentListItem {
  id: string;
  description: string;
  checkpoint_count: number;
  created_at: string;
  closed_at: string | null;
}

const columns: GridColDef<IntentListItem>[] = [
  {
    field: "status",
    headerName: "",
    width: 80,
    sortable: false,
    renderCell: (params) => (
      <StatusChip closed={params.row.closed_at !== null} />
    ),
  },
  {
    field: "description",
    headerName: "Intent",
    flex: 1,
    minWidth: 300,
  },
  {
    field: "checkpoint_count",
    headerName: "Checkpoints",
    width: 120,
    align: "center",
    headerAlign: "center",
  },
  {
    field: "created_at",
    headerName: "Created",
    width: 180,
    valueFormatter: (value: string) => {
      return new Date(value).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    },
  },
];

export function IntentListPage() {
  const { data, loading, error } = useApi<IntentListItem[]>("/api/intents");
  const navigate = useNavigate();

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

  return (
    <DataGrid
      rows={data ?? []}
      columns={columns}
      autoHeight
      disableRowSelectionOnClick
      onRowClick={(params) => navigate(`/intents/${params.row.id}`)}
      sx={{
        cursor: "pointer",
        "& .MuiDataGrid-row:hover": {
          bgcolor: "rgba(100, 108, 255, 0.04)",
        },
      }}
      initialState={{
        sorting: {
          sortModel: [{ field: "created_at", sort: "desc" }],
        },
      }}
      pageSizeOptions={[25, 50, 100]}
    />
  );
}
