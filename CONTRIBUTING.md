# Contributing to Soroban-ZK-Std

Thank you for contributing to Soroban-ZK-Std. This document covers the documentation standards, branch conventions, and submission process.

---

## Documentation Policy

### General Documentation

All documentation files in `docs/` (and subdirectories, **except** `docs/specs/`) **must** use the `.mdx` extension.

- Supports embedded React components
- Required for the documentation viewer / Next.js frontend
- Must include YAML frontmatter with at minimum `title` and `description` fields

**Examples:**

```
✅ docs/tutorials/getting-started.mdx
✅ docs/guides/installation.mdx
✅ docs/math/rescue.mdx

❌ docs/tutorials/getting-started.md
❌ docs/guides/setup.md
```

### Algorithm Specifications

Files in `docs/specs/` **may** remain `.md`.

- Intended for mathematical proofs and formal algorithm specifications
- Plain Markdown keeps the content portable and diff-friendly
- React components are not required here

**Examples:**

```
✅ docs/specs/pairing-proof.md
✅ docs/specs/poseidon2-spec.md
```

### Frontmatter Requirements

Every `.mdx` file in `docs/` must open with a YAML frontmatter block:

```yaml
---
title: "Human-readable page title"
description: "One-sentence summary shown in search results and link previews"
protocol: "25"
cap: "CAP-0075"
---
```

The `protocol` and `cap` fields are strongly recommended for all ZK-related docs.

---

## Validation

Before opening a PR, run the doc-extension check locally:

```bash
bash scripts/check-doc-extensions.sh
```

This script traverses `docs/`, skips `docs/specs/`, and exits non-zero if any forbidden `.md` file is found. CI runs the same check on every pull request.

---

## Pull Request Checklist

- [ ] New documentation files use `.mdx` (unless under `docs/specs/`)
- [ ] Frontmatter includes `title` and `description`
- [ ] `bash scripts/check-doc-extensions.sh` exits 0
- [ ] Internal links between docs use `.mdx` paths, not `.md`
- [ ] No unrelated changes bundled into the PR