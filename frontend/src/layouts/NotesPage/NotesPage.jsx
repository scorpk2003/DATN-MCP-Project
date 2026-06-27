import { useMemo, useState } from "react";
import { ErrorState, LoadingState, SectionTitle } from "../../components/ui";
import { useNotesData } from "../../hooks/useNotesData.js";
import { NoteDetail } from "./components/NoteDetail.jsx";
import { NotesList } from "./components/NotesList.jsx";
import { NotesToolbar } from "./components/NotesToolbar.jsx";

const allCoursesLabel = "Tất cả";

function NotesPage() {
  const [activeCourse, setActiveCourse] = useState(allCoursesLabel);
  const [query, setQuery] = useState("");
  const [selectedNoteId, setSelectedNoteId] = useState(null);
  const { data, error, loading, reload } = useNotesData();
  const { noteCourses, notes } = data;

  const filteredNotes = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();

    return notes.filter((note) => {
      const matchesCourse = activeCourse === allCoursesLabel || note.course === activeCourse;
      const matchesQuery =
        normalizedQuery.length === 0 ||
        note.title.toLowerCase().includes(normalizedQuery) ||
        note.excerpt.toLowerCase().includes(normalizedQuery) ||
        note.tags.some((tag) => tag.toLowerCase().includes(normalizedQuery));

      return matchesCourse && matchesQuery;
    });
  }, [activeCourse, notes, query]);

  const selectedNote =
    filteredNotes.find((note) => note.id === selectedNoteId) || filteredNotes[0] || null;

  if (loading) {
    return <LoadingState layout="list" title="Đang tải ghi chú..." />;
  }

  if (error) {
    return <ErrorState onRetry={reload} />;
  }

  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <SectionTitle
        eyebrow="Notes"
        title="Ghi chú học tập"
        className="rounded-[var(--radius-card)] border border-[var(--border-primary)] bg-[var(--bg-surface)] p-5 shadow-[var(--shadow-card)]"
      />
      <NotesToolbar
        courses={noteCourses}
        activeCourse={activeCourse}
        query={query}
        onCourseChange={setActiveCourse}
        onQueryChange={setQuery}
      />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_380px]">
        <NotesList
          notes={filteredNotes}
          selectedNoteId={selectedNote?.id}
          onSelectNote={setSelectedNoteId}
        />
        <NoteDetail note={selectedNote} />
      </div>
    </div>
  );
}

export default NotesPage;
