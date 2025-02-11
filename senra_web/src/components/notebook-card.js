export function notebookCard(containerId, { preview }) {
    const container = document.getElementById(containerId);
    if (!container) {
        console.error(`Container element ${containerId} does not exist`);
        return null;
    }

    // We need load the preview shader or preview image
    if (preview?.type === 'shader') {
        // const shader = loadShader(preview.shader);
        // container.innerHTML = shader;
    } else if (preview?.type === 'image') {
        // const image = loadImage(preview.image);
        // container.innerHTML = image;
    } else {
        console.error(`Preview is not supported`);
    }
}