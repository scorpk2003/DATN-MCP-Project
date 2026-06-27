import { faBars, faBookOpen, faXmark } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useState } from "react";
import { useLocation } from "react-router-dom";
import { Button } from "../../components/ui";
import MainPage from "../MainPage";
import Sidebar from "../Sidebar";

function AppShell() {
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const location = useLocation();

  const closeMobileSidebar = () => {
    setMobileSidebarOpen(false);
  };

  return (
    <div className="min-h-screen bg-[var(--bg-canvas)] text-[var(--text-primary)] lg:grid lg:grid-cols-[280px_minmax(0,1fr)]">
      <div className="sticky top-0 z-30 flex items-center justify-between border-b border-[var(--border-primary)] bg-[var(--bg-surface)] px-4 py-3 shadow-[var(--shadow-sm)] lg:hidden">
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-[var(--bg-inverse)] text-[var(--text-inverse)]">
            <FontAwesomeIcon icon={faBookOpen} />
          </div>
          <div className="min-w-0">
            <p className="truncate text-sm font-bold text-[var(--text-primary)]">SelfLearn</p>
            <p className="truncate text-xs text-[var(--text-muted)]">{location.pathname}</p>
          </div>
        </div>
        <Button
          variant="secondary"
          size="icon"
          aria-label="Mở menu"
          onClick={() => setMobileSidebarOpen(true)}
        >
          <FontAwesomeIcon icon={faBars} />
        </Button>
      </div>

      <div className="hidden lg:block">
        <Sidebar />
      </div>

      {mobileSidebarOpen ? (
        <div className="fixed inset-0 z-40 lg:hidden">
          <button
            type="button"
            className="absolute inset-0 cursor-pointer bg-[var(--overlay-scrim)]"
            aria-label="Đóng menu"
            onClick={closeMobileSidebar}
          />
          <div className="absolute inset-y-0 left-0 flex w-[min(82vw,320px)] flex-col shadow-[var(--shadow-lg)]">
            <div className="flex items-center justify-between bg-[var(--bg-inverse)] px-4 py-3 text-[var(--text-inverse)]">
              <p className="text-sm font-bold">Menu</p>
              <Button variant="ghost" size="icon" aria-label="Đóng menu" onClick={closeMobileSidebar}>
                <FontAwesomeIcon icon={faXmark} />
              </Button>
            </div>
            <div onClick={closeMobileSidebar} className="min-h-0 flex-1 overflow-y-auto">
              <Sidebar />
            </div>
          </div>
        </div>
      ) : null}

      <MainPage />
    </div>
  );
}

export default AppShell;
