import { appState } from '../state.js';
import { authApi } from '../api.js';

export async function checkAuthStatus() {
    try {
        if (await authApi.verifyToken()) {
            appState.setState((state) => ({
                ...state,
                auth: {
                    ...state.auth,
                    isAuthenticated: true,
                },
            }));
            return true;
        } else {
            logout(false);
            return false;
        }
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

        appState.setState((state) => ({
            ...state,
            auth: {
                isAuthenticated: true,
                user: {
                    id: response.id,
                    username: response.username,
                    email: response.email,
                    avatar: response.avatar,
                },
            },
            ui: { ...state.ui, isLoading: false, error: null },
        }));

        return { success: true };
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

        appState.setState((state) => ({
            ...state,
            auth: {
                isAuthenticated: true,
                user: {
                    id: response.id,
                    username: response.username,
                    email: response.email,
                    avatar: response.avatar,
                },
            },
            ui: { ...state.ui, isLoading: false, error: null },
        }));

        return { success: true };
    } catch (error) {
        console.error('Registration failed:', error);

        appState.setState((state) => ({
            ...state,
            ui: { ...state.ui, isLoading: false, error: `Registration failed: ${error.message}` },
        }));

        return { success: false, error: `Registration failed: ${error.message}` };
    }
}

export async function logout(redirect = true) {
    try {
        await authApi.logout();

        appState.setState((state) => ({
            ...state,
            auth: {
                isAuthenticated: false,
                user: null,
            },
        }));

        if (redirect) {
            window.location.reload();
        }
    } catch (error) {
        console.error('Logout failed:', error);
    }
}
