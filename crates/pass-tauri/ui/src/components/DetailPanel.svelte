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

<section class="detail-panel">
  {#if meta === null}
    <p class="placeholder">Select an entry to view details.</p>
  {:else}
    <h2 class="entry-title">{meta.path}</h2>

    <!-- Password row -->
    <div class="field-row password-row">
      <span class="field-key">password</span>
      <span class="field-value password-value" data-testid="password-display">
        {#if revealedPassword !== null}
          <span class="revealed" data-testid="password-revealed">{revealedPassword}</span>
        {:else}
          <span class="masked" data-testid="password-masked" aria-label="Password hidden">••••••••</span>
        {/if}
      </span>
      <div class="field-actions">
        <button
          class="btn-sm"
          onclick={handleReveal}
          disabled={isRevealing}
          data-testid="reveal-button"
        >
          {revealedPassword !== null ? 'Hide' : 'Reveal'}
        </button>
        <button
          class="btn-sm"
          onclick={handleCopy}
          data-testid="copy-button"
        >
          Copy
        </button>
      </div>
    </div>

    <!-- Fields -->
    {#each meta.fields as [key, value]}
      <div class="field-row">
        <span class="field-key">{key}</span>
        <span class="field-value">{value}</span>
      </div>
    {/each}

    <!-- Tags -->
    {#if meta.tags.length > 0}
      <div class="tags-row">
        {#each meta.tags as tag}
          <span class="tag">@{tag}</span>
        {/each}
      </div>
    {/if}

    <!-- OTP -->
    {#if meta.has_otp}
      <div class="otp-row" data-testid="otp-section">
        <span class="field-key">OTP</span>
        {#if otp}
          <span class="otp-code" data-testid="otp-code">{otp.code}</span>
          <span class="otp-timer" data-testid="otp-countdown">{otpCountdown}s</span>
        {:else}
          <span class="otp-loading">Loading…</span>
        {/if}
      </div>
    {/if}
  {/if}
</section>

<style>
  .detail-panel {
    padding: 1rem;
    overflow-y: auto;
    height: 100%;
  }
  .placeholder {
    color: #888;
    font-style: italic;
  }
  .entry-title {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 1rem;
    word-break: break-all;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
    font-size: 0.9rem;
  }
  .field-key {
    font-weight: 600;
    min-width: 7rem;
    color: #555;
    flex-shrink: 0;
  }
  .field-value {
    flex: 1;
    word-break: break-all;
  }
  .password-row {
    flex-wrap: wrap;
  }
  .password-value {
    font-family: monospace;
    letter-spacing: 0.05em;
  }
  .masked {
    letter-spacing: 0.2em;
    color: #888;
  }
  .revealed {
    color: #c62828;
    user-select: text;
  }
  .field-actions {
    display: flex;
    gap: 0.35rem;
    flex-shrink: 0;
  }
  .btn-sm {
    padding: 0.15rem 0.5rem;
    font-size: 0.8rem;
    border: 1px solid #bbb;
    border-radius: 3px;
    background: #fafafa;
    cursor: pointer;
  }
  .btn-sm:hover {
    background: #e0e0e0;
  }
  .btn-sm:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .tags-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-top: 0.5rem;
  }
  .tag {
    background: #e3f2fd;
    color: #1565c0;
    border-radius: 3px;
    padding: 0.1rem 0.4rem;
    font-size: 0.8rem;
  }
  .otp-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .otp-code {
    font-family: monospace;
    font-size: 1.3rem;
    font-weight: 700;
    letter-spacing: 0.15em;
  }
  .otp-timer {
    font-size: 0.85rem;
    color: #888;
  }
  .otp-loading {
    color: #888;
    font-style: italic;
  }
</style>
