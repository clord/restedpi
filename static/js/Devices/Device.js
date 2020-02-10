import { h } from '/js/html.js'
import { Link } from '/js/depend/wouter/'

function DtDd({dt, dd, dds}) {
    if (dds != null) {
        return [h("dt", {className: "font-bold col-start-1"}, dt), dds.map(d => h("dd", {className: "col-start-2 col-span-2"}, d))]
    }
    return [h("dt", {className: "font-bold col-start-1"}, dt), h("dd", {className:"col-start-2 col-span-2"}, dd)]
}

function SensorsOfDevice({sensors}) {
    if (sensors.length === 0) {
        return null
    }

    return [
        h(DtDd, {
            dt: "Sensors",
            dds: sensors.map(sen => [
                sen.type,
                ...(sen.range == null ? [] : [h("br"), h("small", {}, sen.range)])
            ])
        })
    ]
}

function SwitchesOfDevice({switches}) {
    if (switches.length === 0) {
        return null
    }
    return h(DtDd, {
        dt: "Switches",
        dd: switches.length.toString() + ' switches'
    })
}

export function Device({name, description, sensors, switches, datasheet, bus}) {
    return h("article", {className: "max-w-sm rounded overflow-hidden shadow-lg border flex flex-col justify-between px-6 py-4"}, [
        h("div", {}, [
            h("header", {className: ""}, [
                h("h1", {className: "font-bold text-xl mb-2"}, name),
                h("p", {className: "text-gray-700 text-base"}, description)
            ]),
            h("dl", {className: "grid grid-cols-3 gap-2 py-4"}, [
                h(DtDd, {dt: "Bus", dd: bus}, []),
                datasheet == null ? null : h(DtDd, {dt: "Links", dd:
                    h('a', {target: "_blank", href: datasheet, className: " underline text-blue-400"}, [
                        h('i', {className: "fas fa-link text-xs"}),
                        " datasheet"
                    ])
                }, []),
                h(SensorsOfDevice,  {sensors:  sensors  || []}, []),
                h(SwitchesOfDevice, {switches: switches || []}, []),
            ])]
        ),
        h("div", {className: "mx-auto px-3 py-3"}, [h(Link, {href: '/devices/add/' + name, className: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"}, "Add Device")])
    ])
}

