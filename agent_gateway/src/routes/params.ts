import { z } from "zod";

export function routeParam(value: unknown, name: string) {
  return z.string().min(1, `${name} is required`).parse(value);
}
