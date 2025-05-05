/**
 * Create a reactive state store with fine-grained subscriptions
 * @param {Object} initialState - Initial state object
 * @returns {Object} State store with methods
 */
export function createState(initialState = {}) {
    const listeners = new Map(); // path -> Set<callback>
    let state = structuredClone(initialState);

    /**
     * Get nested value by path
     */
    const getByPath = (path) => {
        if (!path || typeof path !== 'string') return state;
        return path.split('.').reduce((obj, key) => obj?.[key], state);
    };

    /**
     * Set nested value by path with immutable updates
     */
    const setByPath = (path, value) => {
        if (!path || typeof path !== 'string') {
            state = typeof value === 'function' ? value(state) : { ...state, ...value };
            return true;
        }

        const keys = path.split('.');
        const lastKey = keys.at(-1);
        const target = keys.slice(0, -1).reduce((obj, key) => {
            obj[key] ??= {};
            return obj[key];
        }, state);

        const oldValue = target[lastKey];
        const newValue = typeof value === 'function' ? value(oldValue) : value;

        if (oldValue !== newValue) {
            target[lastKey] = newValue;
            return true;
        }
        return false;
    };

    /**
     * Get affected paths using functional chain
     */
    const getAffectedPaths = (changedPath) => {
        if (!changedPath || typeof changedPath !== 'string') {
            return new Set();
        }

        const affected = new Set();

        // Add exact path if it has listeners
        listeners.has(changedPath) && affected.add(changedPath);

        // Add parent paths efficiently
        const parts = changedPath.split('.');
        parts.reduce((acc, _, i) => {
            const parentPath = parts.slice(0, i).join('.');
            listeners.has(parentPath) && affected.add(parentPath);
            return acc;
        }, []);

        // Add child paths using filter
        [...listeners.keys()]
            .filter(path => typeof path === 'string' && path.startsWith(changedPath + '.'))
            .forEach(path => affected.add(path));

        return affected;
    };

    /**
     * Notify listeners with error boundaries
     */
    const notify = (paths) => {
        const notified = new Set();

        paths.forEach(path => {
            const pathListeners = listeners.get(path);
            if (!pathListeners) return;

            const currentValue = getByPath(path);
            pathListeners.forEach(listener => {
                if (notified.has(listener)) return;

                notified.add(listener);
                queueMicrotask(() => {
                    try {
                        listener(currentValue, path);
                    } catch (error) {
                        console.error(`State listener error for "${path}":`, error);
                    }
                });
            });
        });
    };

    /**
     * Find changed paths recursively
     */
    const findChanges = (a, b, basePath = '') => {
        return Object.entries(b).flatMap(([key, value]) => {
            const path = basePath ? `${basePath}.${key}` : key;
            const changed = a[key] !== value;

            if (!changed) return [];

            const currentChange = [path];
            const nestedChanges = value && typeof value === 'object' && !Array.isArray(value)
                ? findChanges(a[key] ?? {}, value, path)
                : [];

            return [...currentChange, ...nestedChanges];
        });
    };

    return {
        /**
         * Get state with optional path
         */
        get: (path) => structuredClone(getByPath(path)),

        /**
         * Set state with path or object
         */
        set: (pathOrState, value) => {
            const updates = new Set();

            if (typeof pathOrState === 'string') {
                setByPath(pathOrState, value) && updates.add(pathOrState);
            } else {
                const newState = typeof pathOrState === 'function'
                    ? pathOrState(state)
                    : { ...state, ...pathOrState };

                findChanges(state, newState).forEach(path => updates.add(path));
                state = newState;
            }

            // Notify if there are changes
            if (updates.size > 0) {
                const affectedPaths = new Set([...updates].flatMap(path =>
                    [...getAffectedPaths(path)]
                ));
                notify(affectedPaths);
            }
        },

        /**
         * Subscribe to path changes
         */
        subscribe: (path = '', callback) => {
            const validPath = typeof path === 'string' ? path : '';

            const pathListeners = listeners.get(validPath) ?? new Set();
            if (!listeners.has(validPath)) {
                listeners.set(validPath, pathListeners);
            }

            pathListeners.add(callback);

            queueMicrotask(() => callback(getByPath(validPath), validPath));

            return () => {
                pathListeners.delete(callback);
                pathListeners.size === 0 && listeners.delete(validPath);
            };
        },
    };
}

/**
 * Path utilities using modern string methods
 */
export const normalizePath = (path) => {
    const basePath = import.meta.env?.BASE_URL ?? '/';
    return basePath !== '/' && path.startsWith(basePath)
        ? path.slice(basePath.length - 1)
        : path;
};

export const addBasePath = (path) => {
    const basePath = import.meta.env?.BASE_URL ?? '/';

    if (path.startsWith(basePath) || path.startsWith('http') || basePath === '/') {
        return path;
    }

    const cleanPath = path.startsWith('/') ? path : `/${path}`;
    const cleanBase = basePath.endsWith('/') ? basePath.slice(0, -1) : basePath;

    return `${cleanBase}${cleanPath}`;
};

// Application state with improved structure
export const appState = createState({
    ui: {
        currentPath: '/',
        isLoading: false,
        error: null,
    },
    auth: {
        isAuthenticated: false,
        user: null,
    },
    notebook: {
        current: {
            notebook: null,
            isLoading: false,
            error: null,
            comments: {
                items: [],
                total: 0,
                page: 1,
                isLoading: false,
                hasLoaded: false,
                error: null,
            },
            versions: {
                items: [],
                total: 0,
                page: 1,
                isLoading: false,
                error: null,
            },
        },
        trending: {
            notebooks: [],
            isLoading: false,
            error: null,
        },
        latest: {
            notebooks: [],
            isLoading: false,
            error: null,
        },
    },
});

const createStateAccessor = (path) => ({
    get: () => appState.get(path),
    subscribe: (callback) => appState.subscribe(path, callback),
});

export const authState = createStateAccessor('auth');

export const uiState = createStateAccessor('ui');

export const notebookState = {
    ...createStateAccessor('notebook'),
    current: createStateAccessor('notebook.current'),
    trending: createStateAccessor('notebook.trending'),
    latest: createStateAccessor('notebook.latest'),
};
