import { useCallback, useState } from "/js/depend/react/";
import { h } from "/js/html.js";
import { Form, Text, Submit, Select } from "/js/Forms/Form.js";
import { useLocation } from "/js/depend/wouter/";
import { useAppStore } from "/js/hooks/useApp.js";

function AddMcp23017(props) {
  const [submitting, setSubmitting] = useState(false);
  const addDevice = useAppStore(x => x.devices.add);
  const [location, setLocation] = useLocation();
  const handleSubmit = useCallback(
    form => {
      setSubmitting(true);

      const params = {
        model: { MCP23017: { bank0: {}, bank1: {} } },
        description: form.description,
        name: form.name,
        address: Number(form.address)
      };
      addDevice(params)
        .then(result => {
          setLocation("/devices");
        })
        .finally(() => setSubmitting(false));
    },
    [addDevice]
  );

  return h(Form, { onSubmit: handleSubmit }, [
    h(Text, {
      id: "name",
      label: "Sensor Name",
      required: "Required"
    }),
    h(Text, {
      id: "description",
      label: "Description"
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
    h(Submit, { submitting }, "Create")
  ]);
}

function AddMcp9808(props) {
  const [submitting, setSubmitting] = useState(false);
  const addDevice = useAppStore(x => x.devices.add);
  const [location, setLocation] = useLocation();
  const handleSubmit = useCallback(
    form => {
      setSubmitting(true);

      const params = {
        model: "MCP9808",
        description: form.description,
        name: form.name,
        address: Number(form.address)
      };
      addDevice(params)
        .then(result => {
          setLocation("/devices");
        })
        .finally(() => setSubmitting(false));
    },
    [addDevice]
  );

  return h(Form, { onSubmit: handleSubmit }, [
    h(Text, {
      id: "name",
      label: "Sensor Name",
      required: "Required"
    }),
    h(Text, {
      id: "description",
      label: "Description"
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
    h(Submit, { submitting }, "Create")
  ]);
}

function AddBmp085(props) {
  const [submitting, setSubmitting] = useState(false);
  const addDevice = useAppStore(x => x.devices.add);
  const [location, setLocation] = useLocation();
  const handleSubmit = useCallback(
    form => {
      setSubmitting(true);

      const params = {
        model: { BMP085: { mode: form.resolution } },
        description: form.description,
        name: form.name,
        address: Number(form.address)
      };
      addDevice(params)
        .then(result => {
          setLocation("/devices");
        })
        .finally(() => setSubmitting(false));
    },
    [addDevice]
  );

  return h(Form, { onSubmit: handleSubmit }, [
    h(Text, {
      id: "name",
      label: "Sensor Name",
      required: "Required"
    }),
    h(Text, {
      id: "description",
      label: "Description"
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
    case "MCP23017": {
      component = h(AddMcp23017, {});
      break;
    }
    case "MCP9808": {
      component = h(AddMcp9808, {});
      break;
    }
    case "BMP085": {
      component = h(AddBmp085, {});
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
