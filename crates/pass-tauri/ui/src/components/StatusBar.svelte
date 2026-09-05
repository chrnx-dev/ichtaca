<script lang="ts">
  import type { GitStatus } from '../lib/types';

  interface Props {
    message: string;
    kind?: 'info' | 'error';
    /** Local git state; null when the store is not a git repo — no chip then. */
    git?: GitStatus | null;
    /** True while a pull/push is in flight. */
    syncing?: boolean;
    onsync?: () => void;
  }

  let { message, kind = 'info', git = null, syncing = false, onsync }: Props = $props();

  let synced = $derived(
    git !== null && git.upstream && git.ahead === 0 && git.behind === 0 && git.dirty === 0
  );
</script>

{#if message || git}
  <div
    class="px-3 py-1.5 text-xs border-t flex items-center gap-2
      {message && kind === 'error'
        ? 'bg-error/10 text-error border-error/20'
        : message
          ? 'bg-success/10 text-success border-success/20'
          : 'bg-base-100 text-neutral border-neutral/20'}"
    role="status"
    aria-live="polite"
  >
    {#if message}
      {#if kind === 'error'}
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 flex-shrink-0" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 flex-shrink-0" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
        </svg>
      {/if}
      {message}
    {/if}

    {#if git}
      <!-- Git chip, right-aligned. Click syncs (pull then push). -->
      <button
        class="ml-auto flex items-center gap-1.5 px-1.5 py-0.5 rounded hover:bg-neutral/20
          disabled:opacity-60 disabled:hover:bg-transparent"
        onclick={onsync}
        disabled={syncing || !git.upstream}
        title={git.upstream
          ? 'Pull then push the store repo'
          : 'No upstream branch — run: pass git remote add origin <url>'}
        data-testid="git-chip"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 flex-shrink-0" viewBox="0 0 16 16" fill="currentColor">
          <path d="M11.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 113 2.122V6A2.5 2.5 0 0110 8.5H6a1 1 0 00-1 1v1.128a2.251 2.251 0 11-1.5 0V5.372a2.25 2.25 0 111.5 0v1.836A2.492 2.492 0 016 7h4a1 1 0 001-1v-.628A2.25 2.25 0 019.5 3.25zM4.25 12a.75.75 0 100 1.5.75.75 0 000-1.5zM3.5 3.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0z"/>
        </svg>
        <span class="font-mono">{git.branch}</span>
        {#if git.ahead > 0}<span class="text-primary font-bold">↑{git.ahead}</span>{/if}
        {#if git.behind > 0}<span class="text-info">↓{git.behind}</span>{/if}
        {#if git.dirty > 0}<span class="text-error">●{git.dirty}</span>{/if}
        {#if !git.upstream}
          <span class="opacity-70">no remote</span>
        {:else if syncing}
          <span class="loading loading-spinner loading-xs"></span>
        {:else if synced}
          <span class="text-success">✓</span>
        {/if}
      </button>
    {/if}
  </div>
{/if}
