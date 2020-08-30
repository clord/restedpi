import { newE2EPage } from '@stencil/core/testing';

describe('add-device', () => {
  it('renders', async () => {
    const page = await newE2EPage();
    await page.setContent('<add-device></add-device>');

    const element = await page.find('add-device');
    expect(element).toHaveClass('hydrated');
  });
});
