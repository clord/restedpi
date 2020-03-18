import { useCallback, useState } from "/react/";
import { useAppStore } from "/js/hooks/useApp.js";
import { useLocation } from "/js/depend/wouter/";
import { Form, Text, Submit } from "/js/Forms/Form.js";
import { h } from "/js/html.js";

export default function AddEditMcp23017(props) {
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
      key: "name",
      label: "Device name",
      required: "Required"
    }),
    h(Text, {
      id: "description",
      key: "description",
      label: "Description"
    }),
    h(Text, {
      id: "address",
      key: "address",
      label: "I2C Bus Address",
      required: "Required",
      pattern: {
        value: /^\d+$/,
        message: "Address must be decimal number"
      }
    }),
    h(Submit, { key: "sub", submitting }, "Create")
  ]);
}
