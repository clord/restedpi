import { useEffect, lazy, Suspense } from '/static/js/depend/react/react.js'
import { h, render } from '/static/js/html.js'
import { Route } from '/static/js/depend/wouter/index.js'
import { useApp } from '/static/js/hooks/useApp.js'

import { Header } from './Header.js'

const Switches = lazy(() => import('./Switches/index.js'))
const Sensors = lazy(() => import('./Sensors/index.js'))
const Devices = lazy(() => import('./Devices/index.js'))

function App() {
    useApp()
    return [
        h(Header),
        h(Suspense, {fallback: h("div", {className: "loading"}, "...")},
            h("section", {}, [
                h(Route, {path: "/devices/:rest*", component: Devices}),
                h(Route, {path: "/sensors/:rest*", component: Sensors}),
                h(Route, {path: "/switches/:rest*", component: Switches})
            ])
        ),
        h("footer", {}, "The Footer of Power")
    ]
}

render(h(App), document.getElementById('app'));

