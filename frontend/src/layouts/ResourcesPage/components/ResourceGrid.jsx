import { EmptyState, SectionTitle } from "../../../components/ui";
import { ResourceCard } from "./ResourceCard.jsx";

export function ResourceGrid({ resources, selectedResourceId, onSelectResource }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Library" title="Kho tài liệu" />
      {resources.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-2">
          {resources.map((resource) => (
            <ResourceCard
              key={resource.id}
              active={resource.id === selectedResourceId}
              resource={resource}
              onSelect={onSelectResource}
            />
          ))}
        </div>
      ) : (
        <EmptyState title="Không tìm thấy tài liệu phù hợp" />
      )}
    </section>
  );
}
