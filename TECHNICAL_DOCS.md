# 📖 RunBox Sandbox IDE - Documentación Técnica Detallada

## 📋 Tabla de contenidos

1. [Introducción](#introducción)
2. [Arquitectura](#arquitectura)
3. [API Global](#api-global)
4. [Módulos](#módulos)
5. [Funciones](#funciones)
6. [Lógica de ejecución](#lógica-de-ejecución)
7. [Estructura de datos](#estructura-de-datos)
8. [Ejemplos de uso](#ejemplos-de-uso)
9. [Debugging](#debugging)

---

## Introducción

RunBox Sandbox IDE es un entorno de desarrollo completo que corre en el navegador usando WebAssembly. Permite:

- ✅ Crear y editar archivos de proyecto
- ✅ Ejecutar comandos (npm, ls, cat, etc.)
- ✅ Renderizar código en tiempo real
- ✅ Terminal interactiva integrada
- ✅ Preview de aplicaciones web

### Características técnicas

- **Runtime**: WebAssembly (WASM) - 435 KB compilado
- **Framework**: Vanilla JavaScript (sin dependencias)
- **Build tool**: Vite 5.4.21
- **Package manager**: Bun (recomendado) o NPM
- **Navegadores soportados**: Todos los modernos

---

## Arquitectura

### 🏗️ Capas

```
┌─────────────────────────────────────┐
│  Navegador (Chrome, Firefox, etc)   │
├─────────────────────────────────────┤
│     UI (main.js, ui.js)             │  JavaScript puro
├─────────────────────────────────────┤
│  Lógica (project.js, executor.js)   │  Simulación de comandos
├─────────────────────────────────────┤
│  RunBox WASM (runbox.js + .wasm)    │  Sandbox aislado
├─────────────────────────────────────┤
│  VFS Virtual (memoria)              │  Sistema de archivos simulado
└─────────────────────────────────────┘
```

### 📦 Módulos

#### `main.js` (Punto de entrada)
```javascript
// 1. Importa RunBox WASM
import init, * as runbox from 'runbox';

// 2. Inicializa
await init();

// 3. Crea UI
setupUI();

// 4. Crea archivos
createProjectFiles();

// 5. Expone API global
window.runboxApp = { ... }
```

**Responsabilidades:**
- Bootstrapping de la aplicación
- Orquestación de módulos
- Exposición de API global
- Manejo de errores de inicialización

---

#### `ui.js` (Interface visual)
```javascript
export function setupUI() {
    // Crea la estructura HTML de 3 paneles
    // + terminal
    // + botones de control
}
```

**Estructura generada:**

```html
<div class="sandbox-container">
  <header class="sandbox-header">
    <!-- Estado y título -->
  </header>
  
  <div class="sandbox-grid">
    <!-- Panel 1: Editor de archivos -->
    <div class="panel editor-panel">
      <!-- Árbol de archivos -->
      <!-- Editor de texto -->
    </div>
    
    <!-- Panel 2: Preview -->
    <div class="panel preview-panel">
      <!-- iframe para renderizar -->
    </div>
    
    <!-- Panel 3: Terminal -->
    <div class="panel terminal-panel">
      <!-- Líneas de salida -->
      <!-- Input de comandos -->
    </div>
  </div>
  
  <div class="control-panel">
    <!-- Botones de control -->
  </div>
</div>
```

**Funciones exportadas:**

```javascript
export function setupUI()              // Crea la interfaz
export function addTerminalLine(text, type)  // Agrega línea a terminal
export function clearTerminal()        // Limpia terminal
export function updatePreview(html)    // Renderiza en preview
```

---

#### `project.js` (Gestión de archivos)
```javascript
export function createProjectFiles() {
    // Crea 4 archivos de ejemplo
}
```

**Archivos creados:**

1. **package.json**
```json
{
  "name": "runbox-sandbox-app",
  "type": "module",
  "scripts": {
    "dev": "python -m http.server 3000",
    "build": "echo 'Building...'",
    "test": "echo 'Running tests...'"
  }
}
```

2. **index.html**
```html
<!DOCTYPE html>
<html>
  <head>
    <title>RunBox App</title>
    <link rel="stylesheet" href="style.css">
  </head>
  <body>
    <!-- Contenido -->
    <script src="app.js"></script>
  </body>
</html>
```

3. **style.css**
```css
/* Estilos con gradientes y responsive */
```

4. **app.js**
```javascript
console.log('✅ App.js loaded');
// Lógica de la aplicación
```

**Funciones exportadas:**

```javascript
export function createProjectFiles()              // Crear archivos
export function getProjectFiles()                 // Obtener todos los archivos
export function getFileContent(filename)          // Obtener contenido de archivo
export function updateFileContent(filename, content)  // Actualizar archivo
```

---

#### `executor.js` (Ejecución de comandos)
```javascript
export function executeCommand(command) {
    // Parsea y ejecuta comando
    // Actualiza terminal
    // Puede renderizar preview
}
```

**Comandos implementados:**

| Comando | Función | Salida |
|---------|---------|--------|
| `ls` | Lista archivos | Nombres y tamaños |
| `cat <file>` | Muestra contenido | Contenido del archivo |
| `echo <text>` | Imprime texto | El texto |
| `npm install` | Simula instalación | Mensaje de éxito |
| `npm run dev` | Inicia servidor | URL del servidor |
| `npm run build` | Compila proyecto | Mensaje de build |
| `npm list` | Lista dependencias | Lista de deps |
| `preview` | Renderiza app | Actualiza iframe |
| `run-dev` | Pipeline completo | Todos los pasos |
| `pwd` | Directorio actual | `/sandbox` |
| `clear` | Limpia terminal | Borra historial |
| `help` | Muestra ayuda | Lista de comandos |

**Lógica de ejecución:**

```
executeCommand(string)
  ↓
Parse comando y argumentos
  ↓
Switch por tipo de comando
  ↓
handleXxx() función específica
  ↓
addTerminalLine() para output
  ↓
Actualizar estado si es necesario
```

---

## API Global

Toda la API se expone en `window.runboxApp`:

### Estado

```javascript
// ¿RunBox está listo?
window.runboxApp.runboxReady()  // boolean

// Archivos del proyecto en memoria
window.runboxApp.projectFiles   // object

// Archivo actualmente abierto
window.runboxApp.currentFile    // string | null

// Módulo RunBox WASM
window.runboxApp.runbox         // object
```

### Terminal

```javascript
// Ejecutar comando
window.runboxApp.executeCommand(cmd: string)

// Ejecutar con input directo
window.runboxApp.executeCommand()  // Lee de #terminal-input

// Limpiar terminal
window.runboxApp.clearConsole()
```

### Archivos

```javascript
// Cargar archivo en editor
window.runboxApp.loadFile(filename: string)

// Guardar archivo actual
window.runboxApp.saveFile()

// Obtener todos los archivos
window.runboxApp.getProjectFiles()  // object

// Obtener contenido de archivo
window.runboxApp.getFileContent(filename: string)  // string

// Actualizar archivo
window.runboxApp.updateFileContent(filename: string, content: string)
```

### Desarrollo

```javascript
// Ejecutar dev server
window.runboxApp.runDev()

// Instalar dependencias
window.runboxApp.installDeps()

// Build del proyecto
window.runboxApp.buildProject()

// Mostrar estadísticas
window.runboxApp.showStats()

// Resetear proyecto
window.runboxApp.resetProject()
```

### Control

```javascript
// Manejar tecla Enter en terminal
window.runboxApp.handleCommand(event: KeyboardEvent)

// Actualizar badge de estado
window.runboxApp.updateStatus(text: string, type?: string)

// Inicializar RunBox WASM
window.runboxApp.init()
```

---

## Módulos

### Importación

```javascript
// En main.js
import init, * as runbox from 'runbox';
```

Las funciones iniciales de RunBox WASM:

```javascript
// Inicializar el módulo
await init();

// Después puedes usar todas las funciones de runbox
// Actualmente no expone funciones públicas (WIP)
```

---

## Funciones

### executeCommand(cmd)

**Parámetros:**
- `cmd` (string): Comando a ejecutar (ej: "ls", "cat app.js")

**Comportamiento:**

1. Valida que no esté vacío
2. Agrega línea a terminal: `$ {comando}`
3. Parsea comando y argumentos
4. Ejecuta handler específico
5. Agrega output a terminal

**Ejemplo:**

```javascript
window.runboxApp.executeCommand('ls');
// Salida:
// $ ls
// index.html        342 bytes
// style.css         3000 bytes
// app.js            1200 bytes
// package.json      400 bytes
```

### setupUI()

**Parámetros:** Ninguno

**Retorna:** void

**Comportamiento:**

1. Obtiene elemento #app
2. Inyecta HTML de 3 paneles
3. Registra event listeners
4. Enfoca en terminal input

**Lado effects:**
- Modifica DOM
- Agrega event listeners

---

### createProjectFiles()

**Parámetros:** Ninguno

**Retorna:** void

**Comportamiento:**

1. Define 4 archivos de ejemplo
2. Los almacena en `window.runboxApp.projectFiles`
3. Agrega logs a terminal

**Archivos creados:**
- package.json
- index.html
- style.css
- app.js

---

### addTerminalLine(text, type)

**Parámetros:**
- `text` (string): Texto a mostrar
- `type` (string, opcional): Tipo ['output', 'info', 'success', 'error', 'command']

**Retorna:** void

**Comportamiento:**

1. Crea elemento div
2. Añade clase según tipo
3. Agrega al terminal
4. Hace scroll automático

**Ejemplo:**

```javascript
addTerminalLine('✅ Archivo guardado', 'success');
addTerminalLine('Error: archivo no encontrado', 'error');
```

---

### updatePreview(html)

**Parámetros:**
- `html` (string): HTML completo a renderizar

**Retorna:** void

**Comportamiento:**

1. Obtiene iframe
2. Usa `srcdoc` para inyectar HTML
3. El contenido se ejecuta en sandbox

**Ejemplo:**

```javascript
const html = `
<html>
  <body>
    <h1>Hola</h1>
  </body>
</html>
`;
updatePreview(html);
```

---

### loadFile(filename)

**Parámetros:**
- `filename` (string): Nombre del archivo

**Retorna:** void

**Comportamiento:**

1. Obtiene contenido del archivo
2. Muestra editor si existe
3. Carga contenido en textarea
4. Guarda referencia en `currentFile`

**Ejemplo:**

```javascript
window.runboxApp.loadFile('app.js');
// Muestra el contenido en el editor
```

---

### saveFile()

**Parámetros:** Ninguno

**Retorna:** void

**Comportamiento:**

1. Obtiene archivo actual
2. Obtiene contenido de textarea
3. Actualiza en `projectFiles`
4. Agrega mensaje de éxito a terminal

**Ejemplo:**

```javascript
// Usuario edita archivo
// Hace click en "Guardar"
window.runboxApp.saveFile();
// ✅ app.js guardado
```

---

## Lógica de ejecución

### Flujo de inicialización

```
1. HTML carga
2. <script type="module" src="/src/main.js"></script>
3. main.js importa módulos
4. main() función async
   a. await init() - Inicializa RunBox WASM
   b. setupUI() - Crea interfaz
   c. createProjectFiles() - Crea archivos
   d. updateStatus('RunBox Ready', 'success')
   e. addTerminalLine(...)
5. window.runboxApp expuesto globalmente
6. Usuario interactúa con UI
```

### Flujo de ejecución de comando

```
Usuario escribe "ls" en terminal
         ↓
Presiona Enter
         ↓
handleCommand(event) en main.js
         ↓
window.runboxApp.executeCommand()
         ↓
executeCommand(cmd) en executor.js
         ↓
Valida comando no vacío
         ↓
addTerminalLine(`$ ${cmd}`)
         ↓
Parse: const [cmd, ...args] = cmd.split(' ')
         ↓
Switch por tipo:
  ├─ 'ls' → handleLs()
  ├─ 'cat' → handleCat(args[0])
  ├─ 'npm' → handleNpm(args)
  └─ etc.
         ↓
Ejecuta handler específico
         ↓
addTerminalLine(resultado)
         ↓
Terminal actualizado
```

### Flujo de preview

```
Usuario hace click "Preview"
         ↓
executeCommand('preview')
         ↓
handlePreview()
         ↓
Obtiene archivos del proyecto
  - index.html
  - style.css
  - app.js
         ↓
Inyecta CSS:
  index.html = index.html.replace('</head>', `<style>...</style></head>`)
         ↓
Inyecta JS:
  index.html = html.replace('</body>', `<script>...</script></body>`)
         ↓
updatePreview(html)
         ↓
iframe.srcdoc = html
         ↓
Navegador renderiza en iframe
         ↓
App visible en preview
```

---

## Estructura de datos

### window.runboxApp.projectFiles

```javascript
{
  'package.json': '{...}',  // string JSON
  'index.html': '<!DOCTYPE...>',  // string HTML
  'style.css': 'body { ... }',  // string CSS
  'app.js': 'console.log(...)'  // string JavaScript
}
```

### Formato de comando en terminal

```
Entrada: "ls"
Salida:  $ ls
         index.html        342 bytes
         style.css         3000 bytes
         app.js            1200 bytes
         package.json      400 bytes
```

### Estado de archivo en editor

```javascript
window.runboxApp.currentFile = 'app.js';  // Archivo abierto actualmente
document.getElementById('editor-content').value  // Contenido actual
```

---

## Ejemplos de uso

### Ejemplo 1: Ejecutar un comando

```javascript
// Ejecutar ls
window.runboxApp.executeCommand('ls');

// Output en terminal:
// $ ls
// index.html        342 bytes
// style.css         3000 bytes
// app.js            1200 bytes
// package.json      400 bytes
```

### Ejemplo 2: Ver contenido de archivo

```javascript
// Ver app.js
window.runboxApp.executeCommand('cat app.js');

// Output:
// $ cat app.js
// --- app.js ---
// console.log('✅ App.js cargado en RunBox');
// ...
// --- end app.js ---
```

### Ejemplo 3: Editar y guardar archivo

```javascript
// 1. Cargar en editor
window.runboxApp.loadFile('app.js');

// 2. Editar en el textarea #editor-content
document.getElementById('editor-content').value = 'console.log("Hola");';

// 3. Guardar
window.runboxApp.saveFile();
// ✅ app.js guardado

// 4. Verificar
window.runboxApp.executeCommand('cat app.js');
// Muestra el contenido actualizado
```

### Ejemplo 4: Pipeline completo

```javascript
// Ejecutar todo el pipeline
window.runboxApp.executeCommand('run-dev');

// Output:
// 🚀 Ejecutando pipeline de desarrollo...
//   1. Instalando dependencias...
//   ✅ Dependencies installed
//   2. Compilando proyecto...
//   ✅ Build successful
//   3. Renderizando preview...
//   ✅ Preview actualizado
//   Server running at http://localhost:3000
// ✨ Pipeline completado
```

### Ejemplo 5: Renderizar código personalizado

```javascript
// Obtener archivo
const html = window.runboxApp.getFileContent('index.html');
const css = window.runboxApp.getFileContent('style.css');
const js = window.runboxApp.getFileContent('app.js');

// Combinar
const complete = html
  .replace('</head>', `<style>${css}</style></head>`)
  .replace('</body>', `<script>${js}</script></body>`);

// Renderizar
window.runboxApp.updatePreview(complete);
```

---

## Debugging

### Ver estado actual

```javascript
// ¿RunBox está listo?
console.log(window.runboxApp.runboxReady());

// Ver todos los archivos
console.log(window.runboxApp.getProjectFiles());

// Ver archivo específico
console.log(window.runboxApp.getFileContent('app.js'));

// Ver archivo actual
console.log(window.runboxApp.currentFile);
```

### Ejecutar comando desde DevTools

```javascript
// Ejecutar comando directamente
window.runboxApp.executeCommand('ls');

// Editar archivo
window.runboxApp.updateFileContent('app.js', 'console.log("test");');

// Renderizar
window.runboxApp.executeCommand('preview');
```

### Ver logs de ejecución

1. Abre DevTools (F12)
2. Ve a Console
3. Verás logs de:
   - Inicialización de RunBox
   - Creación de archivos
   - Ejecución de comandos

### Errores comunes

**Error: "RunBox is not ready"**
- Solución: Espera a que termine la inicialización

**Error: "Cannot find module 'runbox'"**
- Solución: Ejecuta `bun link runbox` en test-app

**Preview no se renderiza**
- Solución: Ejecuta `window.runboxApp.executeCommand('preview')`

**Archivo no se guarda**
- Solución: Verifica que `window.runboxApp.currentFile` esté seteado

---

## Resumen de API

```javascript
// Estado
runboxApp.runboxReady()
runboxApp.projectFiles
runboxApp.currentFile

// Terminal
runboxApp.executeCommand(cmd)
runboxApp.clearConsole()
runboxApp.handleCommand(event)

// Archivos
runboxApp.loadFile(filename)
runboxApp.saveFile()
runboxApp.getProjectFiles()
runboxApp.getFileContent(filename)
runboxApp.updateFileContent(filename, content)

// Desarrollo
runboxApp.runDev()
runboxApp.installDeps()
runboxApp.buildProject()
runboxApp.showStats()
runboxApp.resetProject()

// Control
runboxApp.updateStatus(text, type)
runboxApp.init()
```

---

**Fin de documentación técnica**
