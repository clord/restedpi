import { useCallback, useState } from "/react/";
import { useAppStore } from "/js/hooks/useApp.js";
import { useLocation } from "/js/depend/wouter/";
import { Form, Text, Submit } from "/js/Forms/Form.js";
import { h } from "/js/html.js";

export default function AddEditMcp9808(props) {
  const [submitting, setSubmitting] = useState(false);
  const addDevice = useAppStore(x => x.devices.add);
  const editDevice = useAppStore(x => x.devices.edit);
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
      let method;
      if (props.device == null) {
        method = addDevice(params);
      } else {
        method = editDevice(props.name, params);
      }
      method
        .then(result => {
          setLocation("/devices");
        })
        .finally(() => setSubmitting(false));
    },
    [addDevice, editDevice, props.name]
  );

  return h(Form, { onSubmit: handleSubmit, defaultValues: props.device }, [
    h(Text, {
      id: "name",
      key: "name",
      label: "Device Name",
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
    h(
      Submit,
      { key: "sub", submitting },
      props.device == null ? "Create" : "Edit"
    )
  ]);
}
