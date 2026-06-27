import type { RoadmapArtifact, UIArtifact } from "../protocol/index.js";
import { makeId, nowIso } from "../services/id.js";

type CoverageStatus = "good" | "partial" | "missing";

export type NormalizedOutput =
  | {
      type: "artifacts";
      artifacts: UIArtifact[];
      summary?: string;
    }
  | {
      type: "message";
      message: string;
    };

export function normalizeOrchestratorOutput(output: unknown, goal: string): NormalizedOutput {
  const artifact = findRoadmapLikeArtifact(output, goal);
  if (artifact) {
    return {
      type: "artifacts",
      artifacts: [artifact],
      summary: "Roadmap artifact generated.",
    };
  }

  return {
    type: "message",
    message: summarizeUnknownOutput(output),
  };
}

function findRoadmapLikeArtifact(value: unknown, goal: string): RoadmapArtifact | null {
  const seen = new Set<unknown>();
  const candidate = findObject(value, (item) => Array.isArray(item.nodes) || Array.isArray(item.phases) || Array.isArray(item.steps), seen);
  if (!candidate) {
    return null;
  }

  const rawNodes = arrayFrom(candidate.nodes) ?? arrayFrom(candidate.phases) ?? arrayFrom(candidate.steps) ?? [];
  const nodes = rawNodes.slice(0, 40).map((node, index) => {
    const record = isRecord(node) ? node : {};
    const id = stringFrom(record.id) ?? stringFrom(record.nodeId) ?? stringFrom(record.slug) ?? `node_${index + 1}`;
    const title =
      stringFrom(record.title) ??
      stringFrom(record.name) ??
      stringFrom(record.label) ??
      stringFrom(record.topic) ??
      `Learning step ${index + 1}`;

    return {
      id,
      title,
      type: normalizeNodeType(stringFrom(record.type)),
      status: normalizeNodeStatus(stringFrom(record.status), index),
      coverageStatus: normalizeCoverage(stringFrom(record.coverageStatus) ?? stringFrom(record.coverage)),
      lessonId: stringFrom(record.lessonId),
      position: {
        x: 120 + (index % 4) * 220,
        y: 100 + Math.floor(index / 4) * 160,
      },
    };
  });

  if (nodes.length === 0) {
    return null;
  }

  const rawEdges = arrayFrom(candidate.edges);
  const edges = rawEdges
    ? rawEdges.map((edge, index) => {
        const record = isRecord(edge) ? edge : {};
        const source = stringFrom(record.source) ?? stringFrom(record.from) ?? nodes[index]?.id ?? nodes[0]?.id ?? "node_1";
        const target = stringFrom(record.target) ?? stringFrom(record.to) ?? nodes[index + 1]?.id ?? source;
        return {
          id: stringFrom(record.id) ?? `edge_${source}_${target}_${index + 1}`,
          source,
          target,
          type: normalizeEdgeType(stringFrom(record.type)),
        };
      })
    : nodes.slice(1).map((node, index) => ({
        id: `edge_${nodes[index]?.id}_${node.id}`,
        source: nodes[index]?.id ?? nodes[0]?.id ?? node.id,
        target: node.id,
        type: "recommended" as const,
      }));

  return {
    kind: "roadmap",
    id: stringFrom(candidate.id) ?? stringFrom(candidate.roadmapId) ?? makeId("artifact_roadmap"),
    title: stringFrom(candidate.title) ?? "Generated learning roadmap",
    goal: stringFrom(candidate.goal) ?? goal,
    status: "draft",
    coverageStatus: normalizeCoverage(stringFrom(candidate.coverageStatus)),
    nodes,
    edges,
    metadata: {
      generatedAt: nowIso(),
    },
  };
}

function findObject(
  value: unknown,
  predicate: (item: Record<string, unknown>) => boolean,
  seen: Set<unknown>,
): Record<string, unknown> | null {
  if (!isRecord(value) || seen.has(value)) {
    return null;
  }
  seen.add(value);

  if (predicate(value)) {
    return value;
  }

  for (const child of Object.values(value)) {
    if (Array.isArray(child)) {
      for (const item of child) {
        const found = findObject(item, predicate, seen);
        if (found) {
          return found;
        }
      }
    } else {
      const found = findObject(child, predicate, seen);
      if (found) {
        return found;
      }
    }
  }

  return null;
}

function summarizeUnknownOutput(output: unknown) {
  if (typeof output === "string" && output.trim()) {
    return output.trim();
  }

  if (isRecord(output)) {
    const message = stringFrom(output.message) ?? stringFrom(output.summary) ?? stringFrom(output.answer);
    if (message) {
      return message;
    }
  }

  return "Agent completed the run, but the result did not match a supported UI artifact yet.";
}

function normalizeNodeType(value?: string) {
  const allowed = ["foundation", "concept", "skill", "practice", "checkpoint", "project"] as const;
  return allowed.find((item) => item === value) ?? "concept";
}

function normalizeNodeStatus(value: string | undefined, index: number) {
  const allowed = ["locked", "ready", "active", "completed", "blocked"] as const;
  return allowed.find((item) => item === value) ?? (index === 0 ? "active" : "ready");
}

function normalizeCoverage(value?: string): CoverageStatus {
  if (value === "missing" || value === "low") {
    return "missing";
  }
  if (value === "partial") {
    return "partial";
  }
  return "good";
}

function normalizeEdgeType(value?: string): "prerequisite" | "recommended" {
  return value === "recommended" ? "recommended" : "prerequisite";
}

function arrayFrom(value: unknown) {
  return Array.isArray(value) ? value : null;
}

function stringFrom(value: unknown) {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
