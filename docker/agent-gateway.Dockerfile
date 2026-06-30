FROM node:24-alpine AS deps

WORKDIR /app/agent_gateway
COPY agent_gateway/package*.json ./
RUN npm ci

FROM node:24-alpine

WORKDIR /app/agent_gateway
ENV NODE_ENV=production
COPY --from=deps /app/agent_gateway/node_modules ./node_modules
COPY agent_gateway/ ./
COPY tools/ ../tools/

EXPOSE 4000
CMD ["npm", "run", "start"]
