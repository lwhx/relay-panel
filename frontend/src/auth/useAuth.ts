import { useContext } from 'react';
import { AuthContext } from './AuthContext';

/** v0.4.10: the consumer hook for AuthContext. Kept in its own module (not in
 *  AuthContext.tsx) so AuthContext.tsx only exports the provider component —
 *  this satisfies react-refresh/only-export-components (fast refresh requires a
 *  file export either a component OR non-components, not both). */
export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within an AuthProvider');
  return ctx;
}
