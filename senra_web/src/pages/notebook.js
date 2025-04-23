import styles from './notebook.module.css';
import eyeIcon from '../assets/eye.svg?raw';
import heartIcon from '../assets/heart.svg?raw';
import commentIcon from '../assets/chat.svg?raw';
import { notebookService } from '../services/index.js';
import { createNotebookViewer, createCommentList } from '../components/index.js';

export function notebookPage(id) {
    const container = document.createElement('div');
    container.className = styles.container;

    const loader = container.appendChild(document.createElement('div'));
    loader.className = styles.loader;
    loader.innerHTML = '<div class="spinner"></div><p>Loading...</p>';

    const content = container.appendChild(document.createElement('div'));
    content.className = styles.content;
    content.style.display = 'none';

    const errorDisplay = document.createElement('div');
    errorDisplay.className = styles.error;
    errorDisplay.style.display = 'none';
    container.appendChild(errorDisplay);

    const sidebar = document.createElement('div');
    sidebar.className = styles.commentsSidebar;
    container.appendChild(sidebar);

    const toggle = document.createElement('button');
    toggle.className = styles.commentToggle;
    toggle.innerHTML = commentIcon;
    container.appendChild(toggle);

    let viewer = null;
    let commentsBadge = null;
    let commentsComponent = null;

    const prevState = {
        notebookId: null,
        commentsCount: 0,
    };

    const unsubscribe = notebookService.notebookState.subscribe(({ current }) => {
        if (current.isLoading) {
            loader.style.display = 'flex';
            content.style.display = 'none';
            errorDisplay.style.display = 'none';
            return;
        }

        if (current.error) {
            loader.style.display = 'none';
            content.style.display = 'none';
            errorDisplay.style.display = 'block';
            errorDisplay.innerHTML = `<p>${current.error}</p>`;
            return;
        }

        if (!current.notebook) return;

        loader.style.display = 'none';
        content.style.display = 'block';
        errorDisplay.style.display = 'none';

        const isNotebookChanged = current.notebook.id !== prevState.notebookId;
        const currentCommentsCount = current.notebook.stats.comment_count;

        if (isNotebookChanged) {
            renderNotebook(current.notebook);
            prevState.notebookId = current.notebook.id;

            // If the sidebar is open, refresh comments for the new notebook
            if (sidebar.classList.contains(styles.open) && commentsComponent) {
                commentsComponent.load(current.notebook.id);
            }
        }

        if (isNotebookChanged || currentCommentsCount !== prevState.commentsCount) {
            updateCommentBadge(currentCommentsCount);
            prevState.commentsCount = currentCommentsCount;
        }
    });

    const updateCommentBadge = (count) => {
        if (!commentsBadge) {
            commentsBadge = toggle.appendChild(
                Object.assign(document.createElement('span'), {
                    className: styles.commentCount,
                }),
            );
        }
        commentsBadge.textContent = count;

        const commentsToggle = document.getElementById('comments-toggle');
        if (commentsToggle) {
            const countSpan = commentsToggle.querySelector('span.count');
            if (countSpan) {
                countSpan.textContent = count;
            }
        }
    };

    const toggleSidebar = () => {
        const isOpen = sidebar.classList.toggle(styles.open);

        if (isOpen) {
            // Create comments component if it doesn't exist
            if (!commentsComponent) {
                commentsComponent = createCommentList({
                    onSubmit: (content) =>
                        notebookService.createComment(prevState.notebookId, content),
                });
                commentsComponent.onClose(toggleSidebar);
                sidebar.innerHTML = '';
                sidebar.appendChild(commentsComponent.element);
            }

            // Load comments for the current notebook
            commentsComponent.load(prevState.notebookId);
        }
    };

    toggle.addEventListener('click', toggleSidebar);

    setTimeout(() => {
        notebookService.loadNotebookDetails(id).then((notebook) => {
            notebookService.loadComments(notebook.id).then();
        });
    }, 0);

    function renderNotebook(notebook) {
        const { title, author, stats } = notebook;

        content.innerHTML = `
            <header class="${styles.header}">
                <h1>${title}</h1>
                <div class="${styles.meta}">
                    <div class="${styles.author}">
                        <img src="${author.avatar ? `data:image/png;base64,${btoa(String.fromCharCode.apply(null, author.avatar))}` : '/placeholder-avatar.png'}" 
                             alt="${author.username}" class="${styles.avatar}">
                        <span>${author.username}</span>
                    </div>
                    <div class="${styles.stats}">
                        <span title="View">
                            ${eyeIcon}
                            ${stats.view_count}
                        </span>
                        <span title="Like">
                            ${heartIcon}
                            ${stats.like_count}
                        </span>
                        <span title="Comment" id="comments-toggle">
                            ${commentIcon}
                            <span class="count">${stats.comment_count}</span>
                        </span>
                    </div>
                </div>
            </header>
            
            <div class="${styles.mainContent}">
                <div class="${styles.notebookViewerContainer}" id="notebook-viewer-container"></div>
            </div>
        `;

        document.getElementById('comments-toggle')?.addEventListener('click', toggleSidebar);

        setTimeout(() => {
            const viewerContainer = document.getElementById('notebook-viewer-container');
            if (!viewerContainer) return;

            viewer?.destroy();
            viewer = null;

            viewer = createNotebookViewer(viewerContainer, {
                renderMath: true,
                codeSyntaxHighlight: true,
                autoRunShaders: true,
            });

            viewer.loadNotebook(notebook);
        }, 10);
    }

    const cleanup = () => {
        viewer?.destroy();
        commentsComponent?.destroy();
        viewer = null;
        commentsComponent = null;
        unsubscribe();
    };

    const handleRouteChange = () => {
        !window.location.pathname.startsWith(`/notebook/${id}`) && cleanup();
    };

    window.addEventListener('popstate', handleRouteChange);
    container.cleanup = cleanup;

    return container;
}
