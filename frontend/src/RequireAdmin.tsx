import { Navigate } from 'react-router-dom';
import { Spin } from 'antd';
import { useAuth } from './auth/useAuth';

/**
 * v0.4.10: gate admin-only routes. A logged-in non-admin is redirected to /403
 * (a real forbidden page) instead of silently bouncing to /account. The role
 * comes from AuthContext (resolved from /user/me on boot, never trusted
 * long-term from localStorage).
 *
 * This is defense-in-depth alongside the backend's AdminOnly → 403: even if a
 * non-admin navigates directly to /users, they see a 403 page, not the admin
 * UI flashing before the API rejects them.
 */
export function RequireAdmin({ children }: { children: React.ReactNode }) {
  const { isAdmin, authReady } = useAuth();
  if (!authReady) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', padding: 48 }}>
        <Spin />
      </div>
    );
  }
  if (!isAdmin) return <Navigate to="/403" replace />;
  return <>{children}</>;
}
