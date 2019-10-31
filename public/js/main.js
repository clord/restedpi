import { usePost, useGet } from './util.js'
import { html, render } from './html.js'

import { Router, Link } from './depend/preact-router.js';

import { Spinner } from './comp/Spinner.js'

/**
 * Present the current description of the server to the user
 */
function Header(props) {
    const {response, error} = useGet(`/api/about`);
    if (response == null) {
        return html`
            <header>
                <h1>
                    <i class="fas fa-pizza-slice"></i>
                    RestedPI
                </h1>
                <h2>
                    <i class="fas fa-microchip"></i>
                </h2>
            </header>
        `
    }

    return html`
        <header>
            <h1>
                <i class="fas fa-pizza-slice"></i>
                RestedPI
            </h1>
            <h2>
                <i class="fas fa-microchip"></i>
                ${response.server}
            </h2>
        </header>
    `
}

function EvaluateValue(props) {
    console.log("val:", props)
    const {response, error} = usePost(`/api/debug/eval_value`,
		{HoursOfDaylight: [{Const: -53.32},  "DayOfYear" ]});

    if (response == null) {
        return null
    }

    return html`
        <div>
       		<strong>Hours of daylight:</strong> ${response.result}
        </div>
    `
}

function EvaluateBool(props) {
    // POST /api/debug/eval_bool
    const {response, error} = usePost(`/api/debug/eval_bool`, {
    	Equal: [{Const: Number(props.value)}, {Const: 13}]
	});

    if (response == null) {
        return null
    }

    return html`
        <div>
          	Bool Eval: ${response.result ? "true" : "false"}
        </div>
    `
}

const app = html`
    <${Header} />
    <aside style="display:flex; flex-direction:column;">
            <${Link} href="/value">Evaluate</${Link}>
            <${Link} href="/bool/1">check 1</${Link}>
            <${Link} href="/bool/12">check 12</${Link}>
            <${Link} href="/bool/13">check 13</${Link}>
    </aside>
    <section>
      <${Router}>
        <${EvaluateBool} path="/bool/:value" />
        <${EvaluateValue} path="/value" />
      </${Router}>
    </section>
    <footer> Footer</footer>
`

render(app, document.getElementById('app'));

