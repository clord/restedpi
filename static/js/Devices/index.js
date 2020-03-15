import { lazy } from "/react/";
import { Route, Switch } from "/js/depend/wouter/";
import { h } from "/js/html.js";

const DevicesConfigured = lazy(() => import("./DevicesConfigured.js"));
const AvailableDevices = lazy(() => import("./AvailableDevices.js"));
const EditDevice = lazy(() => import("./EditDevice.js"));

export default function Devices(props) {
  return h(Switch, {}, [
    h(Route, { key: 0, path: "/devices/available" }, p =>
      h(AvailableDevices, p)
    ),
    h(Route, { key: 0, path: "/devices/:path" }, p => h(EditDevice, p)),
    h(Route, { key: 1, path: "/devices" }, p => h(DevicesConfigured, p))
  ]);
}
