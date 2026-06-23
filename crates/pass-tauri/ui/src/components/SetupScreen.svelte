<script lang="ts">
  import type { DoctorReport } from '../lib/types';

  let { report }: { report: DoctorReport } = $props();

  const checks: { label: string; ok: boolean }[] = $derived([
    { label: '`pass` binary', ok: report.pass },
    { label: '`gpg` binary', ok: report.gpg },
    { label: `Password store (\`${report.store_dir}\`)`, ok: report.store_dir_exists },
  ]);

  const guidance = $derived(
    report.guidance || report.init_error || ''
  );
</script>

<div
  class="flex flex-col items-center justify-center h-screen bg-[#15131A] text-base-content px-6"
  data-testid="setup-screen"
>
  <div class="w-full max-w-lg">
    <!-- Heading -->
    <h1 class="text-primary font-bold tracking-widest text-sm uppercase mb-1">
      ICHTACA · lo oculto
    </h1>
    <h2 class="text-base-content text-lg font-semibold mb-1">
      Setup required
    </h2>
    <p class="text-neutral text-xs mb-6">
      Ichtaca could not open your password store. Please resolve the issues below.
    </p>

    <!-- Checklist -->
    <div class="mb-6 flex flex-col gap-2" data-testid="setup-checklist">
      {#each checks as check}
        <div class="flex items-center gap-3 text-sm">
          {#if check.ok}
            <span class="text-success font-bold w-4 text-center" aria-label="ok">✓</span>
          {:else}
            <span class="text-error font-bold w-4 text-center" aria-label="fail">✗</span>
          {/if}
          <span class="text-base-content">{check.label}</span>
        </div>
      {/each}
    </div>

    <!-- Guidance / error text -->
    {#if guidance}
      <div class="bg-base-100 border border-neutral/30 rounded p-4">
        <p class="text-neutral text-xs uppercase tracking-wider mb-2">
          {report.guidance ? 'Next steps' : 'Error details'}
        </p>
        <pre
          class="text-xs text-base-content whitespace-pre-wrap break-words font-mono leading-relaxed"
          data-testid="setup-guidance"
        >{guidance}</pre>
      </div>
    {/if}
  </div>
</div>
