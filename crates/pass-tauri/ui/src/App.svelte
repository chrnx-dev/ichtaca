<script lang="ts">
  import { onMount } from 'svelte';
  import TreePanel from './components/TreePanel.svelte';
  import DetailPanel from './components/DetailPanel.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import { list, showMeta, searchFuzzy, buildTree } from './lib/api';
  import type { EntryMeta, EntryNode } from './lib/types';

  let tree = $state<EntryNode[]>([]);
  let selectedPath = $state<string | null>(null);
  let meta = $state<EntryMeta | null>(null);
  let statusMessage = $state('');
  let statusKind = $state<'info' | 'error'>('info');
  let searchQuery = $state('');
  let isLoading = $state(true);
  let allPaths = $state<string[]>([]);

  function showNotice(msg: string) {
    statusMessage = msg;
    statusKind = 'info';
    setTimeout(() => {
      statusMessage = '';
    }, 5000);
  }

  function showError(msg: string) {
    statusMessage = msg;
    statusKind = 'error';
  }

  async function loadEntry(path: string) {
    selectedPath = path;
    meta = null;
    try {
      meta = await showMeta(path);
    } catch (e) {
      showError(`Could not load entry: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  async function handleSearch() {
    const q = searchQuery.trim();
    if (!q) {
      tree = buildTree(allPaths);
      return;
    }
    try {
      const hits = await searchFuzzy(q);
      tree = buildTree(hits);
    } catch (e) {
      showError(`Search error: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  onMount(async () => {
    try {
      allPaths = await list();
      tree = buildTree(allPaths);
    } catch (e) {
      showError(
        `Could not connect to the password store. ` +
        `Make sure 'pass' and 'gpg' are installed and the store is initialised. ` +
        `Error: ${e instanceof Error ? e.message : String(e)}`
      );
    } finally {
      isLoading = false;
    }
  });
</script>

<div class="app-layout">
  <header class="app-header">
    <h1>pass-tauri</h1>
    <input
      class="search-input"
      type="search"
      placeholder="Search…"
      bind:value={searchQuery}
      oninput={handleSearch}
      aria-label="Search entries"
    />
  </header>

  <main class="app-body">
    <aside class="sidebar">
      {#if isLoading}
        <p class="loading">Loading…</p>
      {:else}
        <TreePanel {tree} {selectedPath} onselect={loadEntry} />
      {/if}
    </aside>

    <section class="detail-area">
      <DetailPanel {meta} onnotice={showNotice} onerror={showError} />
    </section>
  </main>

  <footer class="app-footer">
    <StatusBar message={statusMessage} kind={statusKind} />
  </footer>
</div>

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }
  :global(body) {
    margin: 0;
    font-family: system-ui, sans-serif;
    background: #fafafa;
    color: #222;
  }
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .app-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background: #1565c0;
    color: white;
    flex-shrink: 0;
  }
  .app-header h1 {
    margin: 0;
    font-size: 1.1rem;
  }
  .search-input {
    flex: 1;
    max-width: 22rem;
    padding: 0.3rem 0.6rem;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
  }
  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .sidebar {
    width: 260px;
    border-right: 1px solid #ddd;
    overflow-y: auto;
    background: #fff;
    flex-shrink: 0;
  }
  .detail-area {
    flex: 1;
    overflow-y: auto;
    background: #fff;
  }
  .app-footer {
    flex-shrink: 0;
  }
  .loading {
    padding: 1rem;
    color: #888;
  }
</style>
