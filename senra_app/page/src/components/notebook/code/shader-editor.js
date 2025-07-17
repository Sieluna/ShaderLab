import styles from './shader-editor.module.css';
import {
    lineNumbers,
    highlightActiveLineGutter,
    highlightSpecialChars,
    drawSelection,
    dropCursor,
    rectangularSelection,
    crosshairCursor,
    highlightActiveLine,
    keymap,
} from '@codemirror/view';
import { EditorView } from '@codemirror/view';
import { EditorState } from '@codemirror/state';
import {
    foldGutter,
    indentOnInput,
    syntaxHighlighting,
    defaultHighlightStyle,
    bracketMatching,
    foldKeymap,
} from '@codemirror/language';
import { history, defaultKeymap, historyKeymap } from '@codemirror/commands';
import { highlightSelectionMatches, searchKeymap } from '@codemirror/search';
import {
    closeBrackets,
    autocompletion,
    closeBracketsKeymap,
    completionKeymap,
} from '@codemirror/autocomplete';
import { lintKeymap } from '@codemirror/lint';
import { wgsl } from './wgsl/index.js';

/**
 * @typedef {Object} ShaderEditorOptions
 * @property {boolean} [readOnly=false] - Whether the editor is read-only
 * @property {string} [language='wgsl'] - Language mode for the editor
 * @property {function(string): void} [onChange] - Callback when content changes
 */

/**
 * Create shader code editor
 * @param {HTMLElement} container - Container element
 * @param {string} initialCode - Initial code
 * @param {ShaderEditorOptions} options - Editor options
 * @returns {Object} Editor instance API
 */
export function createShaderEditor(container, initialCode = '', options = {}) {
    // Default options
    const editorOptions = {
        readOnly: false,
        language: 'wgsl',
        onChange: null,
        ...options,
    };

    // Create wrapper elements
    const editorWrapper = document.createElement('div');
    editorWrapper.className = styles.editor;
    container.appendChild(editorWrapper);

    // Create language select if needed
    if (!editorOptions.readOnly) {
        const header = document.createElement('div');
        header.className = styles.header;

        const langSelect = document.createElement('select');
        langSelect.className = styles.langSelect;

        const languages = [
            { value: 'wgsl', label: 'WGSL' },
            { value: 'glsl', label: 'GLSL' },
            { value: 'js', label: 'JavaScript' },
        ];

        languages.forEach(({ value, label }) => {
            const option = document.createElement('option');
            option.value = value;
            option.textContent = label;
            if (value === editorOptions.language) {
                option.selected = true;
            }
            langSelect.appendChild(option);
        });

        header.appendChild(langSelect);
        editorWrapper.appendChild(header);
    }

    // Create editor container
    const editorContainer = document.createElement('div');
    editorContainer.className = styles.container;
    editorWrapper.appendChild(editorContainer);

    // Setup editor extensions
    const extensions = [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        foldGutter(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        bracketMatching(),
        closeBrackets(),
        autocompletion(),
        rectangularSelection(),
        crosshairCursor(),
        highlightActiveLine(),
        highlightSelectionMatches(),
        keymap.of([
            ...closeBracketsKeymap,
            ...defaultKeymap,
            ...searchKeymap,
            ...historyKeymap,
            ...foldKeymap,
            ...completionKeymap,
            ...lintKeymap,
        ]),
        wgsl(),
        EditorView.lineWrapping,
        EditorState.tabSize.of(4),
        EditorView.theme({
            '&': {
                fontSize: '14px',
                height: '100%',
            },
            '.cm-content': {
                fontFamily: "'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace",
            },
            '.cm-gutters': {
                backgroundColor: 'var(--theme-hover-bg)',
                color: 'var(--theme-text-color)',
                border: 'none',
            },
        }),
    ];

    // Add onChange handler if provided
    editorOptions.onChange &&
        extensions.push(
            EditorView.updateListener.of((update) => {
                update.docChanged && editorOptions.onChange?.(update.state.doc.toString());
            }),
        );

    // Add readonly extension if needed
    editorOptions.readOnly && extensions.push(EditorView.editable.of(false));

    // Create editor state
    const startState = EditorState.create({
        doc: initialCode,
        extensions,
    });

    // Create editor view
    const view = new EditorView({
        state: startState,
        parent: editorContainer,
    });

    // Return API object
    return {
        /**
         * Get editor content
         * @returns {string} Current content
         */
        getContent: () => view.state.doc.toString(),

        /**
         * Set editor content
         * @param {string} content - New content
         */
        setContent: (content) => {
            view.dispatch({
                changes: {
                    from: 0,
                    to: view.state.doc.length,
                    insert: content,
                },
            });
        },

        /**
         * Update editor options
         * @param {ShaderEditorOptions} newOptions - New options
         */
        updateOptions: (newOptions) => {
            // Only basic options update is supported currently
            if (
                newOptions.readOnly !== undefined &&
                newOptions.readOnly !== editorOptions.readOnly
            ) {
                view.dispatch({
                    effects: EditorView.editable.reconfigure(!newOptions.readOnly),
                });
                editorOptions.readOnly = newOptions.readOnly;
            }
        },

        /**
         * Get the editor view instance
         * @returns {EditorView} Editor view
         */
        getView: () => view,

        /**
         * Destroy editor instance
         */
        destroy: () => view.destroy(),
    };
}
