import { authState } from './state';

const API_URL = import.meta.env.VITE_API_URL ?? 'http://localhost:3000';

console.info('Current API server', API_URL);

export const authApi = {
    login: async (username, password) => {
        const response = await fetch(new URL('/auth/login', API_URL), {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password }),
        });
        if (!response.ok) throw new Error(`Login failed: ${response.status}`);
        return response.json();
    },
    register: async (username, email, password) => {
        const response = await fetch(new URL('/auth/register', API_URL), {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, email, password }),
        });
        if (!response.ok) throw new Error(`Register failed: ${response.status}`);
        return response.json();
    },
    verifyToken: async (token) => {
        const response = await fetch(new URL('/auth/verify', API_URL), {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ token }),
        });
        if (!response.ok) throw new Error(`Verify token failed: ${response.status}`);
        return response.json();
    },
};

export const userApi = {
    getSelf: async (page = 1, perPage = 10) => {
        const url = new URL('/user', API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });

        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${authState.getState().token || ''}` },
        });
        if (!response.ok) throw new Error(`Get self failed: ${response.status}`);
        return response.json();
    },
    getUser: async (id, page = 1, perPage = 10) => {
        const url = new URL(`/user/${id}`, API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });

        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${authState.getState().token || ''}` },
        });
        if (!response.ok) throw new Error(`Get user failed: ${response.status}`);
        return response.json();
    },
    updateUser: async (data) => {
        const response = await fetch(new URL('/user', API_URL), {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${authState.getState().token || ''}`,
            },
            body: JSON.stringify(data),
        });
        if (!response.ok) throw new Error(`Update user failed: ${response.status}`);
        return response.json();
    },
};

export const notebookApi = {
    listNotebooks: async (page = 1, perPage = 10) => {
        const url = new URL('/notebooks', API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });
        const response = await fetch(url);
        if (!response.ok) throw new Error(`List notebooks failed: ${response.status}`);
        return response.json();
    },
    getNotebook: async (id) => {
        const response = await fetch(new URL(`/notebooks/${id}`, API_URL), {
            headers: { Authorization: `Bearer ${authState.getState().token || ''}` },
        });
        if (!response.ok) throw new Error(`Get notebook failed: ${response.status}`);
        return response.json();
    },
    createNotebook: async (data) => {
        const response = await fetch(new URL('/notebooks', API_URL), {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${authState.getState().token || ''}`,
            },
            body: JSON.stringify(data),
        });
        if (!response.ok) throw new Error(`Create notebook failed: ${response.status}`);
        return response.json();
    },
    updateNotebook: async (id, data) => {
        const response = await fetch(new URL(`/notebooks/${id}`, API_URL), {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${authState.getState().token || ''}`,
            },
            body: JSON.stringify(data),
        });
        if (!response.ok) throw new Error(`Update notebook failed: ${response.status}`);
        return response.json();
    },
    deleteNotebook: async (id) => {
        const response = await fetch(new URL(`/notebooks/${id}`, API_URL), {
            method: 'DELETE',
            headers: { Authorization: `Bearer ${authState.getState().token || ''}` },
        });
        if (!response.ok) throw new Error(`Delete notebook failed: ${response.status}`);
        return response.json();
    },
    listComments: async (id, page = 1, perPage = 10) => {
        const url = new URL(`/notebooks/${id}/comments`, API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });
        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${authState.getState().token || ''}` },
        });
        if (!response.ok) throw new Error(`List comments failed: ${response.status}`);
        return response.json();
    },
    createComment: async (id, content) => {
        const response = await fetch(new URL(`/notebooks/${id}/comments`, API_URL), {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${authState.getState().token || ''}`,
            },
            body: JSON.stringify({ content }),
        });
        if (!response.ok) throw new Error(`Create comment failed: ${response.status}`);
        return response.json();
    },
    deleteComment: async (id, commentId) => {
        const response = await fetch(new URL(`/notebooks/${id}/comments/${commentId}`, API_URL), {
            method: 'DELETE',
            headers: { Authorization: `Bearer ${authState.getState().token || ''}` },
        });
        if (!response.ok) throw new Error(`Delete comment failed: ${response.status}`);
        return response.json();
    },
    listVersions: async (id, page = 1, perPage = 10) => {
        const url = new URL(`/notebooks/${id}/versions`, API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });
        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${authState.getState().token || ''}` },
        });
        if (!response.ok) throw new Error(`List versions failed: ${response.status}`);
        return response.json();
    },
};
