import React from "/react/";
import ReactDOM from "/react/react-dom.js";
import htm from "/js/depend/htm.js";

export const html = htm.bind(React.createElement);
export const render = ReactDOM.render;
export const h = React.createElement;
