import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ConfirmModal from '../src/components/ConfirmModal.svelte';

beforeEach(() => {
  vi.clearAllMocks();
});

describe('ConfirmModal', () => {
  it('renders the title and message', () => {
    const { getByTestId } = render(ConfirmModal, {
      props: {
        title: 'Delete entry',
        message: 'Are you sure you want to delete "web/test.com"?',
        onconfirm: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    expect(getByTestId('modal-title').textContent).toBe('Delete entry');
    expect(getByTestId('modal-message').textContent).toBe('Are you sure you want to delete "web/test.com"?');
  });

  it('calls onconfirm when Confirm button is clicked', async () => {
    const onconfirm = vi.fn();
    const oncancel = vi.fn();

    const { getByTestId } = render(ConfirmModal, {
      props: {
        title: 'Delete',
        message: 'Really delete?',
        onconfirm,
        oncancel,
      },
    });

    await fireEvent.click(getByTestId('confirm-button'));

    expect(onconfirm).toHaveBeenCalledTimes(1);
    expect(oncancel).not.toHaveBeenCalled();
  });

  it('calls oncancel when Cancel button is clicked and does NOT call onconfirm', async () => {
    const onconfirm = vi.fn();
    const oncancel = vi.fn();

    const { getByTestId } = render(ConfirmModal, {
      props: {
        title: 'Delete',
        message: 'Really delete?',
        onconfirm,
        oncancel,
      },
    });

    await fireEvent.click(getByTestId('cancel-button'));

    expect(oncancel).toHaveBeenCalledTimes(1);
    expect(onconfirm).not.toHaveBeenCalled();
  });

  it('shows Confirm and Cancel buttons', () => {
    const { getByTestId } = render(ConfirmModal, {
      props: {
        title: 'Delete',
        message: 'Really delete?',
        onconfirm: vi.fn(),
        oncancel: vi.fn(),
      },
    });

    expect(getByTestId('confirm-button')).toBeInTheDocument();
    expect(getByTestId('cancel-button')).toBeInTheDocument();
  });
});
