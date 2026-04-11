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
                    <h1>🎁 RunBox Sandbox IDE</h1>
                    <p class="subtitle">Professional Code Editor with WebAssembly Runtime</p>
                    <div class="status-badge loading">⏳ Inicializando...</div>
                </div>
            </header>

            <!-- Main Grid Layout -->
            <div class="sandbox-grid">
                <!-- Panel 1: File Browser (Izquierda) -->
                <div class="panel file-panel">
                    <div class="panel-header">
                        📂 Archivos
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
                    </div>
                </div>

                <!-- Panel 2: Code Editor (Centro) -->
                <div class="panel editor-panel">
                    <div class="panel-header">
                        ✏️ Editor
                    </div>
                    <div class="panel-content">
                        <div class="editor-wrapper">
                            <div class="editor-header">
                                <div class="editor-filename" id="editor-filename">Sin archivo seleccionado</div>
                            </div>
                            <div class="editor-container">
                                <div class="line-numbers" id="line-numbers"></div>
                                <textarea id="editor-content" class="code-editor" placeholder="Selecciona un archivo para editar..."></textarea>
                            </div>
                            <div class="editor-footer">
                                <div class="editor-info">
                                    <span id="char-count">0 caracteres</span>
                                    <span id="line-count">1 línea</span>
                                </div>
                                <button class="btn-save" onclick="window.runboxApp.saveFile()">💾 Guardar</button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Panel 3: Preview (Derecha-Arriba) -->
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

                <!-- Panel 4: Terminal (Derecha-Abajo) -->
                <div class="panel terminal-panel">
                    <div class="panel-header">
                        💻 Terminal
                        <button class="btn-clear" onclick="window.runboxApp.clearConsole()">🗑️</button>
                    </div>
                    <div class="panel-content">
                        <div class="terminal" id="terminal">
                            <div class="terminal-line">$ RunBox Sandbox Ready</div>
                        </div>
                        <div class="terminal-input-group">
                            <input type="text" id="terminal-input" class="terminal-input"
                                placeholder="$ Escribe un comando: ls, cat, npm install, preview..."
                                onkeypress="window.runboxApp.handleCommand(event)">
                            <button class="btn-execute" onclick="window.runboxApp.executeCommand()">▶️</button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Control Panel (Abajo) -->
            <div class="control-panel">
                <div class="controls">
                    <button class="btn-primary" onclick="window.runboxApp.runDev()">
                        ▶️ Run Dev
                    </button>
                    <button class="btn-secondary" onclick="window.runboxApp.installDeps()">
                        📦 Install
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
    setupEditor();
    setupTerminal();
}

function setupFileTree() {
    const fileItems = document.querySelectorAll('.file-item');
    fileItems.forEach(item => {
        item.addEventListener('click', function() {
            // Remover clase active de todos
            fileItems.forEach(i => i.classList.remove('active'));

            // Agregar clase active al clickeado
            this.classList.add('active');

            const path = this.dataset.path;
            window.runboxApp.loadFile(path);
        });
    });
}

function setupEditor() {
    const editor = document.getElementById('editor-content');

    if (editor) {
        // Actualizar info en tiempo real
        editor.addEventListener('input', updateEditorInfo);
        editor.addEventListener('scroll', syncLineNumberScroll);
    }
}

function updateEditorInfo() {
    const editor = document.getElementById('editor-content');
    const charCount = document.getElementById('char-count');
    const lineCount = document.getElementById('line-count');

    if (editor && charCount && lineCount) {
        const chars = editor.value.length;
        const lines = editor.value.split('\n').length;

        charCount.textContent = `${chars} caracteres`;
        lineCount.textContent = `${lines} línea${lines !== 1 ? 's' : ''}`;
    }

    updateLineNumbers();
}

function updateLineNumbers() {
    const editor = document.getElementById('editor-content');
    const lineNumbers = document.getElementById('line-numbers');

    if (!editor || !lineNumbers) return;

    const lines = editor.value.split('\n').length;
    let html = '';

    for (let i = 1; i <= lines; i++) {
        html += i + '\n';
    }

    lineNumbers.textContent = html;
}

function syncLineNumberScroll() {
    const editor = document.getElementById('editor-content');
    const lineNumbers = document.getElementById('line-numbers');

    if (editor && lineNumbers) {
        lineNumbers.scrollTop = editor.scrollTop;
    }
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

export function updateEditorDisplay(filename, content) {
    const editor = document.getElementById('editor-content');
    const filename_display = document.getElementById('editor-filename');

    if (editor) {
        editor.value = content;
        updateEditorInfo();
    }

    if (filename_display) {
        filename_display.textContent = filename;
    }
}
