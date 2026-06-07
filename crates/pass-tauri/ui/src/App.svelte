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

<div class="flex flex-col h-screen bg-[#15131A] text-base-content">
  <!-- ── Navbar ────────────────────────────────────────────────────────────── -->
  <div class="navbar bg-base-100 border-b border-neutral/30 flex-shrink-0 min-h-12 px-3 gap-3">
    <!-- Brand -->
    <div class="flex-shrink-0 flex items-baseline gap-1.5">
      <span class="text-primary font-bold tracking-widest text-sm uppercase">ICHTACA</span>
      <span class="text-neutral text-xs">· lo oculto</span>
    </div>

    <!-- Search -->
    <div class="flex-1 max-w-xs relative">
      <SearchBar onselect={handleSearchSelect} onclear={handleSearchClear} />
    </div>

    <!-- Actions -->
    <div class="flex-shrink-0 flex items-center gap-1.5">
      <button
        class="btn btn-xs btn-primary"
        onclick={handleNew}
        data-testid="new-button"
      >
        <!-- Plus icon -->
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd"/>
        </svg>
        New
      </button>
      <button
        class="btn btn-xs btn-ghost border border-neutral/40 text-base-content"
        onclick={handleEdit}
        disabled={selectedPath === null}
        data-testid="edit-button"
      >
        <!-- Pencil icon -->
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
          <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"/>
        </svg>
        Edit
      </button>
      <button
        class="btn btn-xs btn-error btn-outline"
        onclick={handleDeleteRequest}
        disabled={selectedPath === null}
        data-testid="delete-button"
      >
        <!-- Trash icon -->
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
        Delete
      </button>
    </div>
  </div>

  <!-- ── Main two-pane layout ──────────────────────────────────────────────── -->
  <main class="flex flex-1 overflow-hidden">
    <!-- Sidebar / Tree -->
    <aside class="w-64 flex-shrink-0 bg-base-100 border-r border-neutral/20 overflow-y-auto">
      {#if isLoading}
        <div class="flex items-center justify-center h-20">
          <span class="loading loading-spinner loading-sm text-primary"></span>
          <span class="ml-2 text-neutral text-sm">Loading…</span>
        </div>
      {:else}
        <TreePanel {tree} {selectedPath} onselect={loadEntry} />
      {/if}
    </aside>

    <!-- Detail pane -->
    <section class="flex-1 overflow-y-auto bg-[#15131A]">
      <DetailPanel {meta} onnotice={showNotice} onerror={showError} />
    </section>
  </main>

  <!-- ── Status footer ─────────────────────────────────────────────────────── -->
  <footer class="flex-shrink-0">
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
