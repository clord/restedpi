import { usePost, useGet } from '/static/js/hooks/network.js'
import { html, render } from '/static/js/html.js'


import { Header } from './Header.js'
import { Switches } from './Switches/index.js'
import { Sensors } from './Sensors/index.js'
import { Devices } from './Devices/index.js'

//import { Router } from 'https://unpkg.com/@reach/router@1.2.1/es/index.js?module'
      // <${Router}>
      // </${Router}>

const app = html`
    <${Header} />
    <section>
        <${Devices} path="/devices" />
        <${Sensors} path="/sensors" />
        <${Switches} path="/switches" />
    </section>
    <footer>Footer bla bla</footer>
`

render(app, document.getElementById('app'));

