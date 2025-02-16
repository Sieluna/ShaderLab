import init, { JsClient } from "senra_api";

const API_URL = globalThis.__APP_API_URL__;

console.info('Current API server', API_URL);

let client = null;

init().then(() => client = new JsClient(API_URL));

export const authApi = {
    login: async (username, password) => {
        if (!client) throw new Error('WASM client not initialized');
        return await client.login(username, password);
    },
    register: async (username, email, password) => {
        if (!client) throw new Error('WASM client not initialized');
        return await client.register(username, email, password);
    },
    verifyToken: async () => {
        if (!client) throw new Error('WASM client not initialized');
        return await client.verify_token();
    },
    logout: async () => {
        if (!client) throw new Error('WASM client not initialized');
        return await client.logout();
    },
};

export const userApi = {
    getSelf: async (page = 1, perPage = 10) => {
        const url = new URL('/user', API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });

        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${client?.token ?? ''}` },
        });
        if (!response.ok) throw new Error(`Get self failed: ${response.status}`);
        return response.json();
    },
    getUser: async (id, page = 1, perPage = 10) => {
        const url = new URL(`/user/${id}`, API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });

        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${client?.token ?? ''}` },
        });
        if (!response.ok) throw new Error(`Get user failed: ${response.status}`);
        return response.json();
    },
    updateUser: async (data) => {
        const response = await fetch(new URL('/user', API_URL), {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${client?.token ?? ''}`,
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
            headers: { Authorization: `Bearer ${client?.token ?? ''}` },
        });
        if (!response.ok) throw new Error(`Get notebook failed: ${response.status}`);
        return response.json();
    },
    createNotebook: async (data) => {
        const response = await fetch(new URL('/notebooks', API_URL), {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${client?.token ?? ''}`,
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
                Authorization: `Bearer ${client?.token ?? ''}`,
            },
            body: JSON.stringify(data),
        });
        if (!response.ok) throw new Error(`Update notebook failed: ${response.status}`);
        return response.json();
    },
    deleteNotebook: async (id) => {
        const response = await fetch(new URL(`/notebooks/${id}`, API_URL), {
            method: 'DELETE',
            headers: { Authorization: `Bearer ${client?.token ?? ''}` },
        });
        if (!response.ok) throw new Error(`Delete notebook failed: ${response.status}`);
        return response.json();
    },
    listComments: async (id, page = 1, perPage = 10) => {
        const url = new URL(`/notebooks/${id}/comments`, API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });
        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${client?.token ?? ''}` },
        });
        if (!response.ok) throw new Error(`List comments failed: ${response.status}`);
        return response.json();
    },
    createComment: async (id, content) => {
        const response = await fetch(new URL(`/notebooks/${id}/comments`, API_URL), {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${client?.token ?? ''}`,
            },
            body: JSON.stringify({ content }),
        });
        if (!response.ok) throw new Error(`Create comment failed: ${response.status}`);
        return response.json();
    },
    deleteComment: async (id, commentId) => {
        const response = await fetch(new URL(`/notebooks/${id}/comments/${commentId}`, API_URL), {
            method: 'DELETE',
            headers: { Authorization: `Bearer ${client?.token ?? ''}` },
        });
        if (!response.ok) throw new Error(`Delete comment failed: ${response.status}`);
        return response.json();
    },
    listVersions: async (id, page = 1, perPage = 10) => {
        const url = new URL(`/notebooks/${id}/versions`, API_URL);
        url.search = new URLSearchParams({ page, per_page: perPage });
        const response = await fetch(url, {
            headers: { Authorization: `Bearer ${client?.token ?? ''}` },
        });
        if (!response.ok) throw new Error(`List versions failed: ${response.status}`);
        return response.json();
    },
};
