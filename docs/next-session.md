# Next Session — Objetivos inmediatos de oxidize-log

Este documento define los **2–3 pasos siguientes** del proyecto, con un enfoque claro, concreto y accionable.  
La idea es avanzar en bloques pequeños pero profundos, manteniendo TDD y una arquitectura limpia.

---

## 🎯 Objetivo 1 — Introducir `LoggerConfig` y `Environment` (P0)

### ¿Por qué este paso?
Hasta ahora el logger tiene un comportamiento fijo.  
Para avanzar hacia un sistema real necesitamos una **capa de configuración** que permita:

- elegir nivel mínimo
- activar/desactivar colores
- seleccionar sinks (por ahora solo consola)
- preparar el terreno para dev/staging/prod

Esto NO debe ir dentro del core del logger, sino en un módulo dedicado.

### Tareas concretas
- Crear archivo `src/config.rs`
- Definir:
  ```rust
  pub enum Environment { Dev, Staging, Prod }
  ```
- Definir:
  ```rust
  pub struct LoggerConfig {
      pub level: LogLevel,
      pub colors: bool,
      pub sinks: Vec<SinkConfig>, // por ahora solo Console
  }
  ```
- Implementar:
  ```rust
  impl LoggerConfig {
      pub fn from_env(env: Environment) -> Self { ... }
  }
  ```
- Añadir tests unitarios:
  - `config_for_dev_has_debug_and_colors`
  - `config_for_staging_has_info_no_colors`
  - `config_for_prod_has_warn_no_colors`

### Resultado esperado
Un módulo de configuración sólido, testeado y listo para integrarse con el logger.

---

## 🎯 Objetivo 2 — Integrar `LoggerConfig` en `Logger::init` (P0)

### ¿Por qué este paso?
Ahora mismo `Logger::init_default()` crea un logger fijo.  
Queremos que el logger pueda inicializarse con una configuración real.

### Tareas concretas
- Modificar `Logger` para aceptar `LoggerConfig`
- Añadir:
  ```rust
  pub fn init(config: LoggerConfig) -> Self
  ```
- Ajustar el filtrado de niveles para usar `config.level`
- Añadir tests:
  - `logger_respects_configured_level`
  - `logger_initializes_with_colors_flag` (aunque aún no se usen)

### Resultado esperado
El logger ya no es rígido: puede configurarse desde fuera y se prepara para soportar sinks y colores.

---

## 🎯 Objetivo 3 — Crear el primer `Sink`: ConsoleSink (P0)

### ¿Por qué este paso?
El logger actual imprime directamente a consola.  
Eso está bien para un prototipo, pero no para un sistema modular.

Necesitamos separar:

- **core** → decide qué log se emite  
- **sink** → decide dónde se escribe  

### Tareas concretas
- Crear archivo `src/sink.rs`
- Definir:
  ```rust
  pub enum SinkConfig {
      Console,
  }
  ```
- Crear trait:
  ```rust
  pub trait Sink {
      fn write(&self, record: &LogRecord);
  }
  ```
- Crear `ConsoleSink`
- Modificar `Logger` para:
  - almacenar una lista de sinks
  - enviar cada log a cada sink

### Tests necesarios
- `console_sink_writes_to_stdout` (capturando salida)
- `logger_sends_record_to_all_sinks` (aunque solo haya uno)

### Resultado esperado
El logger deja de imprimir directamente y pasa a usar un sistema extensible de sinks.

---

# 🧩 Resumen de la sesión siguiente

En la próxima sesión construiremos:

1. **`LoggerConfig` + `Environment`**  
2. **Integración de configuración en `Logger::init`**  
3. **Primer sink real: `ConsoleSink`**

Con estos tres pasos, tu logger pasa de ser un prototipo a tener una **arquitectura real**, modular, extensible y preparada para crecer hacia:

- colores  
- sinks múltiples  
- rotación  
- CloudWatch  
- macros  
- bindings JS/Java  

Todo manteniendo TDD.
