<script lang="ts">
  interface Props {
    message: string;
    kind?: 'info' | 'error';
  }

  let { message, kind = 'info' }: Props = $props();
</script>

{#if message}
  <div
    class="px-3 py-1.5 text-xs border-t flex items-center gap-2
      {kind === 'error'
        ? 'bg-error/10 text-error border-error/20'
        : 'bg-success/10 text-success border-success/20'}"
    role="status"
    aria-live="polite"
  >
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
  </div>
{/if}
