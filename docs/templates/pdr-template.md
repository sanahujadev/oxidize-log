# PDR — [Nombre del feature o cambio en discusión]

> Proposal Design Record. Documento vivo mientras la decisión está abierta.
> Creado por el agente `architect`. Cerrado cuando se toma una decisión — en ese momento
> se convierte en un ADR o se archiva como descartado.
> Estado: `en discusión` | `pendiente de validación` | `descartado`
> Última actualización: [YYYY-MM-DD]

---

## Contexto

<!--
Por qué estamos aquí. Qué situación, problema o necesidad abrió esta discusión.
Sin contexto no hay decisión — esta sección es obligatoria.
Responde: ¿qué está pasando en la librería o su consumo que hace necesario este cambio?
-->

[descripción del contexto que motiva esta propuesta]

---

## Problema

<!--
Formulación precisa del problema a resolver.
Una sola frase si es posible. Si necesitas más, el problema puede estar mal definido.
-->

[enunciado del problema]

---

## Restricciones

<!--
Qué no puede cambiar. Qué está fuera del alcance de esta decisión.
Ejemplos: "No debe requerir std (debe compilar con no_std)."
          "No puede introducir dependencias con macros complejas."
          "La firma del trait Sink debe seguir siendo Send + Sync."
-->

- [restricción 1]
- [restricción 2]

---

## Opciones evaluadas

<!--
Mínimo dos opciones. Si solo hay una, no es una decisión — es una tarea.
Para cada opción: qué es, qué resuelve, qué sacrifica.
No hay opción neutra — todas tienen tradeoffs.
-->

### Opción A — [nombre descriptivo]

**Descripción:** [cómo se implementa en Rust/FFI]

**Ventajas:**
- [ventaja 1]
- [ventaja 2]

**Desventajas / riesgos:**
- [desventaja 1]
- [desventaja 2]

---

### Opción B — [nombre descriptivo]

**Descripción:** [cómo se implementa en Rust/FFI]

**Ventajas:**
- [ventaja 1]

**Desventajas / riesgos:**
- [desventaja 1]

---

## Análisis de Dimensiones Técnicas

<!--
Comparación de las opciones en dimensiones críticas para una librería.
Borra las dimensiones que no apliquen a esta decisión.
-->

- **Diseño del Core y Traits:** [¿Traits genéricos vs dinámicos? ¿Impacto en la firma de APIs?]
- **Bindings y Capa de FFI:** [¿Cómo impacta a JS/Java? ¿Facilidad de mapeo JNI/WASM/N-API?]
- **Rendimiento y Memoria:** [¿Uso de heap (alloc)? ¿ overhead en fast-path? ¿Hilos y bloqueos?]
- **Sinks y Adaptadores:** [¿Operaciones I/O bloqueantes, buffering o background threads?]

---

## Preguntas abiertas

<!--
Lo que no se puede decidir aún porque falta información, validación técnica o pruebas de concepto.
Cada pregunta debe tener un responsable y una fecha límite.
-->

- [ ] [pregunta] — responsable: [agente o humano] — fecha: [YYYY-MM-DD]
- [ ] [pregunta] — responsable: [agente o humano]

---

## Propuesta provisional

<!--
Si el architect tiene una preferencia antes de cerrar la discusión, la expresa aquí.
-->

[opción preferida y por qué, si la hay]

---

## Cierre

<!--
Esta sección la rellena el architect cuando se toma la decisión.
Una vez cerrado, este PDR se convierte en ADR (si se aprueba) o se mueve a archive/ (si se descarta).
-->

**Decisión:** [opción elegida o "descartado"]
**Motivo del cierre:** [una o dos frases]
**Fecha de cierre:** [YYYY-MM-DD]
**ADR resultante:** [`adr-[nombre].md`](adr-[nombre].md) — o "ninguno, descartado"
