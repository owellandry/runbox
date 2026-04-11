/**
 * Executor Module - Ejecutar comandos y manejar sandbox
 */

import { addTerminalLine, updatePreview } from './ui.js';
import { getProjectFiles, getFileContent } from './project.js';

export function executeCommand(command) {
    if (!command.trim()) return;

    addTerminalLine(`$ ${command}`, 'command');

    // Simular ejecución de comandos
    const [cmd, ...args] = command.split(' ');

    switch (cmd) {
        case 'ls':
        case 'ls -la':
            handleLs();
            break;

        case 'cat':
            handleCat(args[0]);
            break;

        case 'npm':
            handleNpm(args);
            break;

        case 'echo':
            handleEcho(args);
            break;

        case 'pwd':
            addTerminalLine('/sandbox', 'output');
            break;

        case 'clear':
            window.runboxApp.clearConsole();
            break;

        case 'help':
            handleHelp();
            break;

        case 'preview':
            handlePreview();
            break;

        case 'run-dev':
            handleRunDev();
            break;

        default:
            addTerminalLine(`Comando no reconocido: ${cmd}`, 'error');
            addTerminalLine('Escribe "help" para ver comandos disponibles', 'info');
    }
}

function handleLs() {
    const files = getProjectFiles();
    addTerminalLine('Archivos del proyecto:', 'output');
    Object.keys(files).forEach(name => {
        const content = files[name];
        const size = new Blob([content]).size;
        addTerminalLine(`  ${name.padEnd(20)} ${size} bytes`, 'output');
    });
}

function handleCat(filename) {
    if (!filename) {
        addTerminalLine('cat: missing filename', 'error');
        return;
    }

    const content = getFileContent(filename);
    if (!content) {
        addTerminalLine(`cat: ${filename}: No such file or directory`, 'error');
        return;
    }

    addTerminalLine(`--- ${filename} ---`, 'output');
    content.split('\n').forEach(line => {
        addTerminalLine(line, 'output');
    });
    addTerminalLine(`--- end ${filename} ---`, 'output');
}

function handleEcho(args) {
    const text = args.join(' ').replace(/^["']|["']$/g, '');
    addTerminalLine(text, 'output');
}

function handleNpm(args) {
    const subcommand = args[0];

    switch (subcommand) {
        case 'install':
            addTerminalLine('📦 Installing dependencies...', 'info');
            addTerminalLine('✅ Dependencies installed', 'success');
            break;

        case 'run':
            const script = args[1];
            if (script === 'dev') {
                addTerminalLine('🚀 Starting dev server...', 'info');
                addTerminalLine('Server running at http://localhost:3000', 'success');
            } else {
                addTerminalLine(`npm run ${script}`, 'output');
            }
            break;

        case 'list':
            addTerminalLine('📦 Project Dependencies:', 'output');
            addTerminalLine('  (no dependencies yet)', 'output');
            break;

        default:
            addTerminalLine(`npm ${subcommand}`, 'output');
    }
}

function handleHelp() {
    const commands = [
        ['ls', 'Lista los archivos del proyecto'],
        ['cat <file>', 'Muestra el contenido de un archivo'],
        ['echo <text>', 'Imprime texto'],
        ['npm install', 'Instala dependencias'],
        ['npm run dev', 'Inicia servidor de desarrollo'],
        ['npm list', 'Lista dependencias'],
        ['preview', 'Renderiza la app en el preview'],
        ['run-dev', 'Ejecuta todo el pipeline de desarrollo'],
        ['pwd', 'Muestra el directorio actual'],
        ['clear', 'Limpia la consola'],
        ['help', 'Muestra esta ayuda'],
    ];

    addTerminalLine('📚 Comandos disponibles:', 'info');
    commands.forEach(([cmd, desc]) => {
        addTerminalLine(`  ${cmd.padEnd(20)} - ${desc}`, 'output');
    });
}

function handlePreview() {
    addTerminalLine('🔄 Renderizando preview...', 'info');

    const files = getProjectFiles();
    let html = files['index.html'] || '';

    if (!html) {
        addTerminalLine('❌ index.html no encontrado', 'error');
        return;
    }

    // Inyectar CSS
    if (files['style.css']) {
        const cssTag = `<style>${files['style.css']}</style>`;
        html = html.replace('</head>', `${cssTag}</head>`);
    }

    // Inyectar JS
    if (files['app.js']) {
        const jsTag = `<script>${files['app.js']}<\/script>`;
        html = html.replace('</body>', `${jsTag}</body>`);
    }

    updatePreview(html);
    addTerminalLine('✅ Preview actualizado', 'success');
}

function handleRunDev() {
    addTerminalLine('🚀 Ejecutando pipeline de desarrollo...', 'info');
    addTerminalLine('  1. Instalando dependencias...', 'output');
    addTerminalLine('  ✅ Dependencies installed', 'success');
    addTerminalLine('  2. Compilando proyecto...', 'output');
    addTerminalLine('  ✅ Build successful', 'success');
    addTerminalLine('  3. Renderizando preview...', 'output');

    handlePreview();

    addTerminalLine('  ✅ Server running at http://localhost:3000', 'success');
    addTerminalLine('✨ Pipeline completado', 'success');
}

export function clearConsole() {
    const terminal = document.getElementById('terminal');
    if (terminal) {
        terminal.innerHTML = '<div class="terminal-line">$ Terminal cleared</div>';
    }
}

export function loadFile(filename) {
    const files = getProjectFiles();
    const content = files[filename];

    if (!content) {
        addTerminalLine(`Error: No se pudo cargar ${filename}`, 'error');
        return;
    }

    const editor = document.getElementById('file-editor');
    const editorContent = document.getElementById('editor-content');

    if (editor && editorContent) {
        editorContent.value = content;
        editor.style.display = 'block';
        window.runboxApp.currentFile = filename;
        editorContent.focus();
    }
}

export function saveFile() {
    const currentFile = window.runboxApp.currentFile;
    const editorContent = document.getElementById('editor-content');

    if (!currentFile || !editorContent) return;

    const newContent = editorContent.value;
    if (!window.runboxApp.projectFiles) {
        window.runboxApp.projectFiles = {};
    }

    window.runboxApp.projectFiles[currentFile] = newContent;
    addTerminalLine(`💾 ${currentFile} guardado`, 'success');
}
