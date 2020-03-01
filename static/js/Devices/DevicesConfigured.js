import { useCallback, useEffect } from "/js/depend/react/";
import { h } from "/js/html.js";
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

  const handleEdit = useCallback(() => {
    console.log("handle edit ", slug);
  }, [slug]);

  return [
    h(
      "button",
      { className: "text-sm text-gray-500 py-1 px-3", onClick: handleRemove },
      "Remove"
    ),
    h(
      "button",
      {
        className:
          "text-sm bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded",
        onClick: handleEdit
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

function AddDevice({ cell }) {
  return h(
    "a",
    {
      className:
        "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
      href: "/devices/available"
    },
    "Add Device"
  );
}

function DevicesConfiguredTable({ data }) {
  return h(Table, {
    columns: [
      { Header: "Name", accessor: "device.name" },
      {
        Header: "Status",
        accessor: "status",
        Cell: DeviceStatus,
        style: { textAlign: "center" }
      },
      { Header: "Model", accessor: "device.model", Cell: ModelCol },
      { Header: "Description", accessor: "device.description" },
      {
        accessor: "slug",
        Cell: ActionCol,
        style: { textAlign: "right" }
      }
    ],
    data
  });
}

export function DevicesConfigured() {
  useEffect(
    useAppStore(x => x.devices.read),
    []
  );
  const configured = useAppStore(x => x.devices.configured);
  const data = Object.keys(configured).map(slug => ({
    device: configured[slug][0],
    status: configured[slug][1],
    slug
  }));

  return h("article", { className: "max-w-sm w-full lg:max-w-full" }, [
    h("div", { className: "flex mb-4 justify-between items-baseline" }, [
      h(
        "h1",
        { className: "text-gray-900 font-bold text-xl mb-3" },
        "Configured Devices"
      ),
      h(AddDevice, {})
    ]),
    h(DevicesConfiguredTable, { data })
  ]);
}
