import styles from './notebook-grid.module.css';
import { createNotebookCard } from './notebook-card.js';

/**
 * Create notebook grid component
 * @param {Object} options Component options
 * @param {Function} options.onItemClick Callback when a notebook card is clicked
 * @returns {Object} Component API
 */
export function createNotebookGrid({ onItemClick } = {}) {
    // Create root element
    const element = document.createElement('div');
    element.className = styles.grid;

    let notebooks = [];
    let cards = new Map();

    // Clean up cards
    function cleanupCards() {
        cards.forEach((card) => card.destroy());
        cards.clear();
    }

    // Render notebooks
    function render() {
        element.innerHTML = '';

        if (notebooks.length === 0) {
            element.innerHTML = '<p class="empty-state">No notebooks</p>';
            return;
        }

        notebooks.forEach((notebook) => {
            const card = createNotebookCard({
                onClick: onItemClick,
            });
            card.setNotebook(notebook);
            element.appendChild(card.element);
            cards.set(notebook.id, card);
        });
    }

    return {
        element,
        setNotebooks: (data) => {
            cleanupCards();
            notebooks = data;
            render();
        },
        destroy: () => {
            cleanupCards();
            element.innerHTML = '';
        },
    };
}
