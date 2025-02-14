import { createState, createDerivedState } from '../state.js';
import { notebookApi } from '../api.js';

export const notebookState = createState({
    current: {
        notebook: null,
        isLoading: false,
        error: null,
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
});

export const currentNotebookState = createDerivedState(notebookState, (state) => state.current);

export const trendingNotebooksState = createDerivedState(notebookState, (state) => state.trending);

export const latestNotebooksState = createDerivedState(notebookState, (state) => state.latest);

export async function loadTrendingNotebooks() {
    const state = notebookState.getState();
    if (state.trending.isLoading) return state.trending.notebooks;

    notebookState.setState((prev) => ({
        ...prev,
        trending: {
            ...prev.trending,
            isLoading: true,
            error: null,
        },
    }));

    try {
        const { notebooks } = await notebookApi.listNotebooks(1, 6);

        notebookState.setState((state) => ({
            ...state,
            trending: {
                notebooks: notebooks || [],
                isLoading: false,
                error: null,
            },
        }));

        return notebooks || [];
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
    const state = notebookState.getState();
    if (state.current.isLoading) return state.current.notebook;

    const shouldReset = state.current.notebook?.id !== +notebookId;
    notebookState.setState((state) => ({
        ...state,
        current: {
            ...state.current,
            notebook: shouldReset ? null : state.current.notebook,
            isLoading: true,
            error: null,
            ...(shouldReset && {
                comments: { items: [], total: 0, page: 1, isLoading: false, error: null },
                versions: { items: [], total: 0, page: 1, isLoading: false, error: null },
            }),
        },
    }));

    try {
        const notebook = await notebookApi.getNotebook(+notebookId);

        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
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
        const notebook = await notebookApi.createNotebook(data);

        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                notebook,
                isLoading: false,
                error: null,
            },
        }));

        return { success: true, data: notebook };
    } catch (error) {
        console.error('Failed to create notebook:', error);
        return { success: false, error: error.message };
    }
}

export async function updateNotebook(notebookId, data) {
    try {
        const notebook = await notebookApi.updateNotebook(+notebookId, data);

        notebookState.setState((state) => {
            if (state.current.notebook && state.current.notebook.id === +notebookId) {
                return {
                    ...state,
                    current: {
                        ...state.current,
                        notebook: {
                            ...state.current.notebook,
                            ...notebook,
                        },
                    },
                };
            }
            return state;
        });

        return { success: true, data: notebook };
    } catch (error) {
        console.error('Failed to update notebook:', error);
        return { success: false, error: error.message };
    }
}

export async function deleteNotebook(notebookId) {
    try {
        await notebookApi.deleteNotebook(+notebookId);

        return { success: true };
    } catch (error) {
        console.error('Failed to delete notebook:', error);
        return { success: false, error: error.message };
    }
}

export async function loadComments(notebookId, page = 1, perPage = 10) {
    const current = notebookState.getState().current;
    if (!current.notebook || current.notebook.id !== +notebookId || current.comments.isLoading) {
        return;
    }

    notebookState.setState((state) => ({
        ...state,
        current: {
            ...state.current,
            comments: {
                ...state.current.comments,
                page,
                isLoading: true,
                error: null,
            },
        },
    }));

    try {
        const { comments, total } = await notebookApi.listComments(+notebookId, page, perPage);

        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                comments: {
                    items: comments || [],
                    total: total || 0,
                    page,
                    isLoading: false,
                    error: null,
                },
            },
        }));

        return { comments, total };
    } catch (error) {
        console.error('Failed to load comments:', error);

        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                comments: {
                    ...state.current.comments,
                    isLoading: false,
                    error: `Loading failed: ${error.message}`,
                },
            },
        }));

        return { comments: [], total: 0 };
    }
}

export async function createComment(notebookId, content) {
    const current = notebookState.getState().current;
    if (!current.notebook || current.notebook.id !== +notebookId || current.comments.isLoading) {
        return;
    }

    try {
        const comment = await notebookApi.createComment(+notebookId, content);
        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                comments: {
                    ...state.current.comments,
                    items: [comment, ...state.current.comments.items],
                    total: state.current.comments.total + 1,
                },
            },
        }));

        return { success: true, data: comment };
    } catch (error) {
        console.error('Failed to create comment:', error);
        return { success: false, error: error.message };
    }
}

export async function deleteComment(notebookId, commentId) {
    try {
        await notebookApi.deleteComment(+notebookId, +commentId);

        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                comments: {
                    ...state.current.comments,
                    items: state.current.comments.items.filter((item) => item.id !== commentId),
                    total: Math.max(0, state.current.comments.total - 1),
                },
            },
        }));

        return { success: true };
    } catch (error) {
        console.error('Failed to delete comment:', error);
        return { success: false, error: error.message };
    }
}

export async function loadVersions(notebookId, page = 1, perPage = 10) {
    const current = notebookState.getState().current;
    if (!current.notebook || current.notebook.id !== +notebookId || current.versions.isLoading) {
        return;
    }

    notebookState.setState((state) => ({
        ...state,
        current: {
            ...state.current,
            versions: {
                ...state.current.versions,
                isLoading: true,
                error: null,
                page,
            },
        },
    }));

    try {
        const { versions, total } = await notebookApi.listVersions(+notebookId, page, perPage);

        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                versions: {
                    items: versions || [],
                    total: total || 0,
                    page,
                    isLoading: false,
                    error: null,
                },
            },
        }));

        return { versions, total };
    } catch (error) {
        console.error('Failed to load versions:', error);

        notebookState.setState((state) => ({
            ...state,
            current: {
                ...state.current,
                versions: {
                    ...state.current.versions,
                    isLoading: false,
                    error: `Loading failed: ${error.message}`,
                },
            },
        }));

        return { versions: [], total: 0 };
    }
}
