import { createContext, useContext, Fragment } from "/react/";
import useForm, {
  FormContext,
  useFormContext
} from "/js/depend/react-hook-form.js";
import { h } from "/js/html.js";

export function Form({ onSubmit, children, defaultValues }) {
  const methods = useForm({ mode: "onBlur", defaultValues });
  const { handleSubmit } = methods;
  return h(
    FormContext,
    methods,
    h(
      "form",
      {
        className: "w-full max-w-sm flex flex-col",
        onSubmit: handleSubmit(onSubmit)
      },
      children
    )
  );
}

export function Submit({ disabled, submitting, children }) {
  const { errors } = useFormContext();
  const hasErrors = Object.entries(errors).length > 0;

  return h(
    "button",
    {
      type: "submit",
      disabled: hasErrors || disabled,
      className: `mt-2 ${
        hasErrors
          ? "bg-gray-300 text-gray-500"
          : "bg-blue-500 hover:bg-blue-700 text-white"
      } font-bold py-2 px-4 rounded`
    },
    children
  );
}

export function Label({ children, ...props }) {
  return h(
    "label",
    {
      ...props,
      className:
        "block uppercase tracking-wide text-gray-700 text-xs font-bold mb-2"
    },
    children
  );
}

export function Group({ name, children }) {
  return h("fieldset", {}, [h("legend", {}, name), h(Fragment, {}, children)]);
}

export function Text({ id, label, placeholder, ...validation }) {
  const { register, errors } = useFormContext();
  const hasError = errors.hasOwnProperty(id);
  return h(Label, { htmlFor: id }, [
    label,
    h("input", {
      type: "text",
      key: 1,
      placeholder,
      className: `appearance-none block w-full text-gray-700 border bg-gray-200
        ${
          hasError
            ? "border-red-500 focus:border-red-400"
            : "border-gray-200 focus:border-gray-500"
        } rounded py-3 px-4 leading-tight focus:outline-none focus:bg-white`,
      id,
      name: id,
      ref: register(validation)
    }),
    errors[id]
      ? h(
          "p",
          { key: "err", className: "text-red-500 text-xs" },
          errors[id].message
        )
      : null
  ]);
}

const ChoiceContext = createContext(null);

export function Radio({ name, children }) {
  const { errors } = useFormContext();
  return h(ChoiceContext.Provider, { value: name }, [
    h(Fragment, {}, children),
    h("aside", {}, errors[name] ? "required" : null)
  ]);
}

export function RadioChoice({ name, label, children }) {
  const groupname = useContext(ChoiceContext);
  const { register } = useFormContext();
  return [
    h(
      "label",
      {
        htmlFor: name,
        key: name,
        className:
          "block uppercase tracking-wide text-gray-700 text-xs font-bold mb-2"
      },
      [label]
    ),
    h("input", {
      name: groupname,
      type: "radio",
      key: `${name}-input`,
      value: name,
      ref: register({ required: true })
    }),
    h(Fragment, {}, children)
  ];
}

Radio.Choice = RadioChoice;

export function Select({ name, label, children }) {
  const { errors, register } = useFormContext();
  return h(ChoiceContext.Provider, { value: name }, [
    h(
      "label",
      {
        htmlFor: name,
        key: name,
        className:
          "block uppercase tracking-wide text-gray-700 text-xs font-bold mb-2"
      },
      [
        h(Fragment, { key: "lab" }, label),
        h("div", { key: "0", className: "relative" }, [
          h(
            "select",
            {
              ref: register({ required: true }),
              key: `${name}-select`,
              name,
              className:
                "block appearance-none w-full bg-gray-200 border border-gray-200 text-gray-700 py-3 px-4 pr-8 rounded leading-tight focus:outline-none focus:bg-white focus:border-gray-500"
            },
            children
          ),
          h(
            "div",
            {
              key: "1",
              className:
                "pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700"
            },
            h(
              "svg",
              {
                className: "fill-current h-4 w-4",
                xmlns: "http://www.w3.org/2000/svg",
                viewBox: "0 0 20 20"
              },
              h("path", {
                d:
                  "M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z"
              })
            )
          )
        ])
      ]
    ),
    h("aside", { key: "err" }, errors[name] ? "required" : null)
  ]);
}

export function SelectChoice({ value, selected, children }) {
  return h("option", { value, selected }, children);
}

Select.Choice = SelectChoice;
