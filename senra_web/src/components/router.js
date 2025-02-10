import styles from './router.module.css';
import { appState } from '../state.js';

export function router(routes = {}) {
    const routerContainer = document.createElement('div');
    routerContainer.className = styles.container;

    const renderRoute = (path) => {
        routerContainer.innerHTML = '';
        const routeHandler = routes[path] || routes['/'] || (() => '');

        if (typeof routeHandler === 'function') {
            const result = routeHandler();
            if (result instanceof HTMLElement) {
                routerContainer.appendChild(result);
            } else {
                routerContainer.innerHTML = result;
            }
        }
    };

    appState.subscribe((state) => {
        const currentPath = state.ui?.currentPath || '/';
        renderRoute(currentPath);
    });

    renderRoute(appState.getState().ui?.currentPath || '/');

    return routerContainer;
}
