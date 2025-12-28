# oxidize-log

**oxidize-log** es un sistema de logging multiplataforma con un core único en Rust y bindings para JavaScript/TypeScript y Java.  
Está diseñado para ofrecer alto rendimiento, seguridad en concurrencia, configuración flexible y una experiencia de desarrollo moderna.

## 🚀 Objetivo

Construir un logger profesional que pueda utilizarse de forma consistente en múltiples lenguajes, manteniendo un único núcleo de lógica:

- Core en Rust: niveles, formateo, sinks, filtros, concurrencia.
- Bindings para JS/TS (Node.js inicialmente).
- Bindings para Java (vía JNI).

## ✨ Características previstas (roadmap resumido)

- Niveles estándar: TRACE, DEBUG, INFO, WARN, ERROR, FATAL.
- Formato texto legible y formato JSON estructurado.
- Metadatos automáticos: timestamp, nivel, archivo, línea, módulo.
- Colores ANSI configurables.
- Sinks múltiples: consola, archivo, rotación, CloudWatch (futuro).
- Concurrencia segura y escritura atómica por línea.
- API consistente en Rust, JS/TS y Java.
- Configuración programática y por archivo (opcional).
- Macros amigables: `info!`, `error!`, etc.

## 📦 Estado actual

Este repositorio contiene la estructura inicial del proyecto y el núcleo mínimo para comenzar a construir el logger.

## 🛠️ Estructura del proyecto

```bash
oxidize-log/
 ├── logger-core/      # Core en Rust (este repo)
 ├── bindings-js/      # Bindings JS/TS (futuro)
 └── bindings-java/    # Bindings Java (futuro)
```

## 🧪 Ejemplo de uso (futuro)

```rust
use oxidize_log::{info, Logger};

fn main() {
    Logger::init_default();
    info!("Hola desde oxidize-log", { "user": "jose" });
}
```

## 📄 Licencia

MIT o Apache-2.0 (por decidir).
