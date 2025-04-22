import styles from './auth-modal.module.css';
import closeIcon from '../../assets/close.svg?raw';

function createTabs({ onTabChange, onClose }) {
    const tabs = document.createElement('div');
    tabs.className = styles.tabs;

    const login = document.createElement('button');
    login.className = `${styles.tab} ${styles.active}`;
    login.dataset.tab = 'login';
    login.textContent = 'Login';

    const register = document.createElement('button');
    register.className = styles.tab;
    register.dataset.tab = 'register';
    register.textContent = 'Register';

    const close = document.createElement('button');
    close.className = styles.close;
    close.innerHTML = closeIcon;

    tabs.append(login, register, close);

    const setActiveTab = (tab) => {
        login.classList.toggle(styles.active, tab === 'login');
        register.classList.toggle(styles.active, tab === 'register');
        onTabChange?.(tab);
    };
    login.addEventListener('click', () => setActiveTab('login'));
    register.addEventListener('click', () => setActiveTab('register'));
    close.addEventListener('click', () => onClose?.());

    return {
        element: tabs,
        setActiveTab,
    };
}

function createForm({ onSubmit }) {
    const form = document.createElement('form');
    form.className = styles.form;

    const username = document.createElement('div');
    username.className = styles.group;
    username.innerHTML = `
        <label for="username">Username</label>
        <input type="text" id="username" name="username" required>
    `;

    const email = document.createElement('div');
    email.className = styles.group;
    email.style.display = 'none';
    email.innerHTML = `
        <label for="email">Email</label>
        <input type="email" id="email" name="email">
    `;

    const password = document.createElement('div');
    password.className = styles.group;
    password.innerHTML = `
        <label for="password">Password</label>
        <input type="password" id="password" name="password" required>
    `;

    const error = document.createElement('div');
    error.className = styles.error;

    const submit = document.createElement('button');
    submit.type = 'submit';
    submit.className = styles.submit;
    submit.textContent = 'Login';

    form.append(username, email, password, error, submit);

    let currentMode = 'login';

    const setMode = (mode) => {
        currentMode = typeof mode === 'string' ? mode.toLowerCase() : 'login';
        email.style.display = currentMode === 'register' ? 'block' : 'none';
        submit.textContent = currentMode === 'login' ? 'Login' : 'Register';
        setError('');
    };

    const setError = (message) => {
        error.textContent = message ?? '';
    };

    form.addEventListener('submit', (e) => {
        e.preventDefault();
        const formData = new FormData(form);
        const data = {
            username: formData.get('username'),
            password: formData.get('password'),
            ...(currentMode === 'register' && { email: formData.get('email') }),
        };
        onSubmit?.(currentMode, data);
    });

    return {
        element: form,
        setMode,
        setError,
    };
}

export function createAuthModal({ onLogin, onRegister }) {
    const modal = document.createElement('div');
    modal.className = styles.modal;
    modal.style.display = 'none';

    const content = modal.appendChild(document.createElement('div'));
    content.className = styles.content;

    const form = createForm({
        onSubmit: (mode, data) => {
            try {
                mode === 'register' ? onRegister(data) : onLogin(data);
            } catch (err) {
                setError(err?.message || String(err));
            }
        },
    });
    const tabs = createTabs({
        onTabChange: form.setMode,
        onClose: () => {
            modal.style.display = 'none';
        },
    });

    content.append(tabs.element, form.element);

    // Outside‑click closes modal
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            modal.style.display = 'none';
        }
    });

    return {
        element: modal,
        show: () => {
            modal.style.display = 'flex';
            tabs.setActiveTab('login');
        },
        hide: () => {
            modal.style.display = 'none';
        },
        reset: () => {
            form.setMode();
            tabs.setActiveTab('login');
        },
        setError: form.setError,
    };
}
