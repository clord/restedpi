import { useCallback, useEffect } from "/react/";
import { h } from "/js/html.js";
import { Link } from "/js/depend/wouter/";
import { useAppStore } from "/js/hooks/useApp.js";
import { Table } from "/js/Table/";

function DeviceStatus({ cell }) {
  switch (cell.value) {
    case "Ok":
      return h("span", {}, "OK");
  }
  return null;
}

function ActionCol({ cell }) {
  const slug = cell.value;
  const removeDevice = useAppStore(x => x.devices.remove);
  const handleRemove = useCallback(() => {
    removeDevice(slug);
  }, [slug]);

  return [
    h(
      "button",
      {
        className: "text-sm text-gray-500 py-1 px-3",
        key: "1",
        onClick: handleRemove
      },
      "Remove"
    ),
    h(
      Link,
      {
        className:
          "text-sm bg-blue-400 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded",
        key: "2",
        to: `/devices/${slug}`
      },
      "Edit"
    )
  ];
}

function ModelCol({ cell }) {
  const { value } = cell;
  if (typeof value === "string") {
    return value;
  }
  return Object.keys(value)[0];
}

function AddDeviceButton({ cell }) {
  return h(
    Link,
    {
      className:
        "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
      to: "/devices/available"
    },
    "Add Device"
  );
}

function DevicesConfiguredTable({ data }) {
  return h(Table, {
    columns: [
      { Header: "Name", accessor: "device.name", className: "text-right" },
      {
        Header: "Status",
        accessor: "status",
        Cell: DeviceStatus,
        className: "text-right"
      },
      {
        Header: "Model",
        accessor: "device.model",
        Cell: ModelCol,
        className: "text-right"
      },
      {
        Header: "Description",
        accessor: "device.description",
        className: "text-left"
      },
      {
        accessor: "slug",
        Cell: ActionCol,
        className: "text-right"
      }
    ],
    data
  });
}

export default function DevicesConfigured() {
  const read = useAppStore(x => x.devices.read);
  useEffect(() => {
    read();
  }, []);
  const configured = useAppStore(x => x.devices.configured);
  const data = Object.keys(configured).map(slug => ({
    device: configured[slug][0],
    status: configured[slug][1],
    key: slug,
    slug
  }));

  return h("article", { className: "max-w-sm w-full lg:max-w-full" }, [
    h(
      "div",
      { className: "flex mb-4 justify-between items-baseline", key: 0 },
      [
        h(
          "h1",
          { className: "text-gray-900 font-bold text-xl mb-3", key: 0 },
          "Configured Devices"
        ),
        h(AddDeviceButton, { key: 1 })
      ]
    ),
    h(DevicesConfiguredTable, { data, key: 1 })
  ]);
}
