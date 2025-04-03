import styles from './avatar.module.css';
import { appState } from '../../state.js';

export function createAvatar({ onLoginClick, onLogoutClick, onProfileClick, onSettingsClick }) {
    const container = document.createElement('div');
    container.className = styles.container;

    const loginBtn = container.appendChild(document.createElement('button'));
    loginBtn.className = `${styles.btn} ${styles.btnLogin}`;
    loginBtn.textContent = 'Login';
    loginBtn.style.display = 'none';

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
            action: (button) => {
                button.addEventListener('click', () => onProfileClick?.());
            },
        },
        {
            text: 'Settings',
            action: (button) => {
                button.addEventListener('click', () => onSettingsClick?.());
            },
        },
        {
            text: 'Logout',
            action: (button) => {
                button.addEventListener('click', () => onLogoutClick?.());
            },
        },
    ];

    menuItems.forEach(({ text, action }) => {
        const button = dropdownMenu.appendChild(document.createElement('button'));
        button.textContent = text;
        action?.(button);
    });

    avatarImg.addEventListener('click', (e) => {
        if (window.innerWidth <= 480) {
            e.preventDefault();
            e.stopPropagation();
            userAvatar.classList.toggle('active');
        }
    });

    document.addEventListener('click', (e) => {
        if (window.innerWidth <= 480) {
            if (!userAvatar.contains(e.target)) {
                userAvatar.classList.remove('active');
            }
        }
    });

    window.addEventListener('resize', () => {
        userAvatar.classList.remove('active');
    });

    const updateAuthState = (state) => {
        const isAuthenticated = state.auth?.isAuthenticated || false;
        const userData = state.auth?.user || null;

        loginBtn.style.display = isAuthenticated ? 'none' : 'block';
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

    loginBtn.addEventListener('click', (e) => onLoginClick?.(e));

    return container;
}
