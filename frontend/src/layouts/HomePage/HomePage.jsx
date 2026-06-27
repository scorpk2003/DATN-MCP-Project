import { useState } from "react";
import { Badge, Card, ErrorState, InlineAlert, LoadingState } from "../../components/ui";
import { useDashboardData } from "../../hooks/useDashboardData.js";
import { createSession, sendIntent, subscribeRun } from "../../services/agentGatewayClient.js";
import { AgentActivityPanel } from "./components/AgentActivityPanel.jsx";
import { CourseSection } from "./components/CourseSection.jsx";
import { HomeHeader } from "./components/HomeHeader.jsx";
import { LearningHero } from "./components/LearningHero.jsx";
import { MetricsStrip } from "./components/MetricsStrip.jsx";

function HomePage() {
  const [prompt, setPrompt] = useState("");
  const [agentSession, setAgentSession] = useState(null);
  const [agentRun, setAgentRun] = useState(null);
  const [agentMessages, setAgentMessages] = useState([]);
  const [agentArtifacts, setAgentArtifacts] = useState([]);
  const [agentError, setAgentError] = useState("");
  const [isSubmittingGoal, setIsSubmittingGoal] = useState(false);
  const { data, error, loading, reload } = useDashboardData();
  const { agentActivities, learner, learningMetrics, quickPrompts, recommendedCourses } = data;

  const handlePromptChange = (event) => {
    setPrompt(event.target.value);
  };

  const handlePromptSelect = (value) => {
    setPrompt(value);
  };

  const handleSubmit = async () => {
    const goal = prompt.trim();
    if (!prompt.trim()) {
      return;
    }

    setIsSubmittingGoal(true);
    setAgentError("");
    setAgentMessages([]);
    setAgentArtifacts([]);

    try {
      const sessionResult =
        agentSession ??
        (await createSession({
          title: goal,
          metadata: {
            source: "web",
            locale: "vi-VN",
          },
        })).session;
      setAgentSession(sessionResult);

      const response = await sendIntent(sessionResult.id, {
        intent: {
          type: "goal.submitted",
          payload: {
            goal,
          },
        },
      });
      setAgentRun(response.run);
      setPrompt("");

      subscribeRun(
        sessionResult.id,
        response.run.id,
        (envelope) => {
          const event = envelope.event;
          if (event.type === "agent.message") {
            setAgentMessages((current) => [...current, event.message]);
          }
          if (event.type === "artifact.created") {
            setAgentArtifacts((current) => [...current.filter((item) => item.id !== event.artifact.id), event.artifact]);
          }
          if (event.type === "run.status_changed") {
            setAgentRun((current) => ({
              ...(current || response.run),
              status: event.status,
              currentStep: event.currentStep,
            }));
            if (["completed", "failed", "cancelled"].includes(event.status)) {
              setIsSubmittingGoal(false);
            }
          }
          if (event.type === "error") {
            setAgentError(event.message);
            setIsSubmittingGoal(false);
          }
        },
        () => {
          setIsSubmittingGoal(false);
        },
      );
    } catch (submitError) {
      setAgentError(submitError instanceof Error ? submitError.message : "Không thể gửi mục tiêu tới Agent Gateway.");
      setIsSubmittingGoal(false);
    }
  };

  if (loading) {
    return <LoadingState layout="dashboard" title="Đang tải tổng quan học tập..." />;
  }

  if (error || !learner) {
    return <ErrorState onRetry={reload} />;
  }

  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <HomeHeader learner={learner} />
      <LearningHero
        learner={learner}
        quickPrompts={quickPrompts}
        prompt={prompt}
        onPromptChange={handlePromptChange}
        onPromptSelect={handlePromptSelect}
        onSubmit={handleSubmit}
        submitLoading={isSubmittingGoal}
      />
      <AgentGatewayStatus run={agentRun} messages={agentMessages} artifacts={agentArtifacts} error={agentError} />
      <MetricsStrip metrics={learningMetrics} />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <CourseSection courses={recommendedCourses} />
        <AgentActivityPanel activities={agentActivities} />
      </div>
    </div>
  );
}

export default HomePage;

function AgentGatewayStatus({ run, messages, artifacts, error }) {
  if (!run && !messages.length && !artifacts.length && !error) {
    return null;
  }

  return (
    <Card className="space-y-4 p-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="text-sm font-bold text-[var(--text-primary)]">Agent Gateway</p>
          <p className="text-sm text-[var(--text-secondary)]">Luồng roadmap đầu tiên đang chạy qua Express Gateway.</p>
        </div>
        {run ? <Badge tone={run.status === "failed" ? "risk" : "agent"}>{run.status}</Badge> : null}
      </div>

      {error ? <InlineAlert tone="risk" title="Gateway error" description={error} /> : null}

      {messages.length ? (
        <div className="space-y-2">
          {messages.slice(-3).map((message, index) => (
            <p key={`${message}-${index}`} className="rounded-[var(--radius-md)] bg-[var(--bg-subtle)] px-3 py-2 text-sm text-[var(--text-secondary)]">
              {message}
            </p>
          ))}
        </div>
      ) : null}

      {artifacts.length ? (
        <div className="flex flex-wrap gap-2">
          {artifacts.map((artifact) => (
            <Badge key={artifact.id} tone="teal">
              {artifact.kind}
            </Badge>
          ))}
        </div>
      ) : null}
    </Card>
  );
}
