import { useEffect, useMemo, useState } from "react";
import { isFirebaseConfigured } from "../config/env.js";
import { AuthContext } from "./AuthContext.js";
import { signOutUser, subscribeToAuthState } from "./authService.js";

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const unsubscribe = subscribeToAuthState((nextUser) => {
      setUser(nextUser);
      setLoading(false);
    });

    return unsubscribe;
  }, []);

  const value = useMemo(
    () => ({
      firebaseConfigured: isFirebaseConfigured,
      loading,
      signOut: signOutUser,
      user,
    }),
    [loading, user],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
