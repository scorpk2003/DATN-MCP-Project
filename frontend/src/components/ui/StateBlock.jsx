import { Card } from "./Card.jsx";
import { Button } from "./Button.jsx";
import { Skeleton, SkeletonCard, SkeletonGrid } from "./Skeleton.jsx";

export function LoadingState({ layout = "default", title = "Đang tải dữ liệu..." }) {
  if (layout === "dashboard") {
    return (
      <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
        <Card className="space-y-4 p-5">
          <p className="text-sm font-semibold text-[var(--text-muted)]">{title}</p>
          <Skeleton className="h-8 w-2/3" />
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-24 w-full rounded-[var(--radius-md)]" />
        </Card>
        <SkeletonGrid count={3} />
      </div>
    );
  }

  if (layout === "list") {
    return (
      <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-4">
        <p className="text-sm font-semibold text-[var(--text-muted)]">{title}</p>
        {Array.from({ length: 4 }).map((_, index) => (
          <SkeletonCard key={index} lines={2} />
        ))}
      </div>
    );
  }

  if (layout === "grid") {
    return (
      <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-4">
        <p className="text-sm font-semibold text-[var(--text-muted)]">{title}</p>
        <SkeletonGrid count={4} />
      </div>
    );
  }

  return (
    <Card className="space-y-3 p-5">
      <p className="text-sm font-semibold text-[var(--text-muted)]">{title}</p>
      <div className="space-y-2">
        <Skeleton className="h-3 w-2/3" />
        <Skeleton className="h-3 w-1/2" />
      </div>
    </Card>
  );
}

export function ErrorState({
  description = "Vui lòng thử lại sau ít phút.",
  onRetry,
  title = "Không thể tải dữ liệu",
}) {
  return (
    <Card className="flex flex-col gap-3 p-5 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p className="text-sm font-semibold text-[var(--color-risk)]">{title}</p>
        <p className="mt-1 text-sm text-[var(--text-secondary)]">{description}</p>
      </div>
      {onRetry ? (
        <Button variant="secondary" onClick={onRetry}>
          Thử lại
        </Button>
      ) : null}
    </Card>
  );
}

export function EmptyState({ action, description, title = "Chưa có dữ liệu" }) {
  return (
    <Card className="flex flex-col gap-3 p-5 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p className="text-sm font-semibold text-[var(--text-muted)]">{title}</p>
        {description ? (
          <p className="mt-1 text-sm text-[var(--text-secondary)]">{description}</p>
        ) : null}
      </div>
      {action}
    </Card>
  );
}
