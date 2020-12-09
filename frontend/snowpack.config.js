module.exports = {
  extends: "@snowpack/app-scripts-react",
  mount: {
    public: '/',
    src: '/_dist_',
  },
  plugins: [
    '@snowpack/plugin-react-refresh',
    '@snowpack/plugin-dotenv',
    '@snowpack/plugin-typescript',
    "@snowpack/plugin-babel",
  ],
  install: [
    "@emotion/react",
    "@emotion/styled",
    "tailwindcss/dist/base.min.css"
  ],
  installOptions: {
    /* ... */
  },
  devOptions: {
    /* ... */
  },
  buildOptions: {
    /* ... */
  },
  proxy: {
    "/api": "https://rip.z.odago.ca:3030/api"
  },
  alias: {
    "@comps": "./src/components",
    "@app": "./src"
  },
};
