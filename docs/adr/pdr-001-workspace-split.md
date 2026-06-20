# PDR-001 — Workspace split del core vs bindings

> **Estado:** en discusión
> **Última actualización:** 2026-06-09
> **Origen:** ADR-001 §Motivo §4

## Contexto
El V0 prioriza una arquitectura multicapa (R1) en la que el core de logging existe de forma independiente a los bindings (JS/Java). En el prototipo inicial, todo vivía en la raíz del proyecto. Tras el refactor a arquitectura hexagonal (ADR-001, Sprint 1), la lógica ya está debidamente aislada, pero el repositorio aún opera como un único crate (`oxidize-log`).

## Problema
Necesitamos decidir en qué momento y bajo qué condiciones transformaremos la estructura del repositorio de un solo crate a un `[workspace]` que albergue el `logger-core` y, posteriormente, los crates para los diferentes bindings.

## Restricciones
- R1: La separación de capas entre el core de lógica y la lógica específica de cada plataforma es una prioridad fundamental (P0).
- No se debe ejecutar el split del workspace antes de que exista la necesidad real (es decir, antes de comenzar el desarrollo del primer binding adicional).
- Se debe preservar el tiempo de compilación rápido y la experiencia de desarrollo sin fricciones.

## Opciones evaluadas
### Opción A — Split ahora (al cierre de Sprint 1)
Migrar el `Cargo.toml` raíz a un workspace y mover todo el código actual a una carpeta `logger-core/` de forma preventiva.
*Pros:* Prepara la estructura final de inmediato.
*Contras:* Añade fricción de reestructuración y un `Cargo.toml` de workspace vacío sin aportar valor práctico en este momento.

### Opción B — Split cuando llegue el primer binding (oxidize-log-js)
Mantener la estructura de un solo crate hasta el Sprint en que se comience a implementar el primer binding.
*Pros:* Evita refactorizaciones preventivas. El split se ejecuta por una necesidad material de compartir código y se puede diseñar acorde a las necesidades reales que surjan.
*Contras:* Requerirá un pequeño esfuerzo de refactorización organizativa en un sprint futuro.

## Análisis de Dimensiones Técnicas
- **Diseño:** Un split impactará los paths relativos de directorios y declaraciones `use` dentro de un futuro workspace.
- **Bindings:** Esta decisión organizativa es la pre-condición estructural principal para abordar los requisitos R23-R30.
- **Rendimiento:** Un workspace afectará ligeramente las compilaciones incrementales dependiendo de la gestión del `Cargo.lock` unificado.
- **Sinks:** Esta estructura organizativa no tiene ningún impacto directo sobre los sinks o la arquitectura hexagonal en sí misma.

## Preguntas abiertas
- ¿Deberíamos utilizar WASM (`wasm-bindgen`) o N-API (`napi-rs`) para JS? → A resolver en **PDR-002**
- ¿Se utilizará JNI de manera directa o se buscará algún wrapper automatizado para la integración con Java? → A resolver en **PDR-002**

## Propuesta provisional
Opción B: esperar al primer binding antes de realizar la migración al workspace.

## Cierre
[Se rellena cuando se decida]
