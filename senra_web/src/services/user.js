import { appState } from '../state.js';
import { userApi } from '../api.js';

export async function getUserProfile(userId = null) {
    const state = appState.getState();
    if (state.ui.isLoading) return state;

    appState.setState((state) => ({
        ...state,
        ui: { ...state.ui, isLoading: true, error: null },
    }));

    try {
        let userData;
        if (userId) {
            userData = await userApi.getUser(userId);
        } else {
            userData = await userApi.getSelf();
        }

        appState.setState((state) => ({
            ...state,
            auth: {
                ...state.auth,
                user: userData,
            },
            ui: { ...state.ui, isLoading: false, error: null },
        }));

        return { success: true, data: userData };
    } catch (error) {
        console.error('Failed to get user profile:', error);

        appState.setState((state) => ({
            ...state,
            ui: {
                ...state.ui,
                isLoading: false,
                error: `Failed to get user profile: ${error.message}`,
            },
        }));

        return { success: false, error: error.message };
    }
}

export async function updateUserProfile(data) {
    const state = appState.getState();
    if (state.ui.isLoading) return state;

    if (!data) {
        return { success: false, error: 'No update data provided' };
    }

    appState.setState((state) => ({
        ...state,
        ui: { ...state.ui, isLoading: true, error: null },
    }));

    try {
        const response = await userApi.updateUser(data);

        appState.setState((state) => ({
            ...state,
            auth: {
                ...state.auth,
                user: {
                    ...state.auth.user,
                    ...response,
                },
            },
            ui: { ...state.ui, isLoading: false, error: null },
        }));

        return { success: true, data: response };
    } catch (error) {
        console.error('Failed to update user profile:', error);

        appState.setState((state) => ({
            ...state,
            ui: {
                ...state.ui,
                isLoading: false,
                error: `Failed to update user profile: ${error.message}`,
            },
        }));

        return { success: false, error: error.message };
    }
}
