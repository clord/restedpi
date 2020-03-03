import { useGet } from "/js/hooks/network.js";
import { h } from "/js/html.js";
import Device from "./Device.js";

export default function AvailableDevices(props) {
  const { response, error } = useGet("/devices/available");

  if (response == null) {
    return null;
  }

  return h(
    "div",
    { className: "grid grid-flow-col grid-cols-3 gap-4" },
    response.result.map(device => h(Device, { key: device.name, ...device }))
  );
}
