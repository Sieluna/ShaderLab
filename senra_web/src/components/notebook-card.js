export function notebookCard(containerId, notebook) {
    const container = document.getElementById(containerId);
    if (!container) {
        console.error(`Container element ${containerId} does not exist`);
        return null;
    }

    if (!notebook || !notebook.shaders || notebook.shaders.length === 0) {
        container.innerHTML = `<div>No Shader</div>`;
        return null;
    }

    if (notebook.preview && notebook.preview.length > 0) {
        try {
            const previewData = new Uint8Array(notebook.preview);
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
            return null;
        } catch (error) {
            console.error('Failed to load preview image:', error);
        }
    }
}
