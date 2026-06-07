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

<div class="search-bar relative w-full" data-testid="search-bar">
  <!-- Input row -->
  <div class="relative flex items-center">
    <!-- Search icon -->
    <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 text-neutral absolute left-2.5 pointer-events-none" viewBox="0 0 20 20" fill="currentColor">
      <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd"/>
    </svg>
    <input
      class="input input-xs w-full bg-base-200/80 border-neutral/30 text-base-content placeholder-neutral/60 pl-7 pr-7 focus:border-primary/50 focus:outline-none"
      type="search"
      placeholder="Search entries…"
      bind:value={query}
      oninput={handleInput}
      aria-label="Search entries"
      aria-controls="search-results"
      data-testid="search-input"
    />
    {#if query}
      <button
        class="absolute right-2 text-neutral hover:text-base-content transition-colors p-0 bg-transparent border-none cursor-pointer text-xs leading-none"
        type="button"
        onclick={handleClear}
        aria-label="Clear search"
        data-testid="clear-search"
      >✕</button>
    {/if}
  </div>

  <!-- Results dropdown -->
  {#if results.length > 0}
    <ul
      id="search-results"
      class="absolute top-[calc(100%+4px)] left-0 right-0 bg-base-100 border border-neutral/30 rounded-lg shadow-xl z-50 max-h-64 overflow-y-auto py-1"
      role="listbox"
      aria-label="Search results"
      data-testid="search-results"
    >
      {#each results as path}
        <li role="option" aria-selected="false">
          <button
            class="flex items-center gap-2 w-full text-left px-3 py-1.5 text-sm text-base-content hover:bg-base-200 hover:text-primary transition-colors"
            type="button"
            onclick={() => handleSelect(path)}
            data-testid="search-result"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-neutral flex-shrink-0" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 8a6 6 0 01-7.743 5.743L10 14l-1 1-1 1H6v2H2v-4l4.257-4.257A6 6 0 1118 8zm-6-4a1 1 0 100 2 2 2 0 012 2 1 1 0 102 0 4 4 0 00-4-4z" clip-rule="evenodd"/>
            </svg>
            {path}
          </button>
        </li>
      {/each}
    </ul>
  {:else if isSearching}
    <div class="absolute top-[calc(100%+4px)] left-0 right-0 bg-base-100 border border-neutral/30 rounded-lg shadow-xl z-50 px-3 py-2">
      <span class="loading loading-dots loading-xs text-primary mr-2"></span>
      <span class="text-neutral text-xs">Searching…</span>
    </div>
  {:else if query.trim() && results.length === 0 && !isSearching}
    <div class="absolute top-[calc(100%+4px)] left-0 right-0 bg-base-100 border border-neutral/30 rounded-lg shadow-xl z-50 px-3 py-2">
      <p class="text-neutral text-xs italic">No results.</p>
    </div>
  {/if}
</div>
