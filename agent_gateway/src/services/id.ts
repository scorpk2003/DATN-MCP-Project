import { nanoid } from "nanoid";

export function makeId(prefix: string) {
  return `${prefix}_${nanoid(12)}`;
}

export function nowIso() {
  return new Date().toISOString();
}
