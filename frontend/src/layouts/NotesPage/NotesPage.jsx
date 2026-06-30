import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../../auth/useAuth.js";
import { ErrorState, LoadingState, SectionTitle } from "../../components/ui";
import { useNotesData } from "../../hooks/useNotesData.js";
import { startLearningFlow } from "../../services/learningFlow.js";
import { createNote } from "../../services/notesService.js";
import { NoteDetail } from "./components/NoteDetail.jsx";
import { NotesList } from "./components/NotesList.jsx";
import { NotesToolbar } from "./components/NotesToolbar.jsx";

const allCoursesLabel = "Tất cả";

function NotesPage() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [activeCourse, setActiveCourse] = useState(allCoursesLabel);
  const [query, setQuery] = useState("");
  const [selectedNoteId, setSelectedNoteId] = useState(null);
  const [creatingNote, setCreatingNote] = useState(false);
  const [savingNote, setSavingNote] = useState(false);
  const [newNoteContent, setNewNoteContent] = useState("");
  const [creatingReview, setCreatingReview] = useState(false);
  const [actionError, setActionError] = useState("");
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

  if (actionError) {
    return <ErrorState title="Không thể tạo phiên ôn tập" description={actionError} onRetry={() => setActionError("")} />;
  }

  const handleCreateReview = async (note) => {
    setCreatingReview(true);
    setActionError("");
    try {
      const { session } = await startLearningFlow({
        user,
        title: `Review note: ${note.title}`,
        intent: {
          type: "note.review.requested",
          payload: {
            noteId: note.noteId || note.id,
            taskId: note.taskId,
            title: note.title,
            content: note.content,
            course: note.course,
            tags: note.tags,
          },
        },
        metadata: { sourcePage: "notes", noteId: note.noteId || note.id },
      });
      navigate("/", { state: { sessionId: session.id } });
    } catch (flowError) {
      setActionError(flowError instanceof Error ? flowError.message : "Không thể tạo ôn tập từ ghi chú.");
    } finally {
      setCreatingReview(false);
    }
  };

  const handleStartCreateNote = () => {
    setCreatingNote(true);
    setActionError("");
  };

  const handleCancelCreateNote = () => {
    setCreatingNote(false);
    setNewNoteContent("");
  };

  const handleCreateNote = async (event) => {
    event?.preventDefault?.();
    const content = newNoteContent.trim();
    if (!content) {
      return;
    }

    setSavingNote(true);
    setActionError("");
    try {
      const { note } = await createNote({ content });
      setSelectedNoteId(note?.id ?? null);
      setNewNoteContent("");
      setCreatingNote(false);
      reload();
    } catch (noteError) {
      setActionError(noteError instanceof Error ? noteError.message : "Khong the tao ghi chu.");
    } finally {
      setSavingNote(false);
    }
  };

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
        creating={creatingNote}
        saving={savingNote}
        newNoteContent={newNoteContent}
        onCourseChange={setActiveCourse}
        onQueryChange={setQuery}
        onNewNoteContentChange={setNewNoteContent}
        onStartCreate={handleStartCreateNote}
        onCreateNote={handleCreateNote}
        onCancelCreate={handleCancelCreateNote}
      />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_380px]">
        <NotesList notes={filteredNotes} selectedNoteId={selectedNote?.id} onSelectNote={setSelectedNoteId} />
        <NoteDetail note={selectedNote} onCreateReview={handleCreateReview} creatingReview={creatingReview} />
      </div>
    </div>
  );
}

export default NotesPage;
