#!/usr/bin/env bash
# Validates that all docs/ files (excluding specs/) use .mdx, not .md.
# Exits non-zero and prints actionable errors if any violations are found.

set -euo pipefail

DOCS_DIR="${1:-docs}"
VIOLATIONS=0

while IFS= read -r -d '' file; do
  rel="${file#./}"
  echo "ERROR: Documentation files must use .mdx. Move ${rel} -> ${rel%.md}.mdx" >&2
  VIOLATIONS=$((VIOLATIONS + 1))
done < <(find "$DOCS_DIR" -path "*/specs" -prune -o -name "*.md" -print0)

if [ "$VIOLATIONS" -gt 0 ]; then
  echo "" >&2
  echo "Found $VIOLATIONS forbidden .md file(s) outside specs/." >&2
  echo "Rename each file to .mdx and update any internal links." >&2
  exit 1
fi

echo "OK: all documentation files use .mdx"