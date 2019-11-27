import { usePost, useGet } from '/static/js/hooks/network.js'
import { h, render } from '/static/js/html.js'
import { Route } from '/static/js/depend/wouter/index.js'

import { Header } from './Header.js'
import { Switches } from './Switches/index.js'
import { Sensors } from './Sensors/index.js'
import { Devices } from './Devices/index.js'

const app = [
    h(Header),
    h("section", {}, [
        h(Route, {path: "/devices/:rest*", component: Devices}),
        h(Route, {path: "/sensors/:rest*", component: Sensors}),
        h(Route, {path: "/switches/:rest*", component: Switches})
    ]),
    h("footer", {}, "The Footer of Power")
]

render(app, document.getElementById('app'));

