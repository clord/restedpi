import { lazy } from '/react/';
import { h } from '/js/html.js';

const AddEditMcp23017 = lazy(() => import('./Mcp23017.js'));
const AddEditMcp9808 = lazy(() => import('./Mcp9808.js'));
const AddEditBmp085 = lazy(() => import('./Bmp085.js'));

export default function AddDevice({ name, description }) {
  let component;

  switch (name) {
    case 'MCP23017': {
      component = h(AddEditMcp23017, { key: name });
      break;
    }
    case 'MCP9808': {
      component = h(AddEditMcp9808, { key: name });
      break;
    }
    case 'BMP085': {
      component = h(AddEditBmp085, { key: name });
      break;
    }
  }

  return [
    h('div', { key: 0, className: '' }, [
      h('header', { key: 0, className: 'mb-4' }, [
        h('h1', { key: 0, className: 'font-bold text-xl mb-2' }, name),
        h('p', { key: 1, className: 'text-gray-700 text-base' }, description),
      ]),
      [component],
    ]),
  ];
}
