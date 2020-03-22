import { useGet } from "/js/hooks/network.js";
import { h } from "/js/html.js";
import { Link } from "/js/depend/wouter/";
import Device from "./Device.js";

export default function AvailableDevices(props) {
  const { response, error } = useGet("/devices/available");

  if (response == null) {
    return null;
  }

  return h("main", {}, [
    h(
      "div",
      { key: 0, className: "flex mb-4 justify-between items-baseline" },
      [
        h(
          "h1",
          { key: 0, className: "text-gray-900 font-bold text-xl mb-3" },
          "Available Devices"
        ),
        h(
          Link,
          { key: 1, className: "font-bold py-2 px-4", to: "/devices" },
          "Back"
        )
      ]
    ),
    h(
      "div",
      {
        key: 1,
        className: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4"
      },
      response.result.map(device => h(Device, { key: device.name, ...device }))
    )
  ]);
}
