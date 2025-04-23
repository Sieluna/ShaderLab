import styles from './notebook-card.module.css';
import eyeIcon from '../assets/eye.svg?raw';
import heartIcon from '../assets/heart.svg?raw';
import commentIcon from '../assets/chat.svg?raw';

function createNotebookPreview() {
    const container = document.createElement('div');
    container.className = styles.preview;

    function renderPreview(preview) {
        if (preview && preview.length > 0) {
            try {
                const previewData = Uint8Array.from(preview);
                const blob = new Blob([previewData], { type: 'image/png' });
                const previewUrl = URL.createObjectURL(blob);

                const img = document.createElement('img');
                img.src = previewUrl;
                img.alt = notebook.title;
                img.style.width = '100%';
                img.style.height = '100%';
                img.style.objectFit = 'cover';

                container.innerHTML = '';
                container.appendChild(img);
                previewRendered = true;
            } catch (error) {
                console.error('Failed to load preview image:', error);
            }
        }
    }
    return {
        element: container,
        renderPreview,
    };
}

export function createNotebookCard({ onClick } = {}) {
    const element = document.createElement('div');
    element.className = styles.card;

    let notebook = null;

    function render() {
        if (!notebook) return;

        // Clear previous content
        element.innerHTML = '';

        // Create and append preview container
        const preview = createNotebookPreview();

        // Add static content using innerHTML
        const content = document.createElement('div');
        content.className = styles.content;
        content.innerHTML = `
            <h3 class="${styles.title}">${notebook.title}</h3>
            <div class="${styles.meta}">
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
        `;

        // Create and append view link
        const view = document.createElement('button');
        view.className = styles.link;
        view.textContent = 'View Details';
        element.append(preview.element, content, view);

        // Handle click events
        view.addEventListener('click', (e) => {
            e.preventDefault();
            onClick?.(notebook, e);
        });
    }

    return {
        element,
        setNotebook: (data) => {
            notebook = data;
            render();
        },
        destroy: () => {
            element.innerHTML = '';
        },
    };
}
