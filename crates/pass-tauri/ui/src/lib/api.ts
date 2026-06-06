/**
 * Typed wrappers over `invoke` from @tauri-apps/api/core.
 * Command names and arg keys match the Rust fn names exactly as registered
 * in generate_handler! in crates/pass-tauri/src/lib.rs.
 *
 * The `invoke` import is the only Tauri boundary — mock it in tests via
 * `vi.mock('@tauri-apps/api/core')`.
 */
import { invoke } from '@tauri-apps/api/core';
import type { EntryMeta, EntryNode, OtpCode } from './types';

// ── Command wrappers ──────────────────────────────────────────────────────────

/** Returns all password entry paths (flat list). */
export function list(): Promise<string[]> {
  return invoke('list');
}

/** Returns metadata for the entry at `path` (no password, no OTP URI). */
export function showMeta(path: string): Promise<EntryMeta> {
  return invoke('show_meta', { path });
}

/**
 * Fetches the plaintext password for `path`.
 * Should only be called when the user explicitly clicks "Reveal".
 */
export function revealPassword(path: string): Promise<string> {
  return invoke('reveal_password', { path });
}

/**
 * Asks the backend to copy the password to the clipboard.
 * The plaintext password NEVER enters JavaScript for this operation.
 */
export function copyPassword(path: string): Promise<void> {
  return invoke('copy_password', { path });
}

/** Returns the current TOTP code and remaining seconds for `path`. */
export function otpCode(path: string): Promise<OtpCode> {
  return invoke('otp_code', { path });
}

/** Returns paths matching `query` using fuzzy matching. */
export function searchFuzzy(query: string): Promise<string[]> {
  return invoke('search_fuzzy', { query });
}

// ── Tree builder ──────────────────────────────────────────────────────────────

/**
 * Builds a tree from a flat list of slash-separated paths.
 * Mirrors the `EntryNode::from_paths` logic on the Rust side.
 *
 * Examples:
 *   ["web/github.com", "web/gitlab.com", "email/work"]
 *   → [
 *       { name: "web",   path: null,           children: [
 *           { name: "github.com", path: "web/github.com", children: [] },
 *           { name: "gitlab.com", path: "web/gitlab.com", children: [] },
 *       ]},
 *       { name: "email", path: null,            children: [
 *           { name: "work",       path: "email/work",     children: [] },
 *       ]},
 *     ]
 *
 * Leaf nodes (no further `/` segment at that depth) carry the full path.
 * Directory nodes have path === null.
 */
export function buildTree(paths: string[]): EntryNode[] {
  // Map from name → mutable node reference
  type MutableNode = { name: string; path: string | null; children: MutableNode[] };

  function insertPath(nodes: MutableNode[], segments: string[], fullPath: string): void {
    if (segments.length === 0) return;
    const [head, ...rest] = segments;
    let node = nodes.find((n) => n.name === head);
    if (!node) {
      node = { name: head, path: null, children: [] };
      nodes.push(node);
    }
    if (rest.length === 0) {
      // Leaf: carry the full path
      node.path = fullPath;
    } else {
      insertPath(node.children, rest, fullPath);
    }
  }

  const roots: MutableNode[] = [];
  for (const p of paths) {
    const segments = p.split('/').filter(Boolean);
    insertPath(roots, segments, p);
  }
  return roots as EntryNode[];
}
