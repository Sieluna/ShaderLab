import styles from './auth-modal.module.css';

export function createAuthModal({ onLogin, onRegister }) {
    const modal = document.createElement('div');
    modal.className = styles.modal;
    modal.style.display = 'none';

    return {
        element: modal,

        show: () => {},

        hide: () => {},

        reset: () => {},
    };
}
