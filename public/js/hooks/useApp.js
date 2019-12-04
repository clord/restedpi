import create from '/static/js/depend/zustand.js'
import {apiGet} from '/static/js/hooks/network.js'

const [useStore, api] = create(set => ({
    server: null,
    setup: async () => {
        const response = await apiGet(`/about`)
        set({server: response.server})
    }
}))

export { useStore as useApp, api }
