import { createState } from '../state.js';
import { notebookApi } from '../api.js';

export const notebookState = createState({
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
    current: {
        notebook: null,
        isLoading: false,
        error: null,
    },
    comments: {
        items: [],
        total: 0,
        page: 1,
        isLoading: false,
        error: null,
    },
});

export async function loadTrendingNotebooks() {
    notebookState.setState((state) => ({
        ...state,
        trending: {
            ...state.trending,
            isLoading: true,
            error: null,
        },
    }));

    try {
        // Get trending notebooks
        const response = await notebookApi.listNotebooks(1, 4);

        notebookState.setState((state) => ({
            ...state,
            trending: {
                notebooks: response.notebooks || [],
                isLoading: false,
                error: null,
            },
        }));

        return response.notebooks || [];
    } catch (error) {
        console.error('Failed to load trending notebooks:', error);

        notebookState.setState((state) => ({
            ...state,
            trending: {
                ...state.trending,
                isLoading: false,
                error: `Loading failed: ${error.message}`,
            },
        }));

        return [];
    }
}

export async function loadNotebookDetails(notebookId) {
    // Set loading state
    notebookState.setState((state) => ({
        ...state,
        current: {
            ...state.current,
            isLoading: true,
            error: null,
        },
    }));

    try {
        // Get notebook details
        const notebook = await notebookApi.getNotebook(notebookId);

        // Update state
        notebookState.setState((state) => ({
            ...state,
            current: {
                notebook,
                isLoading: false,
                error: null,
            },
        }));

        return notebook;
    } catch (error) {
        console.error('Failed to load notebook details:', error);

        // Update error state
        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                isLoading: false,
                error: `Loading failed: ${error.message}`,
            },
        }));

        return null;
    }
}

// Load comments
export async function loadComments(notebookId, page = 1) {
    // Set loading state
    notebookState.setState((state) => ({
        ...state,
        comments: {
            ...state.comments,
            isLoading: true,
            error: null,
            page,
        },
    }));

    try {
        // Get comments list
        const comments = await notebookApi.listComments(notebookId, page);

        // Update state
        notebookState.setState((state) => ({
            ...state,
            comments: {
                items: comments.comments || [],
                total: comments.total || 0,
                page,
                isLoading: false,
                error: null,
            },
        }));

        return comments;
    } catch (error) {
        console.error('Failed to load comments:', error);

        // Update error state
        notebookState.setState((state) => ({
            ...state,
            comments: {
                ...state.comments,
                isLoading: false,
                error: `Loading failed: ${error.message}`,
            },
        }));

        return { comments: [], total: 0 };
    }
}
