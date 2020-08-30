import { newE2EPage } from '@stencil/core/testing';

describe('setup-devices', () => {
  it('renders', async () => {
    const page = await newE2EPage();
    await page.setContent('<setup-devices></setup-devices>');

    const element = await page.find('setup-devices');
    expect(element).toHaveClass('hydrated');
  });

  it('displays the specified name', async () => {
    const page = await newE2EPage({ url: '/profile/joseph' });

    const profileElement = await page.find('app-root >>> setup-devices');
    const element = profileElement.shadowRoot.querySelector('div');
    expect(element.textContent).toContain('Hello! My name is Joseph.');
  });

  // it('includes a div with the class "setup-devices"', async () => {
  //   const page = await newE2EPage({ url: '/profile/joseph' });

  // I would like to use a selector like this above, but it does not seem to work
  //   const element = await page.find('app-root >>> setup-devices >>> div');
  //   expect(element).toHaveClass('setup-devices');
  // });
});
