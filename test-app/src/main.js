/**
 * RunBox Test App - WebAssembly Sandbox
 * Importa RunBox y renderiza código en el sandbox
 */

import init, * as runbox from 'runbox';
import { setupUI, addTerminalLine, clearTerminal, updatePreview } from './ui.js';
import { createProjectFiles, getProjectFiles, getFileContent, updateFileContent } from './project.js';
import { executeCommand, clearConsole, loadFile, saveFile } from './executor.js';

let runboxReady = false;

async function main() {
    try {
        // 1. Inicializar RunBox WASM
        console.log('⏳ Inicializando RunBox WebAssembly...');
        await init();
        runboxReady = true;
        console.log('✅ RunBox inicializado correctamente');

        // 2. Crear la UI
        setupUI();

        // 3. Crear archivos de proyecto en el VFS de RunBox
        console.log('📁 Creando archivos de proyecto en VFS...');
        createProjectFiles();

        // 4. Mostrar status
        updateStatus('✓ RunBox Ready', 'success');
        console.log('✨ Sistema listo para usar');

        addTerminalLine('✨ RunBox Sandbox iniciado', 'success');
        addTerminalLine('Escribe "help" para ver comandos disponibles', 'info');

    } catch (error) {
        console.error('❌ Error al inicializar:', error);
        updateStatus('✗ Error', 'error');
        addTerminalLine('Error: ' + error.message, 'error');
    }
}

function updateStatus(text, type = '') {
    const statusEl = document.querySelector('.status-badge');
    if (statusEl) {
        statusEl.textContent = text;
        statusEl.className = 'status-badge ' + type;
    }
}

// Exponer globalmente para acceso desde UI
window.runboxApp = {
    // Módulos
    runbox,

    // Estado
    runboxReady: () => runboxReady,
    projectFiles: {},
    currentFile: null,

    // Métodos de status
    init,
    updateStatus,

    // Métodos de terminal
    executeCommand(cmd = null) {
        const input = document.getElementById('terminal-input');
        const command = cmd || (input ? input.value : '');

        if (!command.trim()) return;

        executeCommand(command);
        if (input) input.value = '';
    },

    handleCommand(event) {
        if (event.key === 'Enter') {
            this.executeCommand();
        }
    },

    clearConsole() {
        clearTerminal();
    },

    // Métodos de archivos
    loadFile(filename) {
        loadFile(filename);
    },

    saveFile() {
        saveFile();
    },

    // Métodos de desarrollo
    runDev() {
        this.executeCommand('run-dev');
    },

    installDeps() {
        this.executeCommand('npm install');
    },

    buildProject() {
        this.executeCommand('npm run build');
    },

    showStats() {
        this.executeCommand('ls -la');
    },

    resetProject() {
        if (confirm('¿Estás seguro de que quieres resetear el proyecto?')) {
            createProjectFiles();
            addTerminalLine('🔄 Proyecto reseteado', 'success');
        }
    },

    // Acceso a datos
    getProjectFiles,
    getFileContent,
    updateFileContent,
};

// Iniciar cuando DOM esté listo
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', main);
} else {
    main();
}

console.log('💡 Acceso global: window.runboxApp');
