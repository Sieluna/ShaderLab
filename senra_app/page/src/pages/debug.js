import styles from './debug.module.css';
import { createStateTest } from './debug-state.js';
import { createRendererTest } from './debug-notebook.js';

export function deepDiff(prev, curr, path = '') {
    const diffs = [];
    if (typeof prev !== 'object' || typeof curr !== 'object' || prev === null || curr === null) {
        if (prev !== curr) {
            diffs.push({ path, prev, curr });
        }
        return diffs;
    }
    if (Array.isArray(prev) || Array.isArray(curr)) {
        if (JSON.stringify(prev) !== JSON.stringify(curr)) {
            diffs.push({ path, prev, curr });
        }
        return diffs;
    }
    const allKeys = new Set([...Object.keys(prev), ...Object.keys(curr)]);
    for (const key of allKeys) {
        const currentPath = path ? `${path}.${key}` : key;
        const prevVal = prev.hasOwnProperty(key) ? prev[key] : undefined;
        const currVal = curr.hasOwnProperty(key) ? curr[key] : undefined;
        diffs.push(...deepDiff(prevVal, currVal, currentPath));
    }
    return diffs;
}

export function formatContent(content, diffs = []) {
    const json = JSON.stringify(
        content,
        (_, value) => {
            if (Array.isArray(value) && value.length > 5) {
                return {
                    __collapsed: true,
                    length: value.length,
                    preview: value.slice(0, 5),
                };
            }
            return value;
        },
        2,
    );

    return json.replace(
        /{\s*"__collapsed":\s*true,\s*"length":\s*(\d+),\s*"preview":\s*(\[[\s\S]*?\])\s*}/g,
        (_match, len, preview) =>
            `<span class="collapsed-array" data-len="${len}" data-preview='${preview}'>[${preview.slice(1, -1)} … <em>${len} items</em>]</span>`,
    );
}

export function updateTestResult(elementId, result) {
    const element = document.getElementById(elementId);
    if (element) {
        element.classList.add(styles.updated);
        setTimeout(() => element.classList.remove(styles.updated), 500);

        const statusColor = result?.error ? '#ff4444' : '#44ff44';
        element.style.borderLeft = `4px solid ${statusColor}`;

        element.innerHTML = `
            <div class="${styles.meta}">
                <span>${new Date().toLocaleTimeString()}</span>
                ${result?.duration ? `<span>Duration: ${result.duration}ms</span>` : ''}
            </div>
            <pre>${formatContent(result)}</pre>
        `;
        element.scrollIntoView({ behavior: 'smooth' });
    }
}

export function debugPage() {
    const container = document.createElement('div');
    container.className = styles.container;

    const nav = container.appendChild(document.createElement('nav'));
    nav.className = styles.nav;

    const content = container.appendChild(document.createElement('div'));
    content.className = styles.tabs;

    const stateContent = content.appendChild(document.createElement('div'));
    stateContent.className = styles.content;
    stateContent.id = 'tab-state';
    stateContent.appendChild(createStateTest());

    const notebookContent = content.appendChild(document.createElement('div'));
    notebookContent.className = styles.content;
    notebookContent.id = 'tab-notebook';
    notebookContent.appendChild(createRendererTest());

    const tabs = [
        { id: 'state', label: 'State Test', contentId: 'tab-state' },
        { id: 'notebook', label: 'Notebook Test', contentId: 'tab-notebook' },
    ];

    let activeTab = tabs[0].id;

    tabs.forEach((tab) => {
        const button = nav.appendChild(document.createElement('button'));
        button.textContent = tab.label;
        button.className = activeTab === tab.id ? styles.active : '';
        button.dataset.tabId = tab.id;

        stateContent.classList.toggle(styles.active, activeTab === 'state');
        notebookContent.classList.toggle(styles.active, activeTab === 'notebook');

        button.addEventListener('click', () => {
            activeTab = tab.id;

            nav.querySelectorAll('button').forEach((btn) => {
                btn.classList.toggle(styles.active, btn.dataset.tabId === activeTab);
            });

            stateContent.classList.toggle(styles.active, activeTab === 'state');
            notebookContent.classList.toggle(styles.active, activeTab === 'notebook');
        });
    });

    return container;
}
