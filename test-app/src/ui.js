/**
 * UI Module - Setup de interfaz del sandbox
 */

export function setupUI() {
    const app = document.getElementById('app');

    const html = `
        <div class="sandbox-container">
            <!-- Header -->
            <header class="sandbox-header">
                <div class="header-content">
                    <h1>🎁 RunBox Sandbox</h1>
                    <p class="subtitle">WebAssembly Virtual Environment</p>
                    <div class="status-badge loading">⏳ Inicializando...</div>
                </div>
            </header>

            <!-- Main Grid -->
            <div class="sandbox-grid">
                <!-- Editor / Files -->
                <div class="panel editor-panel">
                    <div class="panel-header">
                        📝 Archivos del Proyecto
                    </div>
                    <div class="panel-content">
                        <div class="file-tree" id="file-tree">
                            <div class="file-item" data-path="index.html">
                                <span class="file-icon">📄</span>
                                <span>index.html</span>
                            </div>
                            <div class="file-item" data-path="style.css">
                                <span class="file-icon">🎨</span>
                                <span>style.css</span>
                            </div>
                            <div class="file-item" data-path="app.js">
                                <span class="file-icon">⚙️</span>
                                <span>app.js</span>
                            </div>
                            <div class="file-item" data-path="package.json">
                                <span class="file-icon">📦</span>
                                <span>package.json</span>
                            </div>
                        </div>

                        <div class="file-editor" id="file-editor" style="display: none;">
                            <textarea id="editor-content" placeholder="Edita el contenido aquí"></textarea>
                            <button class="btn-save" onclick="window.runboxApp.saveFile()">💾 Guardar</button>
                        </div>
                    </div>
                </div>

                <!-- Preview -->
                <div class="panel preview-panel">
                    <div class="panel-header">
                        👁️ Preview
                    </div>
                    <div class="panel-content">
                        <iframe id="preview-frame" class="preview-iframe"
                            sandbox="allow-scripts allow-same-origin allow-forms">
                        </iframe>
                    </div>
                </div>

                <!-- Terminal -->
                <div class="panel terminal-panel full-width">
                    <div class="panel-header">
                        💻 Terminal / Console
                        <button class="btn-clear" onclick="window.runboxApp.clearConsole()">🗑️ Clear</button>
                    </div>
                    <div class="panel-content">
                        <div class="terminal" id="terminal">
                            <div class="terminal-line">$ RunBox Sandbox Ready</div>
                        </div>
                        <div class="terminal-input-group">
                            <input type="text" id="terminal-input" class="terminal-input"
                                placeholder="Escribe un comando: npm install, npm run dev, etc..."
                                onkeypress="window.runboxApp.handleCommand(event)">
                            <button class="btn-execute" onclick="window.runboxApp.executeCommand()">▶️ Execute</button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Control Panel -->
            <div class="control-panel">
                <div class="controls">
                    <button class="btn-primary" onclick="window.runboxApp.runDev()">
                        ▶️ Run Dev Server
                    </button>
                    <button class="btn-secondary" onclick="window.runboxApp.installDeps()">
                        📦 Install Dependencies
                    </button>
                    <button class="btn-secondary" onclick="window.runboxApp.buildProject()">
                        🏗️ Build
                    </button>
                    <button class="btn-secondary" onclick="window.runboxApp.showStats()">
                        📊 Stats
                    </button>
                    <button class="btn-secondary" onclick="window.runboxApp.resetProject()">
                        🔄 Reset
                    </button>
                </div>
            </div>
        </div>
    `;

    app.innerHTML = html;

    // Setup event listeners
    setupFileTree();
    setupTerminal();
}

function setupFileTree() {
    const fileItems = document.querySelectorAll('.file-item');
    fileItems.forEach(item => {
        item.addEventListener('click', function() {
            const path = this.dataset.path;
            window.runboxApp.loadFile(path);
        });
    });
}

function setupTerminal() {
    const input = document.getElementById('terminal-input');
    if (input) {
        input.focus();
    }
}

export function addTerminalLine(text, type = 'output') {
    const terminal = document.getElementById('terminal');
    if (terminal) {
        const line = document.createElement('div');
        line.className = `terminal-line terminal-${type}`;
        line.textContent = text;
        terminal.appendChild(line);
        terminal.scrollTop = terminal.scrollHeight;
    }
}

export function clearTerminal() {
    const terminal = document.getElementById('terminal');
    if (terminal) {
        terminal.innerHTML = '<div class="terminal-line">$ Terminal cleared</div>';
    }
}

export function updatePreview(html) {
    const frame = document.getElementById('preview-frame');
    if (frame) {
        frame.srcdoc = html;
    }
}
