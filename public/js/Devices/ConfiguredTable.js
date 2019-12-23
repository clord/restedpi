import {useCallback, useEffect} from '/static/js/depend/react/react.js'
import {h} from '/static/js/html.js'
import {useAppStore} from '/static/js/hooks/useApp.js'
import {Table} from '/static/js/Table/index.js'

function DeviceStatus({cell}) {
    switch (cell.value) {
    case 'ok': return h("span", {}, "OK")
    case 'connected': return h("span", {}, "Connected")
    case 'disconnected': return h("span", {}, "Disconnected")
    }
    return null
}

function ActionCol({cell}) {
    const url = cell.value
    const handleRemove = useCallback(() => {
        console.log("handle remove ", url)
    }, [url])

    const handleEdit = useCallback(() => {
        console.log("handle edit ", url)
    }, [url])

    return [
        h("button", {className: 'tertiary', onClick: handleRemove}, "Remove"),
        h("button", {className: 'secondary', onClick: handleEdit}, "Edit"),
    ]
}

function AddDevice({cell}) {
    const handleAdd = useCallback(() => {
        console.log("handle add ")
    }, [])
    return h("button", {className: 'primary', onClick: handleAdd}, "Add Device")
}

function DevicesConfiguredTable({data}) {
    return h(Table, {
        columns: [
            { Header: 'Name',
              accessor: 'device.name'
            },
            { Header: 'Status',
              accessor: 'device.status',
              Cell: DeviceStatus,
              style: {textAlign: 'center'}
            },
            { Header: 'Description',
              accessor: 'device.description'
            },
            {
              accessor: 'url',
              Cell: ActionCol,
              style: {textAlign:"right"}
            },
        ],
        data
    })
}


export function DevicesConfigured() {
    useEffect(useAppStore(x => x.devices.read), [])
    const configured = useAppStore(x => x.devices.configured)
    const data = [...configured].map(([key, device]) => ({device, key, url: key}))

    return h('article', {className: "part"}, [
        h("h1", {}, "Configured Devices"),
        h(AddDevice, {}),
        h(DevicesConfiguredTable, {data}),
    ])

}
