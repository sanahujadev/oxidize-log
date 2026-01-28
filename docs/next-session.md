# **next-session.md**

## **🎯 Objetivo general de la próxima sesión**
Evolucionar el logger desde un sistema básico con sinks hacia un **logger profesional**, capaz de incluir metadatos (timestamp, file, line), formateo flexible y configuración sobrescribible.

---

## **1. Introducir `LogRecord` como estructura central del logging**
Actualmente `Sink::log` recibe:

```rust
fn log(&self, level: LogLevel, message: &str);
```

Esto es insuficiente para un logger serio.  
El siguiente paso es introducir:

```rust
pub struct LogRecord<'a> {
    pub level: LogLevel,
    pub message: &'a str,
    pub file: &'a str,
    pub line: u32,
    pub timestamp: DateTime<Utc>,
}
```

### **Motivación**
- Permite añadir timestamp  
- Permite capturar file/line automáticamente  
- Permite formateo flexible  
- Permite sinks avanzados (JSON, archivo, remoto…)  
- Separa datos del mensaje del formateo

---

## **2. Actualizar el trait `Sink` para recibir `LogRecord`**
Nuevo trait:

```rust
pub trait Sink {
    fn log(&self, record: &LogRecord);
    fn as_any(&self) -> &dyn Any;
}
```

### **Impacto**
- ConsoleSink deberá formatear el record  
- MockSink deberá almacenar records completos  
- Logger deberá construir el record antes de delegar  

---

## **3. Añadir macros de logging (`info!`, `warn!`, etc.)**
Estas macros capturarán automáticamente:

- `file!()`
- `line!()`
- `message`
- nivel

Ejemplo:

```rust
info!("User {} logged in", user_id);
```

Internamente construirá un `LogRecord`.

### **Motivación**
- API ergonómica  
- Captura automática de metadatos  
- Igual que `log` o `tracing`  

---

## **4. Añadir timestamp automático**
Usaremos `chrono` o `time` (decidiremos en sesión).

Formato inicial:

```
2026-01-28T23:33:12Z
```

---

## **5. Añadir builder pattern a `LoggerConfig`**
Permitir:

```rust
LoggerConfig::from_env(Environment::Dev)
    .with_level(LogLevel::Warn)
    .with_colors(false)
    .with_sinks(vec![SinkConfig::Console]);
```

### **Motivación**
- Overrides limpios  
- Config flexible  
- No depender solo del entorno  

---

## **6. Tests necesarios**
- Logger construye correctamente un `LogRecord`  
- Macros capturan file/line  
- Timestamp existe  
- ConsoleSink formatea correctamente  
- MockSink recibe records completos  
- Config builder sobrescribe valores  

---

## **7. Resultado esperado al final de la sesión**
Un logger que imprime algo así:

```
2026-01-28T23:33:12Z [INFO] (src/main.rs:42) User logged in
```

Y con configuración flexible:

```rust
LoggerConfig::from_env(Environment::Dev)
    .with_level(LogLevel::Error)
    .with_colors(true)
    .with_sinks(vec![SinkConfig::Console]);
```

---

## **8. Preparado para siguientes fases**
Una vez completado este objetivo, estaremos listos para:

- `FileSink`
- `JsonSink`
- `RemoteSink`
- Rotación de archivos
- Formatos personalizados
- Integración con tracing
