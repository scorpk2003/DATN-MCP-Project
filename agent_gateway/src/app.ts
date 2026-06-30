import cors from "cors";
import express from "express";
import type { GatewayConfig } from "./config.js";
import { actionsRouter } from "./routes/actions.routes.js";
import { dataRouter } from "./routes/data.routes.js";
import { intentsRouter } from "./routes/intents.routes.js";
import { runsRouter } from "./routes/runs.routes.js";
import { sessionsRouter } from "./routes/sessions.routes.js";
import { streamRouter } from "./routes/stream.routes.js";
import { errorHandler } from "./middleware/errorHandler.js";
import { RunProcessor } from "./services/runProcessor.js";

export function createApp(config: GatewayConfig) {
  const app = express();
  const runProcessor = new RunProcessor(config);

  app.use(
    cors({
      origin: config.corsOrigin === "*" ? true : config.corsOrigin,
      credentials: true,
    }),
  );
  app.use(express.json({ limit: "1mb" }));

  app.get("/health", (_request, response) => {
    response.json({ status: "ok" });
  });

  app.use(dataRouter(config));
  app.use(sessionsRouter(config));
  app.use(intentsRouter(runProcessor, config));
  app.use(streamRouter(config));
  app.use(actionsRouter(runProcessor, config));
  app.use(runsRouter(runProcessor, config));
  app.use(errorHandler);

  return app;
}
