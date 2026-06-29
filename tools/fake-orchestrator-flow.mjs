import http from "node:http";
import { createApp } from "../agent_gateway/src/app.ts";

const fakeOrchestrator = http.createServer(async (request, response) => {
  const body = await readJson(request);
  response.setHeader("Content-Type", "application/json");

  if (request.url === "/agent/run" && body.goal?.includes("Load or generate a lesson")) {
    response.statusCode = 202;
    response.end(
      JSON.stringify({
        ok: true,
        session_id: body.session_id,
        status: "completed",
        output: {
          ok: true,
          status: "completed",
          lessonDraft: {
            id: "lesson_ipv4_subnetting",
            roadmapId: "roadmap_ccna_standard",
            nodeId: "ipv4-subnetting",
            title: "IPv4 subnetting practice",
            objective: "Calculate subnet ranges, masks, and usable host ranges for CCNA-style questions.",
            explanation:
              "Start from CIDR prefix length, derive the block size, then locate the subnet boundary that contains the target host.",
            resources: [
              {
                id: "res_cisco_subnetting",
                title: "Cisco IP addressing and subnetting",
                url: "https://www.cisco.com/",
                sourceType: "official_docs",
                trustTier: 1,
              },
            ],
            exercises: [
              {
                id: "exercise_subnet_01",
                prompt: "Find the network address and usable range for 192.168.10.65/26.",
                expectedOutput: "192.168.10.64/26, usable 192.168.10.65-192.168.10.126",
                difficulty: "medium",
              },
            ],
            status: "active",
          },
        },
      }),
    );
    return;
  }

  if (request.url === "/agent/run") {
    response.statusCode = 202;
    response.end(
      JSON.stringify({
        ok: true,
        session_id: body.session_id,
        status: "waiting_for_user",
        output: {
          ok: true,
          status: "waiting_for_user",
          session_id: body.session_id,
          approval: {
            step_id: "review_ccna_roadmap_draft",
            question: "Review the generated CCNA roadmap draft before saving it.",
            options: ["approve", "reject", "revise"],
          },
        },
      }),
    );
    return;
  }

  if (request.url === "/agent/resume") {
    response.statusCode = 202;
    response.end(
      JSON.stringify({
        ok: true,
        session_id: body.session_id,
        status: body.approval?.decision === "reject" ? "rejected" : "completed",
        output:
          body.approval?.decision === "reject"
            ? {
                ok: true,
                status: "rejected",
                message: "The learner rejected the draft.",
              }
            : {
                ok: true,
                status: "completed",
                output: {
                  roadmapPreview: {
                    roadmapId: "roadmap_ccna_standard",
                    title: "CCNA standard learner roadmap",
                    goal: "Learn CCNA in 8 weeks",
                    status: "draft",
                    coverageStatus: "partial",
                    nodes: [
                      {
                        nodeId: "networking-foundations",
                        title: "Networking foundations",
                        nodeType: "foundation",
                        status: "ready",
                        coverageStatus: "good",
                      },
                      {
                        nodeId: "ipv4-subnetting",
                        title: "IPv4 subnetting",
                        nodeType: "skill",
                        status: "ready",
                        coverageStatus: "partial",
                      },
                      {
                        nodeId: "switching-vlans",
                        title: "Switching and VLANs",
                        nodeType: "concept",
                        status: "ready",
                        coverageStatus: "partial",
                      },
                      {
                        nodeId: "routing-ospf",
                        title: "Routing and OSPF",
                        nodeType: "concept",
                        status: "ready",
                        coverageStatus: "partial",
                      },
                    ],
                    edges: [
                      {
                        fromNodeId: "networking-foundations",
                        toNodeId: "ipv4-subnetting",
                        edgeType: "prerequisite",
                      },
                      {
                        fromNodeId: "ipv4-subnetting",
                        toNodeId: "switching-vlans",
                        edgeType: "recommended",
                      },
                      {
                        fromNodeId: "switching-vlans",
                        toNodeId: "routing-ospf",
                        edgeType: "recommended",
                      },
                    ],
                    coverageSummary: {
                      totalTopics: 4,
                      coverageGood: 1,
                      coveragePartial: 3,
                      coveragePoor: 0,
                      readyForLessonGeneration: false,
                    },
                  },
                },
              },
      }),
    );
    return;
  }

  response.statusCode = 404;
  response.end(JSON.stringify({ ok: false, error: { message: "Not found" } }));
});

fakeOrchestrator.listen(3999, "127.0.0.1", () => {
  console.log("fake orchestrator listening on http://127.0.0.1:3999");
});

const gateway = createApp({
  host: "127.0.0.1",
  port: 4000,
  orchestratorBaseUrl: "http://127.0.0.1:3999",
  orchestratorTimeoutMs: 10000,
  corsOrigin: "*",
  resourceServiceBaseUrl: "http://127.0.0.1:3200",
  allowDevAuthContext: true,
});

gateway.listen(4000, "127.0.0.1", () => {
  console.log("agent gateway listening on http://127.0.0.1:4000");
});

async function readJson(request) {
  const chunks = [];
  for await (const chunk of request) {
    chunks.push(chunk);
  }
  const text = Buffer.concat(chunks).toString("utf8");
  return text ? JSON.parse(text) : {};
}
