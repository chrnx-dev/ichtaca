import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import TreePanel from '../src/components/TreePanel.svelte';
import type { EntryNode } from '../src/lib/types';

beforeEach(() => {
  vi.clearAllMocks();
});

const tree: EntryNode[] = [
  {
    name: 'web',
    path: null,
    children: [
      { name: 'github.com', path: 'web/github.com', children: [] },
      { name: 'gitlab.com', path: 'web/gitlab.com', children: [] },
    ],
  },
  {
    name: 'email',
    path: null,
    children: [{ name: 'work', path: 'email/work', children: [] }],
  },
];

describe('TreePanel', () => {
  it('renders directory names', () => {
    const { getByText } = render(TreePanel, {
      props: { tree, selectedPath: null, onselect: vi.fn() },
    });
    expect(getByText('web')).toBeInTheDocument();
    expect(getByText('email')).toBeInTheDocument();
  });

  it('expands a directory and shows children on click', async () => {
    const { getByText, queryByText } = render(TreePanel, {
      props: { tree, selectedPath: null, onselect: vi.fn() },
    });

    // Before expanding, leaf entries are not visible
    expect(queryByText('github.com')).not.toBeInTheDocument();

    // Click the 'web' directory button to expand it
    await fireEvent.click(getByText('web'));

    // Now children should be visible
    expect(getByText('github.com')).toBeInTheDocument();
    expect(getByText('gitlab.com')).toBeInTheDocument();
  });

  it('calls onselect with the correct path when a leaf entry is clicked', async () => {
    const onselect = vi.fn();
    const { getByText } = render(TreePanel, {
      props: { tree, selectedPath: null, onselect },
    });

    // Expand web directory first
    await fireEvent.click(getByText('web'));

    // Click a leaf
    await fireEvent.click(getByText('github.com'));

    expect(onselect).toHaveBeenCalledTimes(1);
    expect(onselect).toHaveBeenCalledWith('web/github.com');
  });

  it('highlights the selected entry', async () => {
    const { getByText } = render(TreePanel, {
      props: { tree, selectedPath: 'web/github.com', onselect: vi.fn() },
    });

    // Expand web to see github.com
    await fireEvent.click(getByText('web'));

    const btn = getByText('github.com').closest('button');
    expect(btn).toHaveClass('selected');
  });

  it('auto-reveals the selected entry inside a collapsed folder', () => {
    const infraTree: EntryNode[] = [
      {
        name: 'infra',
        path: null,
        children: [{ name: 'mac-studio', path: 'infra/mac-studio', children: [] }],
      },
    ];

    // No manual expansion: 'infra' starts collapsed, but the selection lives
    // inside it, so the leaf must be revealed (and highlighted).
    const { getByText } = render(TreePanel, {
      props: { tree: infraTree, selectedPath: 'infra/mac-studio', onselect: vi.fn() },
    });

    const leaf = getByText('mac-studio');
    expect(leaf).toBeInTheDocument();
    expect(leaf.closest('button')).toHaveClass('selected');
  });

  it('renders an empty tree without errors', () => {
    const { container } = render(TreePanel, {
      props: { tree: [], selectedPath: null, onselect: vi.fn() },
    });
    expect(container.querySelector('ul')).toBeInTheDocument();
  });
});
