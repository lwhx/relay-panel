import { Navigate } from 'react-router-dom';
import { Spin } from 'antd';
import { useAuth } from './auth/useAuth';

// Wrapper that redirects to /login if no token is present. Kept in its own
// module so the router file only exports route configuration (and thus
// satisfies react-refresh/only-export-components).
//
// v0.4.10: reads auth state from AuthContext (single source of truth) instead
// of localStorage directly. Shows a spinner until authReady flips — this
// avoids a flash of /login on page refresh while /user/me is resolving.
//
// v0.4.10 PR4: if the user must change their password (admin reset with a
// temporary password), redirect to the dedicated /force-password-change page.
// That page is a TOP-LEVEL route (not wrapped by RequireAuth), so it can't
// redirect to itself — no infinite loop. The backend also enforces this
// (403 PASSWORD_CHANGE_REQUIRED on every non-whitelisted endpoint), so this
// guard is UX defense-in-depth, not the security boundary.
export function RequireAuth({ children }: { children: React.ReactNode }) {
  const { token, authReady, mustChangePassword } = useAuth();
  if (!authReady) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', padding: 48 }}>
        <Spin />
      </div>
    );
  }
  if (!token) return <Navigate to="/login" replace />;
  if (mustChangePassword) return <Navigate to="/force-password-change" replace />;
  return <>{children}</>;
}
