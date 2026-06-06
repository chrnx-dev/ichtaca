<script lang="ts">
  import { onMount } from 'svelte';
  import TreePanel from './components/TreePanel.svelte';
  import DetailPanel from './components/DetailPanel.svelte';
  import StatusBar from './components/StatusBar.svelte';
  import Form from './components/Form.svelte';
  import ConfirmModal from './components/ConfirmModal.svelte';
  import SearchBar from './components/SearchBar.svelte';
  import { list, showMeta, remove, buildTree } from './lib/api';
  import type { EntryMeta, EntryNode } from './lib/types';

  let tree = $state<EntryNode[]>([]);
  let selectedPath = $state<string | null>(null);
  let meta = $state<EntryMeta | null>(null);
  let statusMessage = $state('');
  let statusKind = $state<'info' | 'error'>('info');
  let isLoading = $state(true);
  let allPaths = $state<string[]>([]);

  // Modal / form state
  let showCreateForm = $state(false);
  let showEditForm = $state(false);
  let showDeleteModal = $state(false);

  // ── Notifications ─────────────────────────────────────────────────────────────

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

  // ── Tree / entry loading ──────────────────────────────────────────────────────

  async function refreshTree() {
    try {
      allPaths = await list();
      tree = buildTree(allPaths);
    } catch (e) {
      showError(
        `Could not connect to the password store. ` +
        `Make sure 'pass' and 'gpg' are installed and the store is initialised. ` +
        `Error: ${e instanceof Error ? e.message : String(e)}`
      );
    }
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

  // ── Search ────────────────────────────────────────────────────────────────────

  function handleSearchSelect(path: string) {
    loadEntry(path);
  }

  function handleSearchClear() {
    tree = buildTree(allPaths);
  }

  // ── CRUD actions ──────────────────────────────────────────────────────────────

  function handleNew() {
    showCreateForm = true;
  }

  function handleEdit() {
    if (!selectedPath) return;
    showEditForm = true;
  }

  function handleDeleteRequest() {
    if (!selectedPath) return;
    showDeleteModal = true;
  }

  async function handleDeleteConfirm() {
    if (!selectedPath) return;
    const deletedPath = selectedPath;
    showDeleteModal = false;
    try {
      await remove(deletedPath);
      showNotice(`Deleted: ${deletedPath}`);
      selectedPath = null;
      meta = null;
      await refreshTree();
    } catch (e) {
      showError(`Delete failed: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  async function handleSaved() {
    showCreateForm = false;
    showEditForm = false;
    showNotice('Entry saved.');
    await refreshTree();
    // Reload meta if an edit updated the currently selected entry
    if (selectedPath) {
      await loadEntry(selectedPath);
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
    <div class="search-wrapper">
      <SearchBar onselect={handleSearchSelect} onclear={handleSearchClear} />
    </div>
    <div class="header-actions">
      <button class="btn-header" onclick={handleNew} data-testid="new-button">
        + New
      </button>
      <button
        class="btn-header"
        onclick={handleEdit}
        disabled={selectedPath === null}
        data-testid="edit-button"
      >
        Edit
      </button>
      <button
        class="btn-header btn-danger"
        onclick={handleDeleteRequest}
        disabled={selectedPath === null}
        data-testid="delete-button"
      >
        Delete
      </button>
    </div>
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

<!-- Modals (rendered outside the normal flow) -->

{#if showCreateForm}
  <Form
    mode="create"
    onsaved={handleSaved}
    oncancel={() => { showCreateForm = false; }}
  />
{/if}

{#if showEditForm && selectedPath}
  <Form
    mode="edit"
    path={selectedPath}
    onsaved={handleSaved}
    oncancel={() => { showEditForm = false; }}
  />
{/if}

{#if showDeleteModal && selectedPath}
  <ConfirmModal
    title="Delete entry"
    message={`Are you sure you want to permanently delete "${selectedPath}"? This cannot be undone.`}
    onconfirm={handleDeleteConfirm}
    oncancel={() => { showDeleteModal = false; }}
  />
{/if}

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
    white-space: nowrap;
  }
  .search-wrapper {
    flex: 1;
    max-width: 22rem;
    position: relative;
  }
  .header-actions {
    display: flex;
    gap: 0.4rem;
    flex-shrink: 0;
  }
  .btn-header {
    padding: 0.25rem 0.7rem;
    background: rgba(255, 255, 255, 0.15);
    color: white;
    border: 1px solid rgba(255, 255, 255, 0.35);
    border-radius: 4px;
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 0.15s;
  }
  .btn-header:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.28);
  }
  .btn-header:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .btn-danger {
    background: rgba(198, 40, 40, 0.5);
    border-color: rgba(239, 154, 154, 0.5);
  }
  .btn-danger:hover:not(:disabled) {
    background: rgba(183, 28, 28, 0.7);
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
