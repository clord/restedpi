import { useGet } from './util.js'
import { html, render } from './html.js'
import { Link } from './depend/preact-router.js';

export function Nav(props) {
    return html`
        <nav>
            <${Link} href="/devices">
                <i class="fas fa-microchip"></i>
                Devices
            </${Link}>
            <${Link} href="/sensors">
            <i class="fas fa-thermometer-quarter"></i>
                Sensors
            </${Link}>
            <${Link} href="/switches">
            <i class="fas fa-toggle-on"></i>
                Switches
            </${Link}>
        </nav>
    `
}
