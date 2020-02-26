import { useEffect } from "/js/depend/react/";
import produce from "/js/depend/immer.module.js";
import create from "/js/depend/zustand.js";
import { apiGet, apiPost } from "/js/hooks/network.js";

const [useAppStore, api] = create(set => {
  const s = fn => set(produce(fn));
  return {
    serverConfig: {},
    setup: async () => {
      const response = await apiGet(`/config`);
      s(state => void (state.serverConfig = response.serverConfig));
    },
    devices: {
      configured: [],
      read: async () => {
        const response = await apiGet(`/devices/configured`);
        s(state => void (state.devices.configured = response));
      },
      add: async details => {
        const response = await apiPost("/devices/configured", details);
        s(state => void (state.devices.configured = response));
      },
      edit: async (source, details) => {
        s(state => void state.devices.configured.set(source, details));
        const response = await apiPut(source, details);
        s(state => void state.devices.configured.set(source, response));
      },
      remove: async source => {
        const response = await apiDelete(source);
        s(state => void state.devices.configured.delete(source));
      }
    }
  };
});

export { useAppStore, api };

export function useApp() {
  useEffect(
    useAppStore(x => x.setup),
    [useAppStore]
  );
}
