import React from '/static/js/depend/react/react.js'
import ReactDOM from '/static/js/depend/react/react-dom.js'
import htm from '/static/js/depend/htm.js';

const html = htm.bind(React.createElement);
const render = ReactDOM.render

export { html, render }

