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
    versions: {
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
    notebookState.setState((state) => ({
        ...state,
        current: {
            ...state.current,
            isLoading: true,
            error: null,
        },
        comments: {
            items: [],
            total: 0,
            page: 1,
            isLoading: false,
            error: null,
        },
        versions: {
            items: [],
            total: 0,
            page: 1,
            isLoading: false,
            error: null,
        },
    }));

    try {
        const notebook = await notebookApi.getNotebook(notebookId);

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

export async function createNotebook(data) {
    try {
        const response = await notebookApi.createNotebook(data);
        return { success: true, data: response };
    } catch (error) {
        console.error('Failed to create notebook:', error);
        return { success: false, error: error.message };
    }
}

export async function updateNotebook(notebookId, data) {
    try {
        const response = await notebookApi.updateNotebook(notebookId, data);

        notebookState.setState((state) => {
            if (state.current.notebook && state.current.notebook.id === notebookId) {
                return {
                    ...state,
                    current: {
                        ...state.current,
                        notebook: {
                            ...state.current.notebook,
                            ...response,
                        },
                    },
                };
            }
            return state;
        });

        return { success: true, data: response };
    } catch (error) {
        console.error('Failed to update notebook:', error);
        return { success: false, error: error.message };
    }
}

export async function deleteNotebook(notebookId) {
    try {
        await notebookApi.deleteNotebook(notebookId);

        return { success: true };
    } catch (error) {
        console.error('Failed to delete notebook:', error);
        return { success: false, error: error.message };
    }
}

export async function loadComments(notebookId, page = 1, perPage = 10) {
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
        const comments = await notebookApi.listComments(notebookId, page, perPage);

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

export async function createComment(notebookId, content) {
    try {
        const response = await notebookApi.createComment(notebookId, { content });

        notebookState.setState((state) => ({
            ...state,
            comments: {
                ...state.comments,
                items: [response, ...state.comments.items],
                total: state.comments.total + 1,
            },
        }));

        return { success: true, data: response };
    } catch (error) {
        console.error('Failed to create comment:', error);
        return { success: false, error: error.message };
    }
}

export async function deleteComment(notebookId, commentId) {
    try {
        await notebookApi.deleteComment(notebookId, commentId);

        notebookState.setState((state) => ({
            ...state,
            comments: {
                ...state.comments,
                items: state.comments.items.filter((item) => item.id !== commentId),
                total: Math.max(0, state.comments.total - 1),
            },
        }));

        return { success: true };
    } catch (error) {
        console.error('Failed to delete comment:', error);
        return { success: false, error: error.message };
    }
}

export async function loadVersions(notebookId, page = 1, perPage = 10) {
    notebookState.setState((state) => ({
        ...state,
        versions: {
            ...state.versions,
            isLoading: true,
            error: null,
            page,
        },
    }));

    try {
        const response = await notebookApi.listVersions(notebookId, page, perPage);

        notebookState.setState((state) => ({
            ...state,
            versions: {
                items: response.versions || [],
                total: response.total || 0,
                page,
                isLoading: false,
                error: null,
            },
        }));

        return response;
    } catch (error) {
        console.error('Failed to load versions:', error);

        notebookState.setState((state) => ({
            ...state,
            versions: {
                ...state.versions,
                isLoading: false,
                error: `Loading failed: ${error.message}`,
            },
        }));

        return { versions: [], total: 0 };
    }
}
