import axios from 'axios';

const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3000";

console.info("Current API server", API_URL);

const api = axios.create({
    baseURL: API_URL,
    headers: {
        'Content-Type': 'application/json',
    }
});

api.interceptors.request.use(config => {
    const token = localStorage.getItem('token');
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
});

export const authApi = {
    login: (username, password) => api.post('/auth/login', { username, password }),
    register: (username, email, password) => api.post('/auth/register', { username, email, password }),
    verifyToken: (token) => api.post('/auth/verify', { token }),
};

export const userApi = {
    getSelf: (page = 1, perPage = 10) => api.get('/user', { params: { page, per_page: perPage } }),
    getUser: (id, page = 1, perPage = 10) => api.get(`/user/${id}`, { params: { page, per_page: perPage } }),
    updateUser: (data) => api.patch('/user', data),
};

export const notebookApi = {
    listNotebooks: (page = 1, perPage = 10) => api.get('/notebooks', { params: { page, per_page: perPage } }),
    getNotebook: (id) => api.get(`/notebooks/${id}`),
    createNotebook: (data) => api.post('/notebooks', data),
    updateNotebook: (id, data) => api.patch(`/notebooks/${id}`, data),
    deleteNotebook: (id) => api.delete(`/notebooks/${id}`),
    listComments: (id, page = 1, perPage = 10) => api.get(`/notebooks/${id}/comments`, { params: { page, per_page: perPage } }),
    createComment: (id, content) => api.post(`/notebooks/${id}/comments`, { content }),
    deleteComment: (id, commentId) => api.delete(`/notebooks/${id}/comments/${commentId}`),
    listVersions: (id, page = 1, perPage = 10) => api.get(`/notebooks/${id}/versions`, { params: { page, per_page: perPage } }),
};

export default api; 