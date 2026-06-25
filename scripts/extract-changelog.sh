#!/usr/bin/env bash
#
# Extract a single version's section body from CHANGELOG.md.
#
# Used by BOTH:
#   - scripts/release-check.sh  (pre-flight: assert the section exists + is
#     non-empty before tagging)
#   - .github/workflows/binary-release.yml  (generate the GitHub Release body
#     so the dashboard's "view changelog" is never blank / body=null)
#
# Usage:
#   bash scripts/extract-changelog.sh <version> [changelog-file]
#   bash scripts/extract-changelog.sh 0.3.5
#   bash scripts/extract-changelog.sh v0.3.5 CHANGELOG.md
#
# Prints the section body (without the "## [x.y.z] - date" heading and without
# the trailing "---" separator) to stdout. Leading / trailing blank lines are
# trimmed. Exits non-zero (and prints nothing to stdout) when the section is
# missing or contains no non-whitespace content — callers MUST treat that as a
# hard failure so we never publish an empty release body.
#
set -euo pipefail
export MSYS_NO_PATHCONV=1

if [ $# -lt 1 ]; then
    echo "Usage: bash scripts/extract-changelog.sh <version> [changelog-file]" >&2
    exit 2
fi

RAW="$1"
VERSION="${RAW#[vV]}"
FILE="${2:-CHANGELOG.md}"

if [ ! -f "$FILE" ]; then
    echo "extract-changelog: '$FILE' not found" >&2
    exit 1
fi

# awk state machine:
#   - start capturing on the line that begins with "## [<version>]"
#     (index(...)==1 = literal prefix match, so the brackets aren't treated as
#     a regex character class)
#   - stop at the next "## [" section heading OR the "---" separator that ends
#     this section, whichever comes first
#   - trim leading / trailing blank lines from the captured body
BODY="$(awk -v needle="## [${VERSION}]" '
    index($0, needle) == 1 { cap = 1; next }
    cap && (index($0, "## [") == 1 || $0 ~ /^---[[:space:]]*$/) { exit }
    cap { lines[n++] = $0 }
    END {
        start = 0
        while (start < n && lines[start] ~ /^[[:space:]]*$/) start++
        end = n - 1
        while (end >= start && lines[end] ~ /^[[:space:]]*$/) end--
        for (i = start; i <= end; i++) print lines[i]
    }
' "$FILE")"

# Reject an empty / whitespace-only section.
if [ -z "${BODY//[[:space:]]/}" ]; then
    echo "extract-changelog: no non-empty '## [${VERSION}]' section in $FILE" >&2
    exit 1
fi

printf '%s\n' "$BODY"
