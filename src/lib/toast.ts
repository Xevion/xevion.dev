import { toast as sonner, type ExternalToast } from "svelte-sonner";

/**
 * Errors persist until dismissed: a failed action the user must read and act on
 * should never disappear on its own. The Toaster renders a close button, so the
 * user retains explicit control. Success and info auto-dismiss quickly since
 * they are confirmations, not call-to-action.
 */
const ERROR_DURATION = Number.POSITIVE_INFINITY;
const TRANSIENT_DURATION = 3000;

export const toast = {
  success(message: string, opts?: ExternalToast) {
    return sonner.success(message, { duration: TRANSIENT_DURATION, ...opts });
  },
  error(message: string, opts?: ExternalToast) {
    return sonner.error(message, { duration: ERROR_DURATION, ...opts });
  },
  info(message: string, opts?: ExternalToast) {
    return sonner.info(message, { duration: TRANSIENT_DURATION, ...opts });
  },
};
