# Visión y Roadmap de oxidize-log

Este documento detalla la dirección futura, el roadmap y los objetivos arquitectónicos de **oxidize-log**. Para ver el estado actual del repositorio e instrucciones de uso inmediato, consultá el [README.md](file:///home/zitrojj/dev/pro/oxidize-log/README.md).

---

## 🎯 Objetivo General

Construir un logger profesional de alto rendimiento que pueda utilizarse de forma consistente en múltiples lenguajes, manteniendo un único núcleo de lógica compartida:

- **Core en Rust**: Control de niveles, formateo, sinks, filtros y concurrencia.
- **Bindings para JS/TS** (iniciando con soporte para Node.js).
- **Bindings para Java** (vía JNI).

### Arquitectura de Bindings Propuesta

```bash
oxidize-log/ (Monorepo futuro)
 ├── logger-core/      # Core en Rust (actualmente en la raíz)
 ├── bindings-js/      # Bindings para Node.js (JS/TS)
 └── bindings-java/    # Bindings para Java/Kotlin
```

---

## ✨ Características y Funcionalidades Planificadas

### Core de Logging
- **Niveles estándar**: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`.
- **Formateadores**: Texto estructurado legible para humanos y formato JSON optimizado para producción.
- **Metadatos automáticos**: Captura de timestamp, nivel de log, archivo de origen, línea de código y módulo.
- **Colores ANSI**: Soporte configurable para consolas y terminales.
- **Concurrencia**: Garantía de seguridad en entornos concurrentes y escritura atómica garantizada por línea.
- **Macros ergonómicas**: Implementación de macros del estilo `info!`, `warn!`, `error!`, etc.

### Sinks (Destinos de Logs)
- Consola (estándar/error).
- Archivo local simple.
- Rotación de archivos por tamaño y tiempo.
- Sinks para servicios en la nube (ej. AWS CloudWatch, Datadog).

---

## 🧪 Propuesta de API Futura

### En Rust
```rust
use oxidize_log::{info, Logger};

fn main() {
    Logger::init_default();
    info!("Hola desde oxidize-log", { "user": "jose" });
}
```

### En JavaScript / TypeScript
```typescript
import { Logger } from 'oxidize-log';

Logger.initDefault();
Logger.info('Hola desde JS', { user: 'jose' });
```
