import { Card, Col, Row, Statistic, Button, Typography, Result, Space, Empty } from 'antd';
import { ApiOutlined, CloudServerOutlined, LineChartOutlined, PlusOutlined } from '@ant-design/icons';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import api from '../api/client';
import type { ApiEnvelope, ForwardRule, SharedGroupSummary, SharedNodeSummary } from '../api/types';
import { useI18n } from '../i18n/context';
import { useAuth } from '../auth/useAuth';
import { formatBytes } from '../utils/format';

const { Text } = Typography;

/**
 * v0.4.12 PR1: regular user's home page.
 *
 * Data sources (regular-user safe): /groups/shared + /nodes/shared. A new user
 * with NO rules still sees all admin-provided lines. Online/total node counts
 * come from the backend's per-group aggregation (single online-window source of
 * truth) — the frontend does NOT recompute them.
 *
 * Empty states:
 * - 无管理员线路: "管理员暂未提供可用线路，请联系管理员。"
 * - 有线路无在线节点: "当前线路节点暂未在线，请稍后重试或联系管理员。"
 * - 有在线节点: show "创建转发规则" button
 *
 * NODE_TOKEN / install hints never appear here — that is admin-only material.
 */
export default function UserDashboard() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { user } = useAuth();
  const [rules, setRules] = useState<ForwardRule[]>([]);
  const [groups, setGroups] = useState<SharedGroupSummary[]>([]);
  const [nodes, setNodes] = useState<SharedNodeSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadFailed, setLoadFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const [rulesRes, groupsRes, nodesRes] = await Promise.all([
          api.get<unknown, ApiEnvelope<ForwardRule[]>>('/rules'),
          api.get<unknown, ApiEnvelope<SharedGroupSummary[]>>('/groups/shared'),
          api.get<unknown, ApiEnvelope<SharedNodeSummary[]>>('/nodes/shared'),
        ]);
        if (cancelled) return;
        // A non-zero code on the shared endpoints is a load failure, not an
        // empty state — surface it instead of pretending there are no lines.
        if (groupsRes.code !== 0 || nodesRes.code !== 0) {
          setLoadFailed(true);
          return;
        }
        setRules(rulesRes.data || []);
        setGroups(groupsRes.data || []);
        setNodes(nodesRes.data || []);
      } catch {
        if (!cancelled) setLoadFailed(true);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, []);

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', padding: 48 }}>
        <Text type="secondary">{t('loading') || 'Loading...'}</Text>
      </div>
    );
  }

  if (loadFailed) {
    return <Result status="warning" title={t('loadFailed')} subTitle={t('loadFailedRetry')} />;
  }

  // v0.4.13 PR2: nodes is now one row PER NODE. A group with no node yields a
  // placeholder row (empty node_id) — exclude those from the node totals.
  const realNodes = nodes.filter((n) => n.node_id !== '');
  const totalNodes = realNodes.length;
  const onlineNodes = realNodes.filter((n) => n.online).length;

  // Empty state: no admin-provided lines at all.
  if (groups.length === 0) {
    return (
      <Result
        status="info"
        icon={<Empty image={Empty.PRESENTED_IMAGE_SIMPLE} />}
        title={t('adminNoLines')}
      />
    );
  }

  const hasOnlineNode = onlineNodes > 0;
  const hasOfflineNodes = !hasOnlineNode;

  return (
    <>
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title={t('accountRulesLimit')}
              value={user?.current_rules ?? rules.length}
              suffix={`/ ${user?.max_rules ?? '-'}`}
              prefix={<ApiOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title={t('accountTrafficUsed')}
              value={formatBytes(user?.traffic_used ?? 0)}
              suffix={user && user.traffic_limit > 0 ? `/ ${formatBytes(user.traffic_limit)}` : ` / ${t('unlimited')}`}
              prefix={<CloudServerOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title={t('availableGroups')}
              value={groups.length}
              prefix={<CloudServerOutlined />}
            />
          </Card>
        </Col>
        <Col xs={24} sm={12} md={6}>
          <Card>
            <Statistic
              title={t('onlineNodes')}
              value={onlineNodes}
              suffix={`/ ${totalNodes}`}
              prefix={<LineChartOutlined />}
              valueStyle={hasOnlineNode ? { color: '#52c41a' } : { color: 'inherit' }}
            />
          </Card>
        </Col>
      </Row>

      {hasOfflineNodes && (
        <Card style={{ marginTop: 16 }}>
          <Result
            status="warning"
            title={t('allNodesOffline')}
            subTitle={t('allNodesOfflineHint')}
          />
        </Card>
      )}

      {hasOnlineNode && (
        <Card style={{ marginTop: 16 }}>
          <Space direction="vertical" style={{ width: '100%' }}>
            <Text type="secondary">{t('createRuleHint')}</Text>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => navigate('/rules')}
            >
              {t('createRuleButton')}
            </Button>
          </Space>
        </Card>
      )}
    </>
  );
}

