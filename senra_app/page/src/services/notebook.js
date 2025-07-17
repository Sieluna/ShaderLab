import { appState } from '../state.js';
import { notebookApi } from '../api.js';

/**
 * Helper to manage async operations with loading state
 */
const withLoadingState = (statePath) => async (operation) => {
    if (appState.get(`${statePath}.isLoading`)) {
        return { success: false, error: 'Already loading' };
    }

    appState.set(`${statePath}.isLoading`, true);
    appState.set(`${statePath}.error`, null);

    try {
        const result = await operation();
        appState.set(`${statePath}.isLoading`, false);
        return { success: true, ...result };
    } catch (error) {
        console.error('Operation failed:', error);
        appState.set(`${statePath}.isLoading`, false);
        appState.set(`${statePath}.error`, error.message);
        return { success: false, error: error.message };
    }
};

/**
 * Helper to reset notebook current state
 */
const resetCurrentNotebook = () => {
    appState.set('notebook.current', {
        notebook: null,
        isLoading: false,
        error: null,
        comments: {
            items: [],
            total: 0,
            page: 1,
            isLoading: false,
            hasLoaded: false,
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
};

export async function loadTrendingNotebooks() {
    const withTrendingLoading = withLoadingState('notebook.trending');

    return withTrendingLoading(async () => {
        const { notebooks } = await notebookApi.listNotebooks(1, 6);

        appState.set('notebook.trending.notebooks', notebooks || []);
        return { data: notebooks || [] };
    });
}

export async function loadNotebookDetails(notebookId) {
    const currentNotebook = appState.get('notebook.current.notebook');
    const shouldReset = currentNotebook?.id !== +notebookId;

    if (shouldReset) {
        resetCurrentNotebook();
    }

    const withCurrentLoading = withLoadingState('notebook.current');

    return withCurrentLoading(async () => {
        const notebook = await notebookApi.getNotebook(+notebookId);
        appState.set('notebook.current.notebook', notebook);
        return { data: notebook };
    });
}

export async function createNotebook(data) {
    try {
        const notebook = await notebookApi.createNotebook(data);

        appState.set('notebook.current', {
            ...appState.get('notebook.current'),
            notebook,
            isLoading: false,
            error: null,
        });

        return { success: true, data: notebook };
    } catch (error) {
        console.error('Failed to create notebook:', error);
        return { success: false, error: error.message };
    }
}

export async function updateNotebook(notebookId, data) {
    try {
        const notebook = await notebookApi.updateNotebook(+notebookId, data);

        const currentNotebook = appState.get('notebook.current.notebook');
        if (currentNotebook?.id === +notebookId) {
            appState.set('notebook.current.notebook', { ...currentNotebook, ...notebook });
        }

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
    const currentNotebook = appState.get('notebook.current.notebook');

    if (!currentNotebook || currentNotebook.id !== +notebookId) {
        return { success: false, error: 'No matching notebook' };
    }

    const withCommentsLoading = withLoadingState('notebook.current.comments');

    return withCommentsLoading(async () => {
        const { comments, total } = await notebookApi.listComments(+notebookId, page, perPage);

        appState.set('notebook.current.comments', {
            items: comments || [],
            total: total || 0,
            page,
            isLoading: false,
            hasLoaded: true,
            error: null,
        });

        return { data: { comments, total } };
    });
}

export async function createComment(notebookId, content) {
    const currentNotebook = appState.get('notebook.current.notebook');

    if (!currentNotebook || currentNotebook.id !== +notebookId) {
        return { success: false, error: 'No matching notebook' };
    }

    try {
        const comment = await notebookApi.createComment(+notebookId, content);

        const currentComments = appState.get('notebook.current.comments');
        appState.set('notebook.current.comments', {
            ...currentComments,
            items: [comment, ...currentComments.items],
            total: currentComments.total + 1,
        });

        return { success: true, data: comment };
    } catch (error) {
        console.error('Failed to create comment:', error);
        return { success: false, error: error.message };
    }
}

export async function deleteComment(notebookId, commentId) {
    try {
        await notebookApi.deleteComment(+notebookId, +commentId);

        const currentComments = appState.get('notebook.current.comments');
        appState.set('notebook.current.comments', {
            ...currentComments,
            items: currentComments.items.filter(item => item.id !== commentId),
            total: Math.max(0, currentComments.total - 1),
        });

        return { success: true };
    } catch (error) {
        console.error('Failed to delete comment:', error);
        return { success: false, error: error.message };
    }
}

export async function loadVersions(notebookId, page = 1, perPage = 10) {
    const currentNotebook = appState.get('notebook.current.notebook');

    if (!currentNotebook || currentNotebook.id !== +notebookId) {
        return { success: false, error: 'No matching notebook' };
    }

    const withVersionsLoading = withLoadingState('notebook.current.versions');

    return withVersionsLoading(async () => {
        const { versions, total } = await notebookApi.listVersions(+notebookId, page, perPage);

        appState.set('notebook.current.versions', {
            items: versions || [],
            total: total || 0,
            page,
            isLoading: false,
            error: null,
        });

        return { data: { versions, total } };
    });
}
