import { useGet } from '/static/js/hooks/network.js'
import { html, render } from '/static/js/html.js'
// import { Link } from './depend/preact-router.js';

export function Nav(props) {
    return html`
        <nav>
            <a href="/devices">
                <i class="fas fa-microchip"></i>
                Devices
            </a>
            <a href="/sensors">
            <i class="fas fa-thermometer-quarter"></i>
                Sensors
            </a>
            <a href="/switches">
            <i class="fas fa-toggle-on"></i>
                Switches
            </a>
        </nav>
    `
}
