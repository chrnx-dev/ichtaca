<script lang="ts">
  interface Props {
    message: string;
    kind?: 'info' | 'error';
  }

  let { message, kind = 'info' }: Props = $props();
</script>

{#if message}
  <div class="status-bar" class:error={kind === 'error'} role="status" aria-live="polite">
    {message}
  </div>
{/if}

<style>
  .status-bar {
    padding: 0.25rem 0.75rem;
    font-size: 0.85rem;
    background: #e8f5e9;
    color: #2e7d32;
    border-top: 1px solid #c8e6c9;
  }
  .status-bar.error {
    background: #ffebee;
    color: #c62828;
    border-top-color: #ef9a9a;
  }
</style>
