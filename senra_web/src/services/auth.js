import { appState } from '../state.js';
import { authApi } from '../api.js';

/**
 * Helper to manage async operations with UI state
 */
const withUIState = async (operation) => {
    if (appState.get('ui.isLoading')) {
        return { success: false, error: 'Already loading' };
    }

    appState.set('ui', {
        ...appState.get('ui'),
        isLoading: true,
        error: null
    });

    try {
        const result = await operation();

        appState.set('ui.isLoading', false);

        return { success: true, ...result };
    } catch (error) {
        console.error('Operation failed:', error);

        appState.set('ui', {
            ...appState.get('ui'),
            isLoading: false,
            error: error.message,
        });

        return { success: false, error: error.message };
    }
};

/**
 * Helper to update auth state
 */
const setAuthState = (isAuthenticated, user = null) => {
    appState.set('auth', { isAuthenticated, user });
};

export async function checkAuthStatus() {
    try {
        const isValid = await authApi.verifyToken();

        if (isValid) {
            appState.set('auth.isAuthenticated', true);
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

    return withUIState(async () => {
        const response = await authApi.login(username, password);

        setAuthState(true, {
            id: response.id,
            username: response.username,
            email: response.email,
            avatar: response.avatar,
        });

        return { data: response };
    });
}

export async function register(username, email, password) {
    if (!username || !email || !password) {
        return { success: false, error: 'Please fill in all required fields' };
    }

    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
        return { success: false, error: 'Please enter a valid email address' };
    }

    if (password.length < 6) {
        return { success: false, error: 'Password must be at least 6 characters long' };
    }

    return withUIState(async () => {
        const response = await authApi.register(username, email, password);

        setAuthState(true, {
            id: response.id,
            username: response.username,
            email: response.email,
            avatar: response.avatar,
        });

        return { data: response };
    });
}

export async function logout(redirect = true) {
    try {
        await authApi.logout();
        setAuthState(false);

        redirect && window.location.reload();
    } catch (error) {
        console.error('Logout failed:', error);
    }
}
