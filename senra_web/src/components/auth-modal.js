import styles from './auth-modal.module.css';
import closeIcon from '../assets/close.svg?raw';

export function createAuthModal({ onLogin, onRegister }) {
    const modal = document.createElement('div');
    modal.className = styles.modal;
    modal.style.display = 'none';

    modal.innerHTML = `
        <div class="${styles.modalContent}">
            <div class="${styles.tabs}">
                <button class="${styles.tabBtn} ${styles.active}" data-tab="login">Login</button>
                <button class="${styles.tabBtn}" data-tab="register">Register</button>
                <button class="${styles.closeBtn}">${closeIcon}</button>
            </div>
            
            <form id="loginForm" class="${styles.form}" style="display: block;">
                <div class="${styles.formGroup}">
                    <label for="loginUsername">Username</label>
                    <input type="text" id="loginUsername" name="username" required>
                </div>
                <div class="${styles.formGroup}">
                    <label for="loginPassword">Password</label>
                    <input type="password" id="loginPassword" name="password" required>
                </div>
                <button type="submit" class="${styles.submitBtn}">Login</button>
            </form>
            
            <form id="registerForm" class="${styles.form}" style="display: none;">
                <div class="${styles.formGroup}">
                    <label for="registerUsername">Username</label>
                    <input type="text" id="registerUsername" name="username" required>
                </div>
                <div class="${styles.formGroup}">
                    <label for="registerEmail">Email</label>
                    <input type="email" id="registerEmail" name="email" required>
                </div>
                <div class="${styles.formGroup}">
                    <label for="registerPassword">Password</label>
                    <input type="password" id="registerPassword" name="password" required>
                </div>
                <button type="submit" class="${styles.submitBtn}">Register</button>
            </form>
        </div>
    `;

    const closeBtn = modal.querySelector(`.${styles.closeBtn}`);
    const tabBtns = modal.querySelectorAll(`.${styles.tabBtn}`);
    const loginForm = modal.querySelector('#loginForm');
    const registerForm = modal.querySelector('#registerForm');

    tabBtns.forEach((btn) => {
        btn.addEventListener('click', () => {
            const tab = btn.dataset.tab;

            tabBtns.forEach((b) => b.classList.remove(styles.active));
            btn.classList.add(styles.active);

            loginForm.style.display = tab === 'login' ? 'block' : 'none';
            registerForm.style.display = tab === 'register' ? 'block' : 'none';
        });
    });

    closeBtn.addEventListener('click', () => {
        modal.style.display = 'none';
    });

    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            modal.style.display = 'none';
        }
    });

    loginForm.addEventListener('submit', (e) => {
        e.preventDefault();
        const formData = new FormData(loginForm);
        const loginData = {
            username: formData.get('username'),
            password: formData.get('password'),
        };
        onLogin(loginData);
    });

    registerForm.addEventListener('submit', (e) => {
        e.preventDefault();
        const formData = new FormData(registerForm);
        const registerData = {
            username: formData.get('username'),
            email: formData.get('email'),
            password: formData.get('password'),
        };
        onRegister(registerData);
    });

    return {
        element: modal,

        show: () => {
            modal.style.display = 'flex';
        },

        hide: () => {
            modal.style.display = 'none';
        },

        reset: () => {
            loginForm.reset();
            registerForm.reset();

            tabBtns.forEach((btn) => {
                const isLoginTab = btn.dataset.tab === 'login';
                btn.classList.toggle(styles.active, isLoginTab);
            });

            loginForm.style.display = 'block';
            registerForm.style.display = 'none';
        },
    };
}
