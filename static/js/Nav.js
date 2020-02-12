import { useGet } from '/js/hooks/network.js';
import { html, render } from '/js/html.js';
import { Link } from '/js/depend/wouter/';

export function Nav(props) {
  return html`
        <nav class="w-full block flex-grow lg:flex lg:items-center lg:w-auto">
            <${Link} href="/devices" class="block mt-5 lg:inline-block lg:mt-0 text-orange-200 hover:text-white mr-4">
                <i class="fas fa-microchip mr-1"></i>
                Devices
            </${Link}>
            <${Link} href="/sensors" class="block mt-5 lg:inline-block lg:mt-0 text-orange-200 hover:text-white mr-4">
                <i class="fas fa-thermometer-quarter mr-1"></i>
                Sensors
            </${Link}>
            <${Link} href="/switches" class="block mt-5 lg:inline-block lg:mt-0 text-orange-200 hover:text-white mr-4">
                <i class="fas fa-toggle-on mr-1"></i>
                Switches
            </${Link}>
        </nav>
    `;
}
