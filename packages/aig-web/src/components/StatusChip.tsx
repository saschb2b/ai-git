import Chip from "@mui/material/Chip";

export function StatusChip({ closed }: { closed: boolean }) {
  return (
    <Chip
      label={closed ? "done" : "active"}
      size="small"
      color={closed ? "default" : "success"}
      variant="outlined"
      sx={{ fontWeight: 500, fontSize: "0.75rem" }}
    />
  );
}
