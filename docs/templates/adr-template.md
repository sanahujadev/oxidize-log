# ADR — [Nombre de la decisión]

> Architecture Decision Record. Documento inmutable una vez cerrado.
> Creado por el agente `architect`. Nunca se edita — si la decisión cambia, se abre un nuevo ADR
> que referencia y supera a este.
> Estado: `en implementación` | `implementado` | `superado por [adr-nombre.md]`
> Fecha de decisión: [YYYY-MM-DD]

---

## Contexto

<!--
La situación que hizo necesaria esta decisión.
Debe poder leerse de forma independiente — no asumas que quien lee esto
conoce el PDR previo, aunque exista.
-->

[descripción del contexto]

---

## Decisión

<!--
La decisión tomada, expresada con claridad y sin ambigüedad.
Empieza con "Se decide..." o "Adoptamos...".
Una sola decisión por ADR. Si son dos decisiones, son dos ADRs.
-->

[enunciado de la decisión]

---

## Especificaciones Técnicas

<!--
Detalles técnicos definitivos derivados de esta decisión para una librería nativa y multiplataforma.
Borra las dimensiones que no apliquen.
-->

### 1. Diseño del Core y Traits (Rust)
- **Traits expuestos / Puertos:** [traits nuevos o modificados, firmas clave]
- **Abstracción:** [genéricos estáticos (monomorfización) vs trait objects (`dyn`), justificación]

### 2. Bindings y Capa de FFI (Multiplataforma)
- **Tecnología:** [JNI, N-API/WASM, FFI plana]
- **Pasaje de datos:** [tipos nativos, serialización JSON, buffers binarios, manejo de punteros y memoria]

### 3. Rendimiento, Concurrencia y Memoria
- **Sincronización:** [seguridad para hilos (`Send + Sync`), bloqueos (Mutex, RwLock), lock-free, etc.]
- **Asignación de memoria:** [estrategia de alloc / uso de heap, compatibilidad con no_std]
- **Fast-path overhead:** [optimización cuando el nivel de log está desactivado]

### 4. Sinks e Infraestructura (Adaptadores)
- **I/O y Buffering:** [operaciones bloqueantes vs asíncronas, tamaño de buffers de escritura]
- **Dependencias de terceros:** [crates externos añadidos y su impacto]

---

## Motivo

<!--
Por qué esta opción y no las otras.
Si hubo un PDR previo, referéncialo pero no lo copies.
El motivo debe poder entenderse sin leer el PDR.
-->

[justificación de la decisión]

**PDR de origen:** [`pdr-[nombre].md`](pdr-[nombre].md) — o "decisión directa sin PDR previo"

---

## Consecuencias

<!--
Qué cambia en el sistema como resultado de esta decisión.
Incluye tanto las consecuencias positivas como las negativas o las nuevas restricciones que introduce.
-->

**Positivas:**
- [consecuencia 1]
- [consecuencia 2]

**Negativas o restricciones introducidas:**
- [consecuencia negativa o nueva restricción]

---

## Módulos afectados

<!--
Qué módulos cambian, y en qué dirección.
-->

| Módulo / Archivo | Tipo de cambio |
|--------|---------------|
| `[modulo/archivo]` | [nuevo / modificado / eliminado / sin cambio pero afectado] |

---

## Criterio de implementación completa

<!--
Cómo sabe el agente (y el humano) que esta decisión está completamente implementada.
Debe ser verificable — idealmente ligado a tests en verde o a un comportamiento observable.
-->

- [ ] [criterio 1 — verificable]
- [ ] [criterio 2 — verificable]
- [ ] Tests relevantes en verde: [descripción de qué tests cubren esta decisión]

---

## Todo de implementación

<!--
Referencia al todo.md generado por el task-manager para implementar este ADR.
-->

→ [`todo-[nombre].md`](todo-[nombre].md)

---

## Historial de superación

<!--
Solo se rellena si este ADR fue superado por uno posterior.
-->

**Estado:** `superado`
**Superado por:** [`adr-[nombre-nuevo].md`](../[YYYY-MM]/adr-[nombre-nuevo].md)
**Fecha:** [YYYY-MM-DD]
**Motivo:** [una línea — por qué esta decisión ya no aplica]
