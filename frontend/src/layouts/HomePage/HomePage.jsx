import {
  faCheck,
  faCircleNodes,
  faClock,
  faRotateRight,
  faTriangleExclamation,
  faXmark,
} from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useEffect, useMemo, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import { useAuth } from "../../auth/useAuth.js";
import { Badge, Button, Card, Composer, InlineAlert, SectionTitle } from "../../components/ui";
import {
  createSession,
  getSessionState,
  respondToAction,
  sendIntent,
  subscribeRun,
} from "../../services/agentGatewayClient.js";

const QUICK_PROMPTS = [
  "Learn CCNA in 8 weeks",
  "Build a frontend roadmap",
  "Practice SQL joins",
];

const TERMINAL_STATUSES = new Set(["completed", "failed", "cancelled"]);

function HomePage() {
  const location = useLocation();
  const { user } = useAuth();
  const [prompt, setPrompt] = useState(location.state?.draftPrompt || "");
  const [session, setSession] = useState(null);
  const [runs, setRuns] = useState([]);
  const [messages, setMessages] = useState([]);
  const [artifacts, setArtifacts] = useState([]);
  const [pendingActions, setPendingActions] = useState([]);
  const [timeline, setTimeline] = useState([]);
  const [error, setError] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [openingNodeId, setOpeningNodeId] = useState("");
  const [respondingActionId, setRespondingActionId] = useState("");
  const subscriptionsRef = useRef(new Map());

  const activeRun = useMemo(
    () =>
      [...runs]
        .filter((run) => !TERMINAL_STATUSES.has(run.status))
        .sort((a, b) => (b.startedAt || "").localeCompare(a.startedAt || ""))[0] ?? null,
    [runs],
  );

  useEffect(() => {
    const subscriptions = subscriptionsRef.current;
    const handleCreateRoadmap = (event) => {
      setPrompt(event.detail?.draftPrompt || "Create a new learning roadmap");
    };

    window.addEventListener("selflearn:create-roadmap", handleCreateRoadmap);
    return () => {
      window.removeEventListener("selflearn:create-roadmap", handleCreateRoadmap);
      for (const unsubscribe of subscriptions.values()) {
        unsubscribe();
      }
      subscriptions.clear();
    };
  }, []);

  const applyState = (state) => {
    if (!state) {
      return;
    }
    setSession(state.session);
    setPendingActions(state.pendingActions ?? []);
    setMessages(state.messages ?? []);
    setArtifacts(state.artifacts ?? []);
    setTimeline(state.timeline ?? []);
    if (state.activeRun) {
      upsertRun(state.activeRun);
    }
  };

  const refreshState = async (sessionId) => {
    const state = await getSessionState(sessionId);
    applyState(state);
    return state;
  };

  const subscribeToRun = (sessionId, runId) => {
    if (subscriptionsRef.current.has(runId)) {
      return;
    }

    const unsubscribe = subscribeRun(
      sessionId,
      runId,
      (envelope) => {
        handleAgentEvent(envelope.event, envelope);
      },
      () => {
        subscriptionsRef.current.delete(runId);
      },
    );
    subscriptionsRef.current.set(runId, unsubscribe);
  };

  const handleAgentEvent = (event, envelope) => {
    if (event.type === "agent.message") {
      setMessages((current) => [
        ...current,
        {
          id: envelope.id,
          role: "agent",
          content: event.message,
          runId: envelope.runId,
          createdAt: envelope.timestamp,
        },
      ]);
    }

    if (event.type === "agent.thinking" || event.type === "tool.started" || event.type === "tool.completed") {
      setTimeline((current) => [
        ...current,
        {
          id: envelope.id,
          runId: envelope.runId,
          type: event.type,
          label: event.label || event.displayName,
          status: event.type === "tool.completed" ? "completed" : "started",
          createdAt: envelope.timestamp,
        },
      ]);
    }

    if (event.type === "artifact.created") {
      setArtifacts((current) => upsertArtifact(current, event.artifact));
    }

    if (event.type === "ui.action_required") {
      setPendingActions((current) => [
        ...current.filter((action) => action.id !== event.action.id),
        event.action,
      ]);
    }

    if (event.type === "run.status_changed") {
      upsertRun({
        id: envelope.runId,
        sessionId: envelope.sessionId,
        status: event.status,
        currentStep: event.currentStep,
        startedAt: envelope.timestamp,
        completedAt: TERMINAL_STATUSES.has(event.status) ? envelope.timestamp : undefined,
      });
      if (TERMINAL_STATUSES.has(event.status) || event.status === "waiting_for_user") {
        setIsSubmitting(false);
      }
    }

    if (event.type === "error") {
      setError(event.message);
      setIsSubmitting(false);
    }
  };

  const upsertRun = (run) => {
    setRuns((current) => [
      ...current.filter((item) => item.id !== run.id),
      {
        ...(current.find((item) => item.id === run.id) ?? {}),
        ...run,
      },
    ]);
  };

  const handlePromptChange = (event) => {
    setPrompt(event.target.value);
  };

  const handlePromptSelect = (value) => {
    setPrompt(value);
  };

  const handleSubmit = async () => {
    const goal = prompt.trim();
    if (!goal) {
      return;
    }

    setIsSubmitting(true);
    setError("");
    setMessages([]);
    setArtifacts([]);
    setPendingActions([]);
    setTimeline([]);

    try {
      const currentSession =
        session ??
        (
          await createSession({
            userId: user?.uid,
            title: goal,
            metadata: {
              source: "web",
              locale: navigator.language || "vi-VN",
            },
          })
        ).session;
      setSession(currentSession);

      const accepted = await sendIntent(currentSession.id, {
        intent: {
          type: "goal.submitted",
          payload: {
            goal,
          },
        },
      });

      upsertRun(accepted.run);
      setMessages((current) => [
        ...current,
        {
          id: `local_${accepted.run.id}`,
          role: "user",
          content: goal,
          runId: accepted.run.id,
          createdAt: new Date().toISOString(),
        },
      ]);
      setPrompt("");
      subscribeToRun(currentSession.id, accepted.run.id);
      await refreshState(currentSession.id);
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : "Could not send goal to Agent Gateway.");
      setIsSubmitting(false);
    }
  };

  const handleActionResponse = async (action, selectedOptionId) => {
    if (!session) {
      return;
    }

    setRespondingActionId(action.id);
    setError("");
    try {
      const accepted = await respondToAction(session.id, action.id, {
        selectedOptionId,
      });
      setPendingActions((current) => current.filter((item) => item.id !== action.id));
      upsertRun(accepted.run);
      subscribeToRun(session.id, accepted.run.id);
      await refreshState(session.id);
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : "Could not respond to action.");
    } finally {
      setRespondingActionId("");
    }
  };

  const handleOpenLesson = async (roadmapId, nodeId) => {
    if (!session || !roadmapId || !nodeId) {
      return;
    }

    setOpeningNodeId(nodeId);
    setError("");
    try {
      const accepted = await sendIntent(session.id, {
        intent: {
          type: "roadmap.node.selected",
          payload: {
            roadmapId,
            nodeId,
          },
        },
      });
      upsertRun(accepted.run);
      subscribeToRun(session.id, accepted.run.id);
      await refreshState(session.id);
    } catch (lessonError) {
      setError(lessonError instanceof Error ? lessonError.message : "Could not open lesson for this node.");
    } finally {
      setOpeningNodeId("");
    }
  };

  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <GatewayHeader user={user} session={session} activeRun={activeRun} />

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
        <Card className="space-y-5 p-5 sm:p-6">
          <div className="flex flex-wrap items-center gap-2">
            <Badge tone="agent">Agent Gateway</Badge>
            <Badge tone={activeRun ? statusTone(activeRun.status) : "neutral"}>
              {activeRun?.status ?? "idle"}
            </Badge>
          </div>
          <div className="max-w-3xl space-y-2">
            <h2 className="text-2xl font-bold leading-tight text-[var(--text-primary)] sm:text-3xl">
              Create a learning roadmap through the real agent flow.
            </h2>
            <p className="text-sm leading-6 text-[var(--text-secondary)]">
              Submit a goal, review approval gates, then inspect the artifacts returned by Orchestrator.
            </p>
          </div>
          <Composer
            value={prompt}
            onChange={handlePromptChange}
            onSubmit={handleSubmit}
            submitLabel="Run agent"
            submitLoading={isSubmitting}
            disabled={isSubmitting || Boolean(respondingActionId)}
            actions={QUICK_PROMPTS.map((item) => (
              <Button key={item} type="button" variant="ghost" size="sm" onClick={() => handlePromptSelect(item)}>
                {item}
              </Button>
            ))}
          />
        </Card>

        <RunStatusCard session={session} run={activeRun} artifacts={artifacts} pendingActions={pendingActions} />
      </section>

      {error ? <InlineAlert tone="risk" title="Gateway error" description={error} /> : null}

      <PendingActionsPanel
        actions={pendingActions}
        respondingActionId={respondingActionId}
        onRespond={handleActionResponse}
      />

      <section className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_380px]">
        <ArtifactsPanel artifacts={artifacts} openingNodeId={openingNodeId} onOpenLesson={handleOpenLesson} />
        <div className="space-y-6">
          <MessagesPanel messages={messages} />
          <TimelinePanel timeline={timeline} />
        </div>
      </section>
    </div>
  );
}

export default HomePage;

function GatewayHeader({ user, session, activeRun }) {
  return (
    <header className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0">
        <p className="text-sm font-semibold text-[var(--text-muted)]">
          {user?.displayName || user?.email || "SelfLearn user"}
        </p>
        <h1 className="mt-1 text-3xl font-bold leading-tight text-[var(--text-primary)]">
          Agent workspace
        </h1>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {session ? <Badge tone="teal">{session.id}</Badge> : null}
        {activeRun ? <Badge tone={statusTone(activeRun.status)}>{activeRun.status}</Badge> : null}
      </div>
    </header>
  );
}

function RunStatusCard({ session, run, artifacts, pendingActions }) {
  const rows = [
    { label: "Session", value: session?.id ?? "not started" },
    { label: "Run", value: run?.id ?? "none" },
    { label: "Actions", value: String(pendingActions.length) },
    { label: "Artifacts", value: String(artifacts.length) },
  ];

  return (
    <Card tone="inverse" className="flex flex-col justify-between gap-5 p-5">
      <div className="flex items-center justify-between gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-[var(--sl-inverse-surface)]">
          <FontAwesomeIcon icon={faCircleNodes} />
        </div>
        <Badge tone={run ? statusTone(run.status) : "neutral"}>{run?.status ?? "idle"}</Badge>
      </div>
      <div className="space-y-3">
        {rows.map((row) => (
          <div key={row.label} className="flex items-center justify-between gap-4 text-sm">
            <span className="text-[var(--sl-inverse-panel-muted)]">{row.label}</span>
            <span className="max-w-[210px] truncate font-semibold text-[var(--text-inverse)]">{row.value}</span>
          </div>
        ))}
      </div>
    </Card>
  );
}

function PendingActionsPanel({ actions, respondingActionId, onRespond }) {
  if (!actions.length) {
    return null;
  }

  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Human approval" title="Action required" />
      <div className="grid gap-4 lg:grid-cols-2">
        {actions.map((action) => (
          <Card key={action.id} className="space-y-4 p-5">
            <div className="flex items-start gap-3">
              <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-[var(--bg-surface-muted)] text-[var(--color-warning)]">
                <FontAwesomeIcon icon={faTriangleExclamation} />
              </div>
              <div className="min-w-0">
                <h3 className="text-base font-bold text-[var(--text-primary)]">{action.title}</h3>
                {action.description ? (
                  <p className="mt-1 text-sm leading-6 text-[var(--text-secondary)]">{action.description}</p>
                ) : null}
                {action.metadata?.stepId ? (
                  <p className="mt-2 truncate text-xs font-semibold text-[var(--text-muted)]">
                    Step: {action.metadata.stepId}
                  </p>
                ) : null}
              </div>
            </div>
            <div className="flex flex-wrap gap-2">
              {action.options.map((option) => (
                <Button
                  key={option.id}
                  variant={option.variant || optionVariant(option.id)}
                  size="sm"
                  loading={respondingActionId === action.id && option.id === "approve"}
                  disabled={Boolean(respondingActionId)}
                  onClick={() => onRespond(action, option.id)}
                >
                  {optionIcon(option.id)}
                  {option.label}
                </Button>
              ))}
            </div>
          </Card>
        ))}
      </div>
    </section>
  );
}

function ArtifactsPanel({ artifacts, openingNodeId, onOpenLesson }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Artifacts" title="Generated output" />
      {artifacts.length ? (
        <div className="grid gap-4">
          {artifacts.map((artifact) => (
            <ArtifactCard
              key={`${artifact.kind}:${artifact.id}`}
              artifact={artifact}
              openingNodeId={openingNodeId}
              onOpenLesson={onOpenLesson}
            />
          ))}
        </div>
      ) : (
        <EmptyBlock title="No artifacts yet" description="Run a goal and approve pending gates to receive roadmap or lesson artifacts." />
      )}
    </section>
  );
}

function ArtifactCard({ artifact, openingNodeId, onOpenLesson }) {
  if (artifact.kind === "roadmap") {
    return (
      <Card className="space-y-4 p-5">
        <ArtifactHeader artifact={artifact} detail={`${artifact.nodes.length} nodes`} />
        <div className="grid gap-3 md:grid-cols-2">
          {artifact.nodes.slice(0, 6).map((node) => (
            <div key={node.id} className="space-y-3 rounded-[var(--radius-md)] border border-[var(--border-secondary)] p-3">
              <div className="flex items-center justify-between gap-2">
                <p className="truncate text-sm font-bold text-[var(--text-primary)]">{node.title}</p>
                <Badge tone={coverageTone(node.coverageStatus)}>{node.coverageStatus}</Badge>
              </div>
              <div className="flex items-center justify-between gap-3">
                <p className="text-xs font-semibold text-[var(--text-muted)]">{node.type}</p>
                <Button
                  variant="secondary"
                  size="sm"
                  loading={openingNodeId === node.id}
                  disabled={Boolean(openingNodeId)}
                  onClick={() => onOpenLesson?.(artifact.id, node.id)}
                >
                  Open lesson
                </Button>
              </div>
            </div>
          ))}
        </div>
      </Card>
    );
  }

  if (artifact.kind === "resource_readiness") {
    return (
      <Card className="space-y-3 p-5">
        <ArtifactHeader artifact={artifact} detail={artifact.overallStatus} />
        <p className="text-sm leading-6 text-[var(--text-secondary)]">
          {artifact.topicName} needs {artifact.recommendedAction || "source review"}.
        </p>
      </Card>
    );
  }

  if (artifact.kind === "lesson") {
    return (
      <Card className="space-y-3 p-5">
        <ArtifactHeader artifact={artifact} detail={artifact.status} />
        <p className="text-sm leading-6 text-[var(--text-secondary)]">{artifact.objective}</p>
        <p className="text-sm leading-6 text-[var(--text-secondary)]">{artifact.explanation}</p>
        {artifact.exercise ? (
          <div className="rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-subtle)] p-3">
            <p className="text-xs font-bold uppercase tracking-normal text-[var(--text-muted)]">Practice</p>
            <p className="mt-2 text-sm leading-6 text-[var(--text-primary)]">{artifact.exercise.prompt}</p>
            {artifact.exercise.difficulty ? (
              <Badge tone="agent" className="mt-3">
                {artifact.exercise.difficulty}
              </Badge>
            ) : null}
          </div>
        ) : null}
        {artifact.resources.length ? (
          <div className="flex flex-wrap gap-2">
            {artifact.resources.slice(0, 3).map((resource) => (
              <Badge key={resource.id} tone={resource.trustTier === 1 ? "success" : "neutral"}>
                {resource.title}
              </Badge>
            ))}
          </div>
        ) : null}
      </Card>
    );
  }

  return (
    <Card className="space-y-3 p-5">
      <ArtifactHeader artifact={artifact} detail={artifact.status || artifact.kind} />
      <p className="text-sm leading-6 text-[var(--text-secondary)]">
        {artifact.message || "The agent returned a supported UI artifact."}
      </p>
    </Card>
  );
}

function ArtifactHeader({ artifact, detail }) {
  return (
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div className="min-w-0">
        <Badge tone="teal">{artifact.kind}</Badge>
        <h3 className="mt-2 truncate text-lg font-bold text-[var(--text-primary)]">
          {artifact.title || artifact.topicName || artifact.id}
        </h3>
      </div>
      <Badge tone={artifact.coverageStatus ? coverageTone(artifact.coverageStatus) : "neutral"}>{detail}</Badge>
    </div>
  );
}

function MessagesPanel({ messages }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Messages" title="Conversation" />
      {messages.length ? (
        <Card className="max-h-[360px] space-y-3 overflow-y-auto p-4">
          {messages.slice(-12).map((message) => (
            <div key={message.id} className="rounded-[var(--radius-md)] bg-[var(--bg-subtle)] px-3 py-2">
              <div className="mb-1 flex items-center justify-between gap-2">
                <Badge tone={message.role === "user" ? "neutral" : "agent"}>{message.role}</Badge>
                <span className="text-xs text-[var(--text-muted)]">{formatTime(message.createdAt)}</span>
              </div>
              <p className="text-sm leading-6 text-[var(--text-secondary)]">{message.content}</p>
            </div>
          ))}
        </Card>
      ) : (
        <EmptyBlock title="No messages" description="Agent messages will stream here after a run starts." />
      )}
    </section>
  );
}

function TimelinePanel({ timeline }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Runtime" title="Event timeline" />
      {timeline.length ? (
        <Card className="space-y-3 p-4">
          {timeline.slice(-8).map((item) => (
            <div key={item.id} className="flex items-start gap-3">
              <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--bg-surface-muted)] text-[var(--color-info)]">
                <FontAwesomeIcon icon={item.status === "completed" ? faCheck : faClock} />
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-bold text-[var(--text-primary)]">{item.label}</p>
                <p className="text-xs font-semibold text-[var(--text-muted)]">{item.type}</p>
              </div>
            </div>
          ))}
        </Card>
      ) : (
        <EmptyBlock title="No runtime events" description="Tool calls and thinking events will appear here." />
      )}
    </section>
  );
}

function EmptyBlock({ title, description }) {
  return (
    <Card className="p-5">
      <p className="text-sm font-bold text-[var(--text-primary)]">{title}</p>
      <p className="mt-1 text-sm leading-6 text-[var(--text-secondary)]">{description}</p>
    </Card>
  );
}

function upsertArtifact(current, artifact) {
  const key = `${artifact.kind}:${artifact.id}`;
  return [...current.filter((item) => `${item.kind}:${item.id}` !== key), artifact];
}

function statusTone(status) {
  if (status === "completed") {
    return "success";
  }
  if (status === "failed" || status === "cancelled") {
    return "risk";
  }
  if (status === "waiting_for_user") {
    return "warning";
  }
  return "agent";
}

function coverageTone(status) {
  if (status === "good") {
    return "success";
  }
  if (status === "missing" || status === "low") {
    return "risk";
  }
  return "warning";
}

function optionVariant(id) {
  if (id === "reject") {
    return "danger";
  }
  if (id === "approve" || id === "backfill_first") {
    return "primary";
  }
  return "secondary";
}

function optionIcon(id) {
  if (id === "approve" || id === "start_anyway") {
    return <FontAwesomeIcon icon={faCheck} />;
  }
  if (id === "reject") {
    return <FontAwesomeIcon icon={faXmark} />;
  }
  if (id === "revise" || id === "edit_goal") {
    return <FontAwesomeIcon icon={faRotateRight} />;
  }
  return null;
}

function formatTime(value) {
  if (!value) {
    return "";
  }
  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}
