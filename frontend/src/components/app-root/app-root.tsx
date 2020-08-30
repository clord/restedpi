import { Component, State, h } from '@stencil/core';
import { Get } from '../../util/api'

interface ServerConfig {
  serverConfig?: {
    deviceName: string
  }
}

@Component({
  tag: 'app-root',
  styleUrl: 'app-root.css',
  shadow: true
})
export class AppRoot {
  @State() serverConfigData: ServerConfig | null;

  componentWillLoad() {
    this.serverConfigData = null
  }

  async componentDidLoad() {
    this.serverConfigData = await Get("/config")
  }

  render() {
    return (
      <div>
        <header>
          <h1>{this.serverConfigData?.serverConfig.deviceName ?? ""}</h1>
          <nav>
            <stencil-route-link activeClass="active" url="/sensors">Sensors</stencil-route-link>
            <stencil-route-link activeClass="active" url="/switches">Switches</stencil-route-link>
            <stencil-route-link activeClass="active" url="/setup-devices">Config</stencil-route-link>
          </nav>
        </header>

        <main>
          <stencil-router>
            <stencil-route-switch scrollTopOffset={0}>
              <stencil-route url='/' component='app-home' exact={true} />
              <stencil-route url='/sensors' component='app-sensors' />
              <stencil-route url='/switches' component='app-switches' />
              <stencil-route url='/setup-devices' component='setup-devices' />
            </stencil-route-switch>
          </stencil-router>
        </main>
      </div>
    );
  }
}
