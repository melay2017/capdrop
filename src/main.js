// CapDrop - Main frontend entry
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// State
let screenshotB64 = null;
let currentTool = 'select';
let penColor = '#FF0000';
let penSize = 4;
let annotations = [];      // Array of annotation objects
let undoStack = [];
let isDrawing = false;
let startX, startY;
let currentPath = [];       // For freehand pen
let screenshotImage = null; // HTMLImageElement of the screenshot

// DOM refs
const mainView = document.getElementById('main-view');
const settingsView = document.getElementById('settings-view');
const canvasContainer = document.getElementById('canvas-container');
const screenshotCanvas = document.getElementById('screenshot-canvas');
const annotationCanvas = document.getElementById('annotation-canvas');
const sCtx = screenshotCanvas.getContext('2d');
const aCtx = annotationCanvas.getContext('2d');
const statusText = document.getElementById('status-text');

// ===== Toolbar =====
document.querySelectorAll('.tool-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.tool-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        currentTool = btn.dataset.tool;
    });
});

document.getElementById('color-picker').addEventListener('input', e => { penColor = e.target.value; });
document.getElementById('pen-size').addEventListener('change', e => { penSize = parseInt(e.target.value); });
document.getElementById('undo-btn').addEventListener('click', undo);
document.getElementById('redo-btn').addEventListener('click', redo);
document.getElementById('save-btn').addEventListener('click', saveScreenshot);
document.getElementById('cancel-btn').addEventListener('click', cancelScreenshot);
document.getElementById('save-settings-btn').addEventListener('click', saveSettings);
document.getElementById('close-settings-btn').addEventListener('click', () => {
    settingsView.classList.add('hidden');
    if (screenshotB64) mainView.classList.remove('hidden');
});

// ===== Screenshot Capture =====
async function takeScreenshot() {
    statusText.textContent = 'Capturing...';
    try {
        screenshotB64 = await invoke('capture_fullscreen');
        displayScreenshot(screenshotB64);
        statusText.textContent = 'Screenshot captured - annotate and save';
    } catch (e) {
        statusText.textContent = `Error: ${e}`;
        console.error('Capture failed:', e);
    }
}

function displayScreenshot(b64) {
    const img = new Image();
    img.onload = () => {
        screenshotImage = img;
        const maxW = window.innerWidth;
        const maxH = window.innerHeight - 72; // toolbar + status
        let w = img.width, h = img.height;
        const scale = Math.min(maxW / w, maxH / h, 1);
        w = Math.round(w * scale);
        h = Math.round(h * scale);

        screenshotCanvas.width = w;
        screenshotCanvas.height = h;
        annotationCanvas.width = w;
        annotationCanvas.height = h;

        sCtx.drawImage(img, 0, 0, w, h);

        // Reset annotations
        annotations = [];
        undoStack = [];
        redrawAnnotations();

        mainView.classList.remove('hidden');
        settingsView.classList.add('hidden');
    };
    img.src = 'data:image/png;base64,' + b64;
}

// ===== Canvas Drawing =====
annotationCanvas.addEventListener('mousedown', onMouseDown);
annotationCanvas.addEventListener('mousemove', onMouseMove);
annotationCanvas.addEventListener('mouseup', onMouseUp);

function getPos(e) {
    const rect = annotationCanvas.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
}

function onMouseDown(e) {
    isDrawing = true;
    const pos = getPos(e);
    startX = pos.x;
    startY = pos.y;
    currentPath = [{ x: pos.x, y: pos.y }];

    if (currentTool === 'text') {
        isDrawing = false;
        addTextAnnotation(pos.x, pos.y);
    }
}

function onMouseMove(e) {
    if (!isDrawing) return;
    const pos = getPos(e);

    if (currentTool === 'pen') {
        currentPath.push({ x: pos.x, y: pos.y });
    }

    // Preview
    redrawAnnotations();
    drawPreview(pos.x, pos.y);
}

function onMouseUp(e) {
    if (!isDrawing) return;
    isDrawing = false;
    const pos = getPos(e);

    const annotation = createAnnotation(pos.x, pos.y);
    if (annotation) {
        annotations.push(annotation);
        undoStack = []; // Clear redo on new action
    }
    redrawAnnotations();
}

function createAnnotation(endX, endY) {
    switch (currentTool) {
        case 'pen':
            if (currentPath.length < 2) return null;
            return { type: 'pen', path: [...currentPath], color: penColor, size: penSize };
        case 'arrow':
            return { type: 'arrow', x1: startX, y1: startY, x2: endX, y2: endY, color: penColor, size: penSize };
        case 'rect':
            return { type: 'rect', x: Math.min(startX, endX), y: Math.min(startY, endY),
                     w: Math.abs(endX - startX), h: Math.abs(endY - startY), color: penColor, size: penSize };
        case 'circle':
            const cx = (startX + endX) / 2, cy = (startY + endY) / 2;
            const rx = Math.abs(endX - startX) / 2, ry = Math.abs(endY - startY) / 2;
            return { type: 'circle', cx, cy, rx, ry, color: penColor, size: penSize };
        case 'mosaic':
            return { type: 'mosaic', x: Math.min(startX, endX), y: Math.min(startY, endY),
                     w: Math.abs(endX - startX), h: Math.abs(endY - startY), blockSize: 10 };
        default:
            return null;
    }
}

function drawPreview(endX, endY) {
    aCtx.save();
    switch (currentTool) {
        case 'pen':
            drawPenPath(currentPath, penColor, penSize);
            break;
        case 'arrow':
            drawArrow(startX, startY, endX, endY, penColor, penSize);
            break;
        case 'rect':
            aCtx.strokeStyle = penColor;
            aCtx.lineWidth = penSize;
            aCtx.strokeRect(Math.min(startX, endX), Math.min(startY, endY),
                           Math.abs(endX - startX), Math.abs(endY - startY));
            break;
        case 'circle':
            const cx = (startX + endX) / 2, cy = (startY + endY) / 2;
            const rx = Math.abs(endX - startX) / 2, ry = Math.abs(endY - startY) / 2;
            aCtx.strokeStyle = penColor;
            aCtx.lineWidth = penSize;
            aCtx.beginPath();
            aCtx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
            aCtx.stroke();
            break;
        case 'mosaic':
            drawMosaic(Math.min(startX, endX), Math.min(startY, endY),
                      Math.abs(endX - startX), Math.abs(endY - startY), 10);
            break;
    }
    aCtx.restore();
}

// ===== Drawing Functions =====
function redrawAnnotations() {
    aCtx.clearRect(0, 0, annotationCanvas.width, annotationCanvas.height);
    for (const a of annotations) {
        aCtx.save();
        switch (a.type) {
            case 'pen': drawPenPath(a.path, a.color, a.size); break;
            case 'arrow': drawArrow(a.x1, a.y1, a.x2, a.y2, a.color, a.size); break;
            case 'rect':
                aCtx.strokeStyle = a.color;
                aCtx.lineWidth = a.size;
                aCtx.strokeRect(a.x, a.y, a.w, a.h);
                break;
            case 'circle':
                aCtx.strokeStyle = a.color;
                aCtx.lineWidth = a.size;
                aCtx.beginPath();
                aCtx.ellipse(a.cx, a.cy, a.rx, a.ry, 0, 0, Math.PI * 2);
                aCtx.stroke();
                break;
            case 'text':
                aCtx.fillStyle = a.color;
                aCtx.font = `${a.fontSize || 16}px sans-serif`;
                aCtx.fillText(a.text, a.x, a.y);
                break;
            case 'mosaic':
                drawMosaic(a.x, a.y, a.w, a.h, a.blockSize);
                break;
        }
        aCtx.restore();
    }
}

function drawPenPath(path, color, size) {
    if (path.length < 2) return;
    aCtx.strokeStyle = color;
    aCtx.lineWidth = size;
    aCtx.lineCap = 'round';
    aCtx.lineJoin = 'round';
    aCtx.beginPath();
    aCtx.moveTo(path[0].x, path[0].y);
    for (let i = 1; i < path.length; i++) {
        aCtx.lineTo(path[i].x, path[i].y);
    }
    aCtx.stroke();
}

function drawArrow(x1, y1, x2, y2, color, size) {
    const angle = Math.atan2(y2 - y1, x2 - x1);
    const headLen = 12 + size * 2;

    aCtx.strokeStyle = color;
    aCtx.fillStyle = color;
    aCtx.lineWidth = size;
    aCtx.lineCap = 'round';

    // Line
    aCtx.beginPath();
    aCtx.moveTo(x1, y1);
    aCtx.lineTo(x2, y2);
    aCtx.stroke();

    // Arrowhead
    aCtx.beginPath();
    aCtx.moveTo(x2, y2);
    aCtx.lineTo(x2 - headLen * Math.cos(angle - Math.PI / 6),
                y2 - headLen * Math.sin(angle - Math.PI / 6));
    aCtx.lineTo(x2 - headLen * Math.cos(angle + Math.PI / 6),
                y2 - headLen * Math.sin(angle + Math.PI / 6));
    aCtx.closePath();
    aCtx.fill();
}

function drawMosaic(x, y, w, h, blockSize) {
    // Get pixel data from screenshot canvas
    const imgData = sCtx.getImageData(
        Math.max(0, Math.round(x)),
        Math.max(0, Math.round(y)),
        Math.min(Math.round(w), screenshotCanvas.width),
        Math.min(Math.round(h), screenshotCanvas.height)
    );

    for (let by = 0; by < imgData.height; by += blockSize) {
        for (let bx = 0; bx < imgData.width; bx += blockSize) {
            // Average color of this block
            let r = 0, g = 0, b = 0, count = 0;
            for (let dy = 0; dy < blockSize && by + dy < imgData.height; dy++) {
                for (let dx = 0; dx < blockSize && bx + dx < imgData.width; dx++) {
                    const idx = ((by + dy) * imgData.width + (bx + dx)) * 4;
                    r += imgData.data[idx];
                    g += imgData.data[idx + 1];
                    b += imgData.data[idx + 2];
                    count++;
                }
            }
            r = Math.round(r / count);
            g = Math.round(g / count);
            b = Math.round(b / count);

            aCtx.fillStyle = `rgb(${r},${g},${b})`;
            aCtx.fillRect(x + bx, y + by, blockSize, blockSize);
        }
    }
}

function addTextAnnotation(x, y) {
    const input = document.createElement('input');
    input.id = 'text-input-overlay';
    input.style.left = (canvasContainer.offsetLeft + x) + 'px';
    input.style.top = (canvasContainer.offsetTop + y) + 'px';
    input.style.color = penColor;
    document.getElementById('app').appendChild(input);
    input.focus();

    const finalize = () => {
        if (input.value.trim()) {
            annotations.push({
                type: 'text', x, y, text: input.value,
                color: penColor, fontSize: parseInt(penSize) * 4
            });
            undoStack = [];
            redrawAnnotations();
        }
        input.remove();
    };

    input.addEventListener('keydown', e => {
        if (e.key === 'Enter') finalize();
        if (e.key === 'Escape') input.remove();
    });
    input.addEventListener('blur', finalize);
}

// ===== Undo / Redo =====
function undo() {
    if (annotations.length === 0) return;
    undoStack.push(annotations.pop());
    redrawAnnotations();
}

function redo() {
    if (undoStack.length === 0) return;
    annotations.push(undoStack.pop());
    redrawAnnotations();
}

// ===== Save =====
async function saveScreenshot() {
    statusText.textContent = 'Saving...';

    try {
        // Composite screenshot + annotations into one canvas
        const mergedCanvas = document.createElement('canvas');
        mergedCanvas.width = screenshotCanvas.width;
        mergedCanvas.height = screenshotCanvas.height;
        const mCtx = mergedCanvas.getContext('2d');
        mCtx.drawImage(screenshotCanvas, 0, 0);
        mCtx.drawImage(annotationCanvas, 0, 0);

        // Get base64 (strip data URL prefix)
        const dataUrl = mergedCanvas.toDataURL('image/png');
        const b64 = dataUrl.replace('data:image/png;base64,', '');

        const results = await invoke('save_screenshot', { imageB64: b64 });
        const summary = results.map(r => `${r.target}: ${r.success ? 'OK' : r.error}`).join(', ');
        statusText.textContent = `Saved - ${summary}`;
    } catch (e) {
        statusText.textContent = `Save error: ${e}`;
        console.error('Save failed:', e);
    }
}

function cancelScreenshot() {
    screenshotB64 = null;
    mainView.classList.add('hidden');
    statusText.textContent = 'Ready';
}

// ===== Settings =====
async function loadSettings() {
    try {
        const config = await invoke('get_config');
        document.getElementById('setting-hotkey').value = config.hotkey;
        document.getElementById('setting-savepath').value = config.default_save_path;
        document.getElementById('setting-filename').value = config.filename_template;
        document.getElementById('setting-mdpath').value = config.markdown_path || '';
        document.getElementById('setting-clipboard').checked =
            config.save_targets.some(t => t.target_type === 'Clipboard' && t.enabled);
        document.getElementById('setting-local').checked =
            config.save_targets.some(t => t.target_type === 'LocalFile' && t.enabled);
    } catch (e) {
        console.error('Load settings failed:', e);
    }
}

async function saveSettings() {
    const config = await invoke('get_config');
    config.hotkey = document.getElementById('setting-hotkey').value;
    config.default_save_path = document.getElementById('setting-savepath').value;
    config.filename_template = document.getElementById('setting-filename').value;
    config.markdown_path = document.getElementById('setting-mdpath').value || null;

    // Update save targets
    config.save_targets = config.save_targets.map(t => {
        if (t.target_type === 'Clipboard') t.enabled = document.getElementById('setting-clipboard').checked;
        if (t.target_type === 'LocalFile') t.enabled = document.getElementById('setting-local').checked;
        return t;
    });

    try {
        await invoke('update_config', { newConfig: config });
        statusText.textContent = 'Settings saved';
        settingsView.classList.add('hidden');
        if (screenshotB64) mainView.classList.remove('hidden');
    } catch (e) {
        statusText.textContent = `Settings error: ${e}`;
    }
}

// ===== Keyboard Shortcuts =====
document.addEventListener('keydown', e => {
    if (e.ctrlKey && e.key === 's') { e.preventDefault(); saveScreenshot(); }
    if (e.ctrlKey && e.key === 'z') { e.preventDefault(); undo(); }
    if (e.ctrlKey && e.key === 'y') { e.preventDefault(); redo(); }
    if (e.key === 'Escape') { cancelScreenshot(); }
});

// ===== Tauri Event Listeners =====
listen('screenshot:triggered', () => {
    takeScreenshot();
});

listen('settings:open', () => {
    mainView.classList.add('hidden');
    settingsView.classList.remove('hidden');
    loadSettings();
});

// ===== Init =====
statusText.textContent = 'CapDrop ready - press hotkey to capture';
