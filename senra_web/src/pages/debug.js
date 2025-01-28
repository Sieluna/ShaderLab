import './debug.css';
import { appState, authState, uiState } from '../state.js';
import { notebookState } from '../services/notebook.js';
import { login, register, checkAuthStatus } from '../services/auth.js';
import { loadTrendingNotebooks, loadNotebookDetails, loadComments } from '../services/notebook.js';

// 基础测试组件
const createTestSection = (title, tests) => {
    const section = document.createElement('div');
    section.className = 'test-section';

    const controls = tests
        .map(
            (test) => `
        <button id="${test.id}">${test.label}</button>
    `,
        )
        .join('');

    section.innerHTML = `
        <h2>${title}</h2>
        <div class="test-controls">
            ${controls}
        </div>
        <div id="${title.toLowerCase().replace(/\s+/g, '-')}-result" class="test-result"></div>
    `;

    tests.forEach((test) => {
        section.querySelector(`#${test.id}`).addEventListener('click', async () => {
            const result = await test.action();
            updateTestResult(`${title.toLowerCase().replace(/\s+/g, '-')}-result`, result);
        });
    });

    return section;
};

// 状态显示组件
const createStateDisplay = (id, state) => {
    const display = document.createElement('div');
    display.id = id;
    display.className = 'state-display';

    // 初始状态显示
    display.textContent = JSON.stringify(state.getState(), null, 2);

    // 订阅状态更新
    state.subscribe((newState) => {
        display.textContent = JSON.stringify(newState, null, 2);
    });

    return display;
};

// 状态监控组件
const createStateMonitor = () => {
    const section = document.createElement('div');
    section.className = 'state-monitor';

    section.innerHTML = '<h2>State Monitor</h2>';

    // 创建状态显示组件
    const states = [
        { id: 'app-state', state: appState },
        { id: 'auth-state', state: authState },
        { id: 'ui-state', state: uiState },
        { id: 'notebook-state', state: notebookState },
    ];

    states.forEach(({ id, state }) => {
        section.appendChild(createStateDisplay(id, state));
    });

    return section;
};

// 辅助函数
const updateTestResult = (elementId, result) => {
    const element = document.getElementById(elementId);
    if (element) {
        element.textContent = JSON.stringify(result, null, 2);
    }
};

// 主组件
export function StateTest() {
    const container = document.createElement('div');
    container.className = 'state-test';

    // 认证测试配置
    const authTests = [
        {
            id: 'test-login',
            label: 'Test Login',
            action: () => login('testuser', 'password123'),
        },
        {
            id: 'test-register',
            label: 'Test Register',
            action: () => register('testuser', 'test@example.com', 'password123'),
        },
        {
            id: 'test-check-auth',
            label: 'Check Auth Status',
            action: checkAuthStatus,
        },
    ];

    // 笔记本测试配置
    const notebookTests = [
        {
            id: 'test-trending',
            label: 'Load Trending',
            action: loadTrendingNotebooks,
        },
        {
            id: 'test-notebook',
            label: 'Load Notebook Details',
            action: () => loadNotebookDetails('1'),
        },
        {
            id: 'test-comments',
            label: 'Load Comments',
            action: () => loadComments('1'),
        },
    ];

    // 创建测试部分
    const authSection = createTestSection('Authentication State Test', authTests);
    const notebookSection = createTestSection('Notebook State Test', notebookTests);
    const stateMonitor = createStateMonitor();

    container.appendChild(authSection);
    container.appendChild(notebookSection);
    container.appendChild(stateMonitor);

    return container;
}

export function debugPage() {
    const debugContainer = document.createElement('div');
    const debugComponent = StateTest();
    debugContainer.appendChild(debugComponent);
    return debugContainer;
}
