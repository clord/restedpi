import { Route } from '/js/depend/wouter/'
import { useEffect, lazy, Suspense } from '/js/depend/react/'
import { h, render } from '/js/html.js'
import { useApp } from '/js/hooks/useApp.js'

import { Header } from './Header.js'

const Switches = lazy(() => import('./Switches/'))
const Sensors = lazy(() => import('./Sensors/'))
const Devices = lazy(() => import('./Devices/'))

function App() {
    useApp()
    return [
        h(Header),
        h(Suspense, {fallback: h("div", {className: ""}, "...")},
            h("section", {className: "container mx-auto mt-3 h-full"}, [
                h(Route, {path: "/devices/:rest*", component: Devices}),
                h(Route, {path: "/sensors/:rest*", component: Sensors}),
                h(Route, {path: "/switches/:rest*", component: Switches})
            ])
        ),
    ]
}

render(h(App), document.getElementById('app'));

