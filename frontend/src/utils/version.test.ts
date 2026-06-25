import { describe, expect, it } from 'vitest';
import { parseVersion, versionRelation, versionTagColor } from './version';

describe('parseVersion', () => {
  it('strips an optional v / V prefix', () => {
    expect(parseVersion('v0.3.4')).toBe('0.3.4');
    expect(parseVersion('V0.3.4')).toBe('0.3.4');
    expect(parseVersion('0.3.4')).toBe('0.3.4');
  });

  it('keeps a valid pre-release tag intact', () => {
    expect(parseVersion('0.3.4-alpha')).toBe('0.3.4-alpha');
    expect(parseVersion('v0.3.4-rc.1')).toBe('0.3.4-rc.1');
  });

  it('coerces loose 4-segment versions to a comparable form', () => {
    expect(parseVersion('0.3.4.1')).toBe('0.3.4');
  });

  it('returns null for missing or unparseable input', () => {
    expect(parseVersion(undefined)).toBeNull();
    expect(parseVersion(null)).toBeNull();
    expect(parseVersion('')).toBeNull();
    expect(parseVersion('   ')).toBeNull();
    expect(parseVersion('not-a-version')).toBeNull();
  });
});

describe('versionRelation', () => {
  it('reports behind when node < panel', () => {
    expect(versionRelation('0.3.3', '0.3.4')).toBe('behind');
  });

  it('reports same when node == panel (v-prefix agnostic)', () => {
    expect(versionRelation('0.3.4', '0.3.4')).toBe('same');
    expect(versionRelation('v0.3.4', '0.3.4')).toBe('same');
  });

  it('reports ahead when node > panel (never mislabels newer as stale)', () => {
    expect(versionRelation('0.3.5', '0.3.4')).toBe('ahead');
  });

  it('treats a pre-release as behind its stable release', () => {
    expect(versionRelation('0.3.4-alpha', '0.3.4')).toBe('behind');
  });

  it('handles the loose panel version 0.3.4.1 used in field builds', () => {
    // 0.3.4.1 coerces to 0.3.4, so a 0.3.3 node is still behind.
    expect(versionRelation('0.3.3', '0.3.4.1')).toBe('behind');
    expect(versionRelation('0.3.4', '0.3.4.1')).toBe('same');
  });

  it('returns unknown when either side is unparseable (no false stale)', () => {
    expect(versionRelation(undefined, '0.3.4')).toBe('unknown');
    expect(versionRelation('0.3.4', undefined)).toBe('unknown');
    expect(versionRelation('garbage', '0.3.4')).toBe('unknown');
    expect(versionRelation('', '')).toBe('unknown');
  });
});

describe('versionTagColor', () => {
  it('maps unknown to a NEUTRAL tag (undefined), never green', () => {
    // v0.4.14 #76: a regular user has no panel version (/system/version is
    // admin-only) → versionRelation is 'unknown'. It must NOT render green "OK".
    expect(versionTagColor('unknown')).toBeUndefined();
  });

  it('maps comparable relations to their colors', () => {
    expect(versionTagColor('same')).toBe('green');
    expect(versionTagColor('behind')).toBe('orange');
    expect(versionTagColor('ahead')).toBe('blue');
  });
});
