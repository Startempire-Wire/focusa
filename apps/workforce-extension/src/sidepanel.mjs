const status = document.querySelector('#connection-status');
if (!(status instanceof HTMLElement)) {
  throw new Error('connection status element is required');
}
status.dataset.state = 'unconfigured';
