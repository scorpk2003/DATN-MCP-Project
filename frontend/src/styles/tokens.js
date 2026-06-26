export const cssVar = (name) => `var(${name})`;

export const colorVars = {
  background: cssVar("--sl-bg-app"),
  surface: cssVar("--sl-bg-surface"),
  surfaceMuted: cssVar("--sl-bg-muted"),
  inverse: cssVar("--sl-bg-inverse"),
  textPrimary: cssVar("--sl-text-primary"),
  textSecondary: cssVar("--sl-text-secondary"),
  textInverse: cssVar("--sl-text-inverse"),
  border: cssVar("--sl-border-default"),
  accent: cssVar("--sl-action-accent"),
  accentText: cssVar("--sl-action-accent-text"),
  progressTrack: cssVar("--sl-progress-track"),
};

export const statusToneVars = {
  success: {
    fg: cssVar("--sl-status-success"),
    bg: cssVar("--sl-status-success-bg"),
  },
  warning: {
    fg: cssVar("--sl-status-warning"),
    bg: cssVar("--sl-status-warning-bg"),
  },
  risk: {
    fg: cssVar("--sl-status-risk"),
    bg: cssVar("--sl-status-risk-bg"),
  },
  info: {
    fg: cssVar("--sl-status-info"),
    bg: cssVar("--sl-status-info-bg"),
  },
  agent: {
    fg: cssVar("--sl-status-agent"),
    bg: cssVar("--sl-status-agent-bg"),
  },
  neutral: {
    fg: cssVar("--sl-text-secondary"),
    bg: cssVar("--sl-bg-muted"),
  },
};

export const statusToTone = {
  active: "success",
  completed: "success",
  done: "success",
  passed: "success",
  good: "success",
  trusted: "success",
  running: "agent",
  grading: "agent",
  submitting: "agent",
  pending: "neutral",
  queued: "warning",
  warning: "warning",
  partial: "warning",
  "partial coverage": "warning",
  "partial success": "warning",
  "needs confirmation": "warning",
  "needs resource": "warning",
  missing: "risk",
  blocked: "risk",
  failed: "risk",
  error: "risk",
  low: "risk",
  info: "info",
  selected: "info",
  refresh: "info",
};

export const layoutVars = {
  sidebarWidth: cssVar("--sl-sidebar-width"),
  rightPanelSmall: cssVar("--sl-right-panel-sm"),
  rightPanelMedium: cssVar("--sl-right-panel-md"),
  topbarHeight: cssVar("--sl-topbar-height"),
};
