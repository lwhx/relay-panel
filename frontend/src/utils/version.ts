import semver from 'semver';

/** Normalise a version string for semver comparison.
 *
 * Strips an optional `v` / `V` prefix and tolerates non-strict forms (e.g.
 * "0.3.4.1" -> "0.3.4") via `semver.coerce`, while preserving valid pre-release
 * tags (e.g. "0.3.4-alpha"). Returns `null` when there's nothing parseable —
 * callers MUST render a neutral placeholder in that case and never claim a node
 * is stale on a value they cannot compare.
 */
export function parseVersion(v?: string | null): string | null {
  if (!v) return null;
  const cleaned = v.trim().replace(/^v/i, '');
  if (!cleaned) return null;
  // semver.valid keeps a legal pre-release tag intact (so "0.3.4-alpha" sorts
  // below "0.3.4"). coerce is the fallback for 4-segment / otherwise loose
  // strings; it intentionally drops any pre-release suffix.
  if (semver.valid(cleaned)) return cleaned;
  const coerced = semver.coerce(cleaned);
  return coerced ? coerced.version : null;
}

export type VersionRelation = 'unknown' | 'behind' | 'same' | 'ahead';

/** Compare a node's reported version against the panel's.
 *
 *  - 'unknown' : either side unparseable -> render "-" / plain (no judgement)
 *  - 'behind'  : node < panel  -> "upgradable"
 *  - 'same'    : node == panel
 *  - 'ahead'   : node > panel  -> "newer" (never "stale")
 */
export function versionRelation(
  nodeVersion?: string | null,
  panelVersion?: string | null,
): VersionRelation {
  const a = parseVersion(nodeVersion);
  const b = parseVersion(panelVersion);
  if (!a || !b) return 'unknown';
  const cmp = semver.compare(a, b);
  if (cmp < 0) return 'behind';
  if (cmp > 0) return 'ahead';
  return 'same';
}

/** v0.4.14: map a version relation to an antd Tag color. Returns `undefined`
 *  (neutral / default grey tag) for `unknown` — a node whose version can't be
 *  compared (e.g. a regular user has no panel version to compare against, since
 *  /system/version is admin-only) must NOT be painted green "OK". `same` is
 *  green, `behind` orange (upgradable), `ahead` blue (newer). */
export function versionTagColor(rel: VersionRelation): string | undefined {
  switch (rel) {
    case 'behind':
      return 'orange';
    case 'ahead':
      return 'blue';
    case 'same':
      return 'green';
    case 'unknown':
      return undefined;
  }
}
