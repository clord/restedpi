import { lazy, useEffect } from "/react/";
import { h } from "/js/html.js";
import { Link } from "/js/depend/wouter/";
import { useAppStore } from "/js/hooks/useApp.js";

const AddEditMcp23017 = lazy(() => import("./Mcp23017.js"));
const AddEditMcp9808 = lazy(() => import("./Mcp9808.js"));
const AddEditBmp085 = lazy(() => import("./Bmp085.js"));

export default function EditDevice({ path }) {
  const deviceStatus = useAppStore(x => x.devices.configured[path]);
  const refreshDevices = useAppStore(x => x.devices.read);

  let component;

  if (deviceStatus == null) {
    return null;
  }
  const [device] = deviceStatus;

  switch (device.model.name) {
    case "BMP085": {
      component = h(AddEditBmp085, {
        key: device.model.name,
        name: path,
        device
      });
      break;
    }
    case "MCP23017": {
      component = h(AddEditMcp23017, {
        key: device.model.name,
        name: path,
        device
      });
      break;
    }
    case "MCP9808": {
      component = h(AddEditMcp9808, {
        key: device.model.name,
        name: path,
        device
      });
      break;
    }
  }

  return h("article", { className: "max-w-sm w-full lg:max-w-full" }, [
    h(
      "div",
      { key: 0, className: "flex mb-4 justify-between items-baseline" },
      [
        h("header", { key: 0, className: "mb-4" }, [
          h(
            "h1",
            { key: 0, className: "font-bold text-xl mb-2" },
            `Editing ‘${device.name}’`
          ),
          h(
            "p",
            { key: 1, className: "text-gray-700 text-base" },
            device.description
          ),
          h("div", { key: 2, className: " py-3" }, [
            h(
              Link,
              {
                to: "/devices",
                key: 0,
                onClick: refreshDevices,
                className: "font-bold py-2 text-sm"
              },
              "Back to Devices"
            )
          ])
        ]),
        [component]
      ]
    )
  ]);
}
