import { createContext, useContext, Fragment } from '/static/js/depend/react/react.js'
import useForm, {FormContext, useFormContext} from "/static/js/depend/react-hook-form.js"
import { h } from '/static/js/html.js'

export function Form({onSubmit, children}) {
    const methods = useForm({mode: 'onBlur'})
    const { handleSubmit } = methods
	return h(FormContext, methods, h('form', {onSubmit: handleSubmit(onSubmit) }, children))
}

export function Text({name, ...validation}) {
	const {register, errors} = useFormContext()
	return [
		h('input', {
                type: 'text',
                name,
                ref: register(validation)
        }),
        h('p', {}, errors[name] && errors[name].message)
	]
}

const RadioContext = createContext(null)
export function Radio({name, children}) {
	const { errors } = useFormContext()
	return h(RadioContext.Provider, {value: name},
		[h(Fragment, {}, children), h('aside', {}, errors[name] ? "required" : null)])
}

export function Choice({name, children}) {
	const groupname = useContext(RadioContext)
	const {register} = useFormContext()
	return h('label', {}, [
		h('input', {name: groupname, type: "radio", value: name, ref: register({required: true})}),
		h(Fragment, {}, children)
	])
}
