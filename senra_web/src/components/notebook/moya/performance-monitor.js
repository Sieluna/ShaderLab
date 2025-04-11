/**
 * Performance Monitor
 * Responsible for tracking and reporting rendering performance metrics
 */
export class PerformanceMonitor {
    /**
     * Create a performance monitor
     * @param {Object} config - Performance configuration
     * @param {HTMLElement} statsDisplay - Optional statistics display element
     */
    constructor(config, statsDisplay = null) {
        this.config = config || {};
        this.statsDisplay = statsDisplay;

        // Performance tracking
        this.frameCount = 0;
        this.startTime = performance.now();
        this.lastFrameTime = 0;
        this.frameTimes = [];

        // Statistics
        this.stats = {
            fps: 0,
            frameTime: 0,
            drawCalls: 0,
            triangleCount: 0,
        };

        // Adaptive resolution
        this.currentPerformanceFactor = 1.0;
    }

    /**
     * Start a new performance measurement session
     */
    startSession() {
        this.frameCount = 0;
        this.startTime = performance.now();
        this.lastFrameTime = 0;
        this.frameTimes = [];
    }

    /**
     * Update performance statistics
     * @param {number} delta - Frame interval time (seconds)
     * @param {number} drawCalls - Number of draw calls
     * @param {number} triangleCount - Number of triangles
     */
    updateStats(delta, drawCalls, triangleCount) {
        // Update frame time
        this.stats.frameTime = delta * 1000; // Convert to milliseconds

        // Maintain recent 30 frames' frame times
        this.frameTimes.push(delta);
        if (this.frameTimes.length > 30) {
            this.frameTimes.shift();
        }

        // Calculate average FPS
        const averageDelta = this.frameTimes.reduce((a, b) => a + b, 0) / this.frameTimes.length;
        this.stats.fps = Math.round(1 / averageDelta);

        // Update draw calls and triangle count
        this.stats.drawCalls = drawCalls;
        this.stats.triangleCount = triangleCount;

        // Update display (if any)
        this._updateDisplay();
    }

    /**
     * Mark the start of a frame
     * @returns {number} Current timestamp
     */
    beginFrame() {
        return performance.now();
    }

    /**
     * Mark the end of a frame and update statistics
     * @param {number} beginTime - Frame start timestamp
     * @param {number} drawCalls - Number of draw calls
     * @param {number} triangleCount - Number of triangles
     * @returns {number} Frame interval time (seconds)
     */
    endFrame(beginTime, drawCalls, triangleCount) {
        const now = performance.now();
        const delta = (now - beginTime) / 1000; // seconds

        this.lastFrameTime = now;
        this.frameCount++;

        this.updateStats(delta, drawCalls, triangleCount);

        return delta;
    }

    /**
     * Get performance factor for adaptive resolution
     * @returns {number} Performance factor (0.5 to 1.0)
     */
    getPerformanceFactor() {
        if (!this.config.adaptive_resolution) {
            return 1.0;
        }

        // If no FPS data, return highest quality
        if (!this.stats.fps) return 1.0;

        const targetFps = this.config.max_fps || 60;

        // Adjust performance factor based on frame rate
        if (this.stats.fps >= targetFps * 0.9) {
            // Good performance, can increase resolution
            this.currentPerformanceFactor = Math.min(this.currentPerformanceFactor + 0.05, 1.0);
        } else if (this.stats.fps < targetFps * 0.7) {
            // Poor performance, reduce resolution
            this.currentPerformanceFactor = Math.max(this.currentPerformanceFactor - 0.1, 0.5);
        }

        return this.currentPerformanceFactor;
    }

    /**
     * Get current performance statistics
     * @returns {Object} Performance statistics object
     */
    getStats() {
        return { ...this.stats };
    }

    /**
     * Update statistics display
     * @private
     */
    _updateDisplay() {
        if (!this.statsDisplay) return;

        this.statsDisplay.textContent =
            `FPS: ${this.stats.fps} | ` +
            `Frame Time: ${this.stats.frameTime.toFixed(2)}ms | ` +
            `Draw Calls: ${this.stats.drawCalls} | ` +
            `Triangles: ${this.stats.triangleCount}`;
    }

    /**
     * Create statistics display element
     * @param {HTMLElement} container - Container element
     * @returns {HTMLElement} Created statistics display element
     */
    createStatsDisplay(container) {
        if (this.statsDisplay) return this.statsDisplay;

        const display = document.createElement('div');
        display.className = 'renderer-stats';
        display.style.position = 'absolute';
        display.style.top = '0';
        display.style.left = '0';
        display.style.background = 'rgba(0,0,0,0.5)';
        display.style.color = 'white';
        display.style.padding = '5px';
        display.style.fontSize = '12px';
        display.style.fontFamily = 'monospace';
        display.style.zIndex = '100';

        container.appendChild(display);
        this.statsDisplay = display;

        return display;
    }
}
