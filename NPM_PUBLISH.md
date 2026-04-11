# Publicar RunBoxJS en NPM

Guía paso a paso para publicar la librería compilada en npmjs.

## Requisitos Previos

1. **Cuenta en npmjs.com** - Crear en https://www.npmjs.com/signup
2. **Node.js y npm instalados** - Node 16+
3. **Credenciales de npm configuradas** - Haber ejecutado `npm login`

## Pasos para Publicar

### 1. Preparar el Proyecto

```bash
cd runbox
```

**Actualizar versión en `pkg/package.json`:**
```bash
# Opción 1: Editar manualmente
# Cambia "version": "0.1.0" a "0.1.1" (o la versión que corresponda)

# Opción 2: Usar npm (si el proyecto es un monorepo)
npm version patch  # 0.1.0 -> 0.1.1
npm version minor  # 0.1.0 -> 0.2.0
npm version major  # 0.1.0 -> 1.0.0
```

### 2. Compilar el Código Rust a WASM

```bash
wasm-pack build --target bundler --release
```

Esto genera en la carpeta `pkg/`:
- `runbox.js` - Binding de JavaScript
- `runbox.d.ts` - TypeScript definitions
- `runbox_bg.wasm` - El binario WebAssembly compilado
- `runbox_bg.wasm.d.ts` - TypeScript definitions para WASM

### 3. Verificar los Archivos

```bash
ls -la pkg/
```

Deberías ver:
- ✅ `runbox.js`
- ✅ `runbox.d.ts`
- ✅ `runbox_bg.wasm`
- ✅ `runbox_bg.wasm.d.ts`
- ✅ `package.json` (actualizado)
- ✅ `.npmignore`

### 4. Probar Localmente (Opcional)

Antes de publicar, puedes probar el paquete localmente:

```bash
cd pkg
npm link
cd ../test-app
npm link runboxjs
```

Esto permite usar el paquete local en el test-app como si estuviera en npmjs.

### 5. Login en NPM

Si aún no has iniciado sesión:

```bash
npm login
```

Te pedirá:
- Username
- Password
- Email

### 6. Publicar el Paquete

**En la carpeta pkg/**:

```bash
cd pkg
npm publish
```

Si todo va bien verás:
```
npm notice Publishing to https://registry.npmjs.org/
npm notice Publishing runboxjs@0.1.0
+ runboxjs@0.1.0
```

### 7. Verificar la Publicación

```bash
# Visita el paquete en npmjs
# https://www.npmjs.com/package/runboxjs

# O desde la terminal
npm info runboxjs
```

## Comandos Útiles

```bash
# Ver información del paquete publicado
npm info runboxjs

# Ver versiones publicadas
npm view runboxjs versions

# Descargar y probar desde npmjs
npm install runboxjs

# Ver contenido del tarball antes de publicar
npm pack pkg/
tar -tzf runboxjs-0.1.0.tgz
```

## Actualizaciones Futuras

Cada vez que quieras publicar una nueva versión:

1. **Actualizar código Rust**
2. **Compilar**: `wasm-pack build --target bundler --release`
3. **Incrementar versión** en `pkg/package.json`
4. **Publicar**: `cd pkg && npm publish`

## Versionado Semántico

- **patch** (0.1.0 → 0.1.1): Bug fixes
- **minor** (0.1.0 → 0.2.0): Nuevas funcionalidades (backwards compatible)
- **major** (0.1.0 → 1.0.0): Cambios breaking

## Hacer el Paquete Privado

Si en el futuro necesitas mantener el paquete privado:

```bash
npm publish --access restricted
```

O en `pkg/package.json`:
```json
{
  "private": true
}
```

## Solucionar Problemas

### "Package name already exists"
El nombre `runboxjs` está disponible. Si no está, usa otro nombre único.

### "npm ERR! 404 Not Found"
Asegúrate de que:
- Estás en la carpeta `pkg/`
- El package.json existe y tiene la estructura correcta
- Nombre del paquete es único

### "You must be logged in"
Ejecuta `npm login` primero

### WASM file too large
Si el `.wasm` es muy grande:
- Reduce el tamaño en Cargo.toml con optimizaciones
- O publica sin el archivo en .npmignore (NO recomendado)

## Ejemplo Completo

```bash
# 1. Clonar/navegar al repo
cd ~/Documents/solo/runbox

# 2. Compilar
wasm-pack build --target bundler --release

# 3. Actualizar versión
# Editar pkg/package.json y cambiar version

# 4. Verificar contenido
ls -la pkg/*.{js,wasm,d.ts}

# 5. Login si es necesario
npm login

# 6. Publicar
cd pkg
npm publish

# 7. Verificar
npm info runboxjs
```

## Después de Publicar

Los usuarios pueden instalarlo así:

```bash
npm install runboxjs
```

Y usarlo en sus proyectos:

```javascript
import init, * as runbox from 'runboxjs';
await init();
// ¡Listo para usar!
```

---

Para más información: https://docs.npmjs.com/packages-and-modules/
