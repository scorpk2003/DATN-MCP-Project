import { statusToTone, statusToneVars } from "../../styles/tokens.js";

export function cx(...classes) {
  return classes.filter(Boolean).join(" ");
}

export function clampPercent(value = 0, max = 100) {
  const numericValue = Number(value);
  const numericMax = Number(max) || 100;
  const percent = numericMax <= 1 ? numericValue * 100 : (numericValue / numericMax) * 100;

  if (!Number.isFinite(percent)) {
    return 0;
  }

  return Math.min(100, Math.max(0, percent));
}

export function resolveTone(tone = "neutral", status) {
  const mappedTone = status ? statusToTone[status] : tone;
  return statusToneVars[mappedTone] || statusToneVars.neutral;
}
