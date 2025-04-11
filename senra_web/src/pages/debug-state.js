import styles from './debug.module.css';
import { appState } from '../state.js';
import { notebookService, authService, userService } from '../services/index.js';
import { deepDiff, formatContent, updateTestResult } from './debug.js';

function createStateDisplay(id, state) {
    const display = document.createElement('div');
    display.id = id;
    display.className = styles.display;

    const toggle = display.appendChild(document.createElement('button'));
    toggle.className = styles.toggle;
    toggle.textContent = '▼';

    const timestamp = display.appendChild(document.createElement('div'));
    timestamp.className = styles.time;

    const content = display.appendChild(document.createElement('pre'));
    let previousState = state.getState();

    const updateContent = (newState) => {
        const diffs = deepDiff(previousState, newState);
        previousState = newState;
        content.innerHTML = formatContent(newState, diffs);
        timestamp.textContent = `Last Update: ${new Date().toLocaleTimeString()}`;
    };

    toggle.addEventListener('click', () => {
        content.style.display = content.style.display === 'none' ? 'block' : 'none';
        toggle.textContent = content.style.display === 'none' ? '▶' : '▼';
    });

    state.subscribe(updateContent);
    updateContent(state.getState());

    return display;
}

function createInputForm(id, fields, resultId, submitAction) {
    const form = document.createElement('form');
    form.id = id;

    fields.forEach((field) => {
        const container = form.appendChild(document.createElement('div'));
        container.className = styles.field;

        const label = container.appendChild(document.createElement('label'));
        label.htmlFor = `${id}-${field.name}`;
        label.textContent = field.label;

        const input = container.appendChild(document.createElement('input'));
        input.type = field.type || 'text';
        input.id = `${id}-${field.name}`;
        input.name = field.name;
        input.value = field.value || '';
        input.required = field.required || false;
        input.placeholder = field.placeholder || '';
    });

    const submitBtn = form.appendChild(document.createElement('button'));
    submitBtn.type = 'submit';
    submitBtn.textContent = 'Submit';

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const start = performance.now();
        try {
            const formData = {};
            fields.forEach((field) => {
                formData[field.name] = form.querySelector(`#${id}-${field.name}`).value;
            });
            const result = await submitAction(formData);
            updateTestResult(resultId, { ...result, duration: performance.now() - start });
        } catch (error) {
            updateTestResult(resultId, { error: error.message });
        }
    });

    return form;
}

function createTestSection(title, tests) {
    const section = document.createElement('div');
    section.className = styles.section;
    const resultId = `${title.toLowerCase().replace(/\s+/g, '-')}-result`;

    const controls = tests
        .filter((test) => !test.formFields)
        .map((test) => `<button id="${test.id}">${test.label}</button>`)
        .join('');

    section.innerHTML = `
        <h2>${title}</h2>
        <div class="${styles.controls}">${controls}</div>
        <div id="${resultId}" class="${styles.result}"></div>
    `;

    tests.forEach((test) => {
        if (test.formFields) {
            const container = document.createElement('div');
            container.className = styles.form;

            const title = container.appendChild(document.createElement('div'));
            title.className = styles.title;
            title.textContent = test.label;

            container.appendChild(
                createInputForm(`${test.id}-form`, test.formFields, resultId, test.action),
            );

            section.querySelector(`.${styles.controls}`).appendChild(container);
        } else {
            section.querySelector(`#${test.id}`).addEventListener('click', async () => {
                const start = performance.now();
                try {
                    const result = await test.action();
                    updateTestResult(resultId, { ...result, duration: performance.now() - start });
                } catch (error) {
                    updateTestResult(resultId, { error: error.message });
                }
            });
        }
    });

    return section;
}

export function createStateTest() {
    const container = document.createElement('div');
    container.className = styles.section;

    const stateMonitor = document.createElement('div');
    stateMonitor.className = styles.stateMonitor;
    stateMonitor.innerHTML = '<h2>Real-time State Monitor</h2>';
    [
        { id: 'app-state', state: appState },
        { id: 'notebook-state', state: notebookService.notebookState },
    ].forEach(({ id, state }) => stateMonitor.appendChild(createStateDisplay(id, state)));

    const testConfig = {
        auth: [
            {
                id: 'test-login',
                label: 'User Login',
                formFields: [
                    { name: 'username', label: 'Username', value: 'test_user', required: true },
                    { name: 'password', label: 'Password', type: 'password', value: 'test_password', required: true },
                ],
                action: ({ username, password }) => authService.login(username, password),
            },
            {
                id: 'test-register',
                label: 'User Registration',
                formFields: [
                    { name: 'username', label: 'Username', value: 'test_user', required: true },
                    { name: 'email', label: 'Email', type: 'email', value: 'test_email@test.com', required: true },
                    { name: 'password', label: 'Password', type: 'password', value: 'test_password', required: true },
                ],
                action: ({ username, email, password }) => authService.register(username, email, password),
            },
            {
                id: 'test-check-auth',
                label: 'Check Authentication Status',
                action: authService.checkAuthStatus,
            },
        ],
        user: [
            {
                id: 'test-get-user',
                label: 'Get User Profile',
                formFields: [{ name: 'userId', label: 'User ID' }],
                action: ({ userId }) => userService.getUserProfile(userId),
            },
            {
                id: 'test-update-profile',
                label: 'Update Profile',
                formFields: [
                    { name: 'username', label: 'New Username', value: 'test_user_updated' },
                    { name: 'email', label: 'New Email', type: 'email', value: 'test_email_updated@test.com' },
                    { name: 'password', label: 'New Password', type: 'password', value: 'test_password_updated' },
                ],
                action: (data) => userService.updateUserProfile(data),
            },
        ],
        notebook: [
            {
                id: 'test-create-notebook',
                label: 'Create Notebook',
                formFields: [
                    { name: 'title', label: 'Title', value: 'Test Notebook', required: true },
                    { name: 'description', label: 'Description', value: 'This is a test notebook' },
                    { name: 'visibility', label: 'Visibility', value: 'public', placeholder: 'public/private' },
                ],
                action: (data) =>
                    notebookService.createNotebook({
                        ...data,
                        content: JSON.stringify({ cells: [] }),
                        tags: ['Test', 'Example'],
                        resources: [],
                        shaders: [],
                    }),
            },
            {
                id: 'test-update-notebook',
                label: 'Update Notebook',
                formFields: [
                    { name: 'id', label: 'Notebook ID', value: '', required: true },
                    { name: 'title', label: 'New Title', value: 'Updated Test Notebook' },
                    { name: 'description', label: 'New Description', value: 'This is an updated test notebook' },
                    { name: 'visibility', label: 'New Visibility', value: 'private' },
                ],
                action: ({ id, ...data }) => notebookService.updateNotebook(id, data),
            },
            {
                id: 'test-delete-notebook',
                label: 'Delete Notebook',
                formFields: [{ name: 'id', label: 'Notebook ID', value: '', required: true }],
                action: ({ id }) => notebookService.deleteNotebook(id),
            },
            {
                id: 'test-get-notebook',
                label: 'Get Notebook Details',
                formFields: [{ name: 'id', label: 'Notebook ID', value: '', required: true }],
                action: ({ id }) => notebookService.loadNotebookDetails(id),
            },
            {
                id: 'test-get-trending',
                label: 'Get Trending Notebooks',
                action: () => notebookService.loadTrendingNotebooks(),
            },
            {
                id: 'test-get-versions',
                label: 'Get Notebook Versions',
                formFields: [{ name: 'id', label: 'Notebook ID', value: '', required: true }],
                action: ({ id }) => notebookService.loadVersions(id),
            },
            {
                id: 'test-create-comment',
                label: 'Add Comment',
                formFields: [
                    { name: 'notebookId', label: 'Notebook ID', value: '', required: true },
                    { name: 'content', label: 'Comment Content', value: 'This is a test comment', required: true },
                ],
                action: ({ notebookId, content }) => notebookService.createComment(notebookId, content),
            },
            {
                id: 'test-get-comments',
                label: 'Get Comments',
                formFields: [
                    { name: 'notebookId', label: 'Notebook ID', value: '', required: true },
                ],
                action: ({ notebookId }) => notebookService.loadComments(notebookId),
            },
            {
                id: 'test-delete-comment',
                label: 'Delete Comment',
                formFields: [
                    { name: 'notebookId', label: 'Notebook ID', value: '', required: true },
                    { name: 'commentId', label: 'Comment ID', value: '', required: true },
                ],
                action: ({ notebookId, commentId }) => notebookService.deleteComment(notebookId, commentId),
            },
        ],
    };

    container.appendChild(createTestSection('Authentication', testConfig.auth));
    container.appendChild(createTestSection('User Service', testConfig.user));
    container.appendChild(createTestSection('Notebook Service', testConfig.notebook));
    container.appendChild(stateMonitor);

    return container;
}
