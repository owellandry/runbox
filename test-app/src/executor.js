/**
 * Executor Module - Ejecutar comandos y manejar sandbox
 */

import { addTerminalLine, updatePreview } from './ui.js';
import { getProjectFiles, getFileContent } from './project.js';

export function executeCommand(command) {
    if (!command.trim()) return;

    addTerminalLine(`$ ${command}`, 'command');

    // Parsear comando - mejor manejo de espacios
    const parts = command.trim().split(/\s+/);
    const cmd = parts[0];
    const args = parts.slice(1);

    // Manejo especial para npm
    if (cmd === 'npm') {
        handleNpm(args);
        return;
    }

    switch (cmd) {
        case 'ls':
            handleLs();
            break;

        case 'cat':
            handleCat(args[0]);
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
            addTerminalLine(`Command not found: ${cmd}`, 'error');
            addTerminalLine('Type "help" for available commands', 'info');
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
        case 'i':
        case 'in':
            // Si hay argumentos después, es npm install <package>
            if (args.length > 1) {
                const packages = args.slice(1);
                handleNpmAdd(packages);
            } else {
                handleNpmInstall();
            }
            break;

        case 'add':
        case 'a':
            const packages = args.slice(1);
            if (packages.length === 0) {
                addTerminalLine('❌ npm add: missing package name', 'error');
                return;
            }
            handleNpmAdd(packages);
            break;

        case 'run':
            const script = args[1];
            if (!script) {
                addTerminalLine('❌ npm run: missing script name', 'error');
                return;
            }
            if (script === 'dev') {
                addTerminalLine('🚀 Starting dev server...', 'info');
                addTerminalLine('Server running at http://localhost:3000', 'success');
            } else {
                addTerminalLine(`📝 Running script: ${script}`, 'info');
            }
            break;

        case 'list':
        case 'ls':
        case 'l':
            handleNpmList();
            break;

        case 'remove':
        case 'rm':
        case 'r':
            const toRemove = args.slice(1);
            if (toRemove.length === 0) {
                addTerminalLine('❌ npm remove: missing package name', 'error');
                return;
            }
            handleNpmRemove(toRemove);
            break;

        case 'uninstall':
        case 'un':
            const toUninstall = args.slice(1);
            if (toUninstall.length === 0) {
                addTerminalLine('❌ npm uninstall: missing package name', 'error');
                return;
            }
            handleNpmRemove(toUninstall);
            break;

        default:
            addTerminalLine(`❌ npm: unknown command '${subcommand}'`, 'error');
            addTerminalLine('Type "help" for available commands', 'info');
    }
}

function handleNpmInstall() {
    const pkgContent = window.runboxApp.getFileContent('package.json');
    if (!pkgContent) {
        addTerminalLine('❌ package.json no encontrado', 'error');
        return;
    }

    try {
        const pkg = JSON.parse(pkgContent);
        const deps = Object.keys(pkg.dependencies || {});
        const devDeps = Object.keys(pkg.devDependencies || {});
        const allDeps = [...deps, ...devDeps];

        if (allDeps.length === 0) {
            addTerminalLine('📦 up to date — no dependencies found', 'info');
            return;
        }

        addTerminalLine('📦 Installing dependencies...', 'info');
        allDeps.forEach(dep => {
            addTerminalLine(`  ✅ ${dep}`, 'success');
        });
        addTerminalLine(`added ${allDeps.length} packages`, 'success');
    } catch (e) {
        addTerminalLine(`❌ Error parsing package.json: ${e.message}`, 'error');
    }
}

function handleNpmAdd(packages) {
    if (packages.length === 0) {
        addTerminalLine('❌ npm add: missing package name', 'error');
        return;
    }

    const pkgContent = window.runboxApp.getFileContent('package.json');
    if (!pkgContent) {
        addTerminalLine('❌ package.json no encontrado', 'error');
        return;
    }

    try {
        const pkg = JSON.parse(pkgContent);
        pkg.dependencies = pkg.dependencies || {};

        packages.forEach(pkg_name => {
            pkg.dependencies[pkg_name] = '*';
            addTerminalLine(`  ✅ ${pkg_name} added`, 'success');
        });

        const updated = JSON.stringify(pkg, null, 2);
        window.runboxApp.updateFileContent('package.json', updated);
        addTerminalLine(`added ${packages.length} packages`, 'success');
    } catch (e) {
        addTerminalLine(`❌ Error: ${e.message}`, 'error');
    }
}

function handleNpmList() {
    const pkgContent = window.runboxApp.getFileContent('package.json');
    if (!pkgContent) {
        addTerminalLine('❌ package.json no encontrado', 'error');
        return;
    }

    try {
        const pkg = JSON.parse(pkgContent);
        const deps = pkg.dependencies || {};
        const devDeps = pkg.devDependencies || {};

        if (Object.keys(deps).length === 0 && Object.keys(devDeps).length === 0) {
            addTerminalLine('📦 Project Dependencies:', 'output');
            addTerminalLine('  (no dependencies)', 'output');
            return;
        }

        addTerminalLine('📦 Project Dependencies:', 'output');

        if (Object.keys(deps).length > 0) {
            addTerminalLine('  Production:', 'output');
            Object.keys(deps).forEach(name => {
                addTerminalLine(`    ├─ ${name}@${deps[name]}`, 'output');
            });
        }

        if (Object.keys(devDeps).length > 0) {
            addTerminalLine('  Development:', 'output');
            Object.keys(devDeps).forEach(name => {
                addTerminalLine(`    ├─ ${name}@${devDeps[name]}`, 'output');
            });
        }
    } catch (e) {
        addTerminalLine(`❌ Error parsing package.json: ${e.message}`, 'error');
    }
}

function handleNpmRemove(packages) {
    if (packages.length === 0) {
        addTerminalLine('❌ npm remove: missing package name', 'error');
        return;
    }

    const pkgContent = window.runboxApp.getFileContent('package.json');
    if (!pkgContent) {
        addTerminalLine('❌ package.json no encontrado', 'error');
        return;
    }

    try {
        const pkg = JSON.parse(pkgContent);
        pkg.dependencies = pkg.dependencies || {};
        pkg.devDependencies = pkg.devDependencies || {};

        packages.forEach(pkg_name => {
            if (pkg.dependencies[pkg_name]) {
                delete pkg.dependencies[pkg_name];
                addTerminalLine(`  ✅ ${pkg_name} removed`, 'success');
            } else if (pkg.devDependencies[pkg_name]) {
                delete pkg.devDependencies[pkg_name];
                addTerminalLine(`  ✅ ${pkg_name} removed`, 'success');
            } else {
                addTerminalLine(`  ⚠️  ${pkg_name} not found`, 'info');
            }
        });

        const updated = JSON.stringify(pkg, null, 2);
        window.runboxApp.updateFileContent('package.json', updated);
        addTerminalLine(`removed packages`, 'success');
    } catch (e) {
        addTerminalLine(`❌ Error: ${e.message}`, 'error');
    }
}

function handleHelp() {
    const commands = [
        ['ls', 'Lista los archivos del proyecto'],
        ['cat <file>', 'Muestra el contenido de un archivo'],
        ['echo <text>', 'Imprime texto'],
        ['', ''],
        ['npm install', 'Instala todas las dependencias'],
        ['npm add <pkg>', 'Agrega un paquete (ej: npm add express)'],
        ['npm list', 'Lista todas las dependencias'],
        ['npm remove <pkg>', 'Elimina un paquete'],
        ['npm run dev', 'Inicia servidor de desarrollo'],
        ['', ''],
        ['preview', 'Renderiza la app en el preview'],
        ['run-dev', 'Ejecuta todo el pipeline'],
        ['', ''],
        ['pwd', 'Muestra el directorio actual'],
        ['clear', 'Limpia la terminal'],
        ['help', 'Muestra esta ayuda'],
    ];

    addTerminalLine('📚 Comandos disponibles:', 'info');
    commands.forEach(([cmd, desc]) => {
        if (cmd === '') {
            addTerminalLine('', 'output');
        } else {
            addTerminalLine(`  ${cmd.padEnd(25)} - ${desc}`, 'output');
        }
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

    window.runboxApp.currentFile = filename;
    window.runboxApp.updateEditorDisplay(filename, content);

    const editorContent = document.getElementById('editor-content');
    if (editorContent) {
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

    // Hot reload: actualizar preview si el archivo afecta el rendering
    if (['index.html', 'style.css', 'app.js'].includes(currentFile)) {
        setTimeout(() => handlePreview(), 100);
    }
}
