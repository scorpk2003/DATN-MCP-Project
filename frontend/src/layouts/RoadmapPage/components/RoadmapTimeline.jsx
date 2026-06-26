import { SectionTitle } from "../../../components/ui";
import { RoadmapPhaseCard } from "./RoadmapPhaseCard.jsx";

export function RoadmapTimeline({ phases }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Timeline" title="Các chặng học chính" />
      <div className="grid gap-4 md:grid-cols-2">
        {phases.map((phase) => (
          <RoadmapPhaseCard key={phase.id} phase={phase} />
        ))}
      </div>
    </section>
  );
}
