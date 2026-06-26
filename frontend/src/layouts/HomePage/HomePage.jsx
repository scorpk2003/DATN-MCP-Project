import { useState } from "react";
import {
  agentActivities,
  learner,
  learningMetrics,
  quickPrompts,
  recommendedCourses,
} from "../../data/selfLearnDashboard.js";
import { AgentActivityPanel } from "./components/AgentActivityPanel.jsx";
import { CourseSection } from "./components/CourseSection.jsx";
import { HomeHeader } from "./components/HomeHeader.jsx";
import { LearningHero } from "./components/LearningHero.jsx";
import { MetricsStrip } from "./components/MetricsStrip.jsx";

function HomePage() {
  const [prompt, setPrompt] = useState("");

  const handlePromptChange = (event) => {
    setPrompt(event.target.value);
  };

  const handlePromptSelect = (value) => {
    setPrompt(value);
  };

  const handleSubmit = () => {
    if (!prompt.trim()) {
      return;
    }

    setPrompt("");
  };

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
      />
      <MetricsStrip metrics={learningMetrics} />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <CourseSection courses={recommendedCourses} />
        <AgentActivityPanel activities={agentActivities} />
      </div>
    </div>
  );
}

export default HomePage;
