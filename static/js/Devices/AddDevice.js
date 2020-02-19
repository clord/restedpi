import { useCallback, useState } from "/js/depend/react/";
import { h } from "/js/html.js";
import { Form, Text, Submit, Select } from "/js/Forms/Form.js";
import { useAppStore } from "/js/hooks/useApp.js";

function AddBmp085(props) {
  const [submitting, setSubmitting] = useState(false);
  const addDevice = useAppStore(x => x.devices.add);
  const handleSubmit = useCallback(
    form => {
      setSubmitting(true);
      addDevice(form).finally(() => setSubmitting(false));
    },
    [addDevice]
  );

  return h(Form, { onSubmit: handleSubmit }, [
    h(Text, {
      id: "address",
      label: "Bus Address",
      required: "Required",
      pattern: {
        value: /^\d+$/,
        message: "decimal i2c address required"
      }
    }),
    h(Select, { name: "resolution", label: "Resolution" }, [
      h(Select.Choice, { value: "high" }, "High Resolution (slow)"),
      h(Select.Choice, { value: "med", selected: true }, "Medium Resolution"),
      h(Select.Choice, { value: "low" }, "Low Resolution (fast)")
    ]),
    h(Submit, { submitting }, "Create")
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
