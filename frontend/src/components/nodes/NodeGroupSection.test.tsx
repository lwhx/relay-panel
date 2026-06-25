import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import type { Tfn } from './types';
import type { NodeDisplayRow } from '../../api/types';
import { statusTag } from './shared';
import { NodeGroupSection } from './NodeGroupSection';

// A fake t() that echoes the key — assertions match on the i18n KEY, not on a
// translated string, so the tests don't break when wording changes.
const t = ((key: string) => key) as unknown as Tfn;

function row(over: Partial<NodeDisplayRow>): NodeDisplayRow {
  return { group_id: 1, group_name: 'g1', node_id: 'n1', ...over };
}

describe('statusTag', () => {
  it('renders an online tag when the node is online', () => {
    render(<>{statusTag(row({ online: true }), t, 0)}</>);
    expect(screen.getByText('online')).toBeInTheDocument();
  });

  it('renders an offline tag when the node is offline', () => {
    render(<>{statusTag(row({ online: false }), t, 0)}</>);
    expect(screen.getByText('offline')).toBeInTheDocument();
  });

  it('flags a protocol mismatch over online/offline state', () => {
    // online is true, but the node's config protocol disagrees with the panel's
    render(<>{statusTag(row({ online: true, config_protocol_version: 1 }), t, 2)}</>);
    expect(screen.getByText('protocolIncompatible')).toBeInTheDocument();
    expect(screen.queryByText('online')).not.toBeInTheDocument();
  });
});

describe('NodeGroupSection mobile vs desktop', () => {
  // Disambiguate the two layouts: the desktop branch renders an antd Table
  // (.ant-table), the mobile branch renders plain card divs (no table).
  const rows = [row({ node_id: 'n1', online: true, cpu: 10 })];

  it('renders a table on desktop', () => {
    const { container } = render(
      <NodeGroupSection rows={rows} panelProtocol={0} currentVersion="0.4.15" isMobile={false} t={t} openDetail={vi.fn()} />,
    );
    expect(container.querySelector('.ant-table')).not.toBeNull();
  });

  it('renders no table (card list) on mobile', () => {
    const { container } = render(
      <NodeGroupSection rows={rows} panelProtocol={0} currentVersion="0.4.15" isMobile={true} t={t} openDetail={vi.fn()} />,
    );
    expect(container.querySelector('.ant-table')).toBeNull();
  });

  it('shows a "no node reporting" hint for a placeholder-only group', () => {
    const placeholder = [row({ node_id: null, online: false })];
    render(
      <NodeGroupSection rows={placeholder} panelProtocol={0} currentVersion="0.4.15" isMobile={false} t={t} openDetail={vi.fn()} />,
    );
    expect(screen.getByText('noNodeReportingInGroup')).toBeInTheDocument();
  });
});
