import { useCallback } from '/static/js/depend/react/react.js'
import { useGet } from '/static/js/hooks/network.js'
import { html, h, render } from '/static/js/html.js'
import { Route, Link, Switch , useLocation } from '/static/js/depend/wouter/index.js'

function DtDd({dt, dd, dds}) {
    if (dds != null) {
        return [h("dt", {}, dt), dds.map(d => h("dd", {}, d))]
    }
    return [h("dt", {}, dt), h("dd", {}, dd)]
}

function SensorsOfDevice({sensors}) {
    if (sensors.length === 0) {
        return null
    }

    return [h(DtDd, {dt: "Sensors", dds: sensors.map(sen => [
        sen.type,
        ...(sen.range == null ? [] : [h("br"), h("small", {}, sen.range)])
    ])})]
}


function SwitchesOfDevice({switches}) {
    if (switches.length === 0) {
        return null
    }
    return h(DtDd, {dt: "Switches", dd: switches.length.toString() + ' switches'})
}

function Device({name, description, sensors, switches, datasheet, bus}) {
    return h("article", {className: "device"}, [
        h("header", {}, [
            h("h1", {}, name),
            h("p", {}, description)
        ]),
        h("dl", {}, [
            h(DtDd, {dt: "Transport", dd: bus}, []),
            datasheet == null ? null : h(DtDd, {dt: "Datasheet", dd:
                h('a', {target: "_blank", href: datasheet}, [
                    h('i', {className: "fas fa-link"}),
                    " datasheet"
                ])
            }, []),
            h(SensorsOfDevice,  {sensors:  sensors  || []}, []),
            h(SwitchesOfDevice, {switches: switches || []}, []),
        ]),
        h(Link, {href: '/devices/add/' + name}, "Add Device")
    ])
}

function DevicesList(props) {
    const {response, error} = useGet(`/devices`);

    if (response == null) {
        return null
    }
    return h("div", {className: "devices"},
        response.result.map(device => h(Device, device)))
}

function AddDevice(props) {
    return h("div", {className: "devices"}, h("h1", {}, "Add Device: " + props.item))
}

export function Devices(props) {
    const [location] = useLocation()
    console.log(location)
    return (
        h(Switch, {}, [
            h(Route, {path: "/devices/add/:item"}, p => h(AddDevice, p)),
            h(Route, {path: "/devices", component: DevicesList})
        ])
    )
}


