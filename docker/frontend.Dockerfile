FROM node:24-alpine AS build

WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
ARG VITE_AGENT_GATEWAY_URL=/api/agent-gateway
ARG VITE_API_BASE_URL=/api/agent-gateway
ARG VITE_ALLOW_DEV_AUTH=false
ARG VITE_USE_MOCK_API=false
ENV VITE_AGENT_GATEWAY_URL=${VITE_AGENT_GATEWAY_URL}
ENV VITE_API_BASE_URL=${VITE_API_BASE_URL}
ENV VITE_ALLOW_DEV_AUTH=${VITE_ALLOW_DEV_AUTH}
ENV VITE_USE_MOCK_API=${VITE_USE_MOCK_API}
RUN npm run build

FROM nginx:1.29-alpine

COPY --from=build /app/frontend/dist /usr/share/nginx/html
COPY docker/nginx.frontend.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
