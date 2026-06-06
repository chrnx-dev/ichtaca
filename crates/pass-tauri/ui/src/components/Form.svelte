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

  // Use untrack so $state initializer doesn't create a reactive dependency on
  // the `path` prop (the form is recreated per entry, so one-time read is correct).
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

  // ── Local password generator (create form) ────────────────────────────────────
  // Approach: generate password locally in TypeScript for the create flow.
  // The generated value is held in the `password` field and persisted via
  // `insert(path, input, false)` on Save. No backend round-trip is needed.

  function generatePasswordLocally(len = 20, symbols = true): string {
    const alpha = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';
    const digits = '0123456789';
    const sym = '!@#$%^&*()-_=+[]{}|;:,.<>?';
    const charset = alpha + digits + (symbols ? sym : '');
    // Rejection sampling: discard bytes >= floor(256 / charset.length) * charset.length
    // to eliminate modulo bias.
    const limit = Math.floor(256 / charset.length) * charset.length;
    const result: string[] = [];
    let buf = new Uint8Array(len * 2);
    let pos = buf.length; // force refill on first iteration
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

  // ── Template picker (create only) ────────────────────────────────────────────

  function applyTemplate(tpl: Template) {
    selectedTemplate = tpl;
    fields = TEMPLATE_FIELDS[tpl].map(([k, v]) => [k, v] as [string, string]);
  }

  // ── Field row management ─────────────────────────────────────────────────────

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

  // Run prefill once on mount for edit mode using $effect
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

<div class="form-overlay" role="dialog" aria-modal="true" aria-label={mode === 'create' ? 'Create entry' : 'Edit entry'}>
  <div class="form-card">
    <h2 class="form-title">{mode === 'create' ? 'New Entry' : `Edit: ${path}`}</h2>

    {#if isLoading}
      <p class="loading">Loading entry…</p>
    {:else}
      <!-- Path (create only) -->
      {#if mode === 'create'}
        <div class="field-row">
          <label class="field-label" for="entry-path">Path</label>
          <input
            id="entry-path"
            class="field-input"
            type="text"
            placeholder="e.g. web/github.com"
            bind:value={entryPath}
            data-testid="path-input"
          />
        </div>

        <!-- Template picker -->
        <div class="field-row">
          <label class="field-label" for="template-select">Template</label>
          <select
            id="template-select"
            class="field-input"
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
      <div class="field-row">
        <label class="field-label" for="password-input">Password</label>
        <div class="password-row">
          <input
            id="password-input"
            class="field-input password-input"
            type={showPassword ? 'text' : 'password'}
            placeholder="Password"
            bind:value={password}
            data-testid="password-input"
          />
          <button
            class="btn-sm"
            type="button"
            onclick={() => (showPassword = !showPassword)}
            data-testid="password-toggle"
            aria-label={showPassword ? 'Hide password' : 'Show password'}
          >
            {showPassword ? 'Hide' : 'Show'}
          </button>
          {#if mode === 'create'}
            <button
              class="btn-sm"
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
      <div class="section-label">Fields</div>
      {#each fields as [key, value], i}
        <div class="field-row kv-row" data-testid="field-row">
          <input
            class="field-input kv-key"
            type="text"
            placeholder="key"
            value={key}
            oninput={(e) => updateFieldKey(i, (e.target as HTMLInputElement).value)}
            data-testid="field-key-{i}"
          />
          <input
            class="field-input kv-value"
            type="text"
            placeholder="value"
            value={value}
            oninput={(e) => updateFieldValue(i, (e.target as HTMLInputElement).value)}
            data-testid="field-value-{i}"
          />
          <button
            class="btn-sm btn-remove"
            type="button"
            onclick={() => removeField(i)}
            aria-label="Remove field"
            data-testid="remove-field-{i}"
          >
            ✕
          </button>
        </div>
      {/each}
      <button class="btn-sm btn-add-field" type="button" onclick={addField} data-testid="add-field">
        + Add field
      </button>

      <!-- OTP URI -->
      <div class="field-row">
        <label class="field-label" for="otp-input">OTP URI</label>
        <div class="password-row">
          <input
            id="otp-input"
            class="field-input"
            type={showOtp ? 'text' : 'password'}
            placeholder="otpauth://totp/…  (optional)"
            bind:value={otp}
            data-testid="otp-input"
          />
          <button
            class="btn-sm"
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
      <div class="field-row">
        <label class="field-label" for="tag-input">Tags</label>
        <div class="tags-area">
          {#each tags as tag}
            <span class="tag" data-testid="tag-chip">
              @{tag}
              <button
                class="tag-remove"
                type="button"
                onclick={() => removeTag(tag)}
                aria-label="Remove tag {tag}"
              >✕</button>
            </span>
          {/each}
          <input
            id="tag-input"
            class="tag-input"
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
        <p class="error-msg" role="alert" data-testid="form-error">{errorMsg}</p>
      {/if}

      <!-- Actions -->
      <div class="form-actions">
        <button
          class="btn-primary"
          type="button"
          onclick={handleSave}
          disabled={isSaving}
          data-testid="save-button"
        >
          {isSaving ? 'Saving…' : 'Save'}
        </button>
        <button
          class="btn-secondary"
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

<style>
  .form-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .form-card {
    background: #fff;
    border-radius: 6px;
    padding: 1.5rem;
    width: 100%;
    max-width: 520px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.18);
  }
  .form-title {
    margin: 0 0 1rem;
    font-size: 1.1rem;
    font-weight: 700;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.6rem;
  }
  .field-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: #555;
    min-width: 6rem;
    flex-shrink: 0;
  }
  .field-input {
    flex: 1;
    padding: 0.3rem 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 0.9rem;
  }
  .field-input:focus {
    outline: 2px solid #1565c0;
    outline-offset: 1px;
  }
  .password-row {
    display: flex;
    flex: 1;
    gap: 0.4rem;
  }
  .password-input {
    flex: 1;
  }
  .section-label {
    font-size: 0.8rem;
    font-weight: 700;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0.6rem 0 0.3rem;
  }
  .kv-row {
    gap: 0.3rem;
  }
  .kv-key {
    flex: 0 0 8rem;
  }
  .kv-value {
    flex: 1;
  }
  .btn-sm {
    padding: 0.2rem 0.55rem;
    font-size: 0.8rem;
    border: 1px solid #bbb;
    border-radius: 3px;
    background: #fafafa;
    cursor: pointer;
    flex-shrink: 0;
  }
  .btn-sm:hover {
    background: #e0e0e0;
  }
  .btn-remove {
    color: #c62828;
    border-color: #ef9a9a;
  }
  .btn-add-field {
    margin-bottom: 0.6rem;
  }
  .tags-area {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    flex: 1;
    align-items: center;
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 0.25rem 0.4rem;
    min-height: 2rem;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    background: #e3f2fd;
    color: #1565c0;
    border-radius: 3px;
    padding: 0.1rem 0.4rem;
    font-size: 0.8rem;
  }
  .tag-remove {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    font-size: 0.7rem;
    line-height: 1;
    color: #1565c0;
  }
  .tag-input {
    border: none;
    outline: none;
    font-size: 0.85rem;
    flex: 1;
    min-width: 6rem;
  }
  .error-msg {
    color: #c62828;
    font-size: 0.85rem;
    margin: 0.4rem 0;
    padding: 0.35rem 0.5rem;
    background: #ffebee;
    border-radius: 4px;
    border: 1px solid #ef9a9a;
  }
  .form-actions {
    display: flex;
    gap: 0.6rem;
    margin-top: 1rem;
    justify-content: flex-end;
  }
  .btn-primary {
    padding: 0.4rem 1.1rem;
    background: #1565c0;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
  }
  .btn-primary:hover {
    background: #0d47a1;
  }
  .btn-primary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .btn-secondary {
    padding: 0.4rem 1.1rem;
    background: #fafafa;
    color: #333;
    border: 1px solid #bbb;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
  }
  .btn-secondary:hover {
    background: #e0e0e0;
  }
  .btn-secondary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .loading {
    color: #888;
    font-style: italic;
  }
</style>
