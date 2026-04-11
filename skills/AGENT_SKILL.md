# 🤖 RunBox Sandbox IDE - Skill para Agentes de IA

## Descripción

RunBox Sandbox IDE es un entorno de desarrollo web que corre completamente en el navegador. Como agente, puedes:

- Crear y editar archivos de proyecto (HTML, CSS, JavaScript, JSON)
- Ejecutar comandos en el sandbox (npm, ls, cat, preview, etc.)
- Renderizar código y ver el resultado en tiempo real
- Gestionar un proyecto web completo sin servidor backend

---

## Cómo acceder

```javascript
// El sistema expone una API global en window.runboxApp
const api = window.runboxApp;

// Verifica que esté listo
if (api.runboxReady()) {
  // ¡Listo para usar!
}
```

---

## Operaciones principales

### 1. Obtener información del proyecto

**Listar todos los archivos**

```javascript
const files = api.getProjectFiles();
// Retorna: { 'index.html': '...', 'app.js': '...', ... }
```

**Obtener contenido de un archivo**

```javascript
const content = api.getFileContent('app.js');
// Retorna: string con el contenido del archivo
```

**Verificar qué archivo está abierto**

```javascript
const currentFile = api.currentFile;
// Retorna: nombre del archivo o null
```

---

### 2. Crear/Editar archivos

**Crear o actualizar un archivo**

```javascript
api.updateFileContent('nueva.js', 'console.log("hola");');
// Actualiza en memoria
```

**Cargar archivo en el editor**

```javascript
api.loadFile('app.js');
// Abre el archivo en el editor visual
```

**Guardar archivo actual**

```javascript
api.saveFile();
// Guarda el archivo que está en el editor
```

---

### 3. Ejecutar comandos

**Ejecutar comando**

```javascript
api.executeCommand('npm install');
// Ejecuta y muestra output en terminal
```

**Comandos disponibles**

```
ls                    - Listar archivos con tamaño
cat <archivo>         - Ver contenido de archivo
echo <texto>          - Imprimir texto
npm install           - Instalar dependencias
npm run dev           - Iniciar dev server
npm run build         - Compilar proyecto
npm list              - Listar dependencias
preview               - Renderizar app en preview
run-dev               - Pipeline completo
pwd                   - Ver directorio actual
clear                 - Limpiar terminal
help                  - Ver ayuda
```

---

### 4. Renderizar código

**Renderizar HTML en el preview**

```javascript
const html = `
<html>
  <head>
    <title>Mi App</title>
    <style>
      body { background: #667eea; color: white; }
    </style>
  </head>
  <body>
    <h1>¡Hola Mundo!</h1>
    <script>
      console.log('App cargada');
    </script>
  </body>
</html>
`;

// Opción 1: Usar preview directamente
api.executeCommand('preview');

// Opción 2: Renderizar HTML personalizado
updatePreview(html);  // Función interna, usa la primera opción
```

---

### 5. Flujo típico de trabajo

**Escenario: Crear una app web simple**

```javascript
// 1. Crear/actualizar archivos
api.updateFileContent('index.html', `<!DOCTYPE html>
<html>
<head>
  <title>Mi App</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <h1>Mi Aplicación</h1>
  <script src="app.js"></script>
</body>
</html>`);

api.updateFileContent('style.css', `
body {
  font-family: Arial;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  text-align: center;
  padding: 50px;
}
`);

api.updateFileContent('app.js', `
console.log('App iniciada');
document.body.innerHTML += '<p>¡Funciona!</p>';
`);

// 2. Ver lo que creamos
api.executeCommand('ls');

// 3. Renderizar en el preview
api.executeCommand('preview');

// 4. Ver contenido de un archivo
api.executeCommand('cat index.html');
```

---

## Patrones de uso para agentes

### Patrón 1: Crear proyecto desde cero

```javascript
async function createProject(files) {
  // files = { 'index.html': '...', 'app.js': '...', ... }
  
  for (const [filename, content] of Object.entries(files)) {
    api.updateFileContent(filename, content);
    console.log(`✅ Creado: ${filename}`);
  }
  
  api.executeCommand('run-dev');
}
```

### Patrón 2: Modificar archivo existente

```javascript
function editFile(filename, changes) {
  // changes = función que modifica el contenido
  
  const current = api.getFileContent(filename);
  const modified = changes(current);
  api.updateFileContent(filename, modified);
  
  api.executeCommand('preview');
}

// Ejemplo de uso
editFile('app.js', (content) => {
  return content.replace('console.log', '// DISABLED');
});
```

### Patrón 3: Inspeccionar y reportar estado

```javascript
function getProjectStatus() {
  const files = api.getProjectFiles();
  
  return {
    archivos: Object.keys(files),
    totalArchivos: Object.keys(files).length,
    tamaño: Object.values(files)
      .reduce((sum, content) => sum + content.length, 0),
    estado: api.runboxReady() ? 'Listo' : 'Inicializando'
  };
}

const status = getProjectStatus();
console.log(status);
// {
//   archivos: ['index.html', 'app.js', 'style.css', 'package.json'],
//   totalArchivos: 4,
//   tamaño: 5542,
//   estado: 'Listo'
// }
```

### Patrón 4: Ejecutar secuencia de comandos

```javascript
async function runSequence(commands) {
  for (const cmd of commands) {
    console.log(`Ejecutando: ${cmd}`);
    api.executeCommand(cmd);
    // Pequeña pausa para que se procese
    await new Promise(resolve => setTimeout(resolve, 500));
  }
}

// Ejemplo
runSequence([
  'npm install',
  'npm run build',
  'preview',
  'npm list'
]);
```

### Patrón 5: Generar aplicación dinámicamente

```javascript
function generateApp(config) {
  const {
    title = 'Mi App',
    backgroundColor = '#667eea',
    content = 'Hola'
  } = config;
  
  const html = `<!DOCTYPE html>
<html>
<head>
  <title>${title}</title>
  <style>
    body {
      background: ${backgroundColor};
      color: white;
      font-family: Arial;
      text-align: center;
      padding: 50px;
    }
  </style>
</head>
<body>
  <h1>${title}</h1>
  <p>${content}</p>
</body>
</html>`;
  
  api.updateFileContent('index.html', html);
  api.executeCommand('preview');
}

// Uso
generateApp({
  title: 'Mi Proyecto',
  backgroundColor: '#764ba2',
  content: '¡Bienvenido!'
});
```

---

## Manejo de errores

### Validar que RunBox esté listo

```javascript
if (!api.runboxReady()) {
  console.error('RunBox aún está inicializando');
  // Esperar o reintentar
}
```

### Verificar que archivo existe

```javascript
const content = api.getFileContent('app.js');
if (!content) {
  console.error('Archivo no encontrado');
} else {
  console.log('Archivo encontrado:', content.length, 'caracteres');
}
```

### Manejar comandos inexistentes

```javascript
// El sistema muestra un error en la terminal si el comando no existe
// Puedes verificar que comando es válido antes de ejecutar

const validCommands = [
  'ls', 'cat', 'echo', 'npm', 'preview', 'run-dev', 
  'pwd', 'clear', 'help'
];

if (validCommands.includes(command)) {
  api.executeCommand(command);
} else {
  console.error(`Comando no válido: ${command}`);
}
```

---

## Limitaciones y consideraciones

### Límites

- **Archivo máximo**: Sin límite teórico (limitado por memoria del navegador)
- **Número de archivos**: Ilimitado
- **Ejecución de comandos**: Simulada (no real)
- **Instalación de paquetes**: Simulada (no descarga nada)

### Consideraciones

- Los "comandos npm" son simulaciones (no instalan realmente)
- El preview se renderiza en un iframe aislado
- No hay acceso a sistema de archivos real
- Los archivos se almacenan en memoria (se pierden al recargar)

---

## Debugging y troubleshooting

### Ver estado completo

```javascript
console.log({
  ready: api.runboxReady(),
  files: api.getProjectFiles(),
  currentFile: api.currentFile,
  runbox: api.runbox
});
```

### Verificar terminal

```javascript
// Ver elementos en la terminal
const terminal = document.getElementById('terminal');
console.log(terminal.innerText);
```

### Verificar preview

```javascript
// Ver el HTML renderizado
const iframe = document.getElementById('preview-frame');
console.log(iframe.srcdoc);
```

### Limpiar y reiniciar

```javascript
// Limpiar terminal
api.clearConsole();

// Resetear proyecto
api.resetProject();

// Luego los archivos se crean de nuevo
```

---

## Casos de uso para agentes IA

### Caso 1: Asistente de desarrollo web

El agente puede ayudar a crear sitios web:

```javascript
async function assistWebDeveloper(userRequest) {
  // Ejemplo: "Crea una página con un formulario"
  
  // 1. Generar HTML/CSS/JS basado en la solicitud
  const html = generateHTML(userRequest);
  const css = generateCSS();
  const js = generateJS();
  
  // 2. Crear archivos
  api.updateFileContent('index.html', html);
  api.updateFileContent('style.css', css);
  api.updateFileContent('app.js', js);
  
  // 3. Renderizar
  api.executeCommand('preview');
  
  // 4. Reportar
  return 'Página creada y visible en el preview';
}
```

### Caso 2: Generador de código

El agente genera código basado en especificaciones:

```javascript
function generateComponentFromSpec(spec) {
  // spec = { name: 'Button', props: {...}, events: {...} }
  
  const jsx = generateJSX(spec);
  const css = generateComponentCSS(spec);
  
  api.updateFileContent(`components/${spec.name}.js`, jsx);
  api.updateFileContent(`components/${spec.name}.css`, css);
}
```

### Caso 3: Corrector de código

El agente identifica y arregla errores:

```javascript
function fixCodeIssues() {
  const files = api.getProjectFiles();
  
  for (const [filename, content] of Object.entries(files)) {
    const fixed = performLinting(content);
    if (fixed !== content) {
      api.updateFileContent(filename, fixed);
      console.log(`✅ Arreglado: ${filename}`);
    }
  }
}
```

### Caso 4: Documentador automático

El agente genera documentación del proyecto:

```javascript
function documentProject() {
  const files = api.getProjectFiles();
  let docs = '# Documentación del Proyecto\n\n';
  
  for (const [filename, content] of Object.entries(files)) {
    docs += `## ${filename}\n`;
    docs += '```\n' + content.slice(0, 200) + '...\n```\n\n';
  }
  
  api.updateFileContent('DOCS.md', docs);
}
```

---

## API de referencia rápida

```javascript
// Estado
api.runboxReady()                    // boolean
api.projectFiles                     // { filename: content, ... }
api.currentFile                      // string | null
api.runbox                           // objeto WASM

// Archivos
api.getProjectFiles()                // objeto con todos los archivos
api.getFileContent(filename)         // contenido del archivo
api.updateFileContent(filename, content)  // crear/actualizar
api.loadFile(filename)               // abrir en editor
api.saveFile()                       // guardar archivo actual

// Terminal
api.executeCommand(cmd)              // ejecutar comando
api.clearConsole()                   // limpiar terminal
api.handleCommand(event)             // manejador de teclado

// Desarrollo
api.runDev()                         // ejecutar dev
api.installDeps()                    // npm install
api.buildProject()                   // npm run build
api.showStats()                      // mostrar estadísticas
api.resetProject()                   // resetear proyecto

// Control
api.updateStatus(text, type)         // actualizar status badge
api.init()                           // inicializar RunBox
```

---

## Conclusión

RunBox Sandbox IDE proporciona una API JavaScript completa y accesible para que los agentes de IA puedan:

✅ Crear y editar archivos
✅ Ejecutar comandos
✅ Renderizar código
✅ Gestionar proyectos web

Con estos patrones y funciones, un agente puede actuar como:
- Asistente de desarrollo web
- Generador de código
- Corrector de errores
- Documentador automático
- Y mucho más

---

**Fin de Skill para Agentes de IA**
