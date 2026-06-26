import { faBell, faUser } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Button } from "../../../components/ui";

export function HomeHeader({ learner }) {
  return (
    <header className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0">
        <p className="text-sm font-semibold text-[var(--text-muted)]">Xin chào, {learner.name}</p>
        <h1 className="mt-1 text-3xl font-bold leading-tight text-[var(--text-primary)]">
          Tiếp tục hành trình tự học
        </h1>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="icon" aria-label="Thông báo">
          <FontAwesomeIcon icon={faBell} />
        </Button>
        <Button variant="secondary">
          <FontAwesomeIcon icon={faUser} />
          Hồ sơ
        </Button>
      </div>
    </header>
  );
}
