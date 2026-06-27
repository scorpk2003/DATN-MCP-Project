import { noteCourses, notes } from "../data/notesData.js";
import { request } from "../lib/httpClient.js";

export async function getNotesData() {
  return request("/api/notes", {
    fallback: () => ({
      noteCourses,
      notes,
    }),
  });
}
