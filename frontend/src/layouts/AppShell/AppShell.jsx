import MainPage from "../MainPage";
import Sidebar from "../Sidebar";
function AppShell() {
  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] text-[var(--text-primary)] lg:grid lg:grid-cols-[280px_minmax(0,1fr)]">
      <Sidebar />
      <MainPage />
    </div>
  );
}

export default AppShell;
