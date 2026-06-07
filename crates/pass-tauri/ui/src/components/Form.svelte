<script lang="ts">
  import { untrack } from 'svelte';
  import { showMeta, revealPassword, revealOtpUri, insert, updateEntry } from '../lib/api';
  import type { EntryInput } from '../lib/types';

  interface Props {
    mode: 'create' | 'edit';
    /** Required when mode === 'edit' */
    path?: string;
    onsaved: () => void;
    oncancel: () => void;
  }

  let { mode, path = '', onsaved, oncancel }: Props = $props();

  // ── Template definitions ──────────────────────────────────────────────────────

  type Template = 'blank' | 'login' | 'oauth' | 'server' | 'note';

  const TEMPLATE_FIELDS: Record<Template, [string, string][]> = {
    blank:  [],
    login:  [['user', ''], ['url', '']],
    oauth:  [['client_id', ''], ['client_secret', ''], ['url', '']],
    server: [['host', ''], ['user', ''], ['port', '']],
    note:   [],
  };

  // ── Local state ───────────────────────────────────────────────────────────────

  let entryPath = $state(untrack(() => path));
  let password = $state('');
  let fields = $state<[string, string][]>([]);
  let otp = $state('');
  let tagInput = $state('');
  let tags = $state<string[]>([]);
  let selectedTemplate = $state<Template>('blank');

  let isSaving = $state(false);
  let isLoading = $state(false);
  let errorMsg = $state('');

  // ── Show/hide toggles for sensitive fields ────────────────────────────────────
  let showPassword = $state(false);
  let showOtp = $state(false);

  // ── Local password generator ──────────────────────────────────────────────────

  function generatePasswordLocally(len = 20, symbols = true): string {
    const alpha = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';
    const digits = '0123456789';
    const sym = '!@#$%^&*()-_=+[]{}|;:,.<>?';
    const charset = alpha + digits + (symbols ? sym : '');
    const limit = Math.floor(256 / charset.length) * charset.length;
    const result: string[] = [];
    let buf = new Uint8Array(len * 2);
    let pos = buf.length;
    while (result.length < len) {
      if (pos >= buf.length) {
        buf = new Uint8Array(len * 2);
        crypto.getRandomValues(buf);
        pos = 0;
      }
      const b = buf[pos++];
      if (b < limit) {
        result.push(charset[b % charset.length]);
      }
    }
    return result.join('');
  }

  function handleGenerate() {
    password = generatePasswordLocally(20, true);
  }

  // ── Template picker ───────────────────────────────────────────────────────────

  function applyTemplate(tpl: Template) {
    selectedTemplate = tpl;
    fields = TEMPLATE_FIELDS[tpl].map(([k, v]) => [k, v] as [string, string]);
  }

  // ── Field row management ──────────────────────────────────────────────────────

  function addField() {
    fields = [...fields, ['', '']];
  }

  function removeField(index: number) {
    fields = fields.filter((_, i) => i !== index);
  }

  function updateFieldKey(index: number, key: string) {
    fields = fields.map((f, i) => (i === index ? [key, f[1]] : f));
  }

  function updateFieldValue(index: number, value: string) {
    fields = fields.map((f, i) => (i === index ? [f[0], value] : f));
  }

  // ── Tags management ───────────────────────────────────────────────────────────

  function addTag() {
    const t = tagInput.trim().replace(/^@+/, '');
    if (t && !tags.includes(t)) {
      tags = [...tags, t];
    }
    tagInput = '';
  }

  function removeTag(tag: string) {
    tags = tags.filter((t) => t !== tag);
  }

  function handleTagKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag();
    }
  }

  // ── Prefill for edit mode ─────────────────────────────────────────────────────

  async function prefillEdit() {
    if (mode !== 'edit' || !path) return;
    isLoading = true;
    errorMsg = '';
    try {
      const [meta, pw, otpUri] = await Promise.all([
        showMeta(path),
        revealPassword(path),
        revealOtpUri(path),
      ]);
      fields = meta.fields.map(([k, v]) => [k, v] as [string, string]);
      tags = [...meta.tags];
      password = pw;
      otp = otpUri ?? '';
    } catch (e) {
      errorMsg = `Failed to load entry: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      isLoading = false;
    }
  }

  let prefillDone = false;
  $effect(() => {
    if (!prefillDone && mode === 'edit') {
      prefillDone = true;
      prefillEdit();
    }
  });

  // ── Save ──────────────────────────────────────────────────────────────────────

  async function handleSave() {
    errorMsg = '';
    const p = entryPath.trim();
    if (!p) {
      errorMsg = 'Path is required.';
      return;
    }
    if (!password) {
      errorMsg = 'Password is required.';
      return;
    }

    const input: EntryInput = {
      password,
      fields: fields
        .map(([k, v]) => [k.trim(), v] as [string, string])
        .filter(([k]) => k !== ''),
      otp: otp.trim() || null,
      tags: [...tags],
    };

    isSaving = true;
    try {
      if (mode === 'create') {
        await insert(p, input, false);
      } else {
        await updateEntry(p, input);
      }
      onsaved();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      errorMsg = msg;
    } finally {
      isSaving = false;
    }
  }
</script>

<!-- Modal overlay -->
<div
  class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[100]"
  role="dialog"
  aria-modal="true"
  aria-label={mode === 'create' ? 'Create entry' : 'Edit entry'}
>
  <div class="card bg-base-100 w-full max-w-lg max-h-[90vh] overflow-y-auto shadow-2xl border border-neutral/20">
    <div class="card-body p-5">

      <!-- Title -->
      <h2 class="card-title text-primary text-base mb-1">
        {mode === 'create' ? 'New Entry' : `Edit: ${path}`}
      </h2>
      <div class="h-px bg-neutral/20 mb-3"></div>

      {#if isLoading}
        <div class="flex items-center gap-2 py-4 text-neutral text-sm">
          <span class="loading loading-spinner loading-sm text-primary"></span>
          Loading entry…
        </div>
      {:else}
        <!-- Path (create only) -->
        {#if mode === 'create'}
          <div class="form-control mb-2">
            <label class="label py-0.5" for="entry-path">
              <span class="label-text text-xs font-semibold uppercase tracking-wider text-neutral">Path</span>
            </label>
            <input
              id="entry-path"
              class="input input-sm input-bordered bg-base-200 text-base-content w-full"
              type="text"
              placeholder="e.g. web/github.com"
              bind:value={entryPath}
              data-testid="path-input"
            />
          </div>

          <!-- Template picker -->
          <div class="form-control mb-2">
            <label class="label py-0.5" for="template-select">
              <span class="label-text text-xs font-semibold uppercase tracking-wider text-neutral">Template</span>
            </label>
            <select
              id="template-select"
              class="select select-sm select-bordered bg-base-200 text-base-content w-full"
              bind:value={selectedTemplate}
              onchange={() => applyTemplate(selectedTemplate)}
              data-testid="template-select"
            >
              <option value="blank">Blank</option>
              <option value="login">Login</option>
              <option value="oauth">OAuth</option>
              <option value="server">Server</option>
              <option value="note">Note</option>
            </select>
          </div>
        {/if}

        <!-- Password -->
        <div class="form-control mb-2">
          <label class="label py-0.5" for="password-input">
            <span class="label-text text-xs font-semibold uppercase tracking-wider text-neutral">Password</span>
          </label>
          <div class="flex gap-1.5">
            <input
              id="password-input"
              class="input input-sm input-bordered bg-base-200 text-base-content font-mono flex-1 min-w-0"
              type={showPassword ? 'text' : 'password'}
              placeholder="Password"
              bind:value={password}
              data-testid="password-input"
            />
            <button
              class="btn btn-xs btn-ghost border border-neutral/30 flex-shrink-0"
              type="button"
              onclick={() => (showPassword = !showPassword)}
              data-testid="password-toggle"
              aria-label={showPassword ? 'Hide password' : 'Show password'}
            >
              {showPassword ? 'Hide' : 'Show'}
            </button>
            {#if mode === 'create'}
              <button
                class="btn btn-xs btn-ghost border border-primary/40 text-primary flex-shrink-0"
                type="button"
                onclick={handleGenerate}
                data-testid="generate-button"
              >
                Generate
              </button>
            {/if}
          </div>
        </div>

        <!-- Dynamic key/value fields -->
        <div class="mt-1 mb-1">
          <span class="text-xs font-semibold uppercase tracking-wider text-neutral">Fields</span>
        </div>
        {#each fields as [key, value], i}
          <div class="flex gap-1.5 mb-1.5 items-center" data-testid="field-row">
            <input
              class="input input-xs input-bordered bg-base-200 text-base-content w-28 flex-shrink-0"
              type="text"
              placeholder="key"
              value={key}
              oninput={(e) => updateFieldKey(i, (e.target as HTMLInputElement).value)}
              data-testid="field-key-{i}"
            />
            <input
              class="input input-xs input-bordered bg-base-200 text-base-content flex-1 min-w-0"
              type="text"
              placeholder="value"
              value={value}
              oninput={(e) => updateFieldValue(i, (e.target as HTMLInputElement).value)}
              data-testid="field-value-{i}"
            />
            <button
              class="btn btn-xs btn-ghost border border-error/30 text-error flex-shrink-0"
              type="button"
              onclick={() => removeField(i)}
              aria-label="Remove field"
              data-testid="remove-field-{i}"
            >✕</button>
          </div>
        {/each}
        <button
          class="btn btn-xs btn-ghost border border-neutral/30 text-neutral mb-3"
          type="button"
          onclick={addField}
          data-testid="add-field"
        >+ Add field</button>

        <!-- OTP URI -->
        <div class="form-control mb-2">
          <label class="label py-0.5" for="otp-input">
            <span class="label-text text-xs font-semibold uppercase tracking-wider text-neutral">OTP URI</span>
          </label>
          <div class="flex gap-1.5">
            <input
              id="otp-input"
              class="input input-sm input-bordered bg-base-200 text-base-content font-mono flex-1 min-w-0"
              type={showOtp ? 'text' : 'password'}
              placeholder="otpauth://totp/…  (optional)"
              bind:value={otp}
              data-testid="otp-input"
            />
            <button
              class="btn btn-xs btn-ghost border border-neutral/30 flex-shrink-0"
              type="button"
              onclick={() => (showOtp = !showOtp)}
              data-testid="otp-toggle"
              aria-label={showOtp ? 'Hide OTP URI' : 'Show OTP URI'}
            >
              {showOtp ? 'Hide' : 'Show'}
            </button>
          </div>
        </div>

        <!-- Tags -->
        <div class="form-control mb-3">
          <label class="label py-0.5" for="tag-input">
            <span class="label-text text-xs font-semibold uppercase tracking-wider text-neutral">Tags</span>
          </label>
          <div class="flex flex-wrap gap-1.5 items-center border border-neutral/30 rounded-lg bg-base-200 px-2 py-1.5 min-h-[2rem]">
            {#each tags as tag}
              <span class="badge badge-sm text-[#3FA66A] border-[#3FA66A]/40 bg-[#3FA66A]/10 gap-1" data-testid="tag-chip">
                @{tag}
                <button
                  class="text-[#3FA66A]/60 hover:text-error transition-colors"
                  type="button"
                  onclick={() => removeTag(tag)}
                  aria-label="Remove tag {tag}"
                >✕</button>
              </span>
            {/each}
            <input
              id="tag-input"
              class="bg-transparent border-none outline-none text-sm text-base-content placeholder-neutral flex-1 min-w-[6rem]"
              type="text"
              placeholder="Add tag…"
              bind:value={tagInput}
              onkeydown={handleTagKeydown}
              onblur={addTag}
              data-testid="tag-input"
            />
          </div>
        </div>

        <!-- Error message -->
        {#if errorMsg}
          <div class="alert alert-error alert-sm py-2 px-3 mb-3" role="alert" data-testid="form-error">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 flex-shrink-0" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
            </svg>
            <span class="text-sm">{errorMsg}</span>
          </div>
        {/if}

        <!-- Actions -->
        <div class="flex gap-2 justify-end mt-1">
          <button
            class="btn btn-primary btn-sm"
            type="button"
            onclick={handleSave}
            disabled={isSaving}
            data-testid="save-button"
          >
            {#if isSaving}
              <span class="loading loading-spinner loading-xs"></span>
            {/if}
            {isSaving ? 'Saving…' : 'Save'}
          </button>
          <button
            class="btn btn-ghost btn-sm border border-neutral/30"
            type="button"
            onclick={oncancel}
            disabled={isSaving}
            data-testid="cancel-button"
          >
            Cancel
          </button>
        </div>
      {/if}

    </div>
  </div>
</div>
