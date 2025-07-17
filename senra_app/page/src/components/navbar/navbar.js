import styles from './navbar.module.css';
import searchIcon from '../../assets/search.svg?raw';
import { appState } from '../../state.js';
import { createAvatar } from './avatar.js';
import { createAuthModal } from './auth-modal.js';
import { authService } from '../../services/index.js';
import { createLinkHandler } from '../../utils/router.js';

function createSearchBox() {
    const container = document.createElement('div');
    container.className = styles.searchBox;

    const input = container.appendChild(document.createElement('input'));
    input.type = 'text';
    input.placeholder = 'Search Notebook...';

    const button = container.appendChild(document.createElement('button'));
    button.innerHTML = searchIcon;

    return {
        element: container,
    };
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

    const menu = createMenuToggle();

    const navList = document.createElement('ul');
    const renderNavItems = (currentPath) => {
        navList.innerHTML = '';
        items.forEach(({ label, path }) => {
            const li = navList.appendChild(document.createElement('li'));
            const a = li.appendChild(document.createElement('a'));
            a.className = `${styles.navLink} ${currentPath === path ? styles.active : ''}`;
            a.href = path;
            a.textContent = label;
        });
    };

    const search = createSearchBox();

    const modal = createAuthModal({
        onLogin: ({ username, password }) => {
            authService.login(username, password).then((result) => {
                if (result.success) {
                    modal.hide();
                    modal.reset();
                } else {
                    modal.setError(result.error);
                }
            });
        },
        onRegister: ({ username, email, password }) => {
            authService.register(username, email, password).then((result) => {
                if (result.success) {
                    modal.hide();
                    modal.reset();
                } else {
                    modal.setError(result.error);
                }
            });
        },
    });

    const avatar = createAvatar({
        onLoginClick: () => modal.show(),
        onLogoutClick: () => authService.logout(),
        onProfileClick: (userData) => {
            console.log('Profile clicked:', userData);
        },
    });

    container.append(menu, navList, search.element, avatar.element);
    document.body.appendChild(modal.element);

    appState.subscribe('ui.currentPath', (currentPath) => {
        renderNavItems(currentPath || '/');
    });

    const linkHandler = createLinkHandler({
        closeMenu: () => navbar.classList.remove(styles.open)
    });
    
    navbar.addEventListener('click', linkHandler);

    menu.addEventListener('click', () => {
        navbar.classList.toggle(styles.open);
    });

    window.addEventListener('resize', () => {
        if (window.innerWidth > 768 && navbar.classList.contains(styles.open)) {
            navbar.classList.remove(styles.open);
        }
    });

    navbar.appendChild(container);

    renderNavItems(appState.get('ui.currentPath') || '/');

    return navbar;
}
