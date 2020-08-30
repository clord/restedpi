import {Component, h , State} from '@stencil/core';
import {Get} from '../../util/api'

@Component({
  tag: 'list-configured-devices',
  styleUrl: 'list-configured-devices.css',
  shadow: false,
})
export class ListConfiguredDevices {
  @State() devicesData;

  componentWillLoad() {
    this.devicesData = null
  }

  async componentDidLoad() {
    this.devicesData = await Get('/devices/configured');
  }

  devices(f) {
    return Object.entries(this.devicesData).map(([k, x]) => x[1] === "Ok" ? f(x[0], k) : null)
  }


  render() {
    if (this.devicesData == null) {
      return null;
    }

    return (
      <article>
        {this.devices(device => (
          <sl-card class="device-card">
            <h3>{device.name}</h3>
            <p>
              {device.description}
            </p>
            <small>{device.model.name}</small>
            <div slot="footer">
              <sl-button type="primary" size="small">Configure</sl-button>
              <sl-button type="danger" size="small">Remove</sl-button>
            </div>
          </sl-card>
          ))}
      </article>
    );
  }

}
