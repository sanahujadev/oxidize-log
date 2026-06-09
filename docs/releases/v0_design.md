# V0 de Diseño y Requisitos de oxidize-log

Este documento define las especificaciones, requisitos funcionales (F) y no funcionales (NF) clasificados para el desarrollo de **oxidize-log**.

## Clasificación

### Prioridad
- **P0**: Esencial para la primera versión utilizable.
- **P1**: Importante, para la siguiente iteración.
- **P2**: Avanzado / Opcional (*Nice to have*).

---

## 1. Visión General del Proyecto

- **Nombre tentativo**: Logger core en Rust con bindings para JS/TS y Java.
- **Objetivo**: Diseñar y construir un sistema de logging multiplataforma, de alto rendimiento, con un core único en Rust, capaz de funcionar como librería, exponerse a JavaScript/TypeScript y Java, y soportar niveles, formatos, sinks múltiples, y características avanzadas como rotación, CloudWatch, colores y metadatos.

---

## 2. Arquitectura General

### R1. Estructura de proyecto en capas
- **Descripción**: El proyecto debe separarse en al menos tres partes claras:
  - `logger-core` (Rust): lógica principal y API interna.
  - `bindings-js`: integración para JS/TS (WASM o N-API).
  - `bindings-java`: integración para Java (JNI).
- **Prioridad**: P0
- **Tipo**: NF

### R2. Core único de lógica de logging
- **Descripción**: Toda la lógica de negocio (niveles, formateo, sinks, filtros, configuración) debe residir en el core de Rust, sin duplicación en los bindings.
- **Prioridad**: P0
- **Tipo**: NF

---

## 3. Funcionalidades Básicas de Logging

### R3. Niveles de log estándar
- **Descripción**: Soportar al menos: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`.
- **Prioridad**: P0
- **Tipo**: F

### R4. API de logging estructurada
- **Descripción**: API que permita registrar mensajes con:
  - Mensaje simple.
  - Campos adicionales (clave/valor) opcionales.
- **Prioridad**: P1
- **Tipo**: F

### R5. Soporte de formato texto simple
- **Descripción**: Formato por defecto legible tipo: `[timestamp] [LEVEL] [file:line] mensaje`
- **Prioridad**: P0
- **Tipo**: F

### R6. Soporte de formato JSON
- **Descripción**: Posibilidad de emitir logs en JSON estructurado apto para pipelines (log aggregation, ELK, CloudWatch, etc.).
- **Prioridad**: P1
- **Tipo**: F

---

## 4. Metadatos de Contexto (archivo, línea, módulo, función)

### R7. Captura de archivo y línea
- **Descripción**: El logger debe poder incluir `file!()`, `line!()` y `module_path!()` mediante macros, sin requerir backtrace.
- **Prioridad**: P0
- **Tipo**: F

### R8. Captura opcional de nombre de función
- **Descripción**: Cuando sea posible (vía crates como `function_name`), incluir el nombre de la función desde la que se loguea.
- **Prioridad**: P1
- **Tipo**: F

### R9. Desactivación configurable de metadatos
- **Descripción**: Debe ser posible desactivar metadatos (archivo, línea, función) para entornos donde se quiera minimizar tamaño u overhead.
- **Prioridad**: P2
- **Tipo**: F

---

## 5. Colores y Salida a Consola

### R10. Colores en consola según nivel
- **Descripción**: Soporte de colores ANSI para terminal, con mapeo nivel → color (ej: `INFO` verde, `WARN` amarillo, `ERROR` rojo).
- **Prioridad**: P0
- **Tipo**: F

### R11. Desactivación de colores automática o configurable
- **Descripción**: Posibilidad de desactivar colores si el entorno no los soporta (detección de TTY) o por configuración explícita.
- **Prioridad**: P1
- **Tipo**: F

### R12. Soporte de temas de color
- **Descripción**: Permitir configurar el esquema de colores (ej: modo "minimal", "high contrast", etc.).
- **Prioridad**: P2
- **Tipo**: F

---

## 6. Sinks / Destinos de Log

### R13. Sink de consola
- **Descripción**: Destino básico que escribe en `stdout` / `stderr`.
- **Prioridad**: P0
- **Tipo**: F

### R14. Sink de archivo simple
- **Descripción**: Escribir logs en un archivo plano (sin rotación).
- **Prioridad**: P0
- **Tipo**: F

### R15. Sink con rotación de archivos
- **Descripción**: Soportar rotación por tamaño y/o por fecha (ej: daily rotation), con un número máximo de archivos a conservar.
- **Prioridad**: P1
- **Tipo**: F

### R16. Sink para AWS CloudWatch
- **Descripción**: Enviar logs a CloudWatch Logs mediante el SDK oficial de AWS, con soporte para log group/stream configurable y batching básico.
- **Prioridad**: P2
- **Tipo**: F

### R17. Múltiples sinks simultáneos
- **Descripción**: Poder configurar varios destinos a la vez: consola + archivo + CloudWatch, etc.
- **Prioridad**: P1
- **Tipo**: F

### R18. Filtros por sink
- **Descripción**: Permitir que cada sink tenga su propio nivel mínimo (ej: consola `INFO`+, archivo `DEBUG`+).
- **Prioridad**: P2
- **Tipo**: F

---

## 7. Concurrencia y Seguridad en Escritura

### R19. Seguridad de concurrencia en un solo proceso
- **Descripción**: El logger debe ser seguro para uso concurrente desde múltiples threads en el mismo proceso, sin corrupción de datos en los sinks.
- **Prioridad**: P0
- **Tipo**: NF

### R20. Dos threads escribiendo al mismo archivo
- **Descripción**: La solución debe garantizar que las escrituras al archivo desde múltiples threads del mismo proceso sean atómicas a nivel de línea (no mezclar mensajes).
- **Prioridad**: P0
- **Tipo**: NF

### R21. Varios procesos escribiendo al mismo archivo
- **Descripción**: Debe considerarse el caso de múltiples procesos escribiendo en un mismo archivo (definir estrategia: soportado / no recomendado / bloqueo de fichero / advisory locks).
- **Prioridad**: P2
- **Tipo**: NF

### R22. Buffering vs escritura síncrona
- **Descripción**: Permitir configurar si los logs se escriben de forma síncrona (más segura, más lenta) o con buffering/asíncrono (más rápido, riesgo mínimo en crash).
- **Prioridad**: P1
- **Tipo**: F

---

## 8. Bindings para JavaScript / TypeScript

### R23. API JS/TS amigable
- **Descripción**: Exponer una API en JS/TS que se sienta natural, por ejemplo:
  - `logger.info("mensaje", { meta })`
  - `logger.configure({ level, sinks, format })`
- **Prioridad**: P1
- **Tipo**: F

### R24. Implementación basada en core Rust
- **Descripción**: Los bindings deben llamar al core Rust (vía WASM o N-API), no reimplementar lógica.
- **Prioridad**: P1
- **Tipo**: NF

### R25. Soporte para Node.js (inicial)
- **Descripción**: Foco inicial en compatibilidad con Node.js.
- **Prioridad**: P1
- **Tipo**: NF

### R26. Manejo de errores traducido a JS
- **Descripción**: Errores internos de Rust deben exponerse como excepciones u objetos de error útiles en JS.
- **Prioridad**: P1
- **Tipo**: F

---

## 9. Bindings para Java

### R27. API Java sencilla
- **Descripción**: Exponer una clase/servicio Java, por ejemplo:
  - `Logger.info(String message)`
  - `Logger.configure(LoggerConfig config)`
- **Prioridad**: P1
- **Tipo**: F

### R28. Integración vía JNI
- **Descripción**: Usar JNI para llamar al core Rust compilado como librería nativa.
- **Prioridad**: P1
- **Tipo**: NF

### R29. Manejo de errores traducido a excepciones Java
- **Descripción**: Errores en Rust deben traducirse a excepciones claras en Java.
- **Prioridad**: P1
- **Tipo**: F

### R30. Empaquetado para distribución en proyectos Java
- **Descripción**: Proveer una forma razonable de empaquetar/consumir el logger desde Maven/Gradle.
- **Prioridad**: P2
- **Tipo**: NF

---

## 10. Configuración

### R31. Configuración desde código
- **Descripción**: Permitir configurar el logger de forma programática desde Rust, JS y Java.
- **Prioridad**: P0
- **Tipo**: F

### R32. Configuración por archivo (opcional)
- **Descripción**: Opcionalmente soportar configuración por archivo (YAML/JSON/TOML) para casos de despliegue.
- **Prioridad**: P2
- **Tipo**: F

### R33. Niveles configurables por módulo
- **Descripción**: Poder establecer niveles de log por módulo/nombre de logger.
- **Prioridad**: P2
- **Tipo**: F

---

## 11. Rendimiento y Robustez

### R34. Bajo overhead en el "fast path"
- **Descripción**: El coste de loguear cuando el nivel está desactivado debe ser mínimo (ej: evaluación lazy de mensajes).
- **Prioridad**: P1
- **Tipo**: NF

### R35. Medición básica de rendimiento
- **Descripción**: Se deben hacer pruebas básicas de rendimiento para validar que el logger no es un cuello de botella en escenarios típicos.
- **Prioridad**: P2
- **Tipo**: NF

### R36. No panics en producción
- **Descripción**: El logger nunca debe hacer `panic!` en escenarios normales; ante errores de I/O debe degradarse con gracia.
- **Prioridad**: P0
- **Tipo**: NF

---

## 12. DX (Developer Experience)

### R37. Macros amigables en Rust
- **Descripción**: Proveer macros del estilo `info!`, `error!`, etc., que ya capturen metadatos automáticamente.
- **Prioridad**: P0
- **Tipo**: F

### R38. Documentación clara y ejemplos
- **Descripción**: Incluir documentación con ejemplos de uso en Rust, JS y Java.
- **Prioridad**: P1
- **Tipo**: NF

### R39. Comportamiento sensato por defecto
- **Descripción**: Con la mínima configuración, el logger debe:
  - Loguear a consola.
  - Usar niveles estándar.
  - Usar un formato legible.
- **Prioridad**: P0
- **Tipo**: NF
