import { useGet } from './util.js'
import { html, render } from './html.js'
import { Link } from './depend/preact-router.js';

export function Nav(props) {
    return html`
        <nav style="display:flex; flex-direction:column;">
            <${Link} href="/devices">Devices</${Link}>
            <${Link} href="/sensors">Sensors</${Link}>
            <${Link} href="/switches">Switches</${Link}>
        </nav>
    `
}
