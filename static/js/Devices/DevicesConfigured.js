import { useCallback, useState, useEffect } from "/react/";
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

function ConfirmRemove({ onRemove, children }) {
  const [showConfirm, setShowConfirm] = useState(false);
  return h("span", { className: "relative" }, [
    h(
      "button",
      {
        className: "text-sm text-gray-500 py-1 px-3",
        key: "r",
        onClick: () => setShowConfirm(x => !x)
      },
      children
    ),
    ...(showConfirm
      ? [
          h(
            "aside",
            {
              key: "a",
              className:
                "flex flex-col items-end z-40 text-gray-500 top-1 right-0 absolute"
            },
            [
              h(
                "svg",
                {
                  key: "s",
                  className: "fill-current mr-5",
                  width: "14",
                  height: "6",
                  viewBox: "0 0 14 6",
                  xmlns: "http://www.w3.org/2000/svg"
                },
                h("path", { d: "M7 0l6.928 6H.072L7 0z" })
              ),
              h(
                "div",
                {
                  key: "b",
                  className: "bg-gray-500 text-white p-2 rounded-sm"
                },
                [
                  h("h2", { key: "h", className: "mb-4" }, "Are you sure?"),
                  h("div", { key: "d", className: "flex" }, [
                    h(
                      "button",
                      {
                        key: "c",
                        onClick: () => setShowConfirm(false),
                        className: "text-sm text-white py-1 px-3 mr-3"
                      },
                      "Don’t"
                    ),
                    h(
                      "button",
                      {
                        key: "r",
                        onClick: onRemove,
                        className:
                          "text-sm bg-red-400 hover:bg-red-700 text-white font-bold py-1 px-3 rounded"
                      },
                      "Remove"
                    )
                  ])
                ]
              )
            ]
          )
        ]
      : [])
  ]);
}

function ActionCol({ cell }) {
  const slug = cell.value;
  const getDevice = useAppStore(x => x.devices.get);
  const removeDevice = useAppStore(x => x.devices.remove);
  const handleRemove = useCallback(() => {
    removeDevice({ slug });
  }, [slug]);

  return h("div", { className: "relative" }, [
    h(
      ConfirmRemove,
      {
        key: "1",
        onRemove: handleRemove
      },
      "Remove"
    ),
    h(
      Link,
      {
        className:
          "text-sm bg-blue-400 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded",
        key: "2",
        to: `/devices/${slug}`,
        onClick: () => {
          getDevice({ slug });
        }
      },
      "Edit"
    )
  ]);
}

function ModelCol({ cell }) {
  const { value } = cell;
  if (typeof value === "string") {
    return value;
  }
  if (value == null) {
    return null;
  }
  return Object.keys(value)[0];
}

function AddDeviceButton({ cell }) {
  const readAvailable = useAppStore(x => x.devices.readAvailable);
  return h(
    Link,
    {
      className:
        "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
      to: "/devices/available",
      onClick: readAvailable
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
        accessor: "device.model.name",
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
