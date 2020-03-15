import { lazy, useEffect } from "/react/";
import { h } from "/js/html.js";
import { Link } from "/js/depend/wouter/";
import { useAppStore } from "/js/hooks/useApp.js";

const AddEditMcp23017 = lazy(() => import("./Mcp23017.js"));
const AddEditMcp9808 = lazy(() => import("./Mcp9808.js"));
const AddEditBmp085 = lazy(() => import("./Bmp085.js"));

export default function EditDevice({ path }) {
  const get = useAppStore(x => x.devices.get(path));
  useEffect(() => {
    get();
  }, [get]);

  let component;

  // switch (name) {
  //   case "MCP23017": {
  //     component = h(AddEditMcp23017, { key: name });
  //     break;
  //   }
  //   case "MCP9808": {
  //     component = h(AddEditMcp9808, { key: name });
  //     break;
  //   }
  //   case "BMP085": {
  //     component = h(AddEditBmp085, { key: name });
  //     break;
  //   }
  // }

  return h("article", { className: "max-w-sm w-full lg:max-w-full" }, [
    h(
      "div",
      { key: 0, className: "flex mb-4 justify-between items-baseline" },
      [
        h("header", { key: 0, className: "mb-4" }, [
          // h("h1", { key: 0, className: "font-bold text-xl mb-2" }, name),
        ]),
        [component]
      ]
    ),
    h("div", { key: 1, className: "mx-auto px-3 py-3" }, [
      h(
        Link,
        {
          to: "/devices",
          key: 0,
          className: "font-bold py-2 px-4 text-sm"
        },
        "Back to Devices"
      )
    ])
  ]);
}
