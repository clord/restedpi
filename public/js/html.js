import { h, render } from './depend/preact.js';
import htm from './depend/htm.js';

const html = htm.bind(h);

export { html, render }

