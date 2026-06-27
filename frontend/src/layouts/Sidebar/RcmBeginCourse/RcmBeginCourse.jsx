import {
  faArrowRightFromBracket,
  faBookOpen,
  faEllipsisVertical,
  faPlus,
} from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { NavLink } from "react-router-dom";
import { useAuth } from "../../../auth/useAuth.js";
import { Button, InlineAlert, Skeleton } from "../../../components/ui";
import { useNavigationData } from "../../../hooks/useNavigationData.js";

function RcmBeginCourse() {
  const { signOut, user } = useAuth();
  const { data, error, loading } = useNavigationData();
  const { recentChats, sidebarItems } = data;

  return (
    <aside className="flex h-full min-h-0 flex-col gap-6 bg-[var(--bg-inverse)] px-4 py-5 text-[var(--text-inverse)] lg:min-h-screen lg:sticky lg:top-0">
      <div className="flex items-center gap-3 px-2">
        <div className="flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-[var(--sl-inverse-surface)]">
          <FontAwesomeIcon icon={faBookOpen} />
        </div>
        <div className="min-w-0">
          <p className="text-lg font-bold leading-tight">SelfLearn</p>
          <p className="truncate text-xs text-[var(--sl-sidebar-text-muted)]">Adaptive study agent</p>
        </div>
      </div>

      <Button variant="accent" full>
        <FontAwesomeIcon icon={faPlus} />
        Lộ trình mới
      </Button>

      <nav className="space-y-1" aria-label="Điều hướng chính">
        {loading ? (
          <div className="space-y-2 px-3 py-2">
            <Skeleton className="h-3 w-4/5 bg-[var(--sl-sidebar-hover-bg)]" />
            <Skeleton className="h-3 w-3/5 bg-[var(--sl-sidebar-hover-bg)]" />
          </div>
        ) : null}
        {sidebarItems.map((item) => (
          <NavLink
            key={item.id}
            to={item.path}
            end={item.path === "/"}
            className={({ isActive }) =>
              `flex w-full items-center rounded-[var(--radius-md)] px-3 py-2 text-left text-sm font-semibold transition-colors ${
                isActive
                  ? "bg-[var(--sl-sidebar-hover-bg)] text-[var(--text-inverse)]"
                  : "text-[var(--sl-sidebar-text-muted)] hover:bg-[var(--sl-sidebar-hover-bg)] hover:text-[var(--text-inverse)]"
              }`
            }
          >
            <span className="truncate">{item.label}</span>
          </NavLink>
        ))}
      </nav>

      <section className="min-h-0 flex-1">
        <div className="mb-2 flex items-center justify-between px-2">
          <h2 className="text-xs font-bold uppercase tracking-normal text-[var(--sl-sidebar-text-subtle)]">
            Gần đây
          </h2>
        </div>
        <div className="space-y-1 overflow-y-auto pr-1">
          {error ? (
            <InlineAlert title="Không tải được dữ liệu" tone="risk" />
          ) : null}
          {recentChats.map((chat) => (
            <div
              key={chat.id}
              className="flex cursor-pointer items-center gap-2 rounded-[var(--radius-md)] px-3 py-2 text-sm text-[var(--sl-sidebar-text-muted)] hover:bg-[var(--sl-sidebar-hover-bg)] hover:text-[var(--text-inverse)]"
            >
              <span className="min-w-0 flex-1 truncate">{chat.label}</span>
              <FontAwesomeIcon icon={faEllipsisVertical} className="shrink-0 text-xs" />
            </div>
          ))}
        </div>
      </section>

      <div className="rounded-[var(--radius-md)] border border-[var(--sl-sidebar-border)] bg-[var(--sl-inverse-surface)] p-3">
        <p className="truncate text-sm font-bold text-[var(--text-inverse)]">
          {user?.displayName || user?.email || "SelfLearn user"}
        </p>
        {user?.email ? (
          <p className="mt-1 truncate text-xs text-[var(--sl-sidebar-text-muted)]">{user.email}</p>
        ) : null}
        <Button variant="ghost" size="sm" className="mt-3 w-full justify-start" onClick={signOut}>
          <FontAwesomeIcon icon={faArrowRightFromBracket} />
          Đăng xuất
        </Button>
      </div>
    </aside>
  );
}

export default RcmBeginCourse;
