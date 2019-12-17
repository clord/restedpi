import {useEffect} from '/static/js/depend/react/react.js'
import produce from '/static/js/depend/immer.module.js'
import create from '/static/js/depend/zustand.js'
import {apiGet} from '/static/js/hooks/network.js'

const [useAppStore, api] = create(set => {
    const s = fn => set(produce(fn))
    return ({
        serverConfig: {},
        setup: async () => {
            const response = await apiGet(`/config`)
            s(state => void(state.serverConfig = response.serverConfig))
        },
        devices: {
            configured: new Map([]),
            setup: async () => {
                const response = await apiGet(`/devices/configured`)
                const m = new Map(response)
                s(state => void(state.devices.configured = m))
            },
            remove: async (source) => {
                const response = await apiDelete(source)
                s(state => void(state.devices.configured.delete(source)))
            }
        }
    })
})

export { useAppStore, api }

export function useApp() {
    useEffect(useAppStore(x => x.setup), [useAppStore])
}


