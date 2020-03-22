import { html, h } from "./html.js";
import { Nav } from "./Nav.js";
import { useAppStore } from "/js/hooks/useApp.js";

export function About() {
  const name = useAppStore(x => x.serverConfig.deviceName);
  const className = "whitespace-no-wrap text-sm hidden md:block";

  if (name == null) {
    return h("aside", { className }, "loading");
  }
  return h("aside", { className }, name);
}
/**

 * Present the current description of the server to the user
 */
export function Header(props) {
  return html`
    <header
      className="flex flex-row items-baseline justify-between text-white bg-orange-500 p-3"
    >
      <h1 className="flex items-center flex-shrink-0 text-white mr-6 text-lg">
        <i className="fas fa-pizza-slice mr-2"></i> REpi
      </h1>
      <${Nav} />
      <${About} />
    </header>
  `;
}
