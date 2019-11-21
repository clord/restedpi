import { usePost, useGet } from '/static/js/hooks/network.js'
import { html, render } from '/static/js/html.js'
import { Router, Link } from '/static/js/depend/preact-router.js'

import { Header } from './Header.js'
import { Switches } from './Switches/index.js'
import { Sensors } from './Sensors/index.js'
import { Devices } from './Devices/index.js'

const app = html`
    <${Header} />
    <section>
      <${Router}>
        <${Devices} path="/devices" />
        <${Sensors} path="/sensors" />
        <${Switches} path="/switches" />
      </${Router}>
    </section>
    <footer>Footer bla bla</footer>
`

render(app, document.getElementById('app'));

