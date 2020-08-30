import { newSpecPage } from '@stencil/core/testing';
import { AddDevice } from '../add-device';

describe('add-device', () => {
  it('renders', async () => {
    const page = await newSpecPage({
      components: [AddDevice],
      html: `<add-device></add-device>`,
    });
    expect(page.root).toEqualHtml(`
      <add-device>
        <mock:shadow-root>
          <slot></slot>
        </mock:shadow-root>
      </add-device>
    `);
  });
});
