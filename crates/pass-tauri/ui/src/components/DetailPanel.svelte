<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { EntryMeta, OtpCode } from '../lib/types';
  import { revealPassword, copyPassword, otpCode } from '../lib/api';

  interface Props {
    meta: EntryMeta | null;
    onnotice: (msg: string) => void;
    onerror: (msg: string) => void;
  }

  let { meta, onnotice, onerror }: Props = $props();

  // Password reveal state (held only in component state, never persisted)
  let revealedPassword = $state<string | null>(null);
  let isRevealing = $state(false);

  // OTP state
  let otp = $state<OtpCode | null>(null);
  let otpCountdown = $state(0);
  let otpTimer: ReturnType<typeof setInterval> | null = null;

  // Reset when the selected entry changes
  $effect(() => {
    // Track meta changes
    const currentPath = meta?.path;
    revealedPassword = null;
    isRevealing = false;
    otp = null;
    otpCountdown = 0;
    clearOtpTimer();

    if (meta?.has_otp && currentPath) {
      loadOtp(currentPath);
    }
  });

  function clearOtpTimer() {
    if (otpTimer !== null) {
      clearInterval(otpTimer);
      otpTimer = null;
    }
  }

  async function loadOtp(path: string) {
    try {
      const result = await otpCode(path);
      otp = result;
      otpCountdown = result.seconds;
      startOtpCountdown(path);
    } catch (e) {
      onerror(`OTP error: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  function startOtpCountdown(path: string) {
    clearOtpTimer();
    otpTimer = setInterval(() => {
      if (otpCountdown <= 1) {
        // Time to refresh
        loadOtp(path);
      } else {
        otpCountdown -= 1;
      }
    }, 1000);
  }

  async function handleReveal() {
    if (!meta) return;
    if (revealedPassword !== null) {
      // Already revealed — toggle hide
      revealedPassword = null;
      return;
    }
    isRevealing = true;
    try {
      revealedPassword = await revealPassword(meta.path);
    } catch (e) {
      onerror(`Reveal error: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      isRevealing = false;
    }
  }

  async function handleCopy() {
    if (!meta) return;
    try {
      await copyPassword(meta.path);
      onnotice('Password copied (clears in 45s)');
    } catch (e) {
      onerror(`Copy error: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  onDestroy(() => {
    clearOtpTimer();
  });
</script>

<section class="detail-panel p-5 h-full overflow-y-auto">
  {#if meta === null}
    <div class="flex items-center justify-center h-full">
      <p class="text-neutral italic text-sm">Select an entry to view details.</p>
    </div>
  {:else}
    <!-- Entry title -->
    <div class="mb-5">
      <h2 class="text-primary font-bold text-base tracking-wide break-all">{meta.path}</h2>
      <div class="h-px bg-neutral/20 mt-2"></div>
    </div>

    <!-- Card wrapping all fields -->
    <div class="card bg-base-100 shadow-md">
      <div class="card-body p-4 gap-3">

        <!-- Password row -->
        <div class="flex items-center gap-3 flex-wrap">
          <span class="text-neutral text-xs font-semibold uppercase tracking-wider min-w-[5.5rem] flex-shrink-0">password</span>
          <span class="flex-1 font-mono text-sm" data-testid="password-display">
            {#if revealedPassword !== null}
              <span class="text-[#F2C66D] select-text break-all" data-testid="password-revealed">{revealedPassword}</span>
            {:else}
              <span class="text-neutral tracking-[0.2em]" data-testid="password-masked" aria-label="Password hidden">••••••••</span>
            {/if}
          </span>
          <div class="flex gap-1.5 flex-shrink-0">
            <button
              class="btn btn-xs btn-ghost border border-neutral/30 text-base-content/80"
              onclick={handleReveal}
              disabled={isRevealing}
              data-testid="reveal-button"
            >
              <!-- Eye icon -->
              {#if revealedPassword !== null}
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-1.473-1.473A10.014 10.014 0 0019.542 10C18.268 5.943 14.478 3 10 3a9.958 9.958 0 00-4.512 1.074l-1.78-1.781zm4.261 4.26l1.514 1.515a2.003 2.003 0 012.45 2.45l1.514 1.514a4 4 0 00-5.478-5.478z" clip-rule="evenodd"/>
                  <path d="M12.454 16.697L9.75 13.992a4 4 0 01-3.742-3.741L2.335 6.578A9.98 9.98 0 00.458 10c1.274 4.057 5.065 7 9.542 7 .847 0 1.669-.105 2.454-.303z"/>
                </svg>
                Hide
              {:else}
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M10 12a2 2 0 100-4 2 2 0 000 4z"/>
                  <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd"/>
                </svg>
                Reveal
              {/if}
            </button>
            <button
              class="btn btn-xs btn-ghost border border-neutral/30 text-base-content/80"
              onclick={handleCopy}
              data-testid="copy-button"
            >
              <!-- Copy icon -->
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                <path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z"/>
                <path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z"/>
              </svg>
              Copy
            </button>
          </div>
        </div>

        <!-- Divider -->
        {#if meta.fields.length > 0 || meta.tags.length > 0 || meta.has_otp}
          <div class="h-px bg-neutral/15"></div>
        {/if}

        <!-- Fields -->
        {#each meta.fields as [key, value]}
          <div class="flex items-start gap-3">
            <span class="text-neutral text-xs font-semibold uppercase tracking-wider min-w-[5.5rem] flex-shrink-0 pt-0.5">{key}</span>
            <span class="text-base-content text-sm break-all flex-1">{value}</span>
          </div>
        {/each}

        <!-- Tags -->
        {#if meta.tags.length > 0}
          <div class="flex items-center gap-2 flex-wrap mt-1">
            <span class="text-neutral text-xs font-semibold uppercase tracking-wider min-w-[5.5rem] flex-shrink-0">tags</span>
            <div class="flex gap-1.5 flex-wrap">
              {#each meta.tags as tag}
                <span class="badge badge-sm text-[#3FA66A] border-[#3FA66A]/40 bg-[#3FA66A]/10">@{tag}</span>
              {/each}
            </div>
          </div>
        {/if}

        <!-- OTP -->
        {#if meta.has_otp}
          <div class="flex items-center gap-3 mt-1" data-testid="otp-section">
            <span class="text-neutral text-xs font-semibold uppercase tracking-wider min-w-[5.5rem] flex-shrink-0">OTP</span>
            {#if otp}
              <span class="font-mono text-xl font-bold text-[#46D0C0] tracking-[0.15em]" data-testid="otp-code">{otp.code}</span>
              <span class="text-[#2FB6A8] text-xs ml-1" data-testid="otp-countdown">{otpCountdown}s</span>
            {:else}
              <span class="text-neutral italic text-sm">Loading…</span>
            {/if}
          </div>
        {/if}

      </div>
    </div>
  {/if}
</section>
