import { h } from '/static/js/html.js'

export function AddDevice(props) {
    return h("div", {className: "devices"}, h("h1", {}, "Add Device: " + props.item))
}
