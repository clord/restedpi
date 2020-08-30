import { newSpecPage } from '@stencil/core/testing';
import { ListConfiguredDevices } from '../list-configured-devices';

describe('list-configured-devices', () => {
  it('renders', async () => {
    const page = await newSpecPage({
      components: [ListConfiguredDevices],
      html: `<list-configured-devices></list-configured-devices>`,
    });
    expect(page.root).toEqualHtml(`
      <list-configured-devices>
        <mock:shadow-root>
          <slot></slot>
        </mock:shadow-root>
      </list-configured-devices>
    `);
  });
});
