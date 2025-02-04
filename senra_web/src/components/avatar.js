import styles from './avatar.module.css';
import { appState } from '../state.js';

export function createAvatar({ onLoginClick, onLogoutClick, onProfileClick, onSettingsClick }) {
    const container = document.createElement('div');
    container.className = styles.container;

    const loginButton = container.appendChild(document.createElement('button'));
    loginButton.className = `${styles.btn} ${styles.btnLogin}`;
    loginButton.textContent = 'Login';
    loginButton.style.display = 'none';

    const userAvatar = container.appendChild(document.createElement('div'));
    userAvatar.className = styles.avatar;
    userAvatar.style.display = 'none';

    const avatarImg = userAvatar.appendChild(document.createElement('img'));
    Object.assign(avatarImg, {
        src: '/img/default-avatar.png',
        alt: 'Avatar',
        id: 'userAvatar',
    });

    const dropdownMenu = userAvatar.appendChild(document.createElement('div'));
    dropdownMenu.className = styles.dropdown;

    const menuItems = [
        {
            text: 'Profile',
            callback: (button) => {
                button.addEventListener('click', () => onProfileClick?.());
            },
        },
        {
            text: 'Settings',
            callback: (button) => {
                button.addEventListener('click', () => onSettingsClick?.());
            },
        },
        {
            text: 'Logout',
            callback: (button) => {
                button.addEventListener('click', () => onLogoutClick?.());
            },
        },
    ];

    menuItems.forEach(({ text, callback }) => {
        const button = dropdownMenu.appendChild(document.createElement('button'));
        button.textContent = text;
        callback?.(button);
    });

    const updateAuthState = (state) => {
        const isAuthenticated = state.auth?.isAuthenticated || false;
        const userData = state.auth?.user || null;

        loginButton.style.display = isAuthenticated ? 'none' : 'block';
        userAvatar.style.display = isAuthenticated ? 'block' : 'none';

        if (isAuthenticated && userData?.avatar) {
            if (Array.isArray(userData.avatar)) {
                const blob = new Blob([new Uint8Array(userData.avatar)], { type: 'image/png' });
                const url = URL.createObjectURL(blob);
                avatarImg.src = url;
                avatarImg.dataset.blobUrl = url;
            } else {
                avatarImg.src = userData.avatar;
            }
        } else {
            if (avatarImg.dataset.blobUrl) {
                URL.revokeObjectURL(avatarImg.dataset.blobUrl);
                delete avatarImg.dataset.blobUrl;
            }
            avatarImg.src = '/img/default-avatar.png';
        }
    };

    updateAuthState(appState.getState().auth);
    appState.subscribe(updateAuthState);

    loginButton.addEventListener('click', (e) => onLoginClick?.(e));

    return container;
}
