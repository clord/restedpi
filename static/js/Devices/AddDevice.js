import { useCallback } from "/js/depend/react/";
import { h } from "/js/html.js";
import { Form, Text, Label, Select } from "/js/Forms/Form.js";

function AddBmp085(props) {
  const onSubmit = useCallback(values => {
    console.log(values);
  }, []);

  return h(Form, { onSubmit: onSubmit }, [
    h(Text, {
      id: "address",
      label: "Bus Address",
      required: "Required",
      pattern: {
        value: /^\d+$/,
        message: "decimal address required"
      }
    }),
    h(Select, { name: "resolution", label: "Resolution" }, [
      h(Select.Choice, { value: "high" }, "High Accuracy (slow)"),
      h(Select.Choice, { value: "med", selected: true }, "Medium Accuracy"),
      h(Select.Choice, { value: "low" }, "Low (Fast sample rate)")
    ]),
    h(
      "button",
      {
        type: "submit",
        className:
          "mx-auto mt-4 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
      },
      "Create"
    )
  ]);
}

export function AddDevice({
  name,
  description,
  onHideAddDevice: handleHideDevice
}) {
  let component;

  switch (name) {
    case "BMP085": {
      component = h(AddBmp085, { name });
      break;
    }
  }

  return [
    h("div", { className: "" }, [
      h("header", { className: "mb-4" }, [
        h("h1", { className: "font-bold text-xl mb-2" }, name),
        h("p", { className: "text-gray-700 text-base" }, description)
      ]),
      component
    ]),

    h("div", { className: "mx-auto px-3 py-3" }, [
      h(
        "button",
        {
          onClick: handleHideDevice,
          className: "font-bold py-2 px-4 text-sm"
        },
        "Back"
      )
    ])
  ];
}
