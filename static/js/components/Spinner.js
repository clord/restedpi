import { html } from '/js/html.js';

/**
 * Show a Spinner which fills its container
 */
export function Spinner(props) {
  return html`
    <div class="centered-inside">
      <div class="fa-3x">
        <i class="fas fa-cog fa-spin"></i>
      </div>
    </div>
  `;
}
