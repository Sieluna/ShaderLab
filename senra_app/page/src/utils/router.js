import { appState, addBasePath } from '../state.js';

// Navigate to a specific route
export function navigateTo(path, options = {}) {
    const { replace = false } = options;
    const normalizedPath = path.startsWith('/') ? path : `/${path}`;

    appState.set('ui.currentPath', normalizedPath);

    const fullPath = addBasePath(normalizedPath);
    if (replace) {
        window.history.replaceState({}, '', fullPath);
    } else {
        window.history.pushState({}, '', fullPath);
    }

    return normalizedPath;
}

// Replace current route
export function replaceTo(path) {
    return navigateTo(path, { replace: true });
}

// Navigate back
export function goBack() {
    if (window.history.length > 1) {
        window.history.back();
    } else {
        navigateTo('/');
    }
}

// Navigate forward
export function goForward() {
    window.history.forward();
}

// Get current route
export function getCurrentRoute() {
    return appState.get('ui.currentPath') || '/';
}

// Build path with parameters
export function buildPath(pattern, params = {}) {
    let path = pattern;
    Object.entries(params).forEach(([key, value]) => {
        path = path.replace(`:${key}`, encodeURIComponent(value));
    });
    return path;
}

// Create link click handler
export function createLinkHandler(options = {}) {
    const { closeMenu } = options;

    return (event) => {
        const link = event.target.closest('a');
        if (!link) return;

        const href = link.getAttribute('href');
        if (!href || !href.startsWith('/')) return;

        event.preventDefault();
        navigateTo(href);

        if (closeMenu && window.innerWidth <= 768) {
            closeMenu();
        }
    };
}
