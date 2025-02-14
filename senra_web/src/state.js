export function createState(initialState = {}) {
    let state = { ...initialState };
    const listeners = new Set();

    const getState = () => ({ ...state });

    const setState = (newState) => {
        state = typeof newState === 'function' ? newState(state) : { ...state, ...newState };

        listeners.forEach((listener) => listener(state));
    };

    const subscribe = (listener) => {
        listeners.add(listener);
        return () => listeners.delete(listener);
    };

    return {
        getState,
        setState,
        subscribe,
    };
}

export function normalizePath(path) {
    const basePath = import.meta.env?.BASE_URL || '/';
    if (path.startsWith(basePath) && basePath !== '/') {
        return path.substring(basePath.length - 1);
    }
    return path;
}

export function addBasePath(path) {
    const basePath = import.meta.env?.BASE_URL || '/';
    if (path.startsWith(basePath) || path.startsWith('http') || basePath === '/') {
        return path;
    }
    const cleanPath = path.startsWith('/') ? path : `/${path}`;
    const cleanBase = basePath.endsWith('/') ? basePath.slice(0, -1) : basePath;
    return `${cleanBase}${cleanPath}`;
}

export const appState = createState({
    auth: {
        isAuthenticated: false,
        user: null,
        token: localStorage.getItem('token'),
    },
    ui: {
        currentPath: '/',
        isLoading: false,
        error: null,
    },
});

export function createDerivedState(sourceState, selector) {
    const derivedState = createState(selector(sourceState.getState()));

    sourceState.subscribe((state) => {
        derivedState.setState(selector(state));
    });

    return derivedState;
}

export const uiState = createDerivedState(appState, (state) => state.ui);

export const authState = createDerivedState(appState, (state) => state.auth);
