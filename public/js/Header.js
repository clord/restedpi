import { useGet } from './util.js'
import { html, render } from './html.js'

/**
 * Present the current description of the server to the user
 */
export function Header(props) {
    const {response, error} = useGet(`/api/about`);
    if (response == null) {
        return html`
            <h1>
                <i class="fas fa-pizza-slice"></i>
                RestedPI
            </h1>
            <h2>
                <i class="fas fa-microchip"></i>
            </h2>
        `
    }

    return html`
        <h1>
            <i class="fas fa-pizza-slice"></i>
            RestedPI
        </h1>
        <h2>
            <i class="fas fa-microchip"></i>
            ${response.server}
        </h2>
    `
}
