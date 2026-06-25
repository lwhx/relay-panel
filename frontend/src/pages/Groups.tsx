import { Table, Button, Modal, Form, Input, Select, Space, message, Popconfirm, Typography, Tag, Tooltip, Alert } from 'antd';
import { PlusOutlined, ReloadOutlined, CopyOutlined, EditOutlined, CloudServerOutlined, CodeOutlined } from '@ant-design/icons';
import { useCallback, useEffect, useState, type ReactNode } from 'react';
import api from '../api/client';
import type { ApiEnvelope, DeviceGroup, User } from '../api/types';
import { useI18n } from '../i18n/context';
import { copyText } from '../utils/clipboard';
import { useAuth } from '../auth/useAuth';

const { Text } = Typography;

const INSTALL_SCRIPT_URL = 'https://raw.githubusercontent.com/MoeShinX/relay-panel/main/scripts/relay-node-install.sh';

/** Build the one-line install command for a node. */
function buildInstallCommand(token: string, panelUrl: string): string {
  return `bash <(curl -fsSL ${INSTALL_SCRIPT_URL}) -t ${token} -u ${panelUrl}`;
}

/** Check if we're on a localhost address. */
function isLocalhost(): boolean {
  const h = window.location.hostname;
  return h === 'localhost' || h === '127.0.0.1' || h === '::1';
}

export default function Groups() {
  const { t } = useI18n();
  const { isAdmin } = useAuth();
  const [groups, setGroups] = useState<DeviceGroup[]>([]);
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [cmdModalOpen, setCmdModalOpen] = useState(false);
  const [cmdModalContent, setCmdModalContent] = useState<{ title: ReactNode; body: ReactNode }>({ title: null, body: null });
  const [editing, setEditing] = useState<DeviceGroup | null>(null);
  const [createForm] = Form.useForm();
  const [editForm] = Form.useForm();

  const load = useCallback(async () => {
    setLoading(true);
    try {
      // v0.4.10: /admin/users is admin-only and fetched separately so a regular
      // user doesn't 403 and block the groups list. The owner selector is
      // hidden for non-admins (backend forces owner = caller).
      const g = await api.get<unknown, ApiEnvelope<DeviceGroup[]>>('/groups');
      setGroups(g.data || []);
      if (isAdmin) {
        try {
          const u = await api.get<unknown, ApiEnvelope<User[]>>('/admin/users');
          setUsers(u.data || []);
        } catch {
          setUsers([]);
        }
      } else {
        setUsers([]);
      }
    } finally { setLoading(false); }
  }, [isAdmin]);

  useEffect(() => { load(); }, [load]);

  const handleCreate = async (values: { name: string; group_type: string; connect_host: string; port_range: string; owner_uid?: number | null }) => {
    try {
      // owner_uid is admin-only (create on behalf of another user). Omit when
      // empty so the request stays clean; the backend ignores it for non-admins.
      const payload = { ...values, owner_uid: values.owner_uid || undefined };
      const res = await api.post<unknown, ApiEnvelope<DeviceGroup>>('/groups', payload);
      if (res.code !== 0) { message.error(res.message); return; }
      message.success(t('groupCreated'));
      setCreateOpen(false);
      createForm.resetFields();
      load();
    } catch { message.error(t('failedCreateGroup')); }
  };

  const handleEdit = (g: DeviceGroup) => {
    setEditing(g);
    editForm.setFieldsValue({ name: g.name, group_type: g.group_type, connect_host: g.connect_host, port_range: g.port_range });
    setEditOpen(true);
  };

  const handleUpdate = async (values: { name?: string; group_type?: string; connect_host?: string; port_range?: string }) => {
    if (!editing) return;
    const payload: Record<string, unknown> = {};
    if (values.name !== undefined && values.name !== editing.name) payload.name = values.name;
    if (values.group_type !== undefined && values.group_type !== editing.group_type) payload.group_type = values.group_type;
    if (values.connect_host !== undefined && values.connect_host !== editing.connect_host) payload.connect_host = values.connect_host;
    if (values.port_range !== undefined && values.port_range !== editing.port_range) payload.port_range = values.port_range;
    if (Object.keys(payload).length === 0) { setEditOpen(false); return; }
    try {
      const res = await api.put<unknown, ApiEnvelope<null>>(`/groups/${editing.id}`, payload);
      if (res.code !== 0) { message.error(res.message); return; }
      message.success(t('groupUpdated'));
      setEditOpen(false);
      load();
    } catch { message.error(t('failedUpdateGroup')); }
  };

  const handleDelete = async (id: number) => {
    await api.delete(`/groups/${id}`);
    message.success(t('groupDeleted'));
    load();
  };

  const doCopy = async (text: string, successMsg: string) => {
    // Validate command looks correct before copying
    if (!text || text.length < 20) {
      message.error(t('copyFailed'));
      return;
    }
    const ok = await copyText(text);
    if (ok) {
      message.success(successMsg);
    } else {
      message.error(t('copyFailed'));
    }
  };

  const panelUrlRef = async (): Promise<string> => {
    try {
      const resp = await api.get<unknown, { public_panel_url?: string }>("/system/version");
      if (resp.public_panel_url) return resp.public_panel_url;
    } catch { /* ignore */ }
    return window.location.origin;
  };

  const showInstallCommand = async (g: DeviceGroup) => {
    const panelUrl = await panelUrlRef();
    const cmd = buildInstallCommand(g.token, panelUrl);
    setCmdModalContent({
      title: <span>{t('installCommandTitle')}</span>,
      body: (
        <>
{(isLocalhost() || panelUrl.includes("127.0.0.1") || panelUrl.includes("localhost") || panelUrl.includes("0.0.0.0")) && (
            <Alert
              type="warning"
              showIcon
              style={{ marginBottom: 12 }}
              message={t('localhostWarning')}
            />
          )}
          <Input.TextArea
            value={cmd}
            readOnly
            autoSize={{ minRows: 3, maxRows: 5 }}
            style={{ fontFamily: 'var(--rp-font-mono)', fontSize: 12 }}
          />
          <div style={{ textAlign: 'right', marginTop: 8 }}>
            <Button
              type="primary"
              icon={<CopyOutlined />}
              onClick={() => doCopy(cmd, t('installCommandCopied'))}
            >
              {t('copyInstallCommand')}
            </Button>
          </div>
        </>
      ),
    });
    setCmdModalOpen(true);
  };

  const typeColor = (gt: string) => {
    switch (gt) {
      case 'in': return 'green';
      case 'out': return 'cyan';
      case 'monitor': return 'default';
      default: return 'default';
    }
  };

  const columns = [
    { title: t('id'), dataIndex: 'id', key: 'id', width: 60 },
    { title: t('name'), dataIndex: 'name', key: 'name' },
    {
      title: t('type'), dataIndex: 'group_type', key: 'group_type',
      render: (gt: string) => <Tag color={typeColor(gt)}>{gt.toUpperCase()}</Tag>,
    },
    {
      title: t('nodeToken'), dataIndex: 'token', key: 'token',
      render: (tk: string, g: DeviceGroup) => (
        <Space>
          <Text code style={{ maxWidth: 180 }} ellipsis>{tk}</Text>
          <Tooltip title={t('copyInstallCommand')}>
            <Button size="small" type="text" icon={<CodeOutlined />} onClick={() => showInstallCommand(g)} />
          </Tooltip>
        </Space>
      ),
    },
    { title: t('connectHost'), dataIndex: 'connect_host', key: 'connect_host', render: (v: string) => <span className="rp-mono">{v}</span> },
    { title: t('portRange'), dataIndex: 'port_range', key: 'port_range', render: (v: string) => <span className="rp-mono">{v}</span> },
    {
      title: t('action'), key: 'action', width: 120,
      render: (_: unknown, g: DeviceGroup) => (
        <Space>
          <Button size="small" type="text" icon={<EditOutlined />} onClick={() => handleEdit(g)}>{t('edit')}</Button>
          <Popconfirm title={t('deleteGroupConfirm')} onConfirm={() => handleDelete(g.id)}>
            <Button danger size="small" type="text">{t('delete')}</Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  const groupTypeOptions = [
    { value: 'in', label: t('inboundListener') },
    { value: 'out', label: t('outboundEgress') },
    { value: 'monitor', label: t('typeMonitor') },
  ];

  return (
    <>
      <div className="rp-page-header">
        <h2 className="rp-page-title"><CloudServerOutlined /> {t('deviceGroups')}</h2>
        <Space>
          <Button icon={<ReloadOutlined />} onClick={load}>{t('refresh')}</Button>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setCreateOpen(true)}>{t('addGroup')}</Button>
        </Space>
      </div>
      <Table dataSource={groups} columns={columns} rowKey="id" loading={loading} pagination={{ pageSize: 20 }} />

      <Modal title={t('addGroup')} open={createOpen} onCancel={() => setCreateOpen(false)} onOk={() => createForm.submit()} okText={t('create')} cancelText={t('cancel')}>
        <Form form={createForm} onFinish={handleCreate} layout="vertical">
          <Form.Item name="name" label={t('name')} rules={[{ required: true }]}><Input placeholder="tokyo-node-1" /></Form.Item>
          {/* v0.4.10: admin-only owner selector (create on behalf of another
              user; omitted → owned by the caller). Backend ignores a non-admin
              value. Regular users always own what they create. */}
          {isAdmin && (
            <Form.Item name="owner_uid" label={t('owner')} extra={t('ownerHint')}>
              <Select allowClear placeholder={t('ownerSelf')} options={users.map(u => ({ value: u.id, label: u.username }))} />
            </Form.Item>
          )}
          <Form.Item name="group_type" label={t('type')} rules={[{ required: true }]} initialValue="in">
            <Select options={groupTypeOptions} />
          </Form.Item>
          <Form.Item name="connect_host" label={t('connectHost')} rules={[{ required: true }]}><Input placeholder="1.2.3.4 or node.example.com" /></Form.Item>
          <Form.Item name="port_range" label={t('portRange')} rules={[{ required: true }]} initialValue="10000-65535"><Input placeholder="10000-65535" /></Form.Item>
        </Form>
      </Modal>

      <Modal title={t('editGroup')} open={editOpen} onCancel={() => setEditOpen(false)} onOk={() => editForm.submit()} okText={t('save')} cancelText={t('cancel')}>
        <Form form={editForm} onFinish={handleUpdate} layout="vertical">
          <Form.Item name="name" label={t('name')}><Input /></Form.Item>
          <Form.Item name="group_type" label={t('type')}><Select options={groupTypeOptions} /></Form.Item>
          <Form.Item name="connect_host" label={t('connectHost')}><Input /></Form.Item>
          <Form.Item name="port_range" label={t('portRange')}><Input /></Form.Item>
        </Form>
      </Modal>

      <Modal
        title={cmdModalContent.title}
        open={cmdModalOpen}
        onCancel={() => setCmdModalOpen(false)}
        footer={null}
        width={580}
      >
        {cmdModalContent.body}
      </Modal>
    </>
  );
}
