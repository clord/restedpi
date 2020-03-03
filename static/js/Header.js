import { html, h } from "./html.js";
import { Nav } from "./Nav.js";
import { useAppStore } from "/js/hooks/useApp.js";

export function About() {
  const name = useAppStore(x => x.serverConfig.deviceName);

  if (name == null) {
    return h("aside", {}, "loading");
  }
  return h("aside", {}, name);
}
/**

 * Present the current description of the server to the user
 */
export function Header(props) {
  return html`
    <header
      className="flex items-center justify-between flex-wrap text-white bg-orange-500 p-5"
    >
      <h1 className="flex items-center flex-shrink-0 text-white mr-6 text-lg">
        <i className="fas fa-pizza-slice mr-2"></i> REpi
      </h1>
      <${Nav} />
      <${About} />
    </header>
  `;
}
