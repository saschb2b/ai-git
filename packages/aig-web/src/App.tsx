import { Routes, Route } from "react-router";
import { AppShell } from "./components/AppShell";
import { IntentListPage } from "./pages/IntentListPage";
import { IntentDetailPage } from "./pages/IntentDetailPage";
import { TimelinePage } from "./pages/TimelinePage";
import { GraphPage } from "./pages/GraphPage";

export function App() {
  return (
    <AppShell>
      <Routes>
        <Route path="/" element={<IntentListPage />} />
        <Route path="/intents/:id" element={<IntentDetailPage />} />
        <Route path="/timeline" element={<TimelinePage />} />
        <Route path="/graph" element={<GraphPage />} />
      </Routes>
    </AppShell>
  );
}
