import { h } from "/js/html.js";
import { useAppStore } from "/js/hooks/useApp.js";
import { useCallback, useEffect } from "/react/";
import { Table } from "/js/Table/";

function UnitValue({ row }) {
  const { unit, value } = row.original;
  switch (unit) {
    case "DegC":
      return value.toFixed(1) + "℃";
    case "KPa":
      return value.toFixed(1) + " kPa";
    case "Boolean":
      return value == 1 ? "On" : "Off";
  }
  return value;
}

function AllSensorsTable({ data }) {
  return h(Table, {
    columns: [
      { Header: "Name", accessor: "name" },
      {
        Header: "Status",
        accessor: "status"
      },
      {
        Header: "Sensor Value",
        className: "text-right",
        Cell: UnitValue
      }
    ],
    data
  });
}

export function Sensors(props) {
  const sensors = useAppStore(x => x.sensors.all);
  const data = Object.keys(sensors).flatMap(name =>
    sensors[name].map(x => ({ ...x, name }))
  );

  return h("article", { className: "max-w-sm w-full lg:max-w-full" }, [
    h(
      "div",
      { className: "flex mb-4 justify-between items-baseline", key: 0 },
      [
        h(
          "h1",
          { className: "text-gray-900 font-bold text-xl mb-3", key: 0 },
          "All Sensor Values"
        )
      ]
    ),
    h(AllSensorsTable, { data, key: 1 })
  ]);
}

export { Sensors as default };
