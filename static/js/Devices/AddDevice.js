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

      const params = {
        [form.name]: {
          description: "foo TODO",
          address: Number(form.address),
          model: { BMP085: { mode: form.resolution } }
        }
      };
      addDevice(params).finally(() => setSubmitting(false));
    },
    [addDevice]
  );

  return h(Form, { onSubmit: handleSubmit }, [
    h(Text, {
      id: "name",
      label: "Sensor Name",
      required: "Required",
      pattern: {
        value: /^[ \w]+$/,
        message: "Name must be alphanumeric"
      }
    }),
    h(Text, {
      id: "address",
      label: "I2C Bus Address",
      required: "Required",
      pattern: {
        value: /^\d+$/,
        message: "Address must be decimal number"
      }
    }),
    h(Select, { name: "resolution", label: "Resolution" }, [
      h(
        Select.Choice,
        { value: "UltraHighRes" },
        "Ultra High Resolution (slow and power-hungry)"
      ),
      h(Select.Choice, { value: "HighRes" }, "High Resolution (slow)"),
      h(
        Select.Choice,
        { value: "Standard", selected: true },
        "Standard Resolution"
      ),
      h(Select.Choice, { value: "UltraLowPower" }, "Low Power, Low Resolution")
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
