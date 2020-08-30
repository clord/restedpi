import {Component, h , Prop} from '@stencil/core';
import { RouterHistory } from '@stencil/router';

@Component({
  tag: 'setup-devices',
  styleUrl: 'setup-devices.css',
  shadow: true
})
export class AppProfile {
  @Prop() history: RouterHistory;


  handleAddDevice = () => {
    this.history.push(`/setup-devices/add-device`, {});
  }

  render() {
    return (
      <main>
        <aside>
          <h2>Settings</h2>
          <sl-button type="primary" onClick={this.handleAddDevice}>Add Device</sl-button>
          <sl-button>Settings</sl-button>
        </aside>
        <stencil-route-switch scrollTopOffset={0}>
          <stencil-route url='/setup-devices' component='list-configured-devices' exact={true} />
          <stencil-route url='/setup-devices/add-device' component='add-device' />
        </stencil-route-switch>
      </main>
    );
  }
}
