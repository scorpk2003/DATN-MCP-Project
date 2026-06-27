import { faGoogle } from "@fortawesome/free-brands-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useState } from "react";
import { Navigate, useLocation, useNavigate } from "react-router-dom";
import { signInWithEmail, signInWithGoogle } from "../../auth/authService.js";
import { useAuth } from "../../auth/useAuth.js";
import { Button, Card, InlineAlert } from "../../components/ui";

function getAuthErrorMessage(error) {
  const code = error?.code || "";

  if (code.includes("invalid-credential") || code.includes("wrong-password")) {
    return "Email hoặc mật khẩu không đúng.";
  }

  if (code.includes("user-not-found")) {
    return "Không tìm thấy tài khoản với email này.";
  }

  if (code.includes("popup-closed-by-user")) {
    return "Cửa sổ đăng nhập Google đã bị đóng.";
  }

  return error?.message || "Không thể đăng nhập. Vui lòng thử lại.";
}

function LoginPage() {
  const { firebaseConfigured, user } = useAuth();
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [loadingProvider, setLoadingProvider] = useState("");
  const [password, setPassword] = useState("");
  const location = useLocation();
  const navigate = useNavigate();
  const from = location.state?.from?.pathname || "/";

  if (user) {
    return <Navigate to={from} replace />;
  }

  const completeLogin = () => {
    navigate(from, { replace: true });
  };

  const handleEmailLogin = async (event) => {
    event.preventDefault();
    setError("");
    setLoadingProvider("email");

    try {
      await signInWithEmail(email, password);
      completeLogin();
    } catch (nextError) {
      setError(getAuthErrorMessage(nextError));
    } finally {
      setLoadingProvider("");
    }
  };

  const handleGoogleLogin = async () => {
    setError("");
    setLoadingProvider("google");

    try {
      await signInWithGoogle();
      completeLogin();
    } catch (nextError) {
      setError(getAuthErrorMessage(nextError));
    } finally {
      setLoadingProvider("");
    }
  };

  return (
    <main className="grid min-h-screen place-items-center bg-[var(--bg-canvas)] px-4 py-8">
      <Card className="w-full max-w-md space-y-5 p-6">
        <div>
          <p className="text-sm font-semibold text-[var(--text-muted)]">SelfLearn</p>
          <h1 className="mt-1 text-3xl font-bold text-[var(--text-primary)]">Đăng nhập</h1>
        </div>

        {!firebaseConfigured ? (
          <InlineAlert
            tone="warning"
            title="Firebase Auth chưa được cấu hình"
            description="Hãy thêm các biến VITE_FIREBASE_API_KEY, VITE_FIREBASE_AUTH_DOMAIN, VITE_FIREBASE_PROJECT_ID và VITE_FIREBASE_APP_ID trong frontend/.env."
          />
        ) : null}

        {error ? <InlineAlert tone="risk" title="Đăng nhập thất bại" description={error} /> : null}

        <form className="space-y-3" onSubmit={handleEmailLogin}>
          <label className="block text-sm font-semibold text-[var(--text-secondary)]">
            Email
            <input
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              type="email"
              autoComplete="email"
              className="mt-1 h-10 w-full cursor-text rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 text-sm text-[var(--text-primary)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
              required
            />
          </label>
          <label className="block text-sm font-semibold text-[var(--text-secondary)]">
            Mật khẩu
            <input
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              type="password"
              autoComplete="current-password"
              className="mt-1 h-10 w-full cursor-text rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 text-sm text-[var(--text-primary)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
              required
            />
          </label>
          <Button
            type="submit"
            full
            loading={loadingProvider === "email"}
            disabled={!firebaseConfigured || Boolean(loadingProvider)}
          >
            Đăng nhập bằng email
          </Button>
        </form>

        <Button
          variant="secondary"
          full
          loading={loadingProvider === "google"}
          disabled={!firebaseConfigured || Boolean(loadingProvider)}
          onClick={handleGoogleLogin}
        >
          <FontAwesomeIcon icon={faGoogle} />
          Đăng nhập với Google
        </Button>
      </Card>
    </main>
  );
}

export default LoginPage;
