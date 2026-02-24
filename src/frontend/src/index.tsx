/* @refresh reload */
import { render } from 'solid-js/web';
import '~/index.css';
import App from '~/App';

// NOTE: Settings are now initialized in App.tsx via onMount to avoid module import side effects
// that can cause issues with SolidJS lifecycle management

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error('Root element not found.');
}

// Disable browser right-click context menu in production builds
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

render(() => <App />, root!);
