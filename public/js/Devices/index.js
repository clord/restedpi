import { useCallback } from '/static/js/depend/react/react.js'
import { useGet } from '/static/js/hooks/network.js'
import { h } from '/static/js/html.js'
import { Route, Switch } from '/static/js/depend/wouter/index.js'
import { Device } from './Device.js'
import { AddDevice } from './AddDevice.js'

function DevicesList(props) {
    const {response, error} = useGet(`/devices`);

    if (response == null) {
        return null
    }
    return h("div", {className: "devices"},
        response.result.map(device => h(Device, device)))
}

export function Devices(props) {
    return (
        h(Switch, {}, [
            h(Route, {path: "/devices/add/:item"}, p => h(AddDevice, p)),
            h(Route, {path: "/devices", component: DevicesList})
        ])
    )
}


