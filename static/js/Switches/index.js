import { h } from "/js/html.js";
import { useAppStore } from "/js/hooks/useApp.js";
import { useCallback, useEffect } from "/react/";
import { Table } from "/js/Table/";

function AllSwitchesTable({ data }) {
  return h(Table, {
    columns: [
      { Header: "Name", accessor: "name" },
      {
        Header: "Toggle",
        accessor: "value"
      }
    ],
    data
  });
}

export function Switches(props) {
  const switches = useAppStore(x => x.switches.all);
  const data = Object.keys(switches).flatMap(name =>
    switches[name].map(x => ({ ...x, name }))
  );

  return h("article", { className: "max-w-sm w-full lg:max-w-full" }, [
    h(
      "div",
      { className: "flex mb-4 justify-between items-baseline", key: 0 },
      [
        h(
          "h1",
          { className: "text-gray-900 font-bold text-xl mb-3", key: 0 },
          "All Switches"
        )
      ]
    ),
    h(AllSwitchesTable, { data, key: 1 })
  ]);
}

export { Switches as default };
