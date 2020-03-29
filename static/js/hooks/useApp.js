import { useEffect } from "/react/";
import produce from "/js/depend/immer.module.js";
import create from "/js/depend/zustand.js";
import { apiGet, apiDelete, apiPut, apiPost } from "/js/hooks/network.js";

const [useAppStore, api] = create(set => {
  const s = fn => set(produce(fn));
  return {
    serverConfig: {},
    setup: async () => {
      const response = await apiGet(`/config`);
      s(state => void (state.serverConfig = response.serverConfig));
    },
    sensors: {
      all: {},
      read: async () => {
        const response = await apiGet(`/sensors`);
        s(state => void (state.sensors.all = response));
      }
    },
    switches: {
      all: {},
      read: async () => {
        const response = await apiGet(`/switches`);
        s(state => void (state.switches.all = response));
      }
    },
    devices: {
      available: [],
      configured: {},
      readAvailable: async () => {
        const response = await apiGet(`/devices/available`);
        s(state => void (state.devices.available = response));
      },
      read: async () => {
        const response = await apiGet(`/devices/configured`);
        s(state => void (state.devices.configured = response));
      },
      add: async details => {
        const response = await apiPost("/devices/configured", details);
        s(state => void (state.devices.configured = response));
      },
      get: async ({ slug }) => {
        const response = await apiGet(`/devices/configured/${slug}`);
        s(state => void (state.devices.configured[slug] = response));
      },
      edit: async ({ slug }, details) => {
        const response = await apiPut(`/devices/configured/${slug}`, details);
        s(state => void (state.devices.configured[slug] = response));
        const all = await apiGet(`/devices/configured`);
        s(state => void (state.devices.configured = all));
      },
      remove: async ({ slug }) => {
        const response = await apiDelete(`/devices/configured/${slug}`);
        s(state => void (state.devices.configured = response));
      }
    }
  };
});

export { useAppStore, api };

export function useApp() {
  const setup = useAppStore(x => x.setup);
  useEffect(() => {
    setup();
  }, [useAppStore]);
}
