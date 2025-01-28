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

export const appState = createState({
    auth: {
        isAuthenticated: false,
        user: null,
        token: localStorage.getItem('token'),
    },
    ui: {
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
