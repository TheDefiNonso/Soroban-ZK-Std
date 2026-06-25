/**
 * changelog.ts
 *
 * Reads and parses Markdown changelog entries from content/changelog/.
 * Each file has a YAML front-matter block containing release metadata.
 * Returns entries sorted newest-first — ready to be consumed by the
 * RSS route handler or any other page that needs changelog data.
 */

import fs from 'fs';
import path from 'path';

export interface ChangelogEntry {
  /** Human-readable title (from front-matter `title` field) */
  title: string;
  /** ISO date string: "YYYY-MM-DD" */
  date: string;
  /** Semantic type: "minor" | "patch" | "feat" | "fix" | "infra" | … */
  version: string;
  /** GitHub handle of the primary author */
  author: string;
  /** Full 40-char commit hash (optional) */
  commit?: string;
  /** Issues / PRs this entry closes (optional) */
  closes?: string[];
  /** Arbitrary tag labels (optional) */
  tags?: string[];
  /** Markdown body (everything after the closing `---`) */
  body: string;
  /** Source filename, e.g. "2026-05-26-patch.md" */
  filename: string;
}

const CHANGELOG_DIR = path.join(process.cwd(), 'content', 'changelog');

/**
 * Naively parse a YAML front-matter block that only uses simple scalar
 * values and inline arrays (`["a", "b"]` or `[a, b]` syntax).
 * Does NOT handle nested objects or multi-line values — that is intentional:
 * keep the changelog files simple and machine-friendly.
 */
function parseFrontMatter(raw: string): {
  meta: Record<string, unknown>;
  body: string;
} {
  const FM_RE = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$/;
  const match = raw.match(FM_RE);

  if (!match) {
    return { meta: {}, body: raw.trim() };
  }

  const [, yamlBlock, body] = match;
  const meta: Record<string, unknown> = {};

  for (const line of yamlBlock.split(/\r?\n/)) {
    const colonIdx = line.indexOf(':');
    if (colonIdx === -1) continue;

    const key = line.slice(0, colonIdx).trim();
    const rawValue = line.slice(colonIdx + 1).trim();

    // Inline array: ["#1", "#2"]  or  [tag1, tag2]
    if (rawValue.startsWith('[') && rawValue.endsWith(']')) {
      const inner = rawValue.slice(1, -1);
      meta[key] = inner
        .split(',')
        .map((s) =>
          s.trim().replace(/^["']|["']$/g, '')
        )
        .filter(Boolean);
    } else {
      // Scalar — strip surrounding quotes if present
      meta[key] = rawValue.replace(/^["']|["']$/g, '');
    }
  }

  return { meta, body: body.trim() };
}

/**
 * Load all changelog entries from `content/changelog/`, sorted
 * newest-first by the `date` front-matter field.
 */
export function getChangelogEntries(): ChangelogEntry[] {
  if (!fs.existsSync(CHANGELOG_DIR)) return [];

  const files = fs
    .readdirSync(CHANGELOG_DIR)
    .filter((f) => f.endsWith('.md') || f.endsWith('.mdx'))
    .sort()
    .reverse(); // lexicographic descending → newest date prefix first

  const entries: ChangelogEntry[] = [];

  for (const filename of files) {
    const filePath = path.join(CHANGELOG_DIR, filename);
    const raw = fs.readFileSync(filePath, 'utf-8');
    const { meta, body } = parseFrontMatter(raw);

    entries.push({
      title: String(meta.title ?? filename),
      date: String(meta.date ?? ''),
      version: String(meta.version ?? 'patch'),
      author: String(meta.author ?? 'unknown'),
      commit: meta.commit ? String(meta.commit) : undefined,
      closes: Array.isArray(meta.closes) ? (meta.closes as string[]) : undefined,
      tags: Array.isArray(meta.tags) ? (meta.tags as string[]) : undefined,
      body,
      filename,
    });
  }

  // Secondary sort: if filenames share the same date prefix, fall back to
  // insertion order (already newest-first from the reverse sort above).
  return entries.sort(
    (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()
  );
}
