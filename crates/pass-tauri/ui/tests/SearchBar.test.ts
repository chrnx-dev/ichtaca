import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import SearchBar from '../src/components/SearchBar.svelte';

// Mock api
vi.mock('../src/lib/api', () => ({
  searchFuzzy: vi.fn(),
  list: vi.fn(),
  buildTree: vi.fn(),
  showMeta: vi.fn(),
  revealPassword: vi.fn(),
  revealOtpUri: vi.fn(),
  insert: vi.fn(),
  updateEntry: vi.fn(),
  remove: vi.fn(),
  mv: vi.fn(),
  cp: vi.fn(),
  generate: vi.fn(),
  copyPassword: vi.fn(),
  otpCode: vi.fn(),
}));

import { searchFuzzy } from '../src/lib/api';

const mockSearchFuzzy = vi.mocked(searchFuzzy);

beforeEach(() => {
  vi.clearAllMocks();
  vi.useFakeTimers();
});

// Restore real timers after each test
afterEach(() => {
  vi.useRealTimers();
});

describe('SearchBar', () => {
  it('renders the search input', () => {
    const { getByTestId } = render(SearchBar, {
      props: { onselect: vi.fn(), onclear: vi.fn() },
    });
    expect(getByTestId('search-input')).toBeInTheDocument();
  });

  it('calls searchFuzzy after debounce when text is typed', async () => {
    mockSearchFuzzy.mockResolvedValueOnce(['web/github.com', 'web/gitlab.com']);

    const { getByTestId } = render(SearchBar, {
      props: { onselect: vi.fn(), onclear: vi.fn() },
    });

    const input = getByTestId('search-input');
    await fireEvent.input(input, { target: { value: 'git' } });

    // searchFuzzy not called yet (debounced)
    expect(mockSearchFuzzy).not.toHaveBeenCalled();

    // Advance timers past the 250 ms debounce
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      expect(mockSearchFuzzy).toHaveBeenCalledWith('git');
    });
  });

  it('renders results returned by searchFuzzy', async () => {
    mockSearchFuzzy.mockResolvedValueOnce(['web/github.com', 'email/work']);

    const { getByTestId, getAllByTestId } = render(SearchBar, {
      props: { onselect: vi.fn(), onclear: vi.fn() },
    });

    const input = getByTestId('search-input');
    await fireEvent.input(input, { target: { value: 'gi' } });
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      const results = getAllByTestId('search-result');
      expect(results).toHaveLength(2);
      expect(results[0].textContent?.trim()).toBe('web/github.com');
      expect(results[1].textContent?.trim()).toBe('email/work');
    });
  });

  it('emits the selected path when a result is clicked', async () => {
    mockSearchFuzzy.mockResolvedValueOnce(['web/github.com']);

    const onselect = vi.fn();
    const { getByTestId, getAllByTestId } = render(SearchBar, {
      props: { onselect, onclear: vi.fn() },
    });

    const input = getByTestId('search-input');
    await fireEvent.input(input, { target: { value: 'github' } });
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      expect(getAllByTestId('search-result')).toHaveLength(1);
    });

    await fireEvent.click(getAllByTestId('search-result')[0]);

    expect(onselect).toHaveBeenCalledWith('web/github.com');
  });

  it('calls onclear and hides results when query is cleared', async () => {
    mockSearchFuzzy.mockResolvedValueOnce(['web/github.com']);

    const onclear = vi.fn();
    const { getByTestId, queryAllByTestId } = render(SearchBar, {
      props: { onselect: vi.fn(), onclear },
    });

    const input = getByTestId('search-input');
    // Type to search
    await fireEvent.input(input, { target: { value: 'git' } });
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      expect(queryAllByTestId('search-result')).toHaveLength(1);
    });

    // Clear input
    await fireEvent.input(input, { target: { value: '' } });
    vi.advanceTimersByTime(300);

    await waitFor(() => {
      expect(queryAllByTestId('search-result')).toHaveLength(0);
      expect(onclear).toHaveBeenCalled();
    });
  });

  it('does not call searchFuzzy when query is empty', async () => {
    const { getByTestId } = render(SearchBar, {
      props: { onselect: vi.fn(), onclear: vi.fn() },
    });

    const input = getByTestId('search-input');
    await fireEvent.input(input, { target: { value: '' } });
    vi.advanceTimersByTime(300);

    expect(mockSearchFuzzy).not.toHaveBeenCalled();
  });
});
