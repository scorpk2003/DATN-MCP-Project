import { ProtectedRoute } from "./auth/ProtectedRoute.jsx";
import { LoadingState } from "./components/ui";
import { lazy, Suspense } from "react";
import { Route, Routes } from "react-router-dom";

const AppShell = lazy(() => import("./layouts/AppShell"));
const HomePage = lazy(() => import("./layouts/HomePage"));
const LoginPage = lazy(() => import("./layouts/LoginPage"));
const NotesPage = lazy(() => import("./layouts/NotesPage"));
const ResourcesPage = lazy(() => import("./layouts/ResourcesPage"));
const ReviewPage = lazy(() => import("./layouts/ReviewPage"));
const RoadmapPage = lazy(() => import("./layouts/RoadmapPage"));

function App() {
  return (
    <Suspense fallback={<LoadingState layout="dashboard" title="Đang tải giao diện..." />}>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route element={<ProtectedRoute />}>
          <Route element={<AppShell />}>
            <Route path="/" element={<HomePage />} />
            <Route path="/roadmap" element={<RoadmapPage />} />
            <Route path="/review" element={<ReviewPage />} />
            <Route path="/notes" element={<NotesPage />} />
            <Route path="/resources" element={<ResourcesPage />} />
          </Route>
        </Route>
      </Routes>
    </Suspense>
  );
}

export default App;
