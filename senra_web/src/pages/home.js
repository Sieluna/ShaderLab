import styles from './home.module.css';
import { appState } from '../state.js';
import { notebookService } from '../services/index.js';
import { createNotebookGrid } from '../components/notebook-grid.js';

export function homePage() {
    const container = document.createElement('div');
    container.className = styles.container;

    const header = document.createElement('header');
    header.className = styles.header;
    header.innerHTML = `
        <h1>ShaderLab</h1>
        <p>Create, Share, and Explore Real-Time Graphics Shaders</p>
    `;
    container.appendChild(header);

    const trendingSection = document.createElement('section');
    trendingSection.className = styles.trendingSection;
    trendingSection.innerHTML = `<h2>Trending Notebooks</h2>`;

    const grid = createNotebookGrid({
        onItemClick: (notebook) => {
            const path = `/notebook/${notebook.id}`;
            appState.setState((prev) => ({
                ...prev,
                ui: {
                    ...prev.ui,
                    currentPath: path,
                },
            }));
            window.history.pushState({}, '', path);
        },
    });
    trendingSection.appendChild(grid.element);
    container.appendChild(trendingSection);

    setTimeout(async () => {
        const notebooks = await notebookService.loadTrendingNotebooks();
        grid.setNotebooks(notebooks);
    }, 0);

    notebookService.notebookState.subscribe((state) => {
        if (state.trending.notebooks.length > 0) {
            grid.setNotebooks(state.trending.notebooks);
        }
    });

    return container;
}
