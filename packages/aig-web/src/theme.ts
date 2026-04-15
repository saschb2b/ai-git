import { createTheme } from "@mui/material/styles";

export const theme = createTheme({
  palette: {
    mode: "dark",
    primary: {
      main: "#646cff",
      light: "#8b91ff",
      dark: "#4549b2",
    },
    secondary: {
      main: "#a78bfa",
    },
    background: {
      default: "#0a0a0f",
      paper: "#13131a",
    },
    success: { main: "#7ee787" },
    error: { main: "#ff7b72" },
    warning: { main: "#e3b341" },
    text: {
      primary: "#e0e0e6",
      secondary: "#9e9eab",
    },
  },
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
    h5: { fontWeight: 600 },
    h6: { fontWeight: 600 },
  },
  shape: {
    borderRadius: 8,
  },
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        body: {
          scrollbarColor: "#646cff #13131a",
        },
      },
    },
    MuiDrawer: {
      styleOverrides: {
        paper: {
          backgroundColor: "#0f0f17",
          borderRight: "1px solid rgba(100, 108, 255, 0.12)",
        },
      },
    },
    MuiDataGrid: {
      styleOverrides: {
        root: {
          border: "none",
        },
      },
    },
  },
});
