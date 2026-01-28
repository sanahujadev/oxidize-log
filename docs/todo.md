# 🧭 **Roadmap `oxidize-log` — Formato TODO**

Leyenda:  
- **[X]** = hecho  
- **[ ]** = pendiente  
- **🟡** = en progreso  
- **P0/P1/P2** = prioridad  
- **F/NF** = funcional / no funcional  

---

# 1. Visión general  
*(No aplica)*

---

# 2. Arquitectura general

- [ ] **R1 (P0, NF)** Estructura en capas (core + bindings)  
- [X] **R2 (P0, NF)** Core único de lógica (`level.rs`, `logger.rs`, `lib.rs`)

---

# 3. Funcionalidades básicas de logging

- [X] **R3 (P0, F)** Niveles estándar  
- [ ] **R4 (P1, F)** API estructurada (mensaje + campos)  
- [🟡] **R5 (P0, F)** Formato texto simple (falta timestamp y metadatos)  
- [ ] **R6 (P1, F)** Formato JSON  

---

# 4. Metadatos de contexto

- [ ] **R7 (P0, F)** Captura de archivo y línea (requiere macros)  
- [ ] **R8 (P1, F)** Nombre de función opcional  
- [ ] **R9 (P2, F)** Metadatos configurables  

---

# 5. Colores y salida a consola

- [ ] **R10 (P0, F)** Colores por nivel  
- [ ] **R11 (P1, F)** Desactivar colores  
- [ ] **R12 (P2, F)** Temas de color  

---

# 6. Sinks / Destinos

- [🟡] **R13 (P0, F)** Sink consola (mínimo hecho, falta modularización completa)  
- [ ] **R14 (P0, F)** Sink archivo simple  
- [ ] **R15 (P1, F)** Rotación de archivos  
- [ ] **R16 (P2, F)** Sink CloudWatch  
- [ ] **R17 (P1, F)** Múltiples sinks  
- [ ] **R18 (P2, F)** Filtros por sink  

---

# 7. Concurrencia y seguridad

- [ ] **R19 (P0, NF)** Seguridad en un proceso  
- [ ] **R20 (P0, NF)** Escritura atómica en archivo  
- [ ] **R21 (P2, NF)** Varios procesos escribiendo  
- [ ] **R22 (P1, F)** Buffering vs síncrono  

---

# 8. Bindings JS/TS

- [ ] **R23 (P1, F)** API JS amigable  
- [ ] **R24 (P1, NF)** Bindings basados en core  
- [ ] **R25 (P1, NF)** Soporte Node.js  
- [ ] **R26 (P1, F)** Errores traducidos a JS  

---

# 9. Bindings Java

- [ ] **R27 (P1, F)** API Java sencilla  
- [ ] **R28 (P1, NF)** Integración JNI  
- [ ] **R29 (P1, F)** Errores traducidos a Java  
- [ ] **R30 (P2, NF)** Empaquetado Maven/Gradle  

---

# 10. Configuración

- [ ] **R31 (P0, F)** Configuración programática (builder pattern)  
- [ ] **R32 (P2, F)** Configuración por archivo  
- [ ] **R33 (P2, F)** Niveles por módulo  

---

# 11. Rendimiento y robustez

- [🟡] **R34 (P1, NF)** Fast path eficiente (falta lazy evaluation)  
- [ ] **R35 (P2, NF)** Medición de rendimiento  
- [X] **R36 (P0, NF)** No panics en producción  

---

# 12. Developer Experience (DX)

- [ ] **R37 (P0, F)** Macros amigables  
- [ ] **R38 (P1, NF)** Documentación clara  
- [🟡] **R39 (P0, NF)** Defaults sensatos (falta formato y sinks)

---

# 🧠 Resumen

### ✔️ Hecho
- Core inicial  
- Niveles  
- No panics  
- Tests básicos  
- Logger con filtrado  

### 🟡 En progreso
- Formato texto  
- Sink consola  
- Fast path  
- Defaults  

### ⬜ Pendiente
- Metadatos  
- Macros  
- Config builder  
- Sinks avanzados  
- Colores  
- JSON  
- Bindings  
- Concurrencia  
- Rotación  
- Documentación  
