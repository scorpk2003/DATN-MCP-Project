import { noteCourses, notes } from "../data/notesData.js";
import { request } from "../lib/httpClient.js";

export async function getNotesData() {
  return request("/notes", {
    fallback: () => ({
      noteCourses,
      notes,
    }),
  });
}

export async function createNote(input) {
  return request("/notes", {
    method: "POST",
    body: JSON.stringify(input),
  });
}
