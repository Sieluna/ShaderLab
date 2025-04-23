import styles from './comment-list.module.css';
import { createCommentForm } from './comment-form.js';
import { notebookService } from '../../services/index.js';
import { time } from '../../utils/index.js';

/**
 * Creates a comments component with infinite scrolling
 * @param {Object} options - Component options
 * @param {Function} options.onSubmit - Callback when a comment is submitted
 * @returns {Object} Component interface
 */
export function createCommentList({ onSubmit }) {
    const container = document.createElement('div');
    container.className = styles.container;

    const header = document.createElement('div');
    header.className = styles.header;

    const commentsCount = document.createElement('h2');
    commentsCount.textContent = 'Comments (0)';
    header.appendChild(commentsCount);

    const closeBtn = document.createElement('button');
    closeBtn.className = styles.close;
    closeBtn.textContent = '×';
    header.appendChild(closeBtn);

    const formContainer = document.createElement('div');
    const commentForm = createCommentForm({
        onSubmit: async (content) => {
            const result = await onSubmit(content);
            if (result.success) {
                // Force refresh the list to show the new comment
                await loadComments(notebookId, 1, true);
            }
            return result;
        },
    });
    formContainer.appendChild(commentForm.element);

    const listContainer = document.createElement('div');
    listContainer.className = styles.list;
    listContainer.id = 'comments-list';

    const loadingIndicator = document.createElement('div');
    loadingIndicator.className = styles.loading;
    loadingIndicator.innerHTML = '<div class="spinner"></div><p>Loading more comments...</p>';
    loadingIndicator.style.display = 'none';

    // Append all elements to container
    container.append(header, formContainer, listContainer, loadingIndicator);

    // State management
    let isLoading = false;
    let hasMoreComments = true;
    let currentPage = 1;
    const perPage = 10;
    let notebookId = null;
    let cachedComments = {};

    /**
     * Renders a comment to HTML
     */
    const renderComment = (comment) => {
        const commentEl = document.createElement('div');
        commentEl.className = styles.item;
        commentEl.dataset.id = comment.id;

        commentEl.innerHTML = `
            <div class="${styles.header}">
                <div class="${styles.author}">
                    <img src="${
                        comment.author_avatar
                            ? `data:image/png;base64,${btoa(String.fromCharCode.apply(null, comment.author_avatar))}`
                            : '/placeholder-avatar.png'
                    }" 
                        alt="${comment.author}" class="${styles.avatar}">
                    <span>${comment.author}</span>
                </div>
                <div class="${styles.date}">
                    ${time(comment.created_at)}
                </div>
            </div>
            <div class="${styles.content}">
                ${comment.content}
            </div>
        `;

        return commentEl;
    };

    /**
     * Loads comments for the notebook
     */
    const loadComments = async (id, page = 1, forceRefresh = false) => {
        if (isLoading || (!hasMoreComments && page > 1)) return;

        if (id !== notebookId) {
            // Reset when notebook changes
            notebookId = id;
            listContainer.innerHTML = '';
            cachedComments = {};
            currentPage = 1;
            hasMoreComments = true;
        }

        // Check if we already have this page cached
        const cacheKey = `page-${page}`;
        if (!forceRefresh && cachedComments[cacheKey]) {
            renderCachedPage(cacheKey);
            return;
        }

        isLoading = true;
        loadingIndicator.style.display = 'flex';

        try {
            const { comments, total } = await notebookService.loadComments(
                notebookId,
                page,
                perPage,
            );

            if (page === 1) {
                listContainer.innerHTML = '';
                updateCommentsCount(total);
            }

            if (comments.length === 0) {
                hasMoreComments = false;
                if (page === 1) {
                    listContainer.innerHTML = '<div class="empty-state">No comments yet</div>';
                }
            } else {
                // Cache this page
                cachedComments[cacheKey] = comments;

                // Append comments to the list
                comments.forEach((comment) => {
                    listContainer.appendChild(renderComment(comment));
                });

                // Update current page
                currentPage = page;

                // Check if we have more comments
                hasMoreComments = comments.length === perPage;

                // Preload next page if we have more
                if (hasMoreComments) {
                    setTimeout(() => {
                        notebookService
                            .loadComments(notebookId, page + 1, perPage)
                            .then(({ comments: nextComments }) => {
                                if (nextComments.length > 0) {
                                    cachedComments[`page-${page + 1}`] = nextComments;
                                }
                            })
                            .catch((err) => console.error('Failed to preload next page:', err));
                    }, 1000);
                }
            }
        } catch (error) {
            console.error('Failed to load comments:', error);
            listContainer.innerHTML += '<div class="error">Failed to load comments</div>';
        } finally {
            isLoading = false;
            loadingIndicator.style.display = 'none';
        }
    };

    /**
     * Renders a cached page of comments
     */
    const renderCachedPage = (cacheKey) => {
        const comments = cachedComments[cacheKey];
        if (!comments || !comments.length) return;

        comments.forEach((comment) => {
            listContainer.appendChild(renderComment(comment));
        });

        // Extract page number from cache key
        const pageMatch = cacheKey.match(/page-(\d+)/);
        if (pageMatch && pageMatch[1]) {
            currentPage = Math.max(currentPage, parseInt(pageMatch[1], 10));
        }
    };

    /**
     * Updates the comments count in the header
     */
    const updateCommentsCount = (count) => {
        commentsCount.textContent = `Comments (${count})`;
    };

    /**
     * Sets up infinite scrolling for comments
     */
    const setupInfiniteScroll = () => {
        // Use intersection observer to detect when we're near the bottom
        const observer = new IntersectionObserver(
            (entries) => {
                entries.forEach((entry) => {
                    if (entry.isIntersecting && !isLoading && hasMoreComments) {
                        loadComments(notebookId, currentPage + 1);
                    }
                });
            },
            { threshold: 0.1 },
        );

        observer.observe(loadingIndicator);

        return () => observer.disconnect();
    };

    // Setup infinite scrolling
    const cleanupScroll = setupInfiniteScroll();

    return {
        element: container,

        /**
         * Loads comments for a notebook
         * @param {number} id - Notebook ID
         */
        load: (id) => {
            loadComments(id, 1, true);
        },

        /**
         * Updates the comments list with a new comment
         * @param {Object} comment - Comment data
         */
        addComment: (comment) => {
            if (listContainer.querySelector('.empty-state')) {
                listContainer.innerHTML = '';
            }

            listContainer.insertBefore(renderComment(comment), listContainer.firstChild);

            const currentCount = parseInt(commentsCount.textContent.match(/\d+/)[0] || '0', 10);
            updateCommentsCount(currentCount + 1);
        },

        /**
         * Refreshes the comments list
         */
        refresh: () => {
            loadComments(notebookId, 1, true);
        },

        /**
         * Sets the close button click handler
         * @param {Function} handler - Close button handler
         */
        onClose: (handler) => {
            closeBtn.addEventListener('click', handler);
        },

        /**
         * Cleans up the component
         */
        destroy: () => {
            cleanupScroll();
        },
    };
}
