import { appState } from '../state.js';
import { authApi } from '../api.js';

export async function checkAuthStatus() {
    try {
        const token = appState.getState().auth.token;

        if (!token) {
            logout(false);
            return false;
        }

        const response = await authApi.verifyToken(token);

        if (response?.token) {
            appState.setState((state) => ({
                ...state,
                auth: {
                    ...state.auth,
                    isAuthenticated: true,
                    token: response.token,
                },
            }));
        }

        return true;
    } catch (error) {
        console.error('Failed to check authentication status:', error);
        logout(false);
        return false;
    }
}

export async function login(username, password) {
    if (!username || !password) {
        return { success: false, error: 'Please enter username and password' };
    }

    appState.setState((state) => ({
        ...state,
        ui: { ...state.ui, isLoading: true, error: null },
    }));

    try {
        const response = await authApi.login(username, password);

        if (response && response.token) {
            localStorage.setItem('token', response.token);

            appState.setState((state) => ({
                ...state,
                auth: {
                    isAuthenticated: true,
                    user: response.user,
                    token: response.token,
                },
                ui: { ...state.ui, isLoading: false, error: null },
            }));

            return { success: true };
        } else {
            appState.setState((state) => ({
                ...state,
                ui: {
                    ...state.ui,
                    isLoading: false,
                    error: 'Login failed, please check username and password',
                },
            }));

            return { success: false, error: 'Login failed, please check username and password' };
        }
    } catch (error) {
        console.error('Login failed:', error);

        appState.setState((state) => ({
            ...state,
            ui: { ...state.ui, isLoading: false, error: `Login failed: ${error.message}` },
        }));

        return { success: false, error: `Login failed: ${error.message}` };
    }
}

export async function register(username, email, password) {
    if (!username || !email || !password) {
        return { success: false, error: 'Please fill in all required fields' };
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
        return { success: false, error: 'Please enter a valid email address' };
    }

    if (password.length < 6) {
        return { success: false, error: 'Password must be at least 6 characters long' };
    }

    appState.setState((state) => ({
        ...state,
        ui: { ...state.ui, isLoading: true, error: null },
    }));

    try {
        const response = await authApi.register(username, email, password);

        if (response && response.token) {
            localStorage.setItem('token', response.token);

            appState.setState((state) => ({
                ...state,
                auth: {
                    isAuthenticated: true,
                    user: response.user,
                    token: response.token,
                },
                ui: { ...state.ui, isLoading: false, error: null },
            }));

            return { success: true };
        } else {
            appState.setState((state) => ({
                ...state,
                ui: {
                    ...state.ui,
                    isLoading: false,
                    error: 'Registration failed, please try again later',
                },
            }));

            return { success: false, error: 'Registration failed, please try again later' };
        }
    } catch (error) {
        console.error('Registration failed:', error);

        appState.setState((state) => ({
            ...state,
            ui: { ...state.ui, isLoading: false, error: `Registration failed: ${error.message}` },
        }));

        return { success: false, error: `Registration failed: ${error.message}` };
    }
}

export function logout(redirect = true) {
    localStorage.removeItem('token');

    appState.setState((state) => ({
        ...state,
        auth: {
            isAuthenticated: false,
            user: null,
            token: null,
        },
    }));

    if (redirect) {
        window.location.reload();
    }
}
