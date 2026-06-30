import { useMemo, useState } from "react";
import { ErrorState, LoadingState, SectionTitle } from "../../components/ui";
import { useResourcesData } from "../../hooks/useResourcesData.js";
import { ResourceDetail } from "./components/ResourceDetail.jsx";
import { ResourceGrid } from "./components/ResourceGrid.jsx";
import { ResourceToolbar } from "./components/ResourceToolbar.jsx";

const allFilterLabel = "Tất cả";

function ResourcesPage() {
  const [activeCourse, setActiveCourse] = useState(allFilterLabel);
  const [activeType, setActiveType] = useState(allFilterLabel);
  const [query, setQuery] = useState("");
  const [selectedResourceId, setSelectedResourceId] = useState(null);
  const { data, error, loading, reload } = useResourcesData();
  const { resourceCourses, resources, resourceTypes } = data;

  const filteredResources = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();

    return resources.filter((resource) => {
      const matchesCourse = activeCourse === allFilterLabel || resource.course === activeCourse;
      const matchesType = activeType === allFilterLabel || resource.type === activeType;
      const matchesQuery =
        normalizedQuery.length === 0 ||
        resource.title.toLowerCase().includes(normalizedQuery) ||
        resource.description.toLowerCase().includes(normalizedQuery) ||
        resource.source.toLowerCase().includes(normalizedQuery);

      return matchesCourse && matchesType && matchesQuery;
    });
  }, [activeCourse, activeType, query, resources]);

  const selectedResource =
    filteredResources.find((resource) => resource.id === selectedResourceId) || filteredResources[0] || null;

  if (loading) {
    return <LoadingState layout="grid" title="Đang tải kho tài liệu..." />;
  }

  if (error) {
    return <ErrorState onRetry={reload} />;
  }

  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <SectionTitle
        eyebrow="Resources"
        title="Kho tài liệu"
        className="rounded-[var(--radius-card)] border border-[var(--border-primary)] bg-[var(--bg-surface)] p-5 shadow-[var(--shadow-card)]"
      />
      <ResourceToolbar
        courses={resourceCourses}
        types={resourceTypes}
        activeCourse={activeCourse}
        activeType={activeType}
        query={query}
        onCourseChange={setActiveCourse}
        onQueryChange={setQuery}
        onTypeChange={setActiveType}
      />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <ResourceGrid
          resources={filteredResources}
          selectedResourceId={selectedResource?.id}
          onSelectResource={setSelectedResourceId}
        />
        <ResourceDetail resource={selectedResource} />
      </div>
    </div>
  );
}

export default ResourcesPage;
