import { newE2EPage } from '@stencil/core/testing';

describe('list-configured-devices', () => {
  it('renders', async () => {
    const page = await newE2EPage();
    await page.setContent('<list-configured-devices></list-configured-devices>');

    const element = await page.find('list-configured-devices');
    expect(element).toHaveClass('hydrated');
  });
});
