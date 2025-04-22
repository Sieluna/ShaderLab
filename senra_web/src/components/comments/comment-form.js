import styles from './comment-form.module.css';
import { authState } from '../../state.js';

/**
 * Creates a comment form component
 * @param {Object} options - Component options
 * @param {Function} options.onSubmit - Callback when form is submitted
 * @returns {Object} Component interface
 */
export function createCommentForm({ onSubmit }) {
    const container = document.createElement('div');

    if (authState.getState().isAuthenticated) {
        container.className = styles.form;
        container.innerHTML = `
            <textarea placeholder="Add a comment..." id="comment-input"></textarea>
            <button id="submit-comment">Submit</button>
            <div id="comment-status" class="${styles.status}"></div>
        `;

        const commentInput = container.querySelector('#comment-input');
        const submitButton = container.querySelector('#submit-comment');
        const statusDiv = container.querySelector('#comment-status');

        submitButton.addEventListener('click', async () => {
            if (!commentInput || !commentInput.value.trim()) return;

            submitButton.disabled = true;
            submitButton.textContent = 'Submitting...';
            statusDiv.textContent = '';

            try {
                const result = await onSubmit(commentInput.value.trim());
                if (result.success) {
                    statusDiv.textContent = 'Comment submitted';
                    statusDiv.className = `${styles.status} ${styles.success}`;
                    commentInput.value = '';
                } else {
                    statusDiv.textContent = `Submission failed: ${result.error || 'Unknown error'}`;
                    statusDiv.className = `${styles.status} ${styles.error}`;
                }
            } catch (error) {
                statusDiv.textContent = `Submission failed: ${error.message || 'Unknown error'}`;
                statusDiv.className = `${styles.status} ${styles.error}`;
            } finally {
                submitButton.disabled = false;
                submitButton.textContent = 'Submit';
            }
        });
    } else {
        container.className = styles.login;
        container.innerHTML = `
            <p>Please login to comment</p>
        `;
    }

    return {
        element: container,

        // Utility method to clear the input
        clear: () => {
            const input = container.querySelector('#comment-input');
            if (input) input.value = '';
        },

        // Utility method to show success/error message
        showMessage: (message, isError = false) => {
            const statusDiv = container.querySelector('#comment-status');
            if (statusDiv) {
                statusDiv.textContent = message;
                statusDiv.className = `${styles.status} ${isError ? styles.error : styles.success}`;
            }
        },
    };
}
