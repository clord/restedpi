import React from '/static/js/depend/react/react.js'
import ReactDOM from '/static/js/depend/react/react-dom.js'
import htm from '/static/js/depend/htm.js';

export const html = htm.bind(React.createElement);
export const render = ReactDOM.render
export const h = React.createElement

