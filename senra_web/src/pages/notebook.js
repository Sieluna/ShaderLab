import styles from './notebook.module.css';
import eyeIcon from '../assets/eye.svg?raw';
import heartIcon from '../assets/heart.svg?raw';
import commentIcon from '../assets/chat.svg?raw';
import { appState } from '../state.js';
import { notebookService } from '../services/index.js';
import { createNotebookViewer } from '../components/index.js';

export function notebookPage(id) {
    const container = document.createElement('div');
    container.className = styles.container;

    const loader = document.createElement('div');
    loader.className = styles.loader;
    loader.innerHTML = '<div class="spinner"></div><p>Loading...</p>';
    container.appendChild(loader);

    const content = document.createElement('div');
    content.className = styles.content;
    content.style.display = 'none';
    container.appendChild(content);

    const errorDisplay = document.createElement('div');
    errorDisplay.className = styles.error;
    errorDisplay.style.display = 'none';
    container.appendChild(errorDisplay);

    let notebookViewer = null;

    const unsubscribeNotebook = notebookService.notebookState.subscribe((state) => {
        if (state.current.isLoading) {
            loader.style.display = 'flex';
            content.style.display = 'none';
            errorDisplay.style.display = 'none';
        } else if (state.current.error) {
            loader.style.display = 'none';
            content.style.display = 'none';
            errorDisplay.style.display = 'block';
            errorDisplay.innerHTML = `<p>${state.current.error}</p>`;
        } else if (state.current.notebook) {
            loader.style.display = 'none';
            content.style.display = 'block';
            errorDisplay.style.display = 'none';

            renderNotebook(state.current.notebook);

            const comments = state.current.comments;

            if (comments.items.length === 0 && !comments.isLoading && !comments.hasLoaded) {
                notebookService.loadComments(state.current.notebook.id);
            }

            const commentsList = document.getElementById('comments-list');
            if (!commentsList) return;

            if (comments.isLoading) {
                commentsList.innerHTML = '<div class="comments-loader">Loading comments...</div>';
            } else if (comments.error) {
                commentsList.innerHTML = `<div class="error">Failed to load comments: ${comments.error}</div>`;
            } else if (comments.items.length > 0) {
                commentsList.innerHTML = comments.items
                    .map(
                        (comment) => `
                    <div class="${styles.commentItem}">
                        <div class="${styles.commentHeader}">
                            <div class="${styles.commentAuthor}">
                                <img src="${comment.author_avatar ? `data:image/png;base64,${btoa(String.fromCharCode.apply(null, comment.author_avatar))}` : '/placeholder-avatar.png'}" 
                                     alt="${comment.author}" class="${styles.avatar}">
                                <span>${comment.author}</span>
                            </div>
                            <div class="${styles.commentDate}">
                                ${new Date(comment.created_at).toLocaleString()}
                            </div>
                        </div>
                        <div class="${styles.commentContent}">
                            ${comment.content}
                        </div>
                    </div>
                `,
                    )
                    .join('');
            } else {
                commentsList.innerHTML = '<div class="empty-state">No comments yet</div>';
            }
        }
    });

    setTimeout(() => {
        notebookService.loadNotebookDetails(id);
    }, 0);

    function renderNotebook(notebook) {
        content.innerHTML = `
            <header class="${styles.header}">
                <h1>${notebook.title}</h1>
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
            </header>
            
            <div class="${styles.mainContent}">
                <div class="${styles.notebookViewerContainer}" id="notebook-viewer-container"></div>
                
                <div class="${styles.commentsSection}">
                    <h2>Comments (${notebook.stats.comment_count})</h2>
                    <div class="${styles.commentForm}">
                        <textarea placeholder="Add a comment..." id="comment-input"></textarea>
                        <button id="submit-comment">Submit Comment</button>
                    </div>
                    <div class="${styles.commentsList}" id="comments-list">
                        <div class="${styles.commentsLoader}">Loading comments...</div>
                    </div>
                </div>
            </div>
        `;

        setTimeout(() => {
            const viewerContainer = document.getElementById('notebook-viewer-container');
            if (viewerContainer) {
                if (notebookViewer) {
                    notebookViewer.destroy();
                    notebookViewer = null;
                }

                notebookViewer = createNotebookViewer(viewerContainer, {
                    renderMath: true,
                    codeSyntaxHighlight: true,
                    autoRunShaders: true,
                });

                notebookViewer.loadNotebook(notebook);
            }
        }, 10);

        const commentInput = document.getElementById('comment-input');
        const submitButton = document.getElementById('submit-comment');

        if (submitButton && commentInput) {
            submitButton.addEventListener('click', async () => {
                const content = commentInput.value.trim();
                if (!content) return;

                const isAuthenticated = appState.getState().auth.isAuthenticated;
                if (!isAuthenticated) {
                    alert('Please login first');
                    return;
                }

                const result = await notebookService.createComment(notebook.id, content);
                if (result.success) {
                    commentInput.value = '';
                }
            });
        }
    }

    const cleanup = () => {
        if (notebookViewer) {
            notebookViewer.destroy();
            notebookViewer = null;
        }

        unsubscribeNotebook();
    };

    const handleRouteChange = () => {
        const currentPath = window.location.pathname;
        if (!currentPath.startsWith(`/notebook/${id}`)) {
            cleanup();
        }
    };

    window.addEventListener('popstate', handleRouteChange);

    container.cleanup = cleanup;

    return container;
}
