# Equipo de Agentes y Roles

Este documento detalla la estructura organizativa, responsabilidades y perfiles del equipo de agentes (internos y externos) asignados al proyecto **oxidize-log**.

---

## 🏛️ Arquitectura y Diseño (Agentes Externos)

### MiniMax
- **Rol**: Arquitecto en Jefe.
- **Responsabilidad**: Liderar las decisiones arquitectónicas del sistema, diseñar la estructura del core, definir los lineamientos principales y aprobar propuestas de diseño técnico. Crear las versiones de ADR y PDR a implementar. Entender las revisiones de Qwen e incluirlas si lo amerita, o rechazarlas con justificación técnica.

### Qwen
- **Rol**: Arquitecto Segundo.
- **Responsabilidad**: Apoyar al Arquitecto en Jefe, proponer alternativas de diseño, resolver problemas técnicos complejos en el core y colaborar en el refinamiento de especificaciones. Hacer revisiones de los ADR y PDR generados por MiniMax.

---

## 🔍 Aseguramiento de Calidad y Revisión

### Kimi
- **Rol**: Revisor de Código (*Code Reviewer*).
- **Responsabilidad**: Analizar y revisar las implementaciones de código, verificar el cumplimiento de TDD, asegurar la eficiencia y robustez de Rust, y proveer feedback antes de fusionar cambios. Análisis de mejores prácticas y corner cases.

---

## ⚙️ Ejecución y Desarrollo

### agy-desktop / Jules
- **Rol**: Ejecutor de ADR / PDR.
- **Responsabilidad**: Llevar a cabo la implementación técnica y el desarrollo de código basado estrictamente en las decisiones cerradas (ADR) y propuestas aprobadas (PDR).

---

## 🕵️‍♂️ Investigación y Documentación

### agy
- **Rol**: Investigador Privado del Proyecto.
- **Responsabilidad**: Investigar el estado del codebase, documentar decisiones, afinar los criterios de aceptación y pautas técnicas para los agentes externos, y mantener la coherencia del repositorio.
