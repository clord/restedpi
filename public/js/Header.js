import { html, h } from './html.js'
import { Nav } from './Nav.js'
import { useAppStore } from '/static/js/hooks/useApp.js'

export function About() {
    const name = useAppStore(x => x.serverConfig.deviceName)

    if (name == null) {
        return h("aside", {}, "loading")
    }
    return h("aside", {}, name)
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
