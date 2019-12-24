import { Route } from '/static/js/depend/wouter'

import { useEffect, lazy, Suspense } from '/static/js/depend/react/react.js'
import { h, render } from '/static/js/html.js'
import { useApp } from '/static/js/hooks/useApp.js'

import { Header } from './Header.js'

const Switches = lazy(() => import('./Switches'))
const Sensors = lazy(() => import('./Sensors'))
const Devices = lazy(() => import('./Devicess'))

function App() {
    useApp()
    return [
        h(Header),
        h(Suspense, {fallback: h("div", {className: "loading"}, "...")},
            h("section", {className: "main-body"}, [
                h(Route, {path: "/devices/:rest*", component: Devices}),
                h(Route, {path: "/sensors/:rest*", component: Sensors}),
                h(Route, {path: "/switches/:rest*", component: Switches})
            ])
        ),
        h("footer", {}, "The Footer of Power")
    ]
}

render(h(App), document.getElementById('app'));

