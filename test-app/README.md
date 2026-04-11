# 🎁 RunBox Sandbox App

Aplicación web completa que importa y utiliza **RunBox WebAssembly** para crear un sandbox de desarrollo interactivo.

## ¿Qué es?

Un IDE minimalista que corre completamente en el navegador:
- ✅ Importa RunBox compilado a WASM
- ✅ Crea archivos de proyecto (HTML, CSS, JS)
- ✅ Ejecuta comandos como `npm install`, `npm run`, etc.
- ✅ Renderiza preview en tiempo real
- ✅ Terminal interactiva
- ✅ Editor de archivos integrado

## 🏗️ Estructura

```
test-app/
├── index.html              # Entry point (HTML5)
├── package.json            # Configuración
└── src/
    ├── main.js             # Bootstrap + orquestación global
    ├── ui.js               # Setup de interfaz
    ├── project.js          # Crear/gestionar archivos
    ├── executor.js         # Ejecutar comandos
    └── styles.css          # Estilos del sandbox
```

## 📦 Módulos

### `main.js` (Bootstrap)
```javascript
import init, * as runbox from '../../pkg/runbox.js';
import { setupUI } from './ui.js';
import { createProjectFiles } from './project.js';

// 1. Inicializa RunBox WASM
// 2. Crea la interfaz
// 3. Configura archivos de proyecto
// 4. Expone API global en window.runboxApp
```

### `ui.js` (Interfaz)
```javascript
export function setupUI() {
    // Crea 3 paneles:
    // 1. Editor de archivos (izquierda)
    // 2. Preview (derecha)
    // 3. Terminal (abajo)
    // 4. Panel de control
}
```

### `project.js` (Gestión de archivos)
```javascript
export function createProjectFiles() {
    // Crea archivos de ejemplo:
    // - package.json
    // - index.html
    // - style.css
    // - app.js
}
```

### `executor.js` (Ejecución de comandos)
```javascript
export function executeCommand(command) {
    // Ejecuta comandos como:
    // - ls, cat, echo
    // - npm install, npm run
    // - preview (renderiza la app)
    // - help (muestra comandos)
}
```

## 🎮 Interfaz

```
┌─────────────────────────────────────────────────────────────┐
│  🎁 RunBox Sandbox  |  WebAssembly Virtual Environment     │
│  Status: ✓ Running                                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────┐      ┌──────────────────────┐   │
│  │ 📝 Archivos          │      │ 👁️  Preview          │   │
│  │                      │      │                      │   │
│  │ 📄 index.html        │      │ [App rendered here] │   │
│  │ 🎨 style.css         │      │                      │   │
│  │ ⚙️  app.js            │      │                      │   │
│  │ 📦 package.json      │      │                      │   │
│  │                      │      │                      │   │
│  └──────────────────────┘      └──────────────────────┘   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ 💻 Terminal / Console                                       │
│ $ help                                                      │
│ 📚 Comandos disponibles:                                    │
│   ls - Lista archivos                                       │
│   cat <file> - Muestra contenido                            │
│   npm install - Instala deps                                │
│   npm run dev - Inicia servidor                             │
│   preview - Renderiza app                                   │
│   [command] $ _                                             │
├─────────────────────────────────────────────────────────────┤
│ [▶️ Run Dev] [📦 Install] [🏗️ Build] [📊 Stats] [🔄 Reset] │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 🎯 Flujo de uso

### 1. Abre la app
```
http://localhost:8080/test-app/
```

### 2. RunBox se inicializa
- Carga el módulo WASM
- Crea archivos en VFS
- Muestra la interfaz

### 3. Interactúa
```bash
$ ls                    # Ver archivos
$ cat index.html        # Ver contenido
$ preview               # Renderizar app
$ npm install           # Instalar deps
$ npm run dev           # Iniciar servidor
$ help                  # Ver todos los comandos
```

### 4. Edita archivos
- Click en el archivo en la lista
- Modifica en el editor
- Click "💾 Guardar"

### 5. Ve los cambios
- Click "👁️ Preview"
- O usa `run-dev` para pipeline completo

## 🔧 Comandos disponibles

| Comando | Descripción |
|---------|-------------|
| `ls` | Lista los archivos del proyecto |
| `cat <file>` | Muestra el contenido de un archivo |
| `echo <text>` | Imprime texto |
| `npm install` | Instala dependencias |
| `npm run dev` | Inicia servidor de desarrollo |
| `npm list` | Lista dependencias |
| `npm run build` | Compila el proyecto |
| `preview` | Renderiza la app en el preview |
| `run-dev` | Ejecuta pipeline completo (install + build + preview) |
| `pwd` | Muestra directorio actual |
| `clear` | Limpia la terminal |
| `help` | Muestra todos los comandos |

## 🚀 API Global (window.runboxApp)

Desde DevTools (F12 → Console) puedes acceder:

```javascript
// Módulos
window.runboxApp.runbox              // Objeto RunBox WASM
window.runboxApp.init()               // Inicialización WASM

// Estado
window.runboxApp.runboxReady()         // ¿RunBox está listo?
window.runboxApp.projectFiles          // Archivos en memoria
window.runboxApp.currentFile           // Archivo actual

// Terminal
window.runboxApp.executeCommand('ls')  // Ejecutar comando
window.runboxApp.clearConsole()        // Limpiar terminal

// Archivos
window.runboxApp.loadFile('index.html') // Cargar en editor
window.runboxApp.saveFile()             // Guardar archivo
window.runboxApp.getProjectFiles()      // Ver todos los archivos
window.runboxApp.getFileContent('app.js') // Obtener contenido

// Desarrollo
window.runboxApp.runDev()               // Ejecutar dev
window.runboxApp.installDeps()          // npm install
window.runboxApp.buildProject()         // npm run build
window.runboxApp.showStats()            // Mostrar stats
window.runboxApp.resetProject()         // Resetear proyecto
```

## 📝 Ejemplos de uso

### Ver un archivo
```bash
$ cat app.js
```

### Ejecutar un script
```bash
$ npm run dev
🚀 Starting dev server...
Server running at http://localhost:3000
```

### Renderizar la app
```bash
$ preview
🔄 Renderizando preview...
✅ Preview actualizado
```

### Pipeline completo
```bash
$ run-dev
🚀 Ejecutando pipeline de desarrollo...
  1. Instalando dependencias...
  ✅ Dependencies installed
  2. Compilando proyecto...
  ✅ Build successful
  3. Renderizando preview...
  ✅ Preview actualizado
  4. Server running at http://localhost:3000
✨ Pipeline completado
```

## 🎨 Personalización

### Cambiar archivos de proyecto

Edita `src/project.js`:

```javascript
function createIndexHtml() {
    return `<!DOCTYPE html>
<html>
<head>
    <title>Mi App</title>
</head>
<body>
    <h1>Hola Mundo</h1>
</body>
</html>`;
}
```

### Agregar comandos

En `src/executor.js`, en la función `executeCommand()`:

```javascript
case 'mi-comando':
    handleMiComando();
    break;

function handleMiComando() {
    addTerminalLine('Mi comando ejecutado', 'output');
}
```

### Cambiar estilos

Edita `src/styles.css` - los cambios se aplican inmediatamente.

## 🧪 Debugging

Abre DevTools (F12) y prueba:

```javascript
// Ver estado de RunBox
window.runboxApp.runboxReady()

// Ejecutar comando desde console
window.runboxApp.executeCommand('ls')

// Ver archivos
console.log(window.runboxApp.projectFiles)

// Obtener contenido de archivo
window.runboxApp.getFileContent('app.js')

// Guardar archivo desde console
window.runboxApp.updateFileContent('test.txt', 'contenido')
window.runboxApp.saveFile()
```

## 📚 Documentación

- **main.js** — Punto de entrada y orquestación
- **ui.js** — Setup de interfaz
- **project.js** — Gestión de archivos
- **executor.js** — Ejecución de comandos
- **styles.css** — Estilos del sandbox

---

✨ **RunBox Sandbox v1.0** - WebAssembly IDE en el navegador
