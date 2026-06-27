import { initializeApp } from "firebase/app";
import {
  GoogleAuthProvider,
  getAuth,
  onAuthStateChanged,
  signInWithEmailAndPassword,
  signInWithPopup,
  signOut,
} from "firebase/auth";
import { firebaseConfig, isFirebaseConfigured } from "../config/env.js";

let firebaseApp;
let firebaseAuth;

export function getFirebaseAuth() {
  if (!isFirebaseConfigured) {
    return null;
  }

  if (!firebaseApp) {
    firebaseApp = initializeApp(firebaseConfig);
    firebaseAuth = getAuth(firebaseApp);
  }

  return firebaseAuth;
}

export function subscribeToAuthState(callback) {
  const auth = getFirebaseAuth();

  if (!auth) {
    callback(null);
    return () => {};
  }

  return onAuthStateChanged(auth, callback);
}

export async function signInWithEmail(email, password) {
  const auth = getFirebaseAuth();

  if (!auth) {
    throw new Error("Firebase Auth chưa được cấu hình.");
  }

  return signInWithEmailAndPassword(auth, email, password);
}

export async function signInWithGoogle() {
  const auth = getFirebaseAuth();

  if (!auth) {
    throw new Error("Firebase Auth chưa được cấu hình.");
  }

  const provider = new GoogleAuthProvider();
  return signInWithPopup(auth, provider);
}

export async function signOutUser() {
  const auth = getFirebaseAuth();

  if (!auth) {
    return;
  }

  await signOut(auth);
}

export async function getAuthToken() {
  const auth = getFirebaseAuth();

  if (!auth?.currentUser) {
    return null;
  }

  return auth.currentUser.getIdToken();
}
