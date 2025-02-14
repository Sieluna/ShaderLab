import styles from './home.module.css';
import eyeIcon from '../assets/eye.svg?raw';
import heartIcon from '../assets/heart.svg?raw';
import commentIcon from '../assets/chat.svg?raw';
import { notebook } from '../services/index.js';
import { addBasePath } from '../state.js';
import { notebookCard } from '../components/index.js';

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
    trendingSection.innerHTML = `
        <h2>Trending Notebooks</h2>
        <div class="${styles.notebookGrid}" id="trending-notebooks"></div>
    `;
    container.appendChild(trendingSection);

    setTimeout(async () => {
        const notebooks = await notebook.loadTrendingNotebooks();
        renderNotebooks(notebooks);
    }, 0);

    notebook.notebookState.subscribe((state) => {
        if (state.trending.notebooks.length > 0) {
            renderNotebooks(state.trending.notebooks);
        }
    });

    function renderNotebooks(notebooks) {
        const grid = document.getElementById('trending-notebooks');
        if (!grid) return;

        grid.innerHTML = '';

        if (notebooks.length === 0) {
            grid.innerHTML = '<p class="empty-state">No trending notebooks</p>';
            return;
        }

        notebooks.forEach((notebook) => {
            const card = document.createElement('div');
            card.className = styles.notebookCard;

            const previewId = `preview-${notebook.id}`;

            card.innerHTML = `
                <div class="${styles.previewContainer}" id="${previewId}"></div>
                <div class="${styles.cardContent}">
                    <h3 class="${styles.cardTitle}">${notebook.title}</h3>
                    <div class="${styles.cardMeta}">
                        <div class="${styles.author}">
                            <img src="${notebook.author.avatar ? `data:image/png;base64,${btoa(String.fromCharCode.apply(null, notebook.author.avatar))}` : '/placeholder-avatar.png'}" 
                                 alt="${notebook.author.username}" class="${styles.avatar}">
                            <span>${notebook.author.username}</span>
                        </div>
                        <div class="${styles.stats}">
                            <span title="View">
                                ${eyeIcon}
                                ${notebook.stats.view_count}
                            </span>
                            <span title="Like">
                                ${heartIcon}
                                ${notebook.stats.like_count}
                            </span>
                            <span title="Comment">
                                ${commentIcon}
                                ${notebook.stats.comment_count}
                            </span>
                        </div>
                    </div>
                </div>
                <a href="/notebook/${notebook.id}" class="${styles.cardLink}">View Details</a>
            `;

            grid.appendChild(card);

            setTimeout(() => {
                const previewContainer = document.getElementById(previewId);
                if (previewContainer) {
                    notebookCard(previewId, notebook);
                }
            }, 10);

            card.addEventListener('click', (e) => {
                if (!e.target.matches(`.${styles.cardLink}`)) {
                    window.location.href = addBasePath(`/notebook/${notebook.id}`);
                }
            });
        });
    }

    return container;
}
