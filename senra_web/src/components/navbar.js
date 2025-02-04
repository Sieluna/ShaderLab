import styles from './navbar.module.css';
import { appState } from '../state.js';
import { createAvatar } from './avatar.js';
import { createAuthModal } from './auth-modal.js';
import { auth } from '../services/index.js';

function createNavItem({ label, path, isActive }) {
    const item = document.createElement('a');
    item.className = `${styles.navLink} ${isActive ? styles.active : ''}`;
    item.href = path;
    item.textContent = label;
    return item;
}

function createSearchBox() {
    const container = document.createElement('div');
    container.className = styles.searchBox;

    const input = container.appendChild(document.createElement('input'));
    input.type = 'text';
    input.placeholder = 'Search Notebook...';

    const button = container.appendChild(document.createElement('button'));
    button.innerHTML = '<i class="icon-search"></i>';

    return container;
}

export function navbar(items) {
    const navbar = document.createElement('nav');
    navbar.className = styles.navbar;

    const container = document.createElement('div');
    container.className = styles.container;

    const navList = container.appendChild(document.createElement('ul'));
    const renderNavItems = (currentPath) => {
        navList.innerHTML = '';
        items.forEach((item) => {
            const li = navList.appendChild(document.createElement('li'));
            li.appendChild(
                createNavItem({
                    ...item,
                    isActive: currentPath === item.path,
                }),
            );
        });
    };

    container.appendChild(createSearchBox());

    auth.checkAuthStatus();

    const authModal = createAuthModal({
        onLogin: ({ username, password }) => {
            auth.login(username, password);

            authModal.hide();
            authModal.reset();
        },
        onRegister: ({ username, email, password }) => {
            auth.register(username, email, password);

            authModal.hide();
            authModal.reset();
        },
    });
    document.body.appendChild(authModal.element);

    container.appendChild(
        createAvatar({
            onLoginClick: () => authModal.show(),
            onLogoutClick: () => auth.logout(),
            onProfileClick: (userData) => {
                console.log('Profile clicked:', userData);
            },
        }),
    );

    appState.subscribe((state) => {
        const currentPath = state.ui?.currentPath || '/';
        renderNavItems(currentPath);
    });

    navbar.addEventListener('click', (e) => {
        if (e.target.matches(`.${styles.navLink}`)) {
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

    navbar.appendChild(container);
    renderNavItems(appState.getState().ui?.currentPath || '/');

    return navbar;
}
