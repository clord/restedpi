import { html, h } from './html.js'
import { Nav } from './Nav.js'
import { useApp } from '/static/js/hooks/useApp.js'

export function About() {
    const server = useApp(x => x.server)

    if (server == null) {
        return h("aside", {}, "loading")
    }
    return h("aside", {}, server)
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
