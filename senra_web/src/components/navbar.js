import styles from './navbar.module.css';
import searchIcon from '../assets/search.svg?raw';
import { appState } from '../state.js';
import { createAvatar } from './avatar.js';
import { createAuthModal } from './auth-modal.js';
import { auth, user } from '../services/index.js';

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
    button.innerHTML = searchIcon;

    return container;
}

function createMenuToggle() {
    const toggle = document.createElement('button');
    toggle.className = styles.menuToggle;
    toggle.setAttribute('aria-label', 'Menu');

    toggle.innerHTML = `
    <svg viewBox="0 0 24 24" width="30" height="30" xmlns="http://www.w3.org/2000/svg">
        <path class="${styles.line1}" d="M4 6H20" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <path class="${styles.line2}" d="M4 12H20" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        <path class="${styles.line3}" d="M4 18H20" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
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
    user.getUserProfile();

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
