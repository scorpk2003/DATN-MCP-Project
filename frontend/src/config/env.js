export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "";
export const AGENT_GATEWAY_URL = import.meta.env.VITE_AGENT_GATEWAY_URL || "http://localhost:4000";

export const USE_MOCK_API =
  import.meta.env.VITE_USE_MOCK_API === "true" ||
  (!API_BASE_URL && import.meta.env.VITE_USE_MOCK_API !== "false");

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
