import { usePost, useGet } from './util.js'
import { html, render } from './html.js'
import { Router, Link } from './depend/preact-router.js';
import { Header } from './Header.js';
import { Switches } from './Switches.js';
import { Sensors } from './Sensors.js';
import { Devices } from './Devices.js';

const app = html`
    <${Header} />
    <section>
      <${Router}>
        <${Devices} path="/devices" />
        <${Sensors} path="/sensors" />
        <${Switches} path="/switches" />
      </${Router}>
    </section>
    <footer>Footer</footer>
`

render(app, document.getElementById('app'));

