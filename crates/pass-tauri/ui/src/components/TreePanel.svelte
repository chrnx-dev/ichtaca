<script lang="ts">
  import TreePanel from './TreePanel.svelte';
  import type { EntryNode } from '../lib/types';

  interface Props {
    tree: EntryNode[];
    selectedPath: string | null;
    onselect: (path: string) => void;
  }

  let { tree, selectedPath, onselect }: Props = $props();

  // Track which directory nodes are expanded
  let expanded = $state<Set<string>>(new Set());

  function toggleDir(name: string) {
    const next = new Set(expanded);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
    }
    expanded = next;
  }

  // True when any descendant leaf of `node` is the currently selected entry.
  // Used to auto-reveal the path to the selected entry: a collapsed folder
  // whose subtree contains the selection is shown as open so the highlighted
  // leaf is visible. The recursion handles arbitrary nesting depth.
  function subtreeContains(node: EntryNode, path: string | null): boolean {
    if (path === null) return false;
    if (node.path !== null) return node.path === path;
    return node.children.some((child) => subtreeContains(child, path));
  }

  // A directory renders its children when manually expanded OR when its subtree
  // contains the selected entry (auto-reveal).
  function isOpen(node: EntryNode): boolean {
    return expanded.has(node.name) || subtreeContains(node, selectedPath);
  }

  function handleKeydown(e: KeyboardEvent, node: EntryNode) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      if (node.path !== null) {
        onselect(node.path);
      } else {
        toggleDir(node.name);
      }
    }
  }
</script>

<nav class="tree-panel py-1" aria-label="Password entries">
  <ul role="tree" class="list-none m-0 p-0 w-full">
    {#each tree as node (node.name)}
      <li role="treeitem" aria-selected={node.path === selectedPath} aria-expanded={node.children.length > 0 ? isOpen(node) : undefined}>
        {#if node.path !== null}
          <!-- Leaf entry -->
          <button
            class="entry-leaf flex items-center gap-2 w-full text-left px-3 py-1.5 text-sm bg-transparent border-0 transition-colors focus:outline-none
              {node.path === selectedPath
                ? 'selected text-primary font-semibold'
                : 'text-base-content hover:text-primary'}"
            onclick={() => onselect(node.path!)}
            onkeydown={(e) => handleKeydown(e, node)}
            aria-current={node.path === selectedPath ? 'true' : undefined}
          >
            <!-- Key icon for leaf entries -->
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 flex-shrink-0 {node.path === selectedPath ? 'text-primary' : 'text-neutral'}" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 8a6 6 0 01-7.743 5.743L10 14l-1 1-1 1H6v2H2v-4l4.257-4.257A6 6 0 1118 8zm-6-4a1 1 0 100 2 2 2 0 012 2 1 1 0 102 0 4 4 0 00-4-4z" clip-rule="evenodd"/>
            </svg>
            {node.name}
          </button>
        {:else}
          <!-- Directory -->
          <button
            class="entry-dir flex items-center gap-2 w-full text-left px-3 py-1.5 text-sm font-semibold bg-transparent border-0 transition-colors focus:outline-none
              text-base-content/80 hover:text-primary"
            onclick={() => toggleDir(node.name)}
            onkeydown={(e) => handleKeydown(e, node)}
            aria-label="Toggle directory {node.name}"
          >
            <!-- Chevron / folder indicator -->
            <span class="dir-icon text-neutral text-xs w-3 flex-shrink-0">
              {isOpen(node) ? '▾' : '▸'}
            </span>
            <!-- Folder icon -->
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 flex-shrink-0 text-neutral" viewBox="0 0 20 20" fill="currentColor">
              <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
            </svg>
            {node.name}
          </button>
          {#if isOpen(node)}
            <div class="subtree pl-3 border-l border-neutral/20 ml-3">
              <TreePanel tree={node.children} {selectedPath} {onselect} />
            </div>
          {/if}
        {/if}
      </li>
    {/each}
  </ul>
</nav>

<style>
  /* Keep the .tree-panel and .selected class present for the Vitest test
     that checks toHaveClass('selected') */
  .tree-panel {
    overflow-y: auto;
    height: 100%;
  }
</style>
