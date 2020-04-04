import { Route } from '/js/depend/wouter/';
import { Component, useEffect, lazy, Suspense } from '/react/';
import { h, render } from '/js/html.js';
import { useApp } from '/js/hooks/useApp.js';
import { useRefreshOnMount } from '/js/useRefreshOnMount.js';

import { Header } from './Header.js';

const Switches = lazy(() => import('./Switches/'));
const Sensors = lazy(() => import('./Sensors/'));
const Devices = lazy(() => import('./Devices/'));

class ErrorBoundary extends Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true };
  }
  componentDidCatch(error, errorInfo) {
    console.error(error, errorInfo);
  }
  render() {
    if (this.state.hasError) {
      return h('h1', [], 'Something went wrong');
    }

    return this.props.children;
  }
}

function App() {
  useApp();
  useRefreshOnMount('/devices', x => x.devices.read);
  useRefreshOnMount('/sensors', x => x.sensors.read);
  return h(ErrorBoundary, {}, [
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
  ]);
}

render(h(App), document.getElementById('app'));
