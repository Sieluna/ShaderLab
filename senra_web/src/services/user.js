import { appState } from '../state.js';
import { userApi } from '../api.js';

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

export async function getUserProfile(userId = null) {
    return withUIState(async () => {
        const userData = userId
            ? await userApi.getUser(userId)
            : await userApi.getSelf();

        appState.set('auth.user', userData);

        return { data: userData };
    });
}

export async function updateUserProfile(data) {
    if (!data) {
        return { success: false, error: 'No update data provided' };
    }

    return withUIState(async () => {
        const response = await userApi.updateUser(data);

        const currentUser = appState.get('auth.user');
        appState.set('auth.user', { ...currentUser, ...response });

        return { data: response };
    });
}
