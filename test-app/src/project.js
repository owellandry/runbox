/**
 * Project Module - Crear archivos de ejemplo en RunBox VFS
 */

import { addTerminalLine } from './ui.js';

export function createProjectFiles() {
    const files = {
        'package.json': createPackageJson(),
        'index.html': createIndexHtml(),
        'style.css': createStyleCss(),
        'app.js': createAppJs(),
    };

    Object.entries(files).forEach(([name, content]) => {
        addTerminalLine(`📝 ${name}`, 'info');
    });

    // Guardar en memoria/estado global
    window.runboxApp.projectFiles = files;
    window.runboxApp.currentFile = null;

    addTerminalLine('✅ 4 files created', 'success');
    addTerminalLine('📦 Dependencies: express@^4.18.0, vite@^5.0.0', 'info');
}

function createPackageJson() {
    return JSON.stringify({
        name: "runbox-sandbox-app",
        version: "1.0.0",
        type: "module",
        description: "A web application running in RunBox WebAssembly sandbox",
        scripts: {
            "dev": "python -m http.server 3000",
            "build": "echo 'Building...'",
            "test": "echo 'Running tests...'"
        },
        dependencies: {
            "express": "^4.18.0"
        },
        devDependencies: {
            "vite": "^5.0.0"
        }
    }, null, 2);
}

function createIndexHtml() {
    return `<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RunBox App</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>🎁 Mi App en RunBox</h1>
            <p class="subtitle">Ejecutando en WebAssembly Sandbox</p>
        </header>

        <main>
            <div class="card">
                <h2>✨ Bienvenido</h2>
                <p>Esta es una aplicación renderizada dentro del sandbox RunBox.</p>
                <p>Puedes editar los archivos y ver los cambios en tiempo real.</p>
            </div>

            <div class="stats">
                <div class="stat-item">
                    <span class="icon">⚡</span>
                    <span class="label">WebAssembly</span>
                </div>
                <div class="stat-item">
                    <span class="icon">🔒</span>
                    <span class="label">Aislado</span>
                </div>
                <div class="stat-item">
                    <span class="icon">🚀</span>
                    <span class="label">Rápido</span>
                </div>
                <div class="stat-item">
                    <span class="icon">🎨</span>
                    <span class="label">Interactivo</span>
                </div>
            </div>

            <div class="form-section">
                <h3>Prueba el formulario:</h3>
                <form onsubmit="handleSubmit(event)">
                    <input type="text" id="name-input" placeholder="Tu nombre" required>
                    <button type="submit">Enviar</button>
                </form>
                <p id="form-result"></p>
            </div>
        </main>

        <footer>
            <p>Hecho con ❤️ en RunBox WebAssembly</p>
        </footer>
    </div>

    <script src="app.js"><\/script>
</body>
</html>`;
}

function createStyleCss() {
    return `* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    padding: 20px;
}

.container {
    max-width: 800px;
    margin: 0 auto;
    background: white;
    border-radius: 15px;
    overflow: hidden;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 40px 20px;
    text-align: center;
}

header h1 {
    font-size: 2.5em;
    margin-bottom: 10px;
}

.subtitle {
    opacity: 0.9;
    font-size: 1.1em;
}

main {
    padding: 40px 20px;
}

.card {
    background: #f5f5f5;
    padding: 20px;
    border-radius: 10px;
    margin-bottom: 30px;
    border-left: 4px solid #667eea;
}

.card h2 {
    color: #333;
    margin-bottom: 15px;
}

.card p {
    color: #666;
    line-height: 1.6;
    margin-bottom: 10px;
}

.stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 15px;
    margin: 30px 0;
}

.stat-item {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 20px;
    border-radius: 10px;
    text-align: center;
    transition: transform 0.3s;
}

.stat-item:hover {
    transform: translateY(-5px);
}

.stat-item .icon {
    font-size: 2em;
    display: block;
    margin-bottom: 10px;
}

.stat-item .label {
    font-weight: 600;
}

.form-section {
    margin-top: 30px;
}

.form-section h3 {
    color: #333;
    margin-bottom: 15px;
}

form {
    display: flex;
    gap: 10px;
    margin-bottom: 15px;
}

input[type="text"] {
    flex: 1;
    padding: 12px;
    border: 1px solid #ddd;
    border-radius: 5px;
    font-size: 1em;
}

button[type="submit"] {
    padding: 12px 24px;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 5px;
    cursor: pointer;
    font-weight: 600;
    transition: background 0.3s;
}

button[type="submit"]:hover {
    background: #764ba2;
}

#form-result {
    color: #667eea;
    font-weight: 600;
}

footer {
    background: #f5f5f5;
    padding: 20px;
    text-align: center;
    color: #666;
    border-top: 1px solid #ddd;
}

@media (max-width: 600px) {
    header h1 {
        font-size: 1.8em;
    }

    form {
        flex-direction: column;
    }
}`;
}

function createAppJs() {
    return `console.log('✅ App.js cargado en RunBox');

function handleSubmit(event) {
    event.preventDefault();
    const nameInput = document.getElementById('name-input');
    const result = document.getElementById('form-result');

    const name = nameInput.value;
    result.textContent = \`¡Hola \${name}! Bienvenido a RunBox 🚀\`;
    nameInput.value = '';
}

// Mensaje inicial
console.log('🎁 Aplicación renderizada en RunBox WebAssembly');
console.log('🔒 Ejecución completamente aislada');
console.log('⚡ Sin requerir servidor backend');`;
}

// Exportar archivos para acceso global
export function getProjectFiles() {
    return window.runboxApp.projectFiles || {};
}

export function getFileContent(filename) {
    return getProjectFiles()[filename] || '';
}

export function updateFileContent(filename, content) {
    if (!window.runboxApp.projectFiles) {
        window.runboxApp.projectFiles = {};
    }
    window.runboxApp.projectFiles[filename] = content;
}
