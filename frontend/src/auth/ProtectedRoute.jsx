import { Navigate, Outlet, useLocation } from "react-router-dom";
import { LoadingState } from "../components/ui";
import { useAuth } from "./useAuth.js";

export function ProtectedRoute() {
  const { loading, user } = useAuth();
  const location = useLocation();

  if (loading) {
    return <LoadingState layout="dashboard" title="Đang kiểm tra phiên đăng nhập..." />;
  }

  if (!user) {
    return <Navigate to="/login" replace state={{ from: location }} />;
  }

  return <Outlet />;
}
