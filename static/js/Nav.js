import { useGet } from "/js/hooks/network.js";
import { html, render } from "/js/html.js";
import { Link } from "/js/depend/wouter/";

export function Nav(props) {
  return html`
        <nav className="w-full flex flex-row items-center w-auto">
            <${Link} href="/devices" className="block inline-block mt-0 text-orange-200 hover:text-white mr-4">
                <i className="fas fa-microchip mr-1"></i>
                Devices
            </${Link}>
            <${Link} href="/sensors" className="block inline-block mt-0 text-orange-200 hover:text-white mr-4">
                <i className="fas fa-thermometer-quarter mr-1"></i>
                Sensors
            </${Link}>
            <${Link} href="/switches" className="block inline-block mt-0 text-orange-200 hover:text-white mr-4">
                <i className="fas fa-toggle-on mr-1"></i>
                Switches
            </${Link}>
        </nav>
    `;
}
