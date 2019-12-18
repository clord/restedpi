import {useCallback, useEffect} from '/static/js/depend/react/react.js'
import { useGet } from '/static/js/hooks/network.js'
import { h } from '/static/js/html.js'
import { Route, Switch } from '/static/js/depend/wouter/index.js'
import { useAppStore } from '/static/js/hooks/useApp.js'

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

function DeviceStatus(props) {
    switch (props.status) {
    case 'connected': return h("div", {}, "Connected")
    case 'disconnected': return h("div", {}, "Disconnected")
    }
    return null
}

function ConfiguredDevice({device, url}) {
    const handleRemove = useCallback(() => {
        console.log("handle remove ", url)
    }, [url])

    const handleEdit = useCallback(() => {
        console.log("handle edit ", url)
    }, [url])

    return h("div", {className: "configured-device"},
        [ h("h1", {}, device.name),
          h(DeviceStatus, {status: device.status}),
          h("p", {}, device.description),
          h("button", {onClick: handleRemove}, "Remove"),
          h("button", {onClick: handleEdit}, "Edit")
        ]
    )
}

function DevicesConfiguredList(props) {
    useEffect(useAppStore(x => x.devices.read), [])
    const configured = useAppStore(x => x.devices.configured)

    return (
        h("article", {className: "configured-devices"}, [
            h("header", {}, [
                h("a", {href: "/devices/available"}, "Add")
            ]),
            h("div", {},
                [...configured.entries()].map(([key, device]) =>
                    h(ConfiguredDevice, {device, key, url: key}))
            )
        ])
    )
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

export { Devices as default }

