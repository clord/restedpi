import {useCallback, useEffect} from '/js/depend/react/'
import {Route, Switch} from '/js/depend/wouter/'
import {useGet} from '/js/hooks/network.js'
import {h} from '/js/html.js'
import {useAppStore} from '/js/hooks/useApp.js'

import {Device} from './Device.js'
import {AddDevice} from './AddDevice.js'
import {DevicesConfigured} from './ConfiguredTable.js'

function DevicesList(props) {
    const {response, error} = useGet('/devices/available');

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
            h(Route, {path: "/devices/available", component: DevicesList}),
            h(Route, {path: "/devices"}, p => h(DevicesConfigured, p)),
        ])
    )
}

export { Devices as default }

