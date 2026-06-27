import { getNotesData } from "../services/notesService.js";
import { useAsyncResource } from "./useAsyncResource.js";

const initialNotesData = {
  noteCourses: [],
  notes: [],
};

export function useNotesData() {
  return useAsyncResource(getNotesData, initialNotesData);
}
