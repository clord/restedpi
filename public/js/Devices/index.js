import { useGet } from '/static/js/hooks/network.js'
import { useCallback } from '/static/js/depend/preact.hooks.js'
import { html, render } from '/static/js/html.js'

function SensorsOfDevice({sensors}) {
    if (sensors.length === 0) {
        return null
    }
    return html`
        <dt>Sensors</dt>
        ${sensors.map(sen =>
            html`<dd>
                ${sen.type}
                ${sen.range == null ? null :
                    html`<br/><small>${sen.range}</small>`}
            </dd>`
        )}`
}

function SwitchesOfDevice({switches}) {
    if (switches.length === 0) {
        return null
    }
    return html`
        <dt>Switches</dt>
        <dd>${switches.length} switches</dd>
        `
}

function Device({name, description, sensors, switches, datasheet, bus}) {
    const handleClick = useCallback((e) => {
        console.log("did click", e)
    }, [])
    return html`
        <div class="device">
            <header>
                <h1>${name}</h1>
                <p>
                ${description}
                </p>
            </header>
            <dl>
                <dt>Transport</dt><dd>${bus}</dd>
                ${datasheet == null ? null :
                    html`<dt>Data sheet</dt>
                        <dd>
                            <a target="_blank" href=${datasheet}>
                                <i class="fas fa-link"></i> datasheet
                            </a>
                        </dd>`}
                <${SensorsOfDevice} sensors=${sensors || []} />
                <${SwitchesOfDevice} switches=${switches || []} />
            </dl>
            <button onClick=${handleClick}>Add Device</button>
        </div>
    `
}

export function Devices(props) {
    const {response, error} = useGet(`/devices`);

    if (response == null) {
        return null
    }

    return html`
        <div class="devices">
            ${response.result.map(device =>
                html`<${Device} ...${device} />`)}
        </div>
    `
}


