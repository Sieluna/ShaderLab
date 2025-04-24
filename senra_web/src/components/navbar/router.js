import styles from './router.module.css';
import { appState } from '../../state.js';

function matchRoute(path, routes) {
    if (routes[path]) {
        return { handler: routes[path], params: {} };
    }

    for (const [pattern, handler] of Object.entries(routes)) {
        if (pattern.includes(':')) {
            const regex = new RegExp('^' + pattern.replace(/:[^/]+/g, '([^/]+)') + '$');
            const match = path.match(regex);

            if (match) {
                const params = {};
                const paramNames = pattern.match(/:[^/]+/g)?.map(p => p.slice(1)) || [];
                paramNames.forEach((name, index) => {
                    params[name] = match[index + 1];
                });
                return { handler, params };
            }
        }
    }

    return null;
}

function createLoadingComponent() {
    const div = document.createElement('div');
    div.className = `${styles.container} ${styles.loading}`;
    div.innerHTML = `
        <div class="${styles.spinner}"></div>
        <p>Loading...</p>
    `;
    return div;
}

function createErrorComponent(error) {
    const div = document.createElement('div');
    div.className = `${styles.container} ${styles.error}`;
    div.innerHTML = `
        <h2>Route Error</h2>
        <p>${error.message || 'Unknown error'}</p>
        <button onclick="window.location.reload()">Reload Page</button>
    `;
    return div;
}

async function transitionToRoute(container, newComponent) {
    const currentComponent = container.firstElementChild;

    if (!currentComponent || !('animate' in document.createElement('div'))) {
        container.replaceChildren(newComponent);
        return;
    }

    newComponent.style.opacity = '0';
    container.appendChild(newComponent);

    await Promise.all([
        currentComponent.animate([{ opacity: 1 }, { opacity: 0 }],
            { duration: 150, easing: 'ease-out' }).finished,
        newComponent.animate([{ opacity: 0 }, { opacity: 1 }],
            { duration: 150, easing: 'ease-in' }).finished
    ]);

    currentComponent.remove();
    newComponent.style.opacity = '';
}

export function router(routes = {}) {
    const container = document.createElement('div');
    container.className = styles.container;

    let currentPath = '';
    let isNavigating = false;

    const renderRoute = async (path) => {
        if (isNavigating || path === currentPath) return;

        isNavigating = true;
        currentPath = path;

        try {
            const matchResult = matchRoute(path, routes);
            const routeHandler = matchResult?.handler || routes['/'] ||
                (() => createErrorComponent(new Error('Route not found')));

            const isAsync = typeof routeHandler === 'function' &&
                routeHandler.constructor.name === 'AsyncFunction';

            if (isAsync) {
                await transitionToRoute(container, createLoadingComponent());
            }

            let component;
            if (typeof routeHandler === 'function') {
                component = matchResult?.params ?
                    await routeHandler(matchResult.params) :
                    await routeHandler();
            } else {
                component = routeHandler;
            }

            let finalComponent;
            if (component instanceof HTMLElement) {
                finalComponent = component;
            } else {
                finalComponent = document.createElement('div');
                finalComponent.innerHTML = component || '';
            }

            if (matchResult?.params) {
                Object.assign(finalComponent.dataset, matchResult.params);
            }

            await transitionToRoute(container, finalComponent);

        } catch (error) {
            console.error('Route rendering error:', error);
            await transitionToRoute(container, createErrorComponent(error));
        } finally {
            isNavigating = false;
        }
    };

    const unsubscribe = appState.subscribe('ui.currentPath', (newPath) => {
        renderRoute(newPath || '/');
    });

    const cleanup = () => {
        unsubscribe();
    };

    const observer = new MutationObserver(mutations => {
        mutations.forEach(mutation => {
            mutation.removedNodes.forEach(node => {
                if (node === container) {
                    cleanup();
                    observer.disconnect();
                }
            });
        });
    });

    const startObserving = () => {
        if (container.parentNode) {
            observer.observe(container.parentNode, { childList: true });
        } else {
            requestAnimationFrame(startObserving);
        }
    };
    startObserving();

    renderRoute(appState.get('ui.currentPath') || '/');

    container.cleanup = cleanup;

    return container;
}
