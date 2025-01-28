import * as api from './api.js';
import * as services from './services/index.js';
import { navbar, router } from './components/index.js';
import { homePage, debugPage } from './pages/index.js';
import './style.css';

const app = document.querySelector('#app');

app.appendChild(
    navbar([
        { label: 'Home', path: '/' },
        { label: 'Debug', path: '/debug' },
    ]),
);
app.appendChild(
    router({
        '/': homePage,
        '/debug': debugPage,
    }),
);
