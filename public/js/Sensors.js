import {  useGet } from './util.js'
import { html, render } from './html.js'

export function Sensors(props) {
    // GET /api/sensors
    const {response, error} = useGet(`/api/sensors`);

    if (response == null) {
        return null
    }

    return html`
        <div>
          	Sensors: ${response.result ? "true" : "false"}
        </div>
    `
}

