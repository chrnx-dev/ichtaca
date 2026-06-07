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
  class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[200]"
  role="dialog"
  aria-modal="true"
  aria-label={title}
  tabindex="-1"
  onkeydown={handleKeydown}
>
  <div class="card bg-base-100 w-full max-w-sm shadow-2xl border border-error/20">
    <div class="card-body p-5">

      <!-- Title with danger icon -->
      <div class="flex items-center gap-2 mb-2">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-error flex-shrink-0" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
        <h2 class="text-error font-bold text-base" data-testid="modal-title">{title}</h2>
      </div>

      <div class="h-px bg-error/20 mb-3"></div>

      <p class="text-base-content/80 text-sm leading-relaxed mb-4" data-testid="modal-message">{message}</p>

      <div class="flex gap-2 justify-end">
        <button
          class="btn btn-error btn-sm"
          type="button"
          onclick={onconfirm}
          data-testid="confirm-button"
        >
          <!-- Trash icon -->
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd"/>
          </svg>
          Confirm
        </button>
        <button
          class="btn btn-ghost btn-sm border border-neutral/30"
          type="button"
          onclick={oncancel}
          data-testid="cancel-button"
        >
          Cancel
        </button>
      </div>

    </div>
  </div>
</div>
