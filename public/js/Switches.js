import { useGet } from './util.js'
import { html, render } from './html.js'

export function Switches(props) {
    const {response, error} = useGet(`/api/switches`);

    if (response == null) {
        return null
    }

    return html`
        <div>
          	Switches: ${response.result ? "true" : "false"}
        </div>
    `
}

