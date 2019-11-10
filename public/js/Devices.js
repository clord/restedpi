import { useGet } from './util.js'
import { html, render } from './html.js'

export function Devices(props) {
    const {response, error} = useGet(`/api/devices`);

    if (response == null) {
        return null
    }

    return html`
        <div>
          	Devices: ${response.result ? "true" : "false"}
        </div>
    `
}


