import { Component, Listen, h } from '@stencil/core';

@Component({
  tag: 'add-device',
  styleUrl: 'add-device.css',
  shadow: true,
})
export class AddDevice {

  @Listen('slSubmit')
  handleSubmit(event) {
    let output = "";
    const formData = event.detail.formData;
    for (const entry of formData.entries()) {
      output += `${entry[0]}: ${entry[1]}\n`;
    }
    console.log(output);
  }

  render() {
    return (
      <div>
        <h2>Add new device</h2>
        <p>
          Devices allow your pi to sense or act in the world in some manner.
          Temperature sensors, GPIO banks, cameras, timers. Also, virtual devices.
        </p>
        <sl-card>
          <sl-form name="add-device">
            <div style={{marginBottom: "12px"}}>
              <sl-input name="name" type="text" label="Device Name"></sl-input>
            </div>
            <sl-button submit>Create Device</sl-button>
          </sl-form>
        </sl-card>
      </div>
    );
  }
}
