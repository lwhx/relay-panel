import { Spin } from 'antd';
import { useAuth } from './auth/useAuth';
import Dashboard from './pages/Dashboard';
import UserDashboard from './pages/UserDashboard';

/**
 * v0.4.10: the index-route switch. Renders the admin Dashboard or the regular
 * user's UserDashboard based on the server-verified role. Kept in its own
 * module (not in router.tsx) so router.tsx only exports route config — this
 * satisfies react-refresh/only-export-components.
 *
 * Shows a spinner until authReady flips, so a page refresh doesn't flash the
 * wrong dashboard while /user/me resolves the real role.
 */
export default function RoleHome() {
  const { isAdmin, authReady } = useAuth();
  if (!authReady) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', padding: 48 }}>
        <Spin />
      </div>
    );
  }
  return isAdmin ? <Dashboard /> : <UserDashboard />;
}
