// =========================================
// SmartScan — Frontend Logic
// =========================================

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const appWindow = getCurrentWindow();

// --- Window Controls ---
document.addEventListener('DOMContentLoaded', () => {
    document.getElementById('btn-minimize').addEventListener('click', () => appWindow.minimize());
    document.getElementById('btn-maximize').addEventListener('click', async () => {
        const maximized = await appWindow.isMaximized();
        if (maximized) { appWindow.unmaximize(); } else { appWindow.maximize(); }
    });
    document.getElementById('btn-close').addEventListener('click', () => appWindow.close());
});

// --- State ---
let scanResult = null;
let activeCategory = 'all';
let isScanning = false;
let currentScanId = null;
let isImportedScan = false;

// --- Category Icons (SVG paths) ---
const categoryIcons = {
    shield: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>',
    monitor: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>',
    cpu: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="14" x2="23" y2="14"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="14" x2="4" y2="14"/></svg>',
    fingerprint: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12C2 6.5 6.5 2 12 2a10 10 0 0 1 8 4"/><path d="M5 19.5C5.5 18 6 15 6 12c0-3.5 2.5-6 6-6 2 0 3.7.9 4.8 2.5"/><path d="M12 12v8c0 1-1 2-2.5 2"/><path d="M18 12a6 6 0 0 0-6-6"/><path d="M18 12c0 4-1 7-3 9"/></svg>',
    gamepad: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="6" y1="12" x2="10" y2="12"/><line x1="8" y1="10" x2="8" y2="14"/><line x1="15" y1="13" x2="15.01" y2="13"/><line x1="18" y1="11" x2="18.01" y2="11"/><path d="M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.414-1.414A2 2 0 0 1 9.828 16h4.344a2 2 0 0 1 1.414.586L17 18c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-1.544-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z"/></svg>',
    wifi: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/></svg>',
    all: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>',
};

// --- DOM Elements ---
const $ = (id) => document.getElementById(id);

// =========================================
// Particle System
// =========================================
const PARTICLE_COUNT = 120;
let particles = [];
let orbitAnimFrameId = null;
let orbitStartTime = null;

function createParticles() {
    const field = $('particle-field');
    if (!field) return;
    field.innerHTML = '';
    particles = [];
    stopOrbitAnimation();

    const rect = field.getBoundingClientRect();
    const cx = rect.width / 2;
    const cy = rect.height / 2;

    for (let i = 0; i < PARTICLE_COUNT; i++) {
        const el = document.createElement('div');
        el.className = 'particle idle';

        // Distribute in a tight ring around center (like a circle outline)
        const angle = (i / PARTICLE_COUNT) * Math.PI * 2;
        const baseRadius = 56 + Math.random() * 6; // 56-62px — very tight ring
        const x = cx + Math.cos(angle) * baseRadius;
        const y = cy + Math.sin(angle) * baseRadius;

        // Random size (slightly smaller for denser look)
        const size = 1.5 + Math.random() * 3;
        el.style.width = size + 'px';
        el.style.height = size + 'px';

        // Position
        el.style.left = x + 'px';
        el.style.top = y + 'px';

        // Color variation
        const hue = Math.random() > 0.5 ? '190' : '260'; // cyan or purple
        const lightness = 55 + Math.random() * 20;
        el.style.background = `hsl(${hue}, 90%, ${lightness}%)`;
        el.style.boxShadow = `0 0 ${size * 2}px hsl(${hue}, 90%, ${lightness}%)`;

        // Subtle float animation (keep tight ring shape)
        el.style.setProperty('--float-x', (Math.random() * 6 - 3) + 'px');
        el.style.setProperty('--float-y', (Math.random() * 6 - 3) + 'px');
        el.style.animationDelay = (Math.random() * 4) + 's';
        el.style.animationDuration = (3 + Math.random() * 3) + 's';

        // Store data
        el._angle = angle;
        el._baseRadius = baseRadius;
        el._cx = cx;
        el._cy = cy;
        el._size = size;

        field.appendChild(el);
        particles.push(el);
    }
}

function setParticlesScanning() {
    const field = $('particle-field');
    const rect = field.getBoundingClientRect();
    const cx = rect.width / 2;
    const cy = rect.height / 2;

    // For each particle: capture current position, compute angle, begin JS-driven orbit
    particles.forEach((el, i) => {
        // Switch to orbiting class (no CSS animation, JS controls position)
        el.className = 'particle orbiting';

        // Read current position from inline style
        const currentLeft = parseFloat(el.style.left) || cx;
        const currentTop = parseFloat(el.style.top) || cy;

        // Calculate current angle & radius relative to center
        const dx = currentLeft - cx;
        const dy = currentTop - cy;
        const currentAngle = Math.atan2(dy, dx);
        const currentRadius = Math.sqrt(dx * dx + dy * dy);

        // Target orbit parameters
        const targetRadius = 65 + Math.random() * 55; // 65–120px orbit
        const orbitSpeed = 0.6 + Math.random() * 0.7;  // rad/s

        // Store on element
        el._orbitCx = cx;
        el._orbitCy = cy;
        el._startAngle = currentAngle;
        el._startRadius = currentRadius;
        el._targetRadius = targetRadius;
        el._orbitSpeed = (i % 2 === 0 ? 1 : -1) * orbitSpeed; // alternate direction
    });

    // Kick off JS animation loop
    orbitStartTime = performance.now();
    animateOrbit();
}

function animateOrbit() {
    const now = performance.now();
    const elapsed = (now - orbitStartTime) / 1000; // seconds

    particles.forEach((el) => {
        // Smoothly transition radius from current→target over 1.2s (easeOutCubic)
        const t = Math.min(elapsed / 1.2, 1);
        const eased = 1 - Math.pow(1 - t, 3);
        const radius = el._startRadius + (el._targetRadius - el._startRadius) * eased;

        // Advance angle continuously from starting angle
        const angle = el._startAngle + el._orbitSpeed * elapsed;

        const x = el._orbitCx + Math.cos(angle) * radius;
        const y = el._orbitCy + Math.sin(angle) * radius;

        el.style.left = x + 'px';
        el.style.top = y + 'px';

        // Store live values for explode
        el._liveAngle = angle;
        el._liveX = x;
        el._liveY = y;
    });

    orbitAnimFrameId = requestAnimationFrame(animateOrbit);
}

function stopOrbitAnimation() {
    if (orbitAnimFrameId) {
        cancelAnimationFrame(orbitAnimFrameId);
        orbitAnimFrameId = null;
    }
    orbitStartTime = null;
}

function explodeParticles() {
    // Stop the JS orbit loop first
    stopOrbitAnimation();

    particles.forEach((el) => {
        // Switch to explode CSS class (transition-based)
        el.className = 'particle explode';

        // Fly outward from current live position
        const angle = el._liveAngle || el._startAngle || el._angle || 0;
        const flyDist = 200 + Math.random() * 350;
        const curX = el._liveX || parseFloat(el.style.left);
        const curY = el._liveY || parseFloat(el.style.top);
        const targetX = curX + Math.cos(angle) * flyDist;
        const targetY = curY + Math.sin(angle) * flyDist;

        el.style.left = targetX + 'px';
        el.style.top = targetY + 'px';
        el.style.opacity = '0';
        el.style.transform = `scale(${0.3 + Math.random() * 1.5})`;
        el.style.transitionDelay = (Math.random() * 0.2) + 's';
    });
}

function resetParticles() {
    stopOrbitAnimation();
    const field = $('particle-field');
    if (field) field.innerHTML = '';
    particles = [];
}

// Initialize particles on load
document.addEventListener('DOMContentLoaded', () => {
    setTimeout(createParticles, 100);

    // Re-create on resize
    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(createParticles, 200);
    });
});

// --- Start Scan ---
window.startScan = async function () {
    if (isScanning) return;
    isScanning = true;

    const scanScreen = $('scan-screen');
    const resultsScreen = $('results-screen');
    const scanButton = $('scan-button');

    // Switch to scan screen if on results
    resultsScreen.classList.remove('active');
    scanScreen.classList.add('active');

    // Re-create particles if needed
    if (particles.length === 0) {
        createParticles();
        await sleep(100);
    }

    const statusText = $('scan-status-text');

    // Hide button with fade
    scanButton.disabled = true;
    scanButton.classList.add('hidden');
    await sleep(400); // wait for button fade-out

    // Show scanning text & start particle animation
    statusText.classList.add('visible');
    setParticlesScanning();

    try {
        // Call Rust backend
        const result = await invoke('run_scan');
        scanResult = result;

        // Hide scanning text
        statusText.classList.remove('visible');

        // Explode particles outward
        explodeParticles();
        await sleep(1200);

        // Switch to results screen
        activeCategory = 'all';
        renderResults(result);
        scanScreen.classList.remove('active');
        resultsScreen.classList.add('active');

    } catch (err) {
        console.error('Scan failed:', err);
    } finally {
        isScanning = false;
        statusText.classList.remove('visible');
        // Clean up particles
        resetParticles();
        // Keep button hidden — only restore after scan-screen is no longer visible
        scanButton.disabled = false;
        // Re-create idle particles for next time
        setTimeout(() => {
            createParticles();
            // Only un-hide button after a short delay so screen transition is complete
            scanButton.classList.remove('hidden');
        }, 400);
    }
};

// --- Render Results ---
function renderResults(result) {
    renderTabs(result.categories);
    renderEntries(result.categories, 'all');
    updateSummary(result);
}

function renderTabs(categories) {
    const tabsContainer = $('category-tabs');
    tabsContainer.innerHTML = '';

    // "All" tab
    const allTab = createTab('all', 'All', 'all', categories.reduce((s, c) => s + c.entries.length, 0));
    allTab.classList.add('active');
    tabsContainer.appendChild(allTab);

    // Category tabs
    for (const cat of categories) {
        const tab = createTab(cat.id, cat.name, cat.icon, cat.entries.length);
        tabsContainer.appendChild(tab);
    }
}

function createTab(id, name, icon, count) {
    const tab = document.createElement('button');
    tab.className = 'category-tab';
    tab.dataset.id = id;
    tab.innerHTML = `
        <span class="tab-icon">${categoryIcons[icon] || categoryIcons.all}</span>
        <span>${name}</span>
        <span class="tab-count">${count}</span>
    `;
    tab.addEventListener('click', () => {
        document.querySelectorAll('.category-tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        activeCategory = id;
        renderEntries(scanResult.categories, id);
    });
    return tab;
}

function renderEntries(categories, filterId) {
    const container = $('results-container');
    container.innerHTML = '';

    const categoriesToShow = filterId === 'all'
        ? categories
        : categories.filter(c => c.id === filterId);

    let entryIndex = 0;
    for (const cat of categoriesToShow) {
        // Section header (only in "all" view)
        if (filterId === 'all') {
            const header = document.createElement('div');
            header.className = 'category-section-header';
            header.innerHTML = `
                <span class="tab-icon">${categoryIcons[cat.icon] || ''}</span>
                <span>${cat.name}</span>
                <span class="section-line"></span>
            `;
            container.appendChild(header);
        }

        for (const entry of cat.entries) {
            const el = document.createElement('div');
            el.className = 'result-entry';
            el.style.animationDelay = `${entryIndex * 0.03}s`;
            el.innerHTML = `
                <span class="status-dot ${entry.status}"></span>
                <span class="entry-label">${escapeHtml(entry.label)}</span>
                <span class="entry-value ${entry.status}">${escapeHtml(entry.value)}</span>
            `;
            container.appendChild(el);
            entryIndex++;
        }
    }
}

function updateSummary(result) {
    const allEntries = result.categories.flatMap(c => c.entries);
    $('total-checks').textContent = allEntries.length;
    $('ok-count').textContent = allEntries.filter(e => e.status === 'ok').length;
    $('warning-count').textContent = allEntries.filter(e => e.status === 'warning').length;
    $('error-count').textContent = allEntries.filter(e => e.status === 'error').length;
}

// --- Copy Results ---
window.copyResults = function () {
    if (!scanResult) return;

    let text = '═══════════════════════════════════════\n';
    text += '         SmartScan Report v1.0.0\n';
    text += '═══════════════════════════════════════\n\n';

    for (const cat of scanResult.categories) {
        text += `── ${cat.name} ──────────────────────\n`;
        for (const entry of cat.entries) {
            const statusIcon = { ok: '✅', warning: '⚠️', error: '❌', info: 'ℹ️' }[entry.status] || '•';
            text += `  ${statusIcon} ${entry.label}: ${entry.value}\n`;
        }
        text += '\n';
    }

    text += '═══════════════════════════════════════\n';
    const allEntries = scanResult.categories.flatMap(c => c.entries);
    text += `Total: ${allEntries.length} checks | `;
    text += `OK: ${allEntries.filter(e => e.status === 'ok').length} | `;
    text += `Warnings: ${allEntries.filter(e => e.status === 'warning').length} | `;
    text += `Issues: ${allEntries.filter(e => e.status === 'error').length}\n`;

    navigator.clipboard.writeText(text).then(() => {
        showToast('Copied to clipboard!');
    }).catch(() => {
        showToast('Failed to copy');
    });
};

// --- Toast ---
function showToast(message) {
    const toast = $('toast');
    const toastText = $('toast-text');
    toastText.textContent = message;
    toast.classList.add('show');
    setTimeout(() => toast.classList.remove('show'), 2500);
}

// --- Util ---
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

// --- Share Scan (Upload to Upstash) ---
window.shareScan = async function () {
    if (!scanResult) return;

    const shareBtn = $('share-btn');
    shareBtn.disabled = true;
    showToast('Uploading scan...');

    try {
        const scanId = await invoke('upload_scan', { scanResult });
        currentScanId = scanId;

        // Show share modal with ID
        $('scan-id-text').textContent = scanId;
        $('share-modal').classList.add('active');
    } catch (err) {
        showToast('Upload failed: ' + (err.message || err));
    } finally {
        shareBtn.disabled = false;
    }
};

window.copyScanId = function () {
    const scanId = $('scan-id-text').textContent;
    navigator.clipboard.writeText(scanId).then(() => {
        showToast('Scan-ID copied!');
    });
};

window.closeShareModal = function () {
    $('share-modal').classList.remove('active');
};

// --- Import Scan (Download from Upstash) ---
window.openImportModal = function () {
    $('import-id-input').value = '';
    $('import-status').textContent = '';
    $('import-status').className = 'modal-status';
    $('import-submit').disabled = false;
    $('import-modal').classList.add('active');
    setTimeout(() => $('import-id-input').focus(), 100);
};

window.closeImportModal = function () {
    $('import-modal').classList.remove('active');
};

window.importScan = async function () {
    const input = $('import-id-input');
    const statusEl = $('import-status');
    const submitBtn = $('import-submit');
    const scanId = input.value.trim();

    if (!scanId) {
        statusEl.textContent = 'Please enter a Scan-ID.';
        statusEl.className = 'modal-status error';
        return;
    }

    if (!scanId.startsWith('SmartScan-')) {
        statusEl.textContent = 'Invalid format. Must start with SmartScan-';
        statusEl.className = 'modal-status error';
        return;
    }

    submitBtn.disabled = true;
    statusEl.textContent = 'Loading scan data...';
    statusEl.className = 'modal-status loading';

    try {
        const result = await invoke('import_scan', { scanId });
        scanResult = result;
        currentScanId = scanId;
        isImportedScan = true;

        // Close modal
        $('import-modal').classList.remove('active');

        // Switch to results screen
        activeCategory = 'all';
        renderResults(result);

        // Show imported indicator
        const summaryBar = $('summary-bar');
        // Remove any existing badge
        const existingBadge = summaryBar.querySelector('.imported-badge');
        if (existingBadge) existingBadge.remove();
        // Add imported badge
        const badge = document.createElement('span');
        badge.className = 'imported-badge';
        badge.textContent = 'IMPORTED';
        summaryBar.querySelector('.summary-actions').before(badge);

        $('scan-screen').classList.remove('active');
        $('results-screen').classList.add('active');
        showToast('Scan imported successfully!');

    } catch (err) {
        statusEl.textContent = err.message || err || 'Failed to load scan.';
        statusEl.className = 'modal-status error';
        submitBtn.disabled = false;
    }
};

// Handle Enter key in import input
document.addEventListener('DOMContentLoaded', () => {
    const importInput = document.getElementById('import-id-input');
    if (importInput) {
        importInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                window.importScan();
            }
        });
    }

    // Close modals on overlay click
    document.querySelectorAll('.modal-overlay').forEach(overlay => {
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) {
                overlay.classList.remove('active');
            }
        });
    });
});
