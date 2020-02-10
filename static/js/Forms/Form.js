import { createContext, useContext, Fragment } from '/js/depend/react/'
import useForm, {FormContext, useFormContext} from "/js/depend/react-hook-form.js"
import { h } from '/js/html.js'

export function Form({onSubmit, children}) {
    const methods = useForm({mode: 'onBlur'})
    const { handleSubmit } = methods
    return h(FormContext, methods,
        h('form', {className: 'w-full max-w-lg',
            onSubmit: handleSubmit(onSubmit) }, children))
}

export function Label({children, ...props}) {
    return h('label', {
            ...props,
            className: 'block uppercase tracking-wide text-gray-700 text-xs font-bold mb-2'
        },
        children
    )
}

export function Group({name, children}) {
    return h('fieldset', {},
        [h('legend', {}, name),
         h(Fragment, {}, children)]
    )
}

export function Text({id, label, ...validation}) {
	const {register, errors} = useFormContext()
	return [
        h(Label, {"for": id}, [
            label,
            h('input', {
                type: 'text',
                id,
                ref: register(validation)
            }),
            errors[id] ?
                h('p', {className: 'text-red-500 text-xs italic'}, errors[id].message) :
                null
        ])
	]
}

const RadioContext = createContext(null)

export function Radio({name, children}) {
	const { errors } = useFormContext()
	return h(RadioContext.Provider, {value: name},
        [h(Fragment, {}, children),
         h('aside', {}, errors[name] ? "required" : null)])
}

export function Choice({name, children}) {
	const groupname = useContext(RadioContext)
	const {register} = useFormContext()
	return h('label', {}, [
        h('input', {
            name: groupname,
            type: "radio",
            value: name,
            ref: register({required: true})}),
		h(Fragment, {}, children)
	])
}

