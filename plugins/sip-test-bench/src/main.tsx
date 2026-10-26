import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { createBridge } from './sip-bridge';
import App from './App';
import './styles.css';

// The bridge is created before React mounts so the hello/kickoff is sent as
// early as possible, and it lives on `window` so HMR / StrictMode double-mount
// cannot install a second listener or a second hello loop.
createBridge();

const container = document.getElementById('root');
if (!container) {
  throw new Error('#root element missing from index.html');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
