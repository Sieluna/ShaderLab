import { appState } from '../state.js';
import './navbar.css';

function navItem({ label, path, isActive }) {
    const item = document.createElement('a');
    item.className = `nav-link ${isActive ? 'active' : ''}`;
    item.href = path;
    item.textContent = label;
    return item;
}

export function navbar(items) {
    const nav = document.createElement('nav');
    nav.className = 'navbar';

    const renderNavItems = (currentPath) => {
        nav.innerHTML = '';
        items.forEach((item) => {
            nav.appendChild(
                navItem({
                    ...item,
                    isActive: currentPath === item.path,
                }),
            );
        });
    };

    appState.subscribe((state) => {
        const currentPath = state.ui?.currentPath || '/';
        renderNavItems(currentPath);
    });

    nav.addEventListener('click', (e) => {
        if (e.target.matches('.nav-link')) {
            e.preventDefault();
            const path = e.target.getAttribute('href');
            appState.setState((prev) => ({
                ...prev,
                ui: {
                    ...prev.ui,
                    currentPath: path,
                },
            }));
        }
    });

    renderNavItems(appState.getState().ui?.currentPath || '/');

    return nav;
}
