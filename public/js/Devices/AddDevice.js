import { h } from '/static/js/html.js'
import { Form, Text, Radio, Choice } from '/static/js/Forms/Form.js'

function AddBmp085(props) {
    const onSubmit = values => {
        console.log(values);
    };
    return h(Form, {onSubmit: onSubmit}, [
            h('p', {}, "BMP Form and button that will add it"),
            h(Text, { name: 'address',
                    required: "Required",
                    pattern: {value: /^\d+$/, message: "decimal address required"}}),
            h(Radio, { name: 'resolution'}, [
				h(Choice, {name: "high"}, "High Accuracy (slow)"),
				h(Choice, {name: "med"}, "Medium Accuracy"),
				h(Choice, {name: "low"}, "Low (Fast sample rate)")
			]),
            h('button', {type: 'submit'}, "create")
        ]
    )
}

export function AddDevice(props) {
    switch(props.item) {
        case 'BMP085': return h(AddBmp085, {})
    }

    return h("div", {className: "add-device"},
        h("h1", {}, "configure device: " + props.item)
    )
}
