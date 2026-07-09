# 🧒 Guía sencilla: cómo tomar cosas de la competencia (sin meter la pata)

> La comparativa completa y con datos está en [`COMPARATIVA_OSS.md`](COMPARATIVA_OSS.md).
> **Esta guía es la versión corta y para actuar.** Cada vez que quieras "hacer algo que
> hace otro software", sigue estos pasos. Explicado fácil, caso a caso.

---

## La regla de oro (una sola frase)

> **De la competencia se copian IDEAS, casi nunca CÓDIGO.**
> Y solo puedes copiar código si su licencia es **"permisiva" (verde)**.

---

## El semáforo de licencias 🚦

Piensa en la licencia como el **permiso** que te da el autor para usar su trabajo.

| Luz | Licencias | Qué puedes hacer |
|---|---|---|
| 🟢 **Verde** | MIT, Apache-2.0, BSD, DRL (reglas Sigma) | **Usar su código dentro de RootCause** sin problema |
| 🟡 **Amarillo** | MIT + Commons Clause | Usar la **idea**; el código solo si NO vendes el producto |
| 🔴 **Rojo** | GPL, AGPL | **NO metes su código**. Solo miras cómo lo hace y lo escribes tú a tu manera |
| ⚪ **Gris** | "gratis pero no open source" (ej. Sysmon) | No hay código para tomar; solo te **inspiras** en qué hace |

**Analogía de cocina:**
- 🟢 Verde = *"te presto mi receta y puedes usarla en tu restaurante, incluso cobrando."*
- 🔴 Rojo = *"si usas mi receta, TODO tu restaurante tiene que ser gratis y abierto."* → no te conviene, así que **solo la miras y cocinas la tuya**.

> Truco para no perderte: si ves **`MIT OR Apache-2.0`**, significa "elige la que quieras";
> elige **Apache-2.0** (la de RootCause) y listo. Si ves **GPL** o **AGPL**, luz roja.

---

## Los 5 pasos — cada vez, caso a caso

Antes de "copiar algo de la competencia", responde estas 5 preguntas:

1. **¿Qué idea quiero tomar?** Escríbela en una línea (una función, un formato, una técnica de detección).
2. **¿De dónde viene y qué licencia tiene?** Míralo en su README o en `Cargo.toml` (campo `license`). Aplica el semáforo 🚦.
3. **¿Integro o solo aprendo?** 🟢 verde → puedo usar su código. 🔴 rojo → solo miro y escribo el mío.
4. **¿Sigue siendo RootCause?** Revisa que NO rompa la identidad (ver checklist abajo). Si la rompe → no lo hago, o lo repienso.
5. **Papeleo mínimo:** si integré una crate verde, añado su aviso al archivo de terceros (con `cargo about` se genera solo). Si es una detección, la **etiqueto con su técnica MITRE ATT&CK**.

### Checklist de identidad (paso 4) — RootCause debe seguir siendo…
- ✅ **Ligero y local** (un binario, corre en tu PC).
- ✅ **Cero telemetría** (nada sale de tu equipo).
- ✅ **Forense / de diagnóstico** (avisa e explica; el usuario decide).
- ❌ NO convertirse en **antivirus/EDR** que actúa solo.
- ❌ NO volverse **agente + servidor** (nada de instalar un servidor central).
- ❌ NO instalar un **driver de kernel** (como Sysmon).

Si una idea te obliga a romper un ❌, **no encaja** — por muy buena que sea la idea.

---

## Tarjetas caso a caso (resumen de un vistazo)

**Librerías Rust (para tu código):**
| Herramienta | Semáforo | Qué tomar | Qué evitar |
|---|---|---|---|
| `windows` (windows-rs) | 🟢 integrar | APIs nativas (servicios, registro, red, sesiones) → jubila PowerShell/netstat | nada; solo curva de aprendizaje |
| `evtx` | 🟢 integrar | leer Event Logs en Rust (sin `Get-WinEvent`) | — |
| `ferrisetw` | 🟢 integrar (v2.0) | ETW en vivo estilo Sysmon | bloquear v1.0 con ello (requiere admin) |
| `sysinfo` | 🟢 ya integrada | métricas (mantener) | usarla para lo que windows-rs hace mejor |
| motor Sigma en Rust | 🟢 (verificar cada crate) | reglas declarativas | motor pesado; crates aún jóvenes |

**Proyectos/competidores (solo ideas):**
| Proyecto | Semáforo | Qué tomar (idea) | Qué evitar |
|---|---|---|---|
| Sysmon | ⚪ no-OSS | su **mapa de eventos** de alto valor (qué vigilar) | instalar driver; solo emite, no analiza |
| osquery | 🟢/🔴 doble | la idea "SO como tablas locales" | su despliegue de flota; su rama GPL de código |
| Velociraptor | 🔴 AGPL | catálogo de artefactos forenses Windows | su arquitectura agente+servidor; su código |
| Wazuh / OSSEC | 🔴 GPL | FIM, drift de config, etiquetado ATT&CK | stack pesado agente+servidor; su código |
| BLUESPAWN | 🔴 GPL | el **etiquetado MITRE ATT&CK** de hallazgos | ser EDR que "actúa"; su código |
| PersistenceSniper | 🟡 MIT+CC | su **lista de ~60 técnicas** de persistencia | su código si vendes; su modelo one-shot |
| Hayabusa / Chainsaw | 🔴 AGPL/GPL | referencia de "motor Sigma sobre EVTX en Rust" | embeber su código |

---

## En una frase

> **¿Es una idea?** Tómala (respetando la identidad de RootCause).
> **¿Es código?** Solo si la luz es 🟢 verde.
> **¿Dudas?** Pregúntame caso por caso y te lo digo con el semáforo.

*(Esto es una explicación práctica, no asesoría legal. Para algo con implicaciones serias,
lo confirma un abogado — pero para usar crates permisivas en Rust y aprender de proyectos
GPL, así es como funciona en el día a día.)*
