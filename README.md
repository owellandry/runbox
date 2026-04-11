# 🎁 RunBox - WebAssembly Sandbox IDE

IDE completo en el navegador usando RunBox compilado a WebAssembly.

## 🚀 Inicio rápido

```bash
# Terminal 1
cd /c/Users/burge/Documents/solo/runbox
python serve.py

# Terminal 2
cd test-app
bun run dev

# Navegador
http://localhost:5174/
```

## 🎮 Lo que ves

- **Panel izquierdo**: Árbol de archivos (editable)
- **Panel central**: Preview de la app renderizada
- **Panel inferior**: Terminal interactiva
- **Botones**: Run Dev, Install, Build, Reset

## 💻 Comandos

```bash
$ ls                    # Listar archivos
$ cat <file>            # Ver contenido
$ preview               # Renderizar app
$ npm install           # Instalar deps
$ npm run dev           # Dev server
$ npm run build         # Build
$ run-dev               # Pipeline completo
$ help                  # Ver todos
```

## 📁 Estructura

```
runbox/
├── pkg/                 # WASM compilado (bun link runbox)
├── test-app/
│   ├── vite.config.js   # Config de Vite
│   ├── package.json     # Dependencies: runbox
│   └── src/
│       ├── main.js      # Bootstrap
│       ├── ui.js        # Interfaz
│       ├── project.js   # Archivos
│       ├── executor.js  # Comandos
│       └── styles.css   # Estilos
└── serve.py             # Servidor HTTP
```

## 📦 RunBox como paquete

RunBox está registrado con `bun link` y vinculado en test-app:

```javascript
import init, * as runbox from 'runbox';
await init();
```

## 🎯 Cómo funciona

1. Abre el navegador
2. RunBox WASM se inicializa
3. Crea 4 archivos: `package.json`, `index.html`, `style.css`, `app.js`
4. Renderiza en el preview
5. Ejecuta comandos desde terminal
6. Edita archivos y guarda

## ✨ Lo especial

- ✅ WASM sandbox aislado
- ✅ Terminal interactiva
- ✅ Editor integrado
- ✅ Preview en tiempo real
- ✅ Como WebContainer pero con RunBox

## 🔧 Compilación

```bash
# WebAssembly
wasm-pack build --target web --release

# Native
cargo build --release
```

## 📚 Archivos importantes

- `test-app/src/main.js` - Punto de entrada
- `test-app/vite.config.js` - Config Vite (permite acceso a pkg/)
- `pkg/runbox.js` - Bindings JavaScript
- `pkg/runbox_bg.wasm` - WebAssembly binary

---

**Estado**: ✅ Funcional y listo para usar
