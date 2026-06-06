import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import DetailPanel from '../src/components/DetailPanel.svelte';
import type { EntryMeta } from '../src/lib/types';

// Mock the api module so invoke is never called in tests
vi.mock('../src/lib/api', () => ({
  revealPassword: vi.fn(),
  copyPassword: vi.fn(),
  otpCode: vi.fn(),
  buildTree: vi.fn(),
  list: vi.fn(),
  showMeta: vi.fn(),
  searchFuzzy: vi.fn(),
}));

import { revealPassword, copyPassword, otpCode } from '../src/lib/api';

const mockRevealPassword = vi.mocked(revealPassword);
const mockCopyPassword = vi.mocked(copyPassword);
const mockOtpCode = vi.mocked(otpCode);

const baseMeta: EntryMeta = {
  path: 'web/github.com',
  fields: [['user', 'bob']],
  tags: ['work'],
  has_otp: false,
};

const otpMeta: EntryMeta = {
  path: 'web/github.com',
  fields: [['user', 'bob']],
  tags: [],
  has_otp: true,
};

beforeEach(() => {
  vi.clearAllMocks();
});

// ── Security: mask-until-reveal ───────────────────────────────────────────────

describe('DetailPanel — password masking (security)', () => {
  it('shows dots (••••••••) by default — password NOT exposed', () => {
    const { getByTestId } = render(DetailPanel, {
      props: {
        meta: baseMeta,
        onnotice: vi.fn(),
        onerror: vi.fn(),
      },
    });
    const masked = getByTestId('password-masked');
    expect(masked).toBeInTheDocument();
    expect(masked.textContent).toMatch(/^•+$/);
    // The revealed element should not be present
    expect(() => getByTestId('password-revealed')).toThrow();
  });

  it('shows the password only after the Reveal button is clicked', async () => {
    mockRevealPassword.mockResolvedValueOnce('s3cret!');

    const { getByTestId } = render(DetailPanel, {
      props: {
        meta: baseMeta,
        onnotice: vi.fn(),
        onerror: vi.fn(),
      },
    });

    // Before reveal: dots
    expect(getByTestId('password-masked')).toBeInTheDocument();

    // Click reveal
    await fireEvent.click(getByTestId('reveal-button'));

    // After reveal: actual password visible
    await waitFor(() => {
      expect(getByTestId('password-revealed')).toBeInTheDocument();
      expect(getByTestId('password-revealed').textContent).toBe('s3cret!');
    });

    // revealPassword was called with the correct path
    expect(mockRevealPassword).toHaveBeenCalledWith('web/github.com');
  });
});

// ── Security: copy-doesn't-reveal ─────────────────────────────────────────────

describe('DetailPanel — copy does NOT reveal password (security)', () => {
  it('Copy button calls copyPassword and does NOT call revealPassword', async () => {
    mockCopyPassword.mockResolvedValueOnce(undefined);

    const { getByTestId } = render(DetailPanel, {
      props: {
        meta: baseMeta,
        onnotice: vi.fn(),
        onerror: vi.fn(),
      },
    });

    await fireEvent.click(getByTestId('copy-button'));

    await waitFor(() => {
      expect(mockCopyPassword).toHaveBeenCalledWith('web/github.com');
    });

    // revealPassword must never have been called
    expect(mockRevealPassword).not.toHaveBeenCalled();

    // Password stays masked (no plaintext entered JS)
    expect(getByTestId('password-masked')).toBeInTheDocument();
    expect(() => getByTestId('password-revealed')).toThrow();
  });
});

// ── OTP rendering ─────────────────────────────────────────────────────────────

describe('DetailPanel — OTP rendering', () => {
  it('renders the OTP code and countdown when has_otp is true', async () => {
    mockOtpCode.mockResolvedValueOnce({ code: '987654', seconds: 18 });

    const { getByTestId } = render(DetailPanel, {
      props: {
        meta: otpMeta,
        onnotice: vi.fn(),
        onerror: vi.fn(),
      },
    });

    await waitFor(() => {
      expect(getByTestId('otp-code')).toBeInTheDocument();
      expect(getByTestId('otp-code').textContent).toBe('987654');
    });

    expect(getByTestId('otp-countdown').textContent).toMatch(/18s/);
  });

  it('does not render OTP section when has_otp is false', () => {
    const { queryByTestId } = render(DetailPanel, {
      props: {
        meta: baseMeta,
        onnotice: vi.fn(),
        onerror: vi.fn(),
      },
    });
    expect(queryByTestId('otp-section')).not.toBeInTheDocument();
  });
});

// ── Placeholder when no entry selected ───────────────────────────────────────

describe('DetailPanel — null meta', () => {
  it('shows placeholder text when no entry is selected', () => {
    const { getByText } = render(DetailPanel, {
      props: {
        meta: null,
        onnotice: vi.fn(),
        onerror: vi.fn(),
      },
    });
    expect(getByText(/select an entry/i)).toBeInTheDocument();
  });
});
