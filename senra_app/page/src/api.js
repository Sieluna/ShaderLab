import init, { ApiClient } from 'senra_client';
import { authService, userService } from './services/index.js';

const API_URL = __APP_API_URL__;

console.info('Current API server', API_URL);

let client = null;

const clientInitPromise = init().then(() => {
    client = new ApiClient(API_URL);
    client.wasm_load_token();
    return client;
});

clientInitPromise.then(() => {
    authService.checkAuthStatus().then(() => {
        userService.getUserProfile();
    });
});

const ensureClient = async () => {
    if (!client) {
        await clientInitPromise;
    }
    return client;
};

export const authApi = {
    login: async (username, password) => {
        const client = await ensureClient();
        return await client.wasm_login(username, password);
    },
    register: async (username, email, password) => {
        const client = await ensureClient();
        return await client.wasm_register(username, email, password);
    },
    verifyToken: async () => {
        const client = await ensureClient();
        return await client.wasm_verify_token();
    },
    logout: async () => {
        const client = await ensureClient();
        return await client.wasm_logout();
    },
};

export const userApi = {
    getSelf: async (page = 1, perPage = 10) => {
        const client = await ensureClient();
        return await client.wasm_get_self(page, perPage);
    },
    getUser: async (id, page = 1, perPage = 10) => {
        const client = await ensureClient();
        return await client.wasm_get_user(id, page, perPage);
    },
    updateUser: async (data) => {
        const client = await ensureClient();
        return await client.wasm_update_user(data);
    },
};

export const notebookApi = {
    listNotebooks: async (page = 1, perPage = 10) => {
        const client = await ensureClient();
        return await client.wasm_get_notebooks(page, perPage);
    },
    getNotebook: async (id) => {
        const client = await ensureClient();
        return await client.wasm_get_notebook(id);
    },
    createNotebook: async (data) => {
        const client = await ensureClient();
        return await client.wasm_create_notebook(data);
    },
    updateNotebook: async (id, data) => {
        const client = await ensureClient();
        return await client.wasm_update_notebook(id, data);
    },
    deleteNotebook: async (id) => {
        const client = await ensureClient();
        return await client.wasm_delete_notebook(id);
    },
    likeNotebook: async (id) => {
        const client = await ensureClient();
        return await client.wasm_like_notebook(id);
    },
    unlikeNotebook: async (id) => {
        const client = await ensureClient();
        return await client.wasm_unlike_notebook(id);
    },
    listVersions: async (id, page = 1, perPage = 10) => {
        const client = await ensureClient();
        return await client.wasm_get_notebook_versions(id, page, perPage);
    },
    listComments: async (id, page = 1, perPage = 10) => {
        const client = await ensureClient();
        return await client.wasm_get_notebook_comments(id, page, perPage);
    },
    createComment: async (id, content) => {
        const client = await ensureClient();
        return await client.wasm_create_notebook_comment(id, { content });
    },
    deleteComment: async (id, commentId) => {
        const client = await ensureClient();
        return await client.wasm_delete_notebook_comment(id, commentId);
    },
};

export const wsApi = {
    connect: async () => {
        const client = await ensureClient();
        return await client.wasm_connect_websocket();
    },

    sendMessage: async (messageType, payload) => {
        const client = await ensureClient();
        return await client.wasm_send_ws_message(messageType, payload);
    },

    receiveMessage: async () => {
        const client = await ensureClient();
        return await client.wasm_receive_ws_message();
    },

    messageListeners: new Set(),

    addMessageListener: (listener) => {
        wsApi.messageListeners.add(listener);
    },

    removeMessageListener: (listener) => {
        wsApi.messageListeners.delete(listener);
    },

    startListening: async () => {
        const client = await ensureClient();

        const listenLoop = async () => {
            try {
                while (true) {
                    const message = await client.wasm_receive_ws_message();
                    wsApi.messageListeners.forEach(listener => {
                        try {
                            listener(message);
                        } catch (error) {
                            console.error('Error in WebSocket message listener:', error);
                        }
                    });
                }
            } catch (error) {
                console.error('WebSocket listening stopped:', error);
                setTimeout(() => {
                    console.log('Attempting to restart WebSocket listening...');
                    wsApi.startListening();
                }, 5000);
            }
        };

        listenLoop();
    },

    connectAndListen: async () => {
        await wsApi.connect();
        wsApi.startListening();
    },

    sendNotification: async (payload) => {
        return await wsApi.sendMessage('notification', payload);
    },

    sendChatMessage: async (payload) => {
        return await wsApi.sendMessage('chat', payload);
    },

    sendCommand: async (command, payload) => {
        return await wsApi.sendMessage('command', { command, ...payload });
    }
};
