import styles from './router.module.css';
import { appState } from '../../state.js';

export function router(routes = {}) {
    const container = document.createElement('div');
    container.className = styles.container;

    const renderRoute = (path) => {
        container.innerHTML = '';
        const routeHandler = routes[path] || routes['/'] || (() => '');

        if (typeof routeHandler === 'function') {
            const result = routeHandler();
            if (result instanceof HTMLElement) {
                container.appendChild(result);
            } else {
                container.innerHTML = result;
            }
        }
    };

    appState.subscribe((state) => {
        const currentPath = state.ui?.currentPath || '/';
        renderRoute(currentPath);
    });

    renderRoute(appState.getState().ui?.currentPath || '/');

    return container;
}
