import {  useGet } from '/static/js/hooks/network.js'
import { html, render } from '/static/js/html.js'

export function Sensors(props) {
    const {response, error} = useGet(`/sensors`);

    if (response == null) {
        return null
    }

    return html`
        <div>
          	Sensors: ${response.result ? "true" : "false"}
        </div>
    `
}

export {Sensors as default}
