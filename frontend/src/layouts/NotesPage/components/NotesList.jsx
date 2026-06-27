import { EmptyState, SectionTitle } from "../../../components/ui";
import { NoteCard } from "./NoteCard.jsx";

export function NotesList({ notes, selectedNoteId, onSelectNote }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Library" title="Ghi chú đã lưu" />
      {notes.length > 0 ? (
        <div className="grid gap-3">
          {notes.map((note) => (
            <NoteCard
              key={note.id}
              active={note.id === selectedNoteId}
              note={note}
              onSelect={onSelectNote}
            />
          ))}
        </div>
      ) : (
        <EmptyState title="Không tìm thấy ghi chú phù hợp" />
      )}
    </section>
  );
}
