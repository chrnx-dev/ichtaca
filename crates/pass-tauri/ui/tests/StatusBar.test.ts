import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import StatusBar from '../src/components/StatusBar.svelte';
import type { GitStatus } from '../src/lib/types';

const clean: GitStatus = { branch: 'main', ahead: 0, behind: 0, dirty: 0, upstream: true };

describe('StatusBar git chip', () => {
  it('renders nothing when there is no message and no git repo', () => {
    const { queryByTestId, container } = render(StatusBar, { props: { message: '', git: null } });
    expect(queryByTestId('git-chip')).toBeNull();
    expect(container.textContent?.trim()).toBe('');
  });

  it('shows the branch and a check when clean and in sync', () => {
    const { getByTestId } = render(StatusBar, { props: { message: '', git: clean } });
    const chip = getByTestId('git-chip');
    expect(chip.textContent).toContain('main');
    expect(chip.textContent).toContain('✓');
  });

  it('shows the ahead count when there are unpushed commits', () => {
    const { getByTestId } = render(StatusBar, {
      props: { message: '', git: { ...clean, ahead: 3 } },
    });
    const chip = getByTestId('git-chip');
    expect(chip.textContent).toContain('↑3');
    expect(chip.textContent).not.toContain('✓');
  });

  it('calls onsync when the chip is clicked', async () => {
    const onsync = vi.fn();
    const { getByTestId } = render(StatusBar, {
      props: { message: '', git: { ...clean, ahead: 1 }, onsync },
    });
    await fireEvent.click(getByTestId('git-chip'));
    expect(onsync).toHaveBeenCalledOnce();
  });

  it('disables sync when the branch has no upstream', () => {
    const { getByTestId } = render(StatusBar, {
      props: { message: '', git: { ...clean, upstream: false } },
    });
    const chip = getByTestId('git-chip') as HTMLButtonElement;
    expect(chip.disabled).toBe(true);
    expect(chip.textContent).toContain('no remote');
  });

  it('disables sync while a sync is in flight', () => {
    const { getByTestId } = render(StatusBar, {
      props: { message: '', git: { ...clean, ahead: 1 }, syncing: true },
    });
    expect((getByTestId('git-chip') as HTMLButtonElement).disabled).toBe(true);
  });
});
