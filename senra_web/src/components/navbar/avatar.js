import styles from './avatar.module.css';
import { appState } from '../../state.js';

const WHITE_AVATAR =
    'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAAXNSR0IArs4c6QAAAA1JREFUGFdj+P///38ACfsD/QVDRcoAAAAASUVORK5CYII=';

function createLoginButton({ onLoginClick }) {
    const button = document.createElement('button');
    button.className = styles.button;
    button.textContent = 'Login';
    button.style.display = 'none';
    button.addEventListener('click', (e) => onLoginClick?.(e));

    return {
        element: button,
        show: () => (button.style.display = 'block'),
        hide: () => (button.style.display = 'none'),
    };
}

function createAvatarButton({ onProfileClick, onSettingsClick, onLogoutClick }) {
    const container = document.createElement('div');
    container.className = styles.avatar;
    container.style.display = 'none';

    const img = document.createElement('img');
    Object.assign(img, { src: WHITE_AVATAR, alt: 'Avatar' });

    const dropdown = document.createElement('div');
    dropdown.className = styles.dropdown;

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
        const button = dropdown.appendChild(document.createElement('button'));
        button.textContent = text;
        action?.(button);
    });

    container.append(img, dropdown);

    img.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        container.classList.toggle(styles.active);
    });

    document.addEventListener('click', (e) => {
        if (!container.contains(e.target)) {
            container.classList.remove(styles.active);
        }
    });

    window.addEventListener('resize', () => {
        container.classList.remove(styles.active);
    });

    return {
        element: container,
        show: () => {
            container.style.display = 'flex';
        },
        hide: () => {
            container.style.display = 'none';
        },
        setImage: ({ avatar }) => {
            const buffer = Uint8Array.from(avatar);
            if (buffer.length > 0) {
                const blob = new Blob([buffer], { type: 'image/png' });
                const url = URL.createObjectURL(blob);
                img.src = url;
                img.dataset.blobUrl = url;
            } else {
                img.src = avatar;
            }
        },
    };
}

export function createAvatar({ onLoginClick, onLogoutClick, onProfileClick, onSettingsClick }) {
    const container = document.createElement('div');
    container.className = styles.container;

    const login = createLoginButton({ onLoginClick });
    const avatar = createAvatarButton({ onLogoutClick, onProfileClick, onSettingsClick });

    container.append(login.element, avatar.element);

    const updateAuthState = (state) => {
        const isAuthenticated = state.auth?.isAuthenticated ?? false;
        const userData = state.auth?.user ?? null;

        isAuthenticated ? login.hide() : login.show();
        isAuthenticated ? avatar.show() : avatar.hide();

        avatar.setImage(
            isAuthenticated && userData?.avatar
                ? { avatar: userData.avatar }
                : { avatar: WHITE_AVATAR },
        );
    };

    updateAuthState(appState.getState());
    appState.subscribe(updateAuthState);

    return {
        element: container,
    };
}
