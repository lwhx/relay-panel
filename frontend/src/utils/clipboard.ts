/**
 * Copy text to clipboard with robust fallback for non-HTTPS contexts.
 *
 * navigator.clipboard.writeText is only available in secure contexts (HTTPS
 * or localhost). On HTTP deployments (e.g. http://server-ip:18888), it throws
 * or silently fails. This function tries multiple methods:
 *
 *   1. navigator.clipboard.writeText (modern, HTTPS only)
 *   2. hidden textarea + execCommand('copy') (works on HTTP)
 *   3. Selection API fallback
 *
 * Returns true only when the copy genuinely succeeded.
 */
export async function copyText(text: string): Promise<boolean> {
  if (!text || text.length === 0) return false;

  // Method 1: modern Clipboard API (HTTPS / localhost only)
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // Fall through to fallback methods
    }
  }

  // Method 2: hidden textarea + execCommand('copy')
  // Works in HTTP contexts. Some browsers require the element to be
  // focused and selected before execCommand works.
  try {
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.setAttribute('readonly', '');
    textarea.style.position = 'fixed';
    textarea.style.top = '0';
    textarea.style.left = '0';
    textarea.style.width = '1px';
    textarea.style.height = '1px';
    textarea.style.padding = '0';
    textarea.style.border = 'none';
    textarea.style.outline = 'none';
    textarea.style.boxShadow = 'none';
    textarea.style.background = 'transparent';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);

    // Focus and select
    textarea.focus();
    textarea.select();
    textarea.setSelectionRange(0, textarea.value.length);

    // Also try selecting via the Selection API
    const range = document.createRange();
    range.selectNodeContents(textarea);
    const sel = window.getSelection();
    if (sel) {
      sel.removeAllRanges();
      sel.addRange(range);
    }

    const ok = document.execCommand('copy');
    document.body.removeChild(textarea);

    // Clear selection
    if (sel) sel.removeAllRanges();

    if (ok) return true;
  } catch {
    // Fall through
  }

  // All methods failed
  return false;
}
