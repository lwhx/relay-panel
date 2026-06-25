import { Result, Button } from 'antd';
import { useNavigate } from 'react-router-dom';
import { useI18n } from '../i18n/context';

/**
 * v0.4.10: explicit 403 page. Shown when a regular user navigates directly to
 * an admin-only route (/users, /tunnel-profiles). Replaces the old silent
 * redirect to /account — the user now gets clear feedback that the page is
 * admin-only, with a way back to their home.
 *
 * Note: API-level 403s (a non-admin calling an admin endpoint from an
 * unlocked page) are NOT routed here — those are handled by the caller. This
 * page is only for route-guard rejections (RequireAdmin → Navigate to /403).
 */
export default function Forbidden() {
  const { t } = useI18n();
  const navigate = useNavigate();
  return (
    <Result
      status="403"
      title="403"
      subTitle={t('forbiddenDesc')}
      extra={
        <Button type="primary" onClick={() => navigate('/')}>
          {t('backHome')}
        </Button>
      }
    />
  );
}
