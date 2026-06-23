import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import App from '../src/App.svelte';

// Mock the entire api module so we control what doctor / list return
vi.mock('../src/lib/api', () => ({
  doctor: vi.fn(),
  list: vi.fn(),
  showMeta: vi.fn(),
  buildTree: vi.fn(),
  remove: vi.fn(),
  revealPassword: vi.fn(),
  revealOtpUri: vi.fn(),
  copyPassword: vi.fn(),
  otpCode: vi.fn(),
  insert: vi.fn(),
  updateEntry: vi.fn(),
  searchFuzzy: vi.fn(),
  searchDeep: vi.fn(),
  generatePassword: vi.fn(),
  mv: vi.fn(),
  cp: vi.fn(),
  generate: vi.fn(),
}));

import { doctor, list, buildTree } from '../src/lib/api';

const mockDoctor = vi.mocked(doctor);
const mockList = vi.mocked(list);
const mockBuildTree = vi.mocked(buildTree);

const OK_REPORT = {
  ok: true,
  pass: true,
  gpg: true,
  store_dir_exists: true,
  store_dir: '~/.password-store',
  guidance: '',
  demo: false,
  init_error: null,
};

beforeEach(() => {
  vi.clearAllMocks();
  // Default: buildTree returns the real implementation shape
  mockBuildTree.mockImplementation(() => []);
});

describe('App.svelte – doctor gate', () => {
  it('renders the main UI when doctor returns ok=true', async () => {
    mockDoctor.mockResolvedValueOnce(OK_REPORT);
    mockList.mockResolvedValueOnce([]);

    const { getByTestId } = render(App);

    await waitFor(() => {
      expect(getByTestId('new-button')).toBeInTheDocument();
    });

    expect(mockDoctor).toHaveBeenCalledTimes(1);
    expect(mockList).toHaveBeenCalledTimes(1);
  });

  it('renders SetupScreen and does NOT call list when doctor returns ok=false', async () => {
    mockDoctor.mockResolvedValueOnce({
      ok: false,
      pass: false,
      gpg: true,
      store_dir_exists: false,
      store_dir: '~/.password-store',
      guidance: 'Install pass: brew install pass',
      demo: false,
      init_error: null,
    });

    const { getByTestId, queryByTestId } = render(App);

    await waitFor(() => {
      expect(getByTestId('setup-screen')).toBeInTheDocument();
    });

    expect(mockList).not.toHaveBeenCalled();
    // Main UI navbar actions should not be present
    expect(queryByTestId('new-button')).toBeNull();
  });

  it('shows the DEMO badge in the navbar when doctor reports demo=true', async () => {
    mockDoctor.mockResolvedValueOnce({ ...OK_REPORT, demo: true });
    mockList.mockResolvedValueOnce([]);

    const { getByText } = render(App);

    await waitFor(() => {
      expect(getByText('DEMO')).toBeInTheDocument();
    });
  });

  it('renders SetupScreen (not a permanent spinner) when doctor() throws', async () => {
    mockDoctor.mockRejectedValueOnce(new Error('IPC channel closed'));

    const { getByTestId, queryByTestId } = render(App);

    await waitFor(() => {
      expect(getByTestId('setup-screen')).toBeInTheDocument();
    });

    expect(queryByTestId('new-button')).toBeNull();
    expect(mockList).not.toHaveBeenCalled();
  });
});
