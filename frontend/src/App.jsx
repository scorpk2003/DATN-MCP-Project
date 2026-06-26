import AppShell from "./layouts/AppShell";
import HomePage from "./layouts/HomePage";
import ReviewPage from "./layouts/ReviewPage";
import RoadmapPage from "./layouts/RoadmapPage";
import { Card, SectionTitle } from "./components/ui";
import { Route, Routes } from "react-router-dom";

function PlaceholderPage({ title }) {
  return (
    <div className="mx-auto w-full max-w-[var(--layout-max-width)]">
      <Card className="p-6">
        <SectionTitle eyebrow="Coming soon" title={title} />
      </Card>
    </div>
  );
}

function App() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route path="/" element={<HomePage />} />
        <Route path="/roadmap" element={<RoadmapPage />} />
        <Route path="/review" element={<ReviewPage />} />
        <Route path="/notes" element={<PlaceholderPage title="Ghi chú học tập" />} />
        <Route path="/resources" element={<PlaceholderPage title="Kho tài liệu" />} />
      </Route>
    </Routes>
  );
}

export default App;
