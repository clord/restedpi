import {  useGet } from '/static/js/hooks/network.js'
import { html, render } from '/static/js/html.js'

export function Switches(props) {
    const {response, error} = useGet(`/switches`);

    if (response == null) {
        return null
    }

    return html`
        <div>
          	Switches: ${response.result ? "true" : "false"}
        </div>
    `
}

export {Switches as default}
