# oxidize-log

**oxidize-log** es un núcleo (core) de logging de alto rendimiento escrito en Rust. Este repositorio actualmente contiene la base inicial del logger en Rust, listo para ser utilizado como base de futuros bindings multiplataforma.

Para conocer la visión completa del proyecto y el roadmap de desarrollo, consulta el documento de [Roadmap](file:///home/zitrojj/dev/pro/oxidize-log/docs/roadmap.md).

---

## 📦 Estado Actual del Proyecto

Actualmente, el proyecto se encuentra en su fase inicial con un core mínimo implementado en Rust. La estructura real del repositorio es:

- **`/src`**: Módulos iniciales del logger (`config`, `level`, `logger`, `sink`).
- **`/examples`**: Un ejemplo de integración manual (`test.rs`).
- **`/docs`**: Documentación de desarrollo, notas y roadmap.

---

## 🛠️ Requisitos de Ejecución

Para desarrollar y ejecutar los comandos de este proyecto sin necesidad de instalar Rust de forma local, el entorno está completamente contenedorizado con **Docker** y **Docker Compose**.

### Requisitos locales en tu máquina:
- Docker
- Docker Compose

---

## 🚀 Cómo Empezar y Comandos Útiles

El proyecto cuenta con un script unificado (`run.sh`) que automatiza el ciclo de desarrollo ejecutando Cargo dentro de contenedores efímeros con caché persistente para agilizar las compilaciones consecutivas.

### 1. Ejecutar el ejemplo de desarrollo
Compila y ejecuta el ejemplo ubicado en `examples/test.rs`:
```bash
./run.sh dev
```

### 2. Ejecutar las pruebas unitarias y de integración
Corre la suite completa de tests definidos en el core:
```bash
./run.sh test
```

### 3. Ejecutar flujo de CI local
Ejecuta la suite de tests y posteriormente compila y corre el ejemplo de prueba:
```bash
./run.sh ci
```

---

## 🧪 Ejemplo de Uso Actual

A día de hoy, el logger se inicializa manualmente configurando sus destinos (sinks) y niveles de log de la siguiente manera:

```rust
use oxidize_log::{Logger, LoggerConfig, LogLevel, SinkConfig};

fn main() {
    // 1. Definir la configuración del Logger
    let config = LoggerConfig {
        level: LogLevel::Debug,
        colors: true,
        sinks: vec![SinkConfig::Console],
    };

    // 2. Inicializar el logger global
    let logger = Logger::init(config);

    // 3. Registrar un mensaje
    logger.log(LogLevel::Info, "Hola desde oxidize-log inicializado de forma manual");
}
```
