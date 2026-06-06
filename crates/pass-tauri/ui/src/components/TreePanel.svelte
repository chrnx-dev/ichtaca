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

<nav class="tree-panel" aria-label="Password entries">
  <ul role="tree">
    {#each tree as node (node.name)}
      <li role="treeitem" aria-selected={node.path === selectedPath} aria-expanded={node.children.length > 0 ? expanded.has(node.name) : undefined}>
        {#if node.path !== null}
          <!-- Leaf entry -->
          <button
            class="entry-leaf"
            class:selected={node.path === selectedPath}
            onclick={() => onselect(node.path!)}
            onkeydown={(e) => handleKeydown(e, node)}
            aria-current={node.path === selectedPath ? 'true' : undefined}
          >
            {node.name}
          </button>
        {:else}
          <!-- Directory -->
          <button
            class="entry-dir"
            onclick={() => toggleDir(node.name)}
            onkeydown={(e) => handleKeydown(e, node)}
            aria-label="Toggle directory {node.name}"
          >
            <span class="dir-icon">{expanded.has(node.name) ? '▾' : '▸'}</span>
            {node.name}
          </button>
          {#if expanded.has(node.name)}
            <div class="subtree">
              <TreePanel tree={node.children} {selectedPath} {onselect} />
            </div>
          {/if}
        {/if}
      </li>
    {/each}
  </ul>
</nav>

<style>
  .tree-panel {
    overflow-y: auto;
    height: 100%;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    margin: 0;
  }
  button {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 0.3rem 0.5rem;
    cursor: pointer;
    font-size: 0.9rem;
    color: inherit;
    border-radius: 3px;
  }
  button:hover {
    background: #f0f0f0;
  }
  .entry-leaf.selected {
    background: #1565c0;
    color: white;
  }
  .entry-dir {
    font-weight: 600;
  }
  .dir-icon {
    display: inline-block;
    width: 1em;
    margin-right: 0.2em;
  }
  .subtree {
    padding-left: 1rem;
  }
</style>
