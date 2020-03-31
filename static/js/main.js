import { Route } from '/js/depend/wouter/';
import { useEffect, lazy, Suspense } from '/react/';
import { h, render } from '/js/html.js';
import { useApp } from '/js/hooks/useApp.js';
import { useRefreshOnMount } from '/js/useRefreshOnMount.js';

import { Header } from './Header.js';

const Switches = lazy(() => import('./Switches/'));
const Sensors = lazy(() => import('./Sensors/'));
const Devices = lazy(() => import('./Devices/'));

function App() {
  useApp();
  useRefreshOnMount('/devices', x => x.devices.read);
  useRefreshOnMount('/sensors', x => x.sensors.read);
  return [
    h(Header, { key: 0 }),
    h(
      Suspense,
      { key: 1, fallback: h('div', { className: '' }, '...') },
      h('section', { className: 'container mx-auto mt-4' }, [
        h(Route, { path: '/devices/:rest*', key: 0, component: Devices }),
        h(Route, { path: '/sensors/:rest*', key: 1, component: Sensors }),
        h(Route, { path: '/switches/:rest*', key: 2, component: Switches }),
      ])
    ),
  ];
}

render(h(App), document.getElementById('app'));
