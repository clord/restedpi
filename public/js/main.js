import { useEffect } from '/static/js/depend/react/react.js'
import { usePost, useGet } from '/static/js/hooks/network.js'
import { h, render } from '/static/js/html.js'
import { Route } from '/static/js/depend/wouter/index.js'
import { useApp } from '/static/js/hooks/useApp.js'

import { Header } from './Header.js'
import { Switches } from './Switches/index.js'
import { Sensors } from './Sensors/index.js'
import { Devices } from './Devices/index.js'


function App() {

    const setup = useApp(x => x.setup)
    useEffect(() => {
        setup()
    }, [])

    return [
        h(Header),
        h("section", {}, [
            h(Route, {path: "/devices/:rest*", component: Devices}),
            h(Route, {path: "/sensors/:rest*", component: Sensors}),
            h(Route, {path: "/switches/:rest*", component: Switches})
        ]),
        h("footer", {}, "The Footer of Power")
    ]
}

render(h(App), document.getElementById('app'));

