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
    button.innerHTML = `
    <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <line x1="4" y1="4" x2="20" y2="20" stroke="black" stroke-width="2"/>
        <line x1="20" y1="4" x2="4" y2="20" stroke="black" stroke-width="2"/>
    </svg>
    `;

    return container;
}

function createMenuToggle() {
    const toggle = document.createElement('button');
    toggle.className = styles.menuToggle;
    toggle.setAttribute('aria-label', 'Menu');

    toggle.innerHTML = `
    <svg viewBox="0 0 100 100" width="30" height="30" xmlns="http://www.w3.org/2000/svg">
        <path class="${styles.line1}" d="M20,30 H80" stroke="currentColor" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/>
        <path class="${styles.line2}" d="M20,50 H80" stroke="currentColor" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/>
        <path class="${styles.line3}" d="M20,70 H80" stroke="currentColor" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
    `;

    return toggle;
}

export function navbar(items) {
    const navbar = document.createElement('nav');
    navbar.className = styles.navbar;

    const container = document.createElement('div');
    container.className = styles.container;

    const menuToggle = createMenuToggle();
    container.appendChild(menuToggle);

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

    const avatarContainer = createAvatar({
        onLoginClick: () => authModal.show(),
        onLogoutClick: () => auth.logout(),
        onProfileClick: (userData) => {
            console.log('Profile clicked:', userData);
        },
    });

    container.appendChild(avatarContainer);

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

            if (window.innerWidth <= 768) {
                navbar.classList.remove(styles.menuOpen);
            }
        }
    });

    menuToggle.addEventListener('click', () => {
        navbar.classList.toggle(styles.menuOpen);
    });

    window.addEventListener('resize', () => {
        if (window.innerWidth > 768 && navbar.classList.contains(styles.menuOpen)) {
            navbar.classList.remove(styles.menuOpen);
        }
    });

    navbar.appendChild(container);
    renderNavItems(appState.getState().ui?.currentPath || '/');

    return navbar;
}
