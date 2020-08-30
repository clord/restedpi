import { Component , h } from '@stencil/core';

@Component({
  tag: 'add-device',
  styleUrl: 'add-device.css',
  shadow: true,
})
export class AddDevice {

  handleSubmit = event => {
    console.log("here", event);
    let output;
    const formData = event.detail.formData;
    for (const entry of formData.entries()) {
      output += `${entry[0]}: ${entry[1]}\n`;
    }
    console.log(output);
  }

  render() {
    return (
      <div style={{ margin: '12px'}}>
      <sl-form name="test" slSubmit={this.handleSubmit}>
        <div style={{marginBottom: "12px"}}>
          <sl-input name="something" type="text" label="Name"></sl-input>
        </div>
        <sl-button submit>Create Device</sl-button>
      </sl-form>
      </div>
    );
  }
}
