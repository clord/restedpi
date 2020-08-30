const http = require('http');
const httpProxy = require('http-proxy');

const proxyApi = new httpProxy.createProxyServer({
  target: {
    host: '127.0.0.1',
    port: 3030,
  },
});

const proxyStencil = new httpProxy.createProxyServer({
  target: {
    host: '127.0.0.1',
    port: 3333,
  },
});
const proxyServer = http.createServer(function(req, res) {
  if (req.url.match(/\/api\//)) {
    console.log('to api:', req.url);
    proxyApi.web(req, res);
  } else {
    console.log('to stencil dev:', req.url);
    proxyStencil.web(req, res);
  }
});

//
// Listen to the `upgrade` event and proxy the
// WebSocket requests as well.
//
proxyServer.on('upgrade', function(req, socket, head) {
  proxyStencil.ws(req, socket, head);
});

console.log('Listen: http://127.0.0.1:5000');
proxyServer.listen(5000);
