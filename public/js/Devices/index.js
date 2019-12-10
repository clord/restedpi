import { useGet } from '/static/js/hooks/network.js'
import { h } from '/static/js/html.js'
import { Route, Switch } from '/static/js/depend/wouter/index.js'

import { Device } from './Device.js'
import { AddDevice } from './AddDevice.js'

function DevicesList(props) {
    const {response, error} = useGet('/devices/available');

    if (response == null) {
        return null
    }

    return h("div", {className: "devices"},
        response.result.map(device => h(Device, device)))
}

function ConfiguredDevice(props) {
    return "Configured Device"
}

function DevicesConfiguredList(props) {
    const {response, error} = useGet('/devices/available');

    if (response == null) {
        return null
    }

    return h("div", {}, [
        response.result.map(device => h(ConfiguredDevice, device))
    ])
}

export function Devices(props) {
    return (
        h(Switch, {}, [
            h(Route, {path: "/devices/add/:item"}, p => h(AddDevice, p)),
            h(Route, {path: "/devices/available", component: DevicesList}),
            h(Route, {path: "/devices"}, p => h(DevicesConfiguredList, p)),
        ])
    )
}


