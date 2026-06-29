export const AGENT_GATEWAY_URL = import.meta.env.VITE_AGENT_GATEWAY_URL || "/api/agent-gateway";
export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || AGENT_GATEWAY_URL;

export const USE_MOCK_API = import.meta.env.VITE_USE_MOCK_API === "true";
export const ALLOW_DEV_AUTH = import.meta.env.VITE_ALLOW_DEV_AUTH === "true";

export const firebaseConfig = {
  apiKey: import.meta.env.VITE_FIREBASE_API_KEY || "",
  appId: import.meta.env.VITE_FIREBASE_APP_ID || "",
  authDomain: import.meta.env.VITE_FIREBASE_AUTH_DOMAIN || "",
  messagingSenderId: import.meta.env.VITE_FIREBASE_MESSAGING_SENDER_ID || "",
  projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID || "",
  storageBucket: import.meta.env.VITE_FIREBASE_STORAGE_BUCKET || "",
};

export const isFirebaseConfigured = Boolean(
  firebaseConfig.apiKey &&
    firebaseConfig.appId &&
    firebaseConfig.authDomain &&
    firebaseConfig.projectId,
);
