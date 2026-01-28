# 🧭 **Roadmap `oxidize-log` (con progreso marcado)**

Leyenda:  
- **P0** = esencial  
- **P1** = importante  
- **P2** = nice to have  
- **F** = funcional  
- **NF** = no funcional  
- **✔️** = ya construido  
- **🟡** = parcialmente iniciado  
- **⬜** = pendiente

---

# 1. Visión general del proyecto

*(No tiene requisitos marcables)*

---

# 2. Arquitectura general

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R1 | Estructura en capas (core + bindings) | P0 | NF | ⬜ |
| R2 | Core único de lógica | P0 | NF | ✔️ *(ya tenemos `level.rs`, `logger.rs`, `lib.rs`)* |

---

# 3. Funcionalidades básicas de logging

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R3 | Niveles estándar | P0 | F | ✔️ *(enum LogLevel + tests)* |
| R4 | API estructurada (mensaje + campos) | P1 | F | ⬜ |
| R5 | Formato texto simple | P0 | F | 🟡 *(logger imprime texto, falta timestamp y metadatos)* |
| R6 | Formato JSON | P1 | F | ⬜ |

---

# 4. Metadatos de contexto

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R7 | Captura de archivo y línea | P0 | F | ⬜ *(requiere macros)* |
| R8 | Nombre de función opcional | P1 | F | ⬜ |
| R9 | Metadatos configurables | P2 | F | ⬜ |

---

# 5. Colores y salida a consola

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R10 | Colores por nivel | P0 | F | ⬜ |
| R11 | Desactivar colores | P1 | F | ⬜ |
| R12 | Temas de color | P2 | F | ⬜ |

---

# 6. Sinks / Destinos

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R13 | Sink consola | P0 | F | 🟡 *(logger imprime en consola, falta modularizar como sink)* |
| R14 | Sink archivo simple | P0 | F | ⬜ |
| R15 | Rotación de archivos | P1 | F | ⬜ |
| R16 | Sink CloudWatch | P2 | F | ⬜ |
| R17 | Múltiples sinks | P1 | F | ⬜ |
| R18 | Filtros por sink | P2 | F | ⬜ |

---

# 7. Concurrencia y seguridad

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R19 | Seguridad en un proceso | P0 | NF | ⬜ |
| R20 | Escritura atómica en archivo | P0 | NF | ⬜ |
| R21 | Varios procesos escribiendo | P2 | NF | ⬜ |
| R22 | Buffering vs síncrono | P1 | F | ⬜ |

---

# 8. Bindings JS/TS

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R23 | API JS amigable | P1 | F | ⬜ |
| R24 | Bindings basados en core | P1 | NF | ⬜ |
| R25 | Soporte Node.js | P1 | NF | ⬜ |
| R26 | Errores traducidos a JS | P1 | F | ⬜ |

---

# 9. Bindings Java

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R27 | API Java sencilla | P1 | F | ⬜ |
| R28 | Integración JNI | P1 | NF | ⬜ |
| R29 | Errores traducidos a Java | P1 | F | ⬜ |
| R30 | Empaquetado Maven/Gradle | P2 | NF | ⬜ |

---

# 10. Configuración

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R31 | Configuración programática | P0 | F | ⬜ *(lo hablaste, falta implementarlo)* |
| R32 | Configuración por archivo | P2 | F | ⬜ |
| R33 | Niveles por módulo | P2 | F | ⬜ |

---

# 11. Rendimiento y robustez

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R34 | Fast path eficiente | P1 | NF | 🟡 *(ya filtras por nivel, falta lazy evaluation)* |
| R35 | Medición de rendimiento | P2 | NF | ⬜ |
| R36 | No panics en producción | P0 | NF | ✔️ *(tu logger actual no hace panic)* |

---

# 12. Developer Experience (DX)

| ID | Requisito | Prioridad | Tipo | Estado |
|----|-----------|-----------|------|--------|
| R37 | Macros amigables | P0 | F | ⬜ |
| R38 | Documentación clara | P1 | NF | ⬜ |
| R39 | Defaults sensatos | P0 | NF | 🟡 *(init_default existe, falta formato y sinks)* |

---

# 🧠 Resumen de progreso real

### ✔️ Completado
- R2: Core inicial  
- R3: Niveles  
- R36: No panics  
- Tests unitarios básicos  
- Estructura inicial del crate  
- Logger básico con filtrado  

### 🟡 En progreso
- R5: Formato texto (mínimo)  
- R13: Consola (mínimo)  
- R34: Fast path básico  
- R39: Defaults iniciales  

### ⬜ Pendiente
Todo lo demás: configuración, sinks, macros, colores, bindings, etc.

---

Si quieres, puedo ayudarte a **convertir este roadmap en un `ROADMAP.md` oficial** dentro del repo, o incluso en **issues de GitHub** listos para trabajar en ellos.