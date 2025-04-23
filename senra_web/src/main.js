import { router, navbar } from './components/index.js';
import { homePage, notebookPage, debugPage } from './pages/index.js';
import { appState, normalizePath, addBasePath } from './state.js';
import './style.css';

const mode = __APP_ENV__;

function initializeApp() {
    const app = document.querySelector('#app');
    if (!app) return;

    app.innerHTML = '';

    const navbarItems = [
        { label: 'Home', path: '/' },
        { label: 'Explore', path: '/explore' },
        { label: 'Create', path: '/create' },
        ...(mode === 'development' ? [{ label: 'Debug', path: '/debug' }] : []),
    ];

    app.appendChild(navbar(navbarItems));

    const routes = {
        '/': homePage,
        '/explore': () => {
            const div = document.createElement('div');
            div.textContent = 'Explore Page - Under Development';
            return div;
        },
        '/create': () => {
            const div = document.createElement('div');
            div.textContent = 'Create Page - Under Development';
            return div;
        },
        ...(mode === 'development' && { '/debug': debugPage }),
    };

    appState.subscribe((state) => {
        const path = state.ui?.currentPath || '';

        if (path.startsWith('/notebook/')) {
            const id = parseInt(path.split('/')[2]);
            if (id && !isNaN(id)) {
                routes[path] = () => notebookPage(id);
            }
        }
    });

    const handleRouteChange = () => {
        let path = normalizePath(window.location.pathname);
        if (path === '/index.html') path = '/';

        if (path.match(/^\/notebook\/\d+$/)) {
            const id = parseInt(path.split('/')[2]);
            if (id && !isNaN(id)) {
                routes[path] = () => notebookPage(id);
            }
        }

        appState.setState((prev) => ({
            ...prev,
            ui: {
                ...prev.ui,
                currentPath: path,
            },
        }));
    };

    handleRouteChange();

    window.addEventListener('popstate', handleRouteChange);

    document.addEventListener('click', (e) => {
        const link = e.target.closest('a');
        if (link && link.getAttribute('href')?.startsWith('/')) {
            e.preventDefault();
            const rawPath = link.getAttribute('href');
            const fullPath = addBasePath(rawPath);
            window.history.pushState({}, '', fullPath);
            handleRouteChange();
        }
    });

    app.appendChild(router(routes));
}

document.addEventListener('DOMContentLoaded', initializeApp);
