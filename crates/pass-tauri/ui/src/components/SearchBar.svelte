<script lang="ts">
  import { searchFuzzy } from '../lib/api';

  interface Props {
    onselect: (path: string) => void;
    onclear: () => void;
  }

  let { onselect, onclear }: Props = $props();

  let query = $state('');
  let results = $state<string[]>([]);
  let isSearching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function clearDebounce() {
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
  }

  async function runSearch(q: string) {
    if (!q.trim()) {
      results = [];
      onclear();
      return;
    }
    isSearching = true;
    try {
      results = await searchFuzzy(q.trim());
    } catch {
      results = [];
    } finally {
      isSearching = false;
    }
  }

  function handleInput() {
    clearDebounce();
    if (!query.trim()) {
      results = [];
      onclear();
      return;
    }
    debounceTimer = setTimeout(() => {
      runSearch(query);
    }, 250);
  }

  function handleSelect(path: string) {
    onselect(path);
    query = '';
    results = [];
  }

  function handleClear() {
    clearDebounce();
    query = '';
    results = [];
    onclear();
  }
</script>

<div class="search-bar" data-testid="search-bar">
  <div class="search-input-row">
    <input
      class="search-input"
      type="search"
      placeholder="Search entries…"
      bind:value={query}
      oninput={handleInput}
      aria-label="Search entries"
      aria-controls="search-results"
      data-testid="search-input"
    />
    {#if query}
      <button class="clear-btn" type="button" onclick={handleClear} aria-label="Clear search" data-testid="clear-search">
        ✕
      </button>
    {/if}
  </div>

  {#if results.length > 0}
    <ul
      id="search-results"
      class="results-list"
      role="listbox"
      aria-label="Search results"
      data-testid="search-results"
    >
      {#each results as path}
        <li role="option" aria-selected="false">
          <button
            class="result-item"
            type="button"
            onclick={() => handleSelect(path)}
            data-testid="search-result"
          >
            {path}
          </button>
        </li>
      {/each}
    </ul>
  {:else if isSearching}
    <p class="search-status">Searching…</p>
  {:else if query.trim() && results.length === 0 && !isSearching}
    <p class="search-status">No results.</p>
  {/if}
</div>

<style>
  .search-bar {
    position: relative;
    width: 100%;
  }
  .search-input-row {
    display: flex;
    align-items: center;
    position: relative;
  }
  .search-input {
    flex: 1;
    padding: 0.3rem 2rem 0.3rem 0.6rem;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
    background: rgba(255, 255, 255, 0.9);
    color: #222;
  }
  .search-input:focus {
    outline: 2px solid rgba(255, 255, 255, 0.6);
  }
  .clear-btn {
    position: absolute;
    right: 0.4rem;
    background: none;
    border: none;
    cursor: pointer;
    color: #888;
    font-size: 0.75rem;
    padding: 0 0.2rem;
    line-height: 1;
  }
  .results-list {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 4px;
    list-style: none;
    margin: 0;
    padding: 0.25rem 0;
    z-index: 50;
    max-height: 260px;
    overflow-y: auto;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  }
  .results-list li {
    margin: 0;
  }
  .result-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 0.35rem 0.75rem;
    font-size: 0.9rem;
    cursor: pointer;
    color: #222;
  }
  .result-item:hover {
    background: #e3f2fd;
    color: #1565c0;
  }
  .search-status {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    font-size: 0.85rem;
    color: #888;
    margin: 0;
    z-index: 50;
  }
</style>
