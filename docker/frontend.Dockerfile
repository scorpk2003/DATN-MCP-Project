FROM node:24-alpine AS build

WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
ARG VITE_AGENT_GATEWAY_URL=/api/agent-gateway
ENV VITE_AGENT_GATEWAY_URL=${VITE_AGENT_GATEWAY_URL}
RUN npm run build

FROM nginx:1.29-alpine

COPY --from=build /app/frontend/dist /usr/share/nginx/html
COPY docker/nginx.frontend.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
