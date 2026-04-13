# RunBox — Hoja de Ruta (Roadmap)

> Documento vivo que define la visión, prioridades y plan de implementación para las
> futuras versiones de RunBox. Cada fase incluye objetivos claros, tareas concretas
> y criterios de aceptación.

---

## Tabla de Contenidos

- [Visión General](#visión-general)
- [Fase 1 — v0.4.0: Preview Avanzado y Colaboración](#fase-1--v040-preview-avanzado-y-colaboración)
- [Fase 2 — v0.5.0: Runtimes y Ecosistema](#fase-2--v050-runtimes-y-ecosistema)
- [Fase 3 — v0.6.0: Rendimiento y Escalabilidad](#fase-3--v060-rendimiento-y-escalabilidad)
- [Fase 4 — v0.7.0: Seguridad y Aislamiento](#fase-4--v070-seguridad-y-aislamiento)
- [Fase 5 — v0.8.0: Integración y Extensibilidad](#fase-5--v080-integración-y-extensibilidad)
- [Fase 6 — v0.9.0: Developer Experience (DX)](#fase-6--v090-developer-experience-dx)
- [Fase 7 — v1.0.0: Estabilidad y Producción](#fase-7--v100-estabilidad-y-producción)
- [Ideas Experimentales](#ideas-experimentales)
- [Convenciones de Contribución](#convenciones-de-contribución)
- [Estado Actual](#estado-actual)

---

## Visión General

RunBox es un sandbox de desarrollo basado en WebAssembly que ejecuta workflows de
proyectos directamente en el navegador. Nuestra visión es convertirnos en la
plataforma de referencia para:

1. **Preview instantáneo** — Ver cualquier proyecto web en segundos, sin servidor.
2. **Colaboración en tiempo real** — Compartir previews con dominio propio.
3. **IA nativa** — Agentes AI que manipulan el sandbox como un entorno de desarrollo completo.
4. **Portabilidad total** — Funcionar en cualquier navegador moderno sin dependencias externas.

### Principios de Diseño

- **Puro Rust/WASM** — Sin dependencias de Node.js o binarios nativos en el runtime.
- **Offline-first** — Todo funciona sin conexión a internet después de la carga inicial.
- **API-first** — Cada funcionalidad es accesible vía API (WASM, MCP, AI tools).
- **Seguro por defecto** — Aislamiento completo del sistema host.

---

## Fase 1 — v0.4.0: Preview Avanzado y Colaboración

> **Objetivo**: Llevar el sistema de preview a producción con soporte completo de
> dominio personalizado, colaboración en tiempo real y experiencia de usuario pulida.

### 1.1 Preview en Tiempo Real con WebSocket

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Canal WebSocket bidireccional | Reemplazar el polling de postMessage con WebSocket para comunicación preview ↔ host | Alta |
| Sincronización de estado | Cuando un usuario modifica el VFS, propagar cambios a todos los viewers conectados | Alta |
| Indicador de conexión | Mostrar estado de conexión en la UI del preview (conectado/reconectando/offline) | Media |
| Reconexión automática | Implementar backoff exponencial para reconexión tras desconexión | Media |

### 1.2 Sistema de Sesiones Compartidas

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Tokens de acceso con expiración | Agregar `expires_at` a `share_token` para limitar tiempo de acceso | Alta |
| Permisos por sesión | Definir niveles: `view` (solo lectura), `interact` (puede usar terminal), `edit` (puede modificar archivos) | Alta |
| Panel de sesiones activas | UI para ver quién está conectado al preview compartido | Media |
| Revocación de tokens | Invalidar tokens de compartir desde el panel de control | Media |
| Historial de accesos | Log de quién accedió, cuándo y desde dónde | Baja |

### 1.3 Dominio Personalizado — Implementación Completa

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Guía de configuración DNS | Documentación paso a paso para CNAME, A record y wildcards | Alta |
| Verificación de dominio | Endpoint que verifica si el DNS del usuario apunta correctamente | Alta |
| Certificados SSL automáticos | Integración con Let's Encrypt para HTTPS automático en dominios custom | Media |
| Subdominios por proyecto | Soporte para `proyecto.preview.dominio.com` automático | Media |
| Proxy inverso embebido | Service Worker avanzado que rutea por `Host` header | Baja |

### 1.4 Mejoras de Live Reload

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| HMR real para React/Vue/Svelte | Integrar protocolo HMR nativo de cada framework en lugar de full reload | Alta |
| Inyección CSS sin parpadeo | Aplicar cambios CSS con transición suave (morphing) | Media |
| Preservación de estado | Mantener scroll position, form inputs y estado de componentes durante reload | Media |
| Overlay de errores | Mostrar errores de compilación como overlay en el preview (estilo Vite) | Media |
| Indicador visual de reload | Barra de progreso sutil durante el proceso de reload | Baja |

### 1.5 Metadatos Sociales Avanzados

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Generación automática de screenshots | Capturar screenshot del preview para `og:image` automáticamente | Media |
| Preview cards dinámicas | Generar imágenes OG dinámicas con título, descripción y screenshot | Media |
| Integración con redes sociales | Optimizar meta tags para LinkedIn, Discord, Slack además de Twitter/Facebook | Baja |

**Criterios de aceptación:**
- [ ] Un usuario puede configurar `preview.midominio.com` y compartir la URL
- [ ] Múltiples usuarios pueden ver el mismo preview simultáneamente
- [ ] Los cambios en el VFS se reflejan en < 100ms en todos los viewers
- [ ] Los tokens de compartir expiran y pueden ser revocados

---

## Fase 2 — v0.5.0: Runtimes y Ecosistema

> **Objetivo**: Expandir los runtimes soportados y mejorar la compatibilidad con
> el ecosistema JavaScript/TypeScript moderno.

### 2.1 Runtime de Deno

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| `deno run` básico | Ejecutar scripts TypeScript con permisos por defecto | Alta |
| `deno.json` / `deno.jsonc` | Parsear configuración de Deno y aplicar compiler options | Alta |
| Import maps | Soporte para `imports` en deno.json y import_map.json | Media |
| `deno task` | Ejecutar tasks definidos en deno.json | Media |
| `deno test` | Runner de tests integrado | Baja |
| `deno bench` | Runner de benchmarks | Baja |

### 2.2 Bundler Integrado

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Resolución de módulos ESM | Resolver `import`/`export` siguiendo el algoritmo de Node.js | Alta |
| Tree shaking básico | Eliminar exports no usados para reducir tamaño del bundle | Media |
| Soporte JSX/TSX | Transformar JSX a `React.createElement` o `h()` | Alta |
| CSS Modules | Soporte para `.module.css` con scoping automático | Media |
| Source maps | Generar source maps para debugging en el navegador | Media |
| Code splitting | Dividir el bundle en chunks para carga lazy | Baja |

### 2.3 Package Manager Mejorado

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Cache de paquetes persistente | Almacenar paquetes descargados en IndexedDB para reutilización | Alta |
| Resolución de dependencias real | Resolver árbol de dependencias con semver correcto | Alta |
| Lockfile fiel | Generar lockfiles compatibles con npm/yarn/pnpm reales | Media |
| Workspaces | Soporte para monorepos con workspaces | Media |
| Paquetes privados | Autenticación con registros privados (npm, GitHub Packages) | Baja |

### 2.4 Motor JavaScript Mejorado (boa_engine)

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Async/Await completo | Soporte robusto para promesas y funciones async | Alta |
| `fetch()` polyfill | Implementar fetch sobre la capa de network de RunBox | Alta |
| `setTimeout`/`setInterval` | Event loop básico con timers | Alta |
| Web APIs esenciales | `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`, `crypto.subtle` | Media |
| Node.js built-ins | `path`, `fs` (mapeado a VFS), `process`, `Buffer` | Media |

**Criterios de aceptación:**
- [ ] Un proyecto React + TypeScript compila y se previsualiza sin errores
- [ ] `npm install react react-dom` descarga y cachea los paquetes
- [ ] El bundler genera un bundle funcional con JSX transformado
- [ ] Los source maps funcionan en las DevTools del navegador

---

## Fase 3 — v0.6.0: Rendimiento y Escalabilidad

> **Objetivo**: Optimizar el rendimiento del VFS, el motor JS y la red para
> manejar proyectos grandes (10,000+ archivos).

### 3.1 VFS de Alto Rendimiento

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| VFS basado en B-tree | Reemplazar HashMap anidado con B-tree para búsquedas O(log n) | Alta |
| Lazy loading de archivos | Cargar contenido de archivos bajo demanda en lugar de todo en memoria | Alta |
| Compresión en memoria | Comprimir archivos grandes en el VFS con LZ4 | Media |
| File watchers eficientes | Detección de cambios por hash en lugar de comparación completa | Media |
| Soporte para archivos binarios | Manejar imágenes, fonts y otros binarios sin UTF-8 lossy | Alta |
| Streaming de archivos grandes | Leer/escribir archivos grandes en chunks sin cargar todo en memoria | Media |

### 3.2 Optimización WASM

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Reducción de tamaño del .wasm | Optimizar con `wasm-opt` y eliminar código muerto | Alta |
| Memoria compartida | Usar SharedArrayBuffer para comunicación más rápida con JS | Media |
| Web Workers para tareas pesadas | Mover bundling y compilación a workers separados | Media |
| Caché de compilación WASM | Cachear el módulo WASM compilado en IndexedDB | Alta |
| Profiling integrado | Instrumentar funciones críticas con métricas de rendimiento | Baja |

### 3.3 Red y Caché

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Cache HTTP inteligente | Implementar `Cache-Control`, `ETag` y `If-None-Match` en el Service Worker | Alta |
| Precarga predictiva | Anticipar qué archivos se necesitarán basado en imports | Media |
| CDN para paquetes npm | Servir paquetes desde CDN (esm.sh, skypack) con fallback local | Media |
| Compresión de respuestas | Brotli/gzip para respuestas del Service Worker | Baja |

**Criterios de aceptación:**
- [ ] Un proyecto con 10,000 archivos carga en < 3 segundos
- [ ] El archivo .wasm tiene < 2MB comprimido
- [ ] Las operaciones del VFS (read/write/list) completan en < 1ms para archivos normales
- [ ] El caché reduce las descargas de paquetes npm en un 90%

---

## Fase 4 — v0.7.0: Seguridad y Aislamiento

> **Objetivo**: Garantizar que el sandbox es seguro para ejecutar código no
> confiable y proteger los datos del usuario.

### 4.1 Sandbox Reforzado

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Límites de recursos | CPU time, memoria máxima, número de procesos, tamaño del VFS | Alta |
| Timeout de ejecución | Matar procesos que excedan un tiempo límite configurable | Alta |
| Aislamiento de red | Whitelist de dominios permitidos para fetch() | Alta |
| Content Security Policy | Generar CSP headers dinámicos para el iframe del preview | Media |
| Sanitización de HTML | Limpiar HTML inyectado para prevenir XSS | Media |

### 4.2 Autenticación y Autorización

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| API keys para preview compartido | Autenticar acceso a previews con API keys | Alta |
| OAuth2 para dominios custom | Proteger previews con login (Google, GitHub) | Media |
| Rate limiting | Limitar requests por IP/token para prevenir abuso | Media |
| Audit log | Registrar todas las acciones sensibles (write, exec, share) | Baja |

### 4.3 Privacidad

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Cifrado del VFS | Cifrar archivos en reposo con AES-256-GCM | Media |
| Borrado seguro | Limpiar memoria y storage al cerrar sesión | Media |
| No tracking | Garantizar que RunBox no envía telemetría sin consentimiento | Alta |
| GDPR compliance | Exportación y borrado de datos del usuario | Baja |

**Criterios de aceptación:**
- [ ] Un script malicioso no puede acceder a datos fuera del sandbox
- [ ] Los previews compartidos requieren autenticación si el usuario lo configura
- [ ] Los recursos (CPU, memoria) están limitados y el sandbox se recupera de abusos
- [ ] No se envía ningún dato a servidores externos sin consentimiento

---

## Fase 5 — v0.8.0: Integración y Extensibilidad

> **Objetivo**: Hacer de RunBox una plataforma extensible que se integre con
> el ecosistema de desarrollo existente.

### 5.1 Sistema de Plugins

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| API de plugins | Definir interfaz para plugins de RunBox (lifecycle hooks, API access) | Alta |
| Plugin de ESLint | Linting en tiempo real dentro del sandbox | Media |
| Plugin de Prettier | Formateo automático al guardar | Media |
| Plugin de TypeScript | Type checking en tiempo real con diagnósticos | Alta |
| Plugin de Tailwind CSS | Compilación de Tailwind en el sandbox | Media |
| Marketplace de plugins | Registro de plugins disponibles para instalar | Baja |

### 5.2 Integraciones Externas

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| GitHub/GitLab sync | Sincronizar VFS con un repositorio Git remoto | Alta |
| Deploy a Vercel/Netlify | One-click deploy desde el preview | Media |
| Integración con Figma | Importar componentes de Figma como código | Baja |
| VS Code extension | Extensión para abrir proyectos de VS Code en RunBox | Media |
| CLI de RunBox | Herramienta de línea de comandos para gestionar instancias | Alta |

### 5.3 MCP Server Avanzado

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Streaming de respuestas | Soporte para SSE (Server-Sent Events) en MCP | Media |
| Subscripción a recursos | Notificar a clientes MCP cuando cambian archivos | Alta |
| Prompts dinámicos | Generar prompts basados en el contexto del proyecto actual | Media |
| Multi-tenant | Un servidor MCP manejando múltiples instancias de RunBox | Baja |
| Protocolo MCP v2 | Adoptar la siguiente versión del protocolo MCP cuando se estabilice | Baja |

### 5.4 AI Tools Avanzados

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| `debug_error` | Tool que analiza un error, busca contexto en el código y propone fix | Alta |
| `refactor_code` | Tool para refactoring seguro con preservación de semántica | Media |
| `generate_tests` | Generar tests unitarios para un archivo o función | Media |
| `explain_project` | Análisis completo del proyecto con arquitectura y dependencias | Media |
| `deploy_preview` | Tool para desplegar el preview a un servicio externo | Baja |
| Contexto de proyecto | Alimentar al AI con información del package.json, tsconfig, etc. | Alta |

**Criterios de aceptación:**
- [ ] Un plugin puede registrar hooks para file change, before exec, after exec
- [ ] El VFS puede sincronizarse con un repo de GitHub
- [ ] Los clientes MCP reciben notificaciones en tiempo real de cambios en archivos
- [ ] Un agente AI puede usar los tools para debuggear un error end-to-end

---

## Fase 6 — v0.9.0: Developer Experience (DX)

> **Objetivo**: Hacer que la experiencia de desarrollo con RunBox sea lo más
> fluida y agradable posible.

### 6.1 Editor Integrado

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Monaco Editor embebido | Integrar Monaco (el editor de VS Code) para edición de código | Alta |
| Autocompletado TypeScript | IntelliSense con tipos del proyecto y dependencias | Alta |
| Multi-tab | Abrir múltiples archivos en tabs | Media |
| Minimap | Vista miniatura del archivo actual | Baja |
| Búsqueda global | Buscar texto en todos los archivos del proyecto (Ctrl+Shift+F) | Media |
| Git diff inline | Mostrar cambios git inline en el editor | Media |

### 6.2 Terminal Mejorada

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Autocompletado de comandos | Sugerir comandos y paths al escribir | Alta |
| Historial de comandos | Navegación con flechas arriba/abajo por comandos previos | Alta |
| Múltiples terminales | Abrir varias terminales en tabs o split | Media |
| Copy/paste mejorado | Soporte completo de clipboard en xterm.js | Media |
| Temas de terminal | Personalizar colores y fuentes del terminal | Baja |

### 6.3 DevTools Integradas

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Network tab | Visualizar todas las requests HTTP del preview | Alta |
| Console mejorada | Console con filtros, agrupación y objetos expandibles | Media |
| Elements inspector mejorado | Árbol DOM completo con edición inline de estilos | Media |
| Performance profiler | Timeline de rendimiento del preview | Baja |
| Storage inspector | Visualizar localStorage, sessionStorage, cookies | Baja |

### 6.4 UI/UX General

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Layout responsivo | Panel editor + preview + terminal con resize | Alta |
| Tema oscuro/claro | Soporte para ambos temas con toggle | Media |
| Atajos de teclado | Mapeo completo de shortcuts (Ctrl+S, Ctrl+P, etc.) | Media |
| Onboarding interactivo | Tutorial paso a paso para nuevos usuarios | Baja |
| Internacionalización (i18n) | Soporte para múltiples idiomas (español, inglés, portugués) | Baja |
| Modo offline completo | PWA con service worker para funcionar sin conexión | Media |

**Criterios de aceptación:**
- [ ] Un desarrollador puede editar, previsualizar y depurar sin salir de RunBox
- [ ] El autocompletado de TypeScript funciona con < 200ms de latencia
- [ ] La UI es responsiva y funciona en móviles
- [ ] Los atajos de teclado son consistentes con VS Code

---

## Fase 7 — v1.0.0: Estabilidad y Producción

> **Objetivo**: Llevar RunBox a v1.0 con estabilidad de producción, documentación
> completa y garantías de compatibilidad.

### 7.1 Estabilidad

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| API estable | Congelar la API pública de RunboxInstance (sin breaking changes) | Alta |
| Semver estricto | Adoptar versionado semántico estricto | Alta |
| Deprecation policy | Definir política de deprecación con warnings por 2 minor versions | Media |
| Changelog automático | Generar changelog desde conventional commits | Media |

### 7.2 Testing y CI/CD

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| CI con GitHub Actions | Pipeline de build, test, lint para cada PR | Alta |
| Tests de integración WASM | Tests que corren en el navegador con `wasm-pack test` | Alta |
| Benchmarks automatizados | Detectar regresiones de rendimiento en CI | Media |
| Tests de compatibilidad | Verificar en Chrome, Firefox, Safari, Edge | Alta |
| Coverage > 80% | Alcanzar 80% de cobertura de código | Media |
| Fuzzing | Tests de fuzzing para el parser de shell y el VFS | Baja |

### 7.3 Documentación

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Sitio de docs (mdBook) | Documentación completa en un sitio estático | Alta |
| API reference auto-generada | Generar docs desde `///` comments con `cargo doc` | Alta |
| Guías paso a paso | Tutoriales para cada caso de uso principal | Media |
| Ejemplos interactivos | Demos en vivo embebidos en la documentación | Media |
| Video tutoriales | Series de videos explicando conceptos clave | Baja |

### 7.4 Distribución

| Tarea | Descripción | Prioridad |
|-------|-------------|-----------|
| Publicación en crates.io | Publicar el crate para uso como dependencia Rust | Alta |
| Publicación en npm | Publicar el paquete WASM para uso desde JavaScript | Alta |
| CDN público | Servir RunBox desde un CDN para uso directo en `<script>` | Media |
| Docker image | Imagen Docker con el MCP server listo para usar | Media |
| Homebrew formula | Instalación vía `brew install runbox` | Baja |

**Criterios de aceptación:**
- [ ] La API pública no tiene breaking changes entre v1.0 y v1.x
- [ ] CI pasa en los 4 navegadores principales
- [ ] La documentación cubre el 100% de la API pública
- [ ] El paquete está disponible en crates.io y npm

---

## Ideas Experimentales

> Ideas que podrían ser transformadoras pero requieren investigación previa.
> No tienen timeline definido — se priorizarán según feedback de la comunidad.

### Colaboración en Tiempo Real (CRDT)

Implementar un sistema de edición colaborativa usando CRDTs (Conflict-free
Replicated Data Types) para que múltiples usuarios puedan editar el mismo
proyecto simultáneamente sin conflictos.

- **Tecnología**: Automerge o Yjs compilado a WASM
- **Impacto**: Convertiría RunBox en un "Google Docs para código"
- **Complejidad**: Alta — requiere reescribir el VFS sobre CRDTs

### Ejecución de Contenedores WASI

Explorar la posibilidad de ejecutar contenedores ligeros usando WASI
(WebAssembly System Interface) para soportar lenguajes compilados
como Go, Rust o C dentro del sandbox.

- **Tecnología**: wasmtime/wasmer compilado a WASM32
- **Impacto**: Soporte para cualquier lenguaje que compile a WASI
- **Complejidad**: Muy alta — WASI dentro de WASM32 es experimental

### AI Code Generation Integrada

Integrar un modelo de lenguaje pequeño (como phi-3 o gemma) directamente
en el sandbox para generar código sin conexión a internet.

- **Tecnología**: ONNX Runtime en WASM + modelo cuantizado
- **Impacto**: Desarrollo asistido por IA completamente offline
- **Complejidad**: Alta — modelos grandes no caben en WASM memory

### RunBox Desktop (Tauri)

Crear una aplicación de escritorio con Tauri que empaquete RunBox con
acceso al filesystem real, terminal nativa y mejor rendimiento.

- **Tecnología**: Tauri + RunBox WASM
- **Impacto**: Alternativa ligera a VS Code para desarrollo rápido
- **Complejidad**: Media — Tauri simplifica el empaquetado

### RunBox Mobile

Versión móvil de RunBox que funcione como PWA completa en tablets y
teléfonos, permitiendo desarrollo desde cualquier dispositivo.

- **Tecnología**: PWA + Service Worker + teclado virtual mejorado
- **Impacto**: Desarrollo desde el teléfono
- **Complejidad**: Media — la UI necesita adaptación significativa

### Marketplace de Templates

Sistema de templates preconfigurados que los usuarios pueden instanciar
con un clic: React + Tailwind, Express API, Full-stack Next.js, etc.

- **Tecnología**: Templates almacenados como VFS snapshots comprimidos
- **Impacto**: Onboarding instantáneo para nuevos proyectos
- **Complejidad**: Baja — infraestructura sencilla

### Debugging con Time Travel

Implementar un debugger que permita "retroceder en el tiempo" para
ver el estado de variables y el DOM en cualquier punto de la ejecución.

- **Tecnología**: Snapshots del estado WASM + replay
- **Impacto**: Debugging revolucionario para aplicaciones web
- **Complejidad**: Muy alta — requiere instrumentación profunda

---

## Convenciones de Contribución

### Formato de Commits

Usamos [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: agregar soporte para Deno runtime
fix: corregir leak de memoria en el VFS al borrar directorios
docs: actualizar guía de dominio personalizado
perf: optimizar resolución de módulos ESM
refactor: extraer lógica de CORS a módulo separado
test: agregar tests de integración para preview compartido
chore: actualizar dependencias de wasm-bindgen
```

### Ramas

```
master           ← rama principal estable
devin/{ts}-{slug} ← ramas de feature/fix
release/v0.x.0   ← ramas de release
```

### Prioridades

- **Alta**: Bloquea el release o es crítico para la experiencia del usuario.
- **Media**: Mejora significativa pero no bloquea el release.
- **Baja**: Nice-to-have, se implementa cuando hay tiempo disponible.

### Cómo Contribuir a una Fase

1. Elegir una tarea de la fase actual (la más antigua no completada).
2. Crear un issue en GitHub con el formato: `[Fase X.Y] Título de la tarea`.
3. Crear una rama con el formato: `devin/{timestamp}-{slug}`.
4. Implementar con tests y documentación.
5. Crear PR apuntando a `master`.
6. Solicitar review.

---

## Estado Actual

### v0.3.9 (actual)

**Completado:**
- VFS virtual con tracking de cambios
- Shell con 7 runtimes (bun, npm, pnpm, yarn, git, python, shell)
- Motor JavaScript (boa_engine) con type stripping de TypeScript
- Consola de logs con niveles y timestamps
- Process manager con PID tracking
- Hot reload inteligente (CSS inject, HMR, full reload)
- Inspector DOM (activar, seleccionar, overlay, historial)
- Terminal xterm.js con input/output bidireccional
- Service Worker para intercepción de red
- HTTP server simulado via `globalThis.__runbox_servers`
- MCP Server con 13 tools, 3 prompts y resources dinámicos
- AI Tools (13 tools) con soporte OpenAI, Anthropic y Gemini
- **Sistema de Preview completo** (dominio custom, CORS, live-reload, OG metadata, compartir URLs)
- 60 tests unitarios y de integración
- Documentación completa (Architecture, API Reference, Development, MCP Guide)
- AGENT_SKILL.md exhaustivo para agentes AI

**Próximo:** Fase 1 — Preview Avanzado y Colaboración (v0.4.0)

---

> *Este roadmap se actualiza con cada release. Las fechas son estimaciones
> y pueden cambiar según feedback de la comunidad y prioridades del proyecto.*
>
> **Mantenido por:** Equipo RunBox
> **Última actualización:** Abril 2026
