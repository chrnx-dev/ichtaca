<script lang="ts">
  interface Props {
    title: string;
    message: string;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let { title, message, onconfirm, oncancel }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      oncancel();
    }
  }
</script>

<div
  class="modal-overlay"
  role="dialog"
  aria-modal="true"
  aria-label={title}
  tabindex="-1"
  onkeydown={handleKeydown}
>
  <div class="modal-card">
    <h2 class="modal-title" data-testid="modal-title">{title}</h2>
    <p class="modal-message" data-testid="modal-message">{message}</p>
    <div class="modal-actions">
      <button
        class="btn-danger"
        type="button"
        onclick={onconfirm}
        data-testid="confirm-button"
      >
        Confirm
      </button>
      <button
        class="btn-secondary"
        type="button"
        onclick={oncancel}
        data-testid="cancel-button"
      >
        Cancel
      </button>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .modal-card {
    background: #fff;
    border-radius: 6px;
    padding: 1.5rem;
    width: 100%;
    max-width: 380px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.18);
  }
  .modal-title {
    margin: 0 0 0.6rem;
    font-size: 1.05rem;
    font-weight: 700;
  }
  .modal-message {
    margin: 0 0 1.2rem;
    font-size: 0.9rem;
    color: #444;
    line-height: 1.45;
  }
  .modal-actions {
    display: flex;
    gap: 0.6rem;
    justify-content: flex-end;
  }
  .btn-danger {
    padding: 0.4rem 1.1rem;
    background: #c62828;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
  }
  .btn-danger:hover {
    background: #b71c1c;
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
</style>
