import styles from './home.module.css';
import { appState } from '../state.js';
import { notebookService } from '../services/index.js';
import { createNotebookGrid } from '../components/notebook-grid.js';
import { navigateTo, buildPath } from '../utils/index.js';

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
            const path = buildPath('/notebook/:id', { id: notebook.id });
            navigateTo(path);
        },
    });
    trendingSection.appendChild(grid.element);
    container.appendChild(trendingSection);

    setTimeout(async () => {
        await notebookService.loadTrendingNotebooks();
    }, 0);

    appState.subscribe('notebook.trending.notebooks', (notebooks) => {
        if (notebooks && notebooks.length > 0) {
            grid.setNotebooks(notebooks);
        }
    });

    return container;
}
