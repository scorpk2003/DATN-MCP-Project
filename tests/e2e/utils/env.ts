export const E2E_USER = {
  id: process.env.E2E_USER_ID ?? 'dev-learner',
  email: process.env.E2E_USER_EMAIL ?? 'dev-learner@local.test',
  password: process.env.E2E_USER_PASSWORD ?? 'password123',
};

export const E2E_AUTH = {
  devAuth: process.env.E2E_DEV_AUTH !== 'false',
};

export const ROUTES = {
  login: process.env.E2E_LOGIN_PATH ?? '/login',
  dashboard: process.env.E2E_DASHBOARD_PATH ?? '/',
  roadmap: process.env.E2E_ROADMAP_PATH ?? '/roadmap',
  lesson: process.env.E2E_LESSON_PATH ?? '/',
  orchestrator: process.env.E2E_ORCHESTRATOR_PATH ?? '/',
};
