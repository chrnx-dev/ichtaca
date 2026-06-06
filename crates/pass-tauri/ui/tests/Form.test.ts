import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import Form from '../src/components/Form.svelte';

// Mock the api module
vi.mock('../src/lib/api', () => ({
  insert: vi.fn(),
  updateEntry: vi.fn(),
  showMeta: vi.fn(),
  revealPassword: vi.fn(),
  revealOtpUri: vi.fn(),
  list: vi.fn(),
  buildTree: vi.fn(),
  searchFuzzy: vi.fn(),
  remove: vi.fn(),
  mv: vi.fn(),
  cp: vi.fn(),
  generate: vi.fn(),
  copyPassword: vi.fn(),
  otpCode: vi.fn(),
}));

import { insert, updateEntry, showMeta, revealPassword, revealOtpUri } from '../src/lib/api';

const mockInsert = vi.mocked(insert);
const mockUpdateEntry = vi.mocked(updateEntry);
const mockShowMeta = vi.mocked(showMeta);
const mockRevealPassword = vi.mocked(revealPassword);
const mockRevealOtpUri = vi.mocked(revealOtpUri);

beforeEach(() => {
  vi.clearAllMocks();
});

// ── Create mode — template picker ─────────────────────────────────────────────

describe('Form (create) — template picker', () => {
  it('starts with no fields when template is Blank', () => {
    const { queryAllByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });
    expect(queryAllByTestId('field-row')).toHaveLength(0);
  });

  it('selecting Login template fills user and url rows', async () => {
    const { getByTestId, getAllByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    const select = getByTestId('template-select');
    await fireEvent.change(select, { target: { value: 'login' } });

    const rows = getAllByTestId('field-row');
    expect(rows).toHaveLength(2);
    const key0 = getByTestId('field-key-0') as HTMLInputElement;
    const key1 = getByTestId('field-key-1') as HTMLInputElement;
    expect(key0.value).toBe('user');
    expect(key1.value).toBe('url');
  });

  it('selecting OAuth template fills client_id, client_secret, url rows', async () => {
    const { getByTestId, getAllByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    const select = getByTestId('template-select');
    await fireEvent.change(select, { target: { value: 'oauth' } });

    const rows = getAllByTestId('field-row');
    expect(rows).toHaveLength(3);

    const key0 = getByTestId('field-key-0') as HTMLInputElement;
    const key1 = getByTestId('field-key-1') as HTMLInputElement;
    const key2 = getByTestId('field-key-2') as HTMLInputElement;
    expect(key0.value).toBe('client_id');
    expect(key1.value).toBe('client_secret');
    expect(key2.value).toBe('url');
  });

  it('selecting Server template fills host, user, port rows', async () => {
    const { getByTestId, getAllByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    const select = getByTestId('template-select');
    await fireEvent.change(select, { target: { value: 'server' } });

    const rows = getAllByTestId('field-row');
    expect(rows).toHaveLength(3);

    const key0 = getByTestId('field-key-0') as HTMLInputElement;
    const key1 = getByTestId('field-key-1') as HTMLInputElement;
    const key2 = getByTestId('field-key-2') as HTMLInputElement;
    expect(key0.value).toBe('host');
    expect(key1.value).toBe('user');
    expect(key2.value).toBe('port');
  });

  it('selecting Note template produces no field rows', async () => {
    const { getByTestId, queryAllByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    const select = getByTestId('template-select');
    await fireEvent.change(select, { target: { value: 'note' } });

    expect(queryAllByTestId('field-row')).toHaveLength(0);
  });
});

// ── Create mode — Save calls insert ──────────────────────────────────────────

describe('Form (create) — Save', () => {
  it('calls insert with path, input, and overwrite:false', async () => {
    mockInsert.mockResolvedValueOnce(undefined);

    const onsaved = vi.fn();
    const { getByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved,
        oncancel: vi.fn(),
      },
    });

    // Fill path
    await fireEvent.input(getByTestId('path-input'), { target: { value: 'web/test.com' } });

    // Fill password
    await fireEvent.input(getByTestId('password-input'), { target: { value: 'mypassword' } });

    // Click save
    await fireEvent.click(getByTestId('save-button'));

    await waitFor(() => {
      expect(mockInsert).toHaveBeenCalledWith(
        'web/test.com',
        expect.objectContaining({
          password: 'mypassword',
          fields: [],
          otp: null,
          tags: [],
        }),
        false
      );
    });

    expect(onsaved).toHaveBeenCalled();
  });

  it('shows AlreadyExists error inline and does not close', async () => {
    mockInsert.mockRejectedValueOnce(new Error('AlreadyExists'));

    const onsaved = vi.fn();
    const { getByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved,
        oncancel: vi.fn(),
      },
    });

    await fireEvent.input(getByTestId('path-input'), { target: { value: 'web/test.com' } });
    await fireEvent.input(getByTestId('password-input'), { target: { value: 'pw' } });
    await fireEvent.click(getByTestId('save-button'));

    await waitFor(() => {
      const err = getByTestId('form-error');
      expect(err).toBeInTheDocument();
      expect(err.textContent).toMatch(/AlreadyExists/i);
    });

    // onsaved NOT called
    expect(onsaved).not.toHaveBeenCalled();
  });

  it('Save with OAuth template includes client_id/client_secret/url fields', async () => {
    mockInsert.mockResolvedValueOnce(undefined);

    const { getByTestId, getAllByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    // Apply oauth template
    const select = getByTestId('template-select');
    await fireEvent.change(select, { target: { value: 'oauth' } });

    // Verify fields rendered
    const rows = getAllByTestId('field-row');
    expect(rows).toHaveLength(3);

    // Fill values
    await fireEvent.input(getByTestId('field-value-0'), { target: { value: 'my_client_id' } });
    await fireEvent.input(getByTestId('field-value-1'), { target: { value: 'my_secret' } });
    await fireEvent.input(getByTestId('field-value-2'), { target: { value: 'https://example.com' } });

    // Fill path and password
    await fireEvent.input(getByTestId('path-input'), { target: { value: 'oauth/app' } });
    await fireEvent.input(getByTestId('password-input'), { target: { value: 'token123' } });

    await fireEvent.click(getByTestId('save-button'));

    await waitFor(() => {
      expect(mockInsert).toHaveBeenCalledWith(
        'oauth/app',
        expect.objectContaining({
          password: 'token123',
          fields: [
            ['client_id', 'my_client_id'],
            ['client_secret', 'my_secret'],
            ['url', 'https://example.com'],
          ],
          otp: null,
          tags: [],
        }),
        false
      );
    });
  });

  it('Generate button fills the password field with a non-empty value', async () => {
    const { getByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    const passwordInput = getByTestId('password-input') as HTMLInputElement;
    expect(passwordInput.value).toBe('');

    await fireEvent.click(getByTestId('generate-button'));

    expect(passwordInput.value).toBeTruthy();
    expect(passwordInput.value.length).toBeGreaterThanOrEqual(20);
  });
});

// ── Edit mode — prefills from backend ────────────────────────────────────────

describe('Form (edit) — prefill', () => {
  it('prefills fields from showMeta, revealPassword, and revealOtpUri', async () => {
    mockShowMeta.mockResolvedValueOnce({
      path: 'web/github.com',
      fields: [['user', 'alice'], ['url', 'https://github.com']],
      tags: ['work', 'dev'],
      has_otp: true,
    });
    mockRevealPassword.mockResolvedValueOnce('secret42');
    mockRevealOtpUri.mockResolvedValueOnce('otpauth://totp/GitHub?secret=BASE32SECRET');

    const { getByTestId, getAllByTestId } = render(Form, {
      props: {
        mode: 'edit',
        path: 'web/github.com',
        onsaved: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    await waitFor(() => {
      const passwordInput = getByTestId('password-input') as HTMLInputElement;
      expect(passwordInput.value).toBe('secret42');
    });

    const rows = getAllByTestId('field-row');
    expect(rows).toHaveLength(2);

    const key0 = getByTestId('field-key-0') as HTMLInputElement;
    const key1 = getByTestId('field-key-1') as HTMLInputElement;
    expect(key0.value).toBe('user');
    expect(key1.value).toBe('url');

    const otpInput = getByTestId('otp-input') as HTMLInputElement;
    expect(otpInput.value).toBe('otpauth://totp/GitHub?secret=BASE32SECRET');

    // Tags should be rendered as chips
    const chips = getAllByTestId('tag-chip');
    expect(chips).toHaveLength(2);
    const chipTexts = chips.map((c) => c.textContent?.replace(/✕/, '').trim());
    expect(chipTexts).toContain('@work');
    expect(chipTexts).toContain('@dev');
  });

  it('Save calls updateEntry with the prefilled + any edited data', async () => {
    mockShowMeta.mockResolvedValueOnce({
      path: 'web/github.com',
      fields: [['user', 'alice']],
      tags: ['work'],
      has_otp: false,
    });
    mockRevealPassword.mockResolvedValueOnce('secret42');
    mockRevealOtpUri.mockResolvedValueOnce(null);
    mockUpdateEntry.mockResolvedValueOnce(undefined);

    const onsaved = vi.fn();
    const { getByTestId } = render(Form, {
      props: {
        mode: 'edit',
        path: 'web/github.com',
        onsaved,
        oncancel: vi.fn(),
      },
    });

    // Wait for prefill
    await waitFor(() => {
      const pw = getByTestId('password-input') as HTMLInputElement;
      expect(pw.value).toBe('secret42');
    });

    // Click save without changing anything
    await fireEvent.click(getByTestId('save-button'));

    await waitFor(() => {
      expect(mockUpdateEntry).toHaveBeenCalledWith(
        'web/github.com',
        expect.objectContaining({
          password: 'secret42',
          fields: [['user', 'alice']],
          otp: null,
          tags: ['work'],
        })
      );
    });

    expect(onsaved).toHaveBeenCalled();
  });
});

// ── Cancel button ─────────────────────────────────────────────────────────────

describe('Form — cancel', () => {
  it('calls oncancel when Cancel is clicked', async () => {
    const oncancel = vi.fn();
    const { getByTestId } = render(Form, {
      props: {
        mode: 'create',
        onsaved: vi.fn(),
        oncancel,
      },
    });

    await fireEvent.click(getByTestId('cancel-button'));
    expect(oncancel).toHaveBeenCalledTimes(1);
  });
});
