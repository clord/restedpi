import { useGet } from './util.js'
import { html, render } from './html.js'
import { Nav } from './Nav.js';

export function About(props) {
    const { response, error } = useGet(`/api/about`);
    if (response == null) {
        return html`<aside>loading</aside>`
    }
    return html`<aside>${response.server}</aside>`
}

/**
 * Present the current description of the server to the user
 */
export function Header(props) {
    return html`
        <header>
            <h1>
                <i class="fas fa-pizza-slice"></i>
                REpi
            </h1>
        </header>
        <${Nav} />
        <${About} />
    `
}
