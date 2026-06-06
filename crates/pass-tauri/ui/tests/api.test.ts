import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  list,
  showMeta,
  revealPassword,
  copyPassword,
  otpCode,
  searchFuzzy,
  buildTree,
} from '../src/lib/api';

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
});

// ── API wrapper tests ─────────────────────────────────────────────────────────

describe('list()', () => {
  it('calls invoke with command "list" and no args', async () => {
    mockInvoke.mockResolvedValueOnce(['web/github.com', 'email/work']);
    const result = await list();
    expect(mockInvoke).toHaveBeenCalledWith('list');
    expect(result).toEqual(['web/github.com', 'email/work']);
  });
});

describe('showMeta()', () => {
  it('calls invoke("show_meta") with { path }', async () => {
    const mockMeta = {
      path: 'web/github.com',
      fields: [['user', 'bob']],
      tags: ['work'],
      has_otp: true,
    };
    mockInvoke.mockResolvedValueOnce(mockMeta);
    const result = await showMeta('web/github.com');
    expect(mockInvoke).toHaveBeenCalledWith('show_meta', { path: 'web/github.com' });
    expect(result).toEqual(mockMeta);
  });
});

describe('revealPassword()', () => {
  it('calls invoke("reveal_password") with { path }', async () => {
    mockInvoke.mockResolvedValueOnce('s3cret!');
    const result = await revealPassword('web/github.com');
    expect(mockInvoke).toHaveBeenCalledWith('reveal_password', { path: 'web/github.com' });
    expect(result).toBe('s3cret!');
  });
});

describe('copyPassword()', () => {
  it('calls invoke("copy_password") with { path }', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await copyPassword('web/github.com');
    expect(mockInvoke).toHaveBeenCalledWith('copy_password', { path: 'web/github.com' });
  });

  it('does NOT call "reveal_password" for copy', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    await copyPassword('web/github.com');
    const calls = mockInvoke.mock.calls.map(([cmd]) => cmd);
    expect(calls).not.toContain('reveal_password');
  });
});

describe('otpCode()', () => {
  it('calls invoke("otp_code") with { path }', async () => {
    const mockOtp = { code: '123456', seconds: 25 };
    mockInvoke.mockResolvedValueOnce(mockOtp);
    const result = await otpCode('web/github.com');
    expect(mockInvoke).toHaveBeenCalledWith('otp_code', { path: 'web/github.com' });
    expect(result).toEqual(mockOtp);
  });
});

describe('searchFuzzy()', () => {
  it('calls invoke("search_fuzzy") with { query }', async () => {
    mockInvoke.mockResolvedValueOnce(['web/github.com']);
    const result = await searchFuzzy('git');
    expect(mockInvoke).toHaveBeenCalledWith('search_fuzzy', { query: 'git' });
    expect(result).toEqual(['web/github.com']);
  });
});

// ── buildTree tests ───────────────────────────────────────────────────────────

describe('buildTree()', () => {
  it('returns empty array for empty input', () => {
    expect(buildTree([])).toEqual([]);
  });

  it('groups web/github.com and web/gitlab.com under a "web" directory', () => {
    const tree = buildTree(['web/github.com', 'web/gitlab.com']);
    expect(tree).toHaveLength(1);
    const web = tree[0];
    expect(web.name).toBe('web');
    expect(web.path).toBeNull();
    expect(web.children).toHaveLength(2);
    const names = web.children.map((c) => c.name);
    expect(names).toContain('github.com');
    expect(names).toContain('gitlab.com');
  });

  it('assigns full paths to leaf nodes', () => {
    const tree = buildTree(['web/github.com']);
    const leaf = tree[0].children[0];
    expect(leaf.path).toBe('web/github.com');
  });

  it('creates separate top-level nodes for different dirs', () => {
    const tree = buildTree(['web/github.com', 'email/work']);
    const topNames = tree.map((n) => n.name);
    expect(topNames).toContain('web');
    expect(topNames).toContain('email');
  });

  it('handles deeply nested paths', () => {
    const tree = buildTree(['a/b/c/d']);
    expect(tree[0].name).toBe('a');
    expect(tree[0].path).toBeNull();
    expect(tree[0].children[0].name).toBe('b');
    expect(tree[0].children[0].path).toBeNull();
    expect(tree[0].children[0].children[0].name).toBe('c');
    expect(tree[0].children[0].children[0].path).toBeNull();
    expect(tree[0].children[0].children[0].children[0].name).toBe('d');
    expect(tree[0].children[0].children[0].children[0].path).toBe('a/b/c/d');
  });

  it('handles single-segment paths (no directory)', () => {
    const tree = buildTree(['solo']);
    expect(tree).toHaveLength(1);
    expect(tree[0].name).toBe('solo');
    expect(tree[0].path).toBe('solo');
    expect(tree[0].children).toHaveLength(0);
  });
});
