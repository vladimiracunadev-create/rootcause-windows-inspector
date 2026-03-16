# Marca, naming y branding técnico de RootCause

> Documento maestro para llevar **RootCause** desde nombre de trabajo de repositorio a **marca de producto** implementada de manera legal, visual y técnica.

---

## 1. Objetivo de este documento

Este documento existe para resolver cuatro necesidades distintas, pero relacionadas:

1. **Definir el nombre actual del producto** y cómo debe usarse dentro del repositorio.
2. **Explicar cómo registrar la marca en Chile** paso a paso con enlaces oficiales, costos, plazos y decisiones previas.
3. **Documentar cómo implementar técnicamente la marca** en el software, el ejecutable, el instalador, los accesos directos y el entorno Windows 11.
4. **Dejar una ruta escalable** para que la marca no quede improvisada si el proyecto crece a SaaS, web, telemetría administrada o versión corporativa.

Este documento no reemplaza asesoría legal especializada. Sí deja una guía profesional, concreta y operable.

---

## 2. Decisión actual de naming

### Nombre seleccionado por ahora

**RootCause**

### Razón de la decisión

Se adopta **RootCause** como nombre actual del producto porque comunica la idea principal del software:

- detectar la **causa raíz** del problema,
- evitar el enfoque de “limpiar por limpiar”,
- diferenciarse de herramientas genéricas de “booster” o “optimizer”,
- sonar más profesional ante reclutadores, desarrolladores y usuarios técnicos.

### Cómo debe usarse dentro del repositorio

| Elemento | Valor actual |
|---|---|
| Nombre del producto | `RootCause` |
| Nombre del repositorio | `rootcause-windows-inspector` |
| Ejecutable esperado | `rootcause.exe` |
| Instalador Inno Setup | `RootCause.iss` |
| Icono principal | `RC` |
| Título largo recomendado | `RootCause — Windows Performance Inspector` |

### Regla de consistencia

Mientras no se tome una decisión comercial distinta, el proyecto debe usar **solo RootCause** en:

- interfaz,
- README,
- documentación,
- script de build,
- empaquetado,
- instalador,
- nombre del ejecutable,
- accesos directos,
- material de presentación.

No mezclar con variantes como:

- RootCause One
- RootCause Cleaner
- RootCause Boost
- RC Fix

salvo que exista una decisión de branding posterior documentada.

---

## 3. Diferencia correcta entre patente, marca y branding

### 3.1. Patente

Una **patente** protege una invención o solución técnica nueva.

### 3.2. Marca

Una **marca** protege el nombre, signo, logo o combinación que identifica comercialmente un producto o servicio.

### 3.3. Branding

El **branding** es la implementación práctica y visual de la identidad:

- nombre,
- tono,
- icono,
- colores,
- títulos,
- instalador,
- accesos directos,
- sitio web,
- material comercial.

### Conclusión práctica para este proyecto

Para **RootCause**, lo que corresponde proteger primero es la **marca**, no una patente.

---

## 4. Ruta legal recomendada para RootCause en Chile

### 4.1. Sitios oficiales que debes usar

#### INAPI
- Portal principal: https://www.inapi.cl/
- Buscadores de marcas: https://www.inapi.cl/marcas/buscadores
- Buscador directo de marcas: https://buscadormarcas.inapi.cl/
- Trámites y pago en línea: https://tramites.inapi.cl/
- Preguntas frecuentes de marcas: https://www.inapi.cl/preguntas-frecuentes/marcas
- Información general de marcas: https://www.inapi.cl/marcas/para-informarse

#### Acceso ciudadano
- Clave Única: https://claveunica.gob.cl/

#### Diario Oficial
- Tarifas: https://www.diariooficial.interior.gob.cl/tarifas/

#### Valor UTM
- UTM oficial: https://www.sii.cl/valores_y_fechas/utm/utm2026.htm

---

## 5. Paso a paso legal y administrativo para registrar RootCause

### Paso 1. Confirmar que el nombre se quiere proteger de verdad

Antes de iniciar el trámite, debes responder por escrito estas preguntas:

- ¿El nombre final será `RootCause` exactamente?
- ¿Será una marca denominativa solo de texto o además una marca mixta con ícono?
- ¿Se registrará para software descargable, servicio en línea o ambos?
- ¿La titularidad será persona natural o empresa?
- ¿La marca se usará en Chile solamente al inicio?

**Recomendación inicial para este repositorio:**

- comenzar con una **marca denominativa**: `RootCause`
- luego evaluar una segunda solicitud para versión mixta o logo si el producto madura

La lógica es simple: proteger primero el nombre puro suele ser más flexible que amarrarse desde el inicio a una versión gráfica específica.

---

### Paso 2. Hacer búsqueda previa en la base oficial de INAPI

Ir a:

- https://buscadormarcas.inapi.cl/

Buscar al menos:

- `RootCause`
- `Root Cause`
- `Root-Cause`
- `RootCause Inspector`
- `RC RootCause`
- palabras cercanas o variantes fonéticas

#### Qué debes revisar

- marcas idénticas,
- marcas visual o fonéticamente parecidas,
- cobertura de productos/servicios,
- estado de la marca (solicitada, registrada, abandonada, rechazada, etc.).

#### Qué decisión documentar

Guardar una nota simple en `docs/` o en tus archivos personales con:

- fecha de búsqueda,
- términos usados,
- capturas o hallazgos relevantes,
- si existe riesgo alto, medio o bajo.

---

### Paso 3. Definir la cobertura correcta

No basta con proteger el nombre. Debes definir **para qué** se protege.

Para eso debes usar el **Clasificador oficial de productos y servicios** de INAPI:

- https://tramites.inapi.cl/Trademark/TrademarkNizaClassifier

#### Ruta práctica recomendada para este proyecto

Como punto de partida razonable:

- una clase para **software descargable**,
- otra clase adicional solo si el proyecto luego evoluciona a **servicio en línea / SaaS / monitoreo administrado**.

**Importante:** la redacción exacta debe salir del clasificador oficial, no inventarse a mano si quieres minimizar observaciones.

---

### Paso 4. Definir el tipo de marca a solicitar

Las dos rutas más comunes para este proyecto son:

#### Opción A. Marca denominativa
Protege la palabra `RootCause`.

**Ventaja:**
- más flexible si el icono cambia.

#### Opción B. Marca mixta
Protege palabra + diseño gráfico.

**Ventaja:**
- amarra una identidad visual concreta.

**Recomendación para este proyecto:**

1. Primero registrar **RootCause** como denominativa.
2. Si el producto madura, evaluar luego una segunda protección para la marca visual.

---

### Paso 5. Crear o preparar acceso a INAPI

Ir a:

- https://tramites.inapi.cl/

El portal indica que puedes ingresar con:

- **Clave Única**, o
- una **Clave INAPI** previamente registrada.

Si es primera vez, la ruta recomendada es ingresar con **Clave Única** y completar el registro base.

---

### Paso 6. Presentar solicitud y pagar el primer tramo

Una solicitud nueva de marca en Chile se paga en dos etapas.

#### Costo oficial base por clase

- pago inicial: **1 UTM**
- pago final: **2 UTM**
- total: **3 UTM por cada clase**

### Valor UTM oficial consultado para marzo de 2026

- **1 UTM = $69.889 CLP**

### Conversión práctica

| Concepto | UTM | Monto aproximado marzo 2026 |
|---|---:|---:|
| Pago inicial por clase | 1 | $69.889 |
| Pago final por clase | 2 | $139.778 |
| Total por clase | 3 | $209.667 |
| Total por 2 clases | 6 | $419.334 |

### Importante

Si la marca no llega a registro, **el pago inicial no se devuelve**.

---

### Paso 7. Publicar el extracto en Diario Oficial

Una vez aceptada la solicitud a trámite, debes publicar el extracto.

#### Plazo legal clave

- **20 días hábiles** para requerir y pagar la publicación desde la aceptación a trámite.

Si no lo haces a tiempo, la solicitud puede quedar abandonada.

#### Sitio oficial

- https://www.diariooficial.interior.gob.cl/tarifas/

#### Qué debes saber del costo

El costo del Diario Oficial:

- **no está fijo por marca**,
- depende de la extensión,
- se cobra proporcionalmente según página,
- y el propio Diario Oficial publica sus tarifas vigentes.

#### Qué presupuesto usar de forma profesional

En el presupuesto del proyecto debes separar:

- **costos oficiales INAPI**,
- **costo de publicación en Diario Oficial**,
- **costos opcionales** (abogado, diseñador, branding, dominio, etc.).

---

### Paso 8. Esperar posible oposición de terceros

Tras la publicación, se abre el período de oposición.

#### Plazo legal

- **30 días hábiles** desde la publicación.

Si aparece oposición:

- el caso deja de ser solo administrativo,
- pasa a una fase contenciosa,
- y para responder formalmente debes contar con abogado habilitado.

---

### Paso 9. Examen de fondo de INAPI

Pasado el período de oposición, INAPI revisa:

- semejanzas con marcas previas,
- identidad o parecido con signos ya solicitados o registrados,
- causales de irregistrabilidad,
- distintividad del signo.

#### Si hay observaciones de fondo

Tendrás normalmente:

- **30 días** para responder.

#### Si la solicitud es aceptada

Tendrás:

- **60 días hábiles** para pagar el tramo final.

---

### Paso 10. Pagar el tramo final y cerrar el registro

Si INAPI acepta a registro la marca, debes pagar:

- **2 UTM por clase**

Y acreditar el pago dentro del plazo.

Si no pagas a tiempo, la solicitud puede perderse.

---

### Paso 11. Vigencia y renovación

La marca registrada tiene una duración de:

- **10 años** desde la fecha de registro.

Luego puede renovarse por períodos iguales.

#### Renovación

La documentación oficial indica que puede pedirse:

- durante los **6 meses anteriores** al vencimiento,
- o dentro de los **6 meses siguientes** a la expiración, con sobretasa.

---

## 6. Tiempos de espera: cómo entenderlos bien

### 6.1. Lo que sí está claramente definido por la normativa operativa

Estos plazos sí debes controlar porque dependen de ti o del procedimiento:

- subsanar observaciones de forma: **30 días**
- publicar extracto tras aceptación a trámite: **20 días hábiles**
- oposición de terceros: **30 días hábiles** desde la publicación
- responder objeciones de fondo: **30 días**
- pagar el tramo final tras aceptación: **60 días hábiles**

### 6.2. Lo que NO debes asumir mal

No conviene prometer un tiempo total fijo tipo “30 días y listo”.

La duración completa depende de:

- si hubo observaciones de forma,
- si hubo oposición,
- si el signo fue considerado distintivo o no,
- si la cobertura quedó bien redactada,
- carga interna del sistema,
- tiempos efectivos de resolución administrativa.

### Regla profesional

En tus documentos internos, tratar la tramitación como:

- **plazos legales controlables**, y
- **duración total variable**.

---

## 7. Costeo profesional recomendado para RootCause

### 7.1. Costo oficial mínimo para arrancar

Si partes con una sola clase y sin oposición:

- 3 UTM por clase = **$209.667 CLP** a valores de marzo 2026
- más publicación en Diario Oficial según tarifa vigente

### 7.2. Presupuesto prudente mínimo

#### Escenario 1: una clase
- INAPI: $209.667
- Diario Oficial: variable
- Total: **base oficial + publicación**

#### Escenario 2: dos clases
- INAPI: $419.334
- Diario Oficial: variable
- Total: **base oficial por 2 clases + publicación**

### 7.3. Costos no oficiales, pero que debes contemplar internamente

INAPI no fija estos montos. Debes presupuestarlos aparte si los necesitas:

- búsqueda legal especializada,
- abogado para oposición,
- diseñador de marca,
- dominio web,
- landing page,
- firma digital de binarios,
- certificación, tienda o distribución.

---

## 8. Implementación técnica de la marca dentro del software

Esta parte ya debe quedar reflejada en el repositorio.

### 8.1. Nombre visible en interfaz

El software debe mostrar:

- `RootCause` en el título de la ventana,
- `RootCause` en el encabezado principal,
- un distintivo visual simple `RC`.

### 8.2. Icono mínimo del producto

Se define un ícono genérico y neutral con las letras:

- `RC`

#### Archivos esperados

- `assets/rootcause.ico`
- `assets/rootcause-icon-256.png`
- `assets/rootcause-icon.svg`

### 8.3. Integración del icono en Windows

La implementación técnica recomendada es:

1. **recurso del ejecutable** para que el `.exe` lleve la identidad visual,
2. **ícono del instalador**,
3. **ícono de accesos directos**,
4. nombre consistente en menú Inicio y escritorio.

### 8.4. build.rs

La incrustación de icono y metadatos de producto en Windows debe hacerse con un `build.rs`.

Objetivo:

- que `rootcause.exe` muestre el nombre correcto,
- que el ejecutable tenga ícono propio,
- que el acceso directo no dependa solo del icono genérico de Rust.

### 8.5. Metadatos mínimos recomendados del ejecutable

- ProductName = RootCause
- FileDescription = RootCause - Windows Performance Inspector
- OriginalFilename = rootcause.exe
- CompanyName = Vladimir Acuña Dev

---

## 9. Implementación en instalador y accesos directos

### 9.1. Instalador Inno Setup

El script `packaging/windows/RootCause.iss` debe:

- usar `RootCause` como AppName,
- usar `rootcause.ico` como icono del instalador,
- crear acceso directo en escritorio,
- crear acceso en grupo del menú Inicio,
- apuntar ambos al ejecutable correcto,
- usar el icono de marca en esos accesos.

### 9.2. Resultado esperado tras instalar

El usuario debería ver:

- carpeta del producto: `RootCause`
- acceso directo en escritorio: `RootCause`
- acceso en el menú Inicio: `RootCause`
- ejecutable principal: `rootcause.exe`

---

## 10. Anclaje en Windows 11: qué sí y qué no

### 10.1. Lo que sí se puede hacer de forma razonable

- crear acceso directo en escritorio,
- crear acceso en menú Inicio,
- ejecutar la app para que el usuario la vea en la barra de tareas,
- dejar el ícono correcto para que al fijarla manualmente se vea bien.

### 10.2. Lo que no conviene prometer como si fuera automático

En Windows 11, el anclaje a:

- **Taskbar**
- **Start pinned**

normalmente debe hacerlo el usuario, o bien administrarse por políticas/OEM en escenarios controlados.

### Regla profesional para este proyecto

- el instalador crea accesos directos,
- la documentación enseña al usuario cómo fijarla,
- no se promete “anclaje forzado universal” en un instalador normal de usuario.

### 10.3. Cómo debe explicarse al usuario final

#### Anclar a la barra de tareas
1. Abrir `RootCause`.
2. Hacer clic derecho sobre el icono activo en la barra de tareas.
3. Elegir **Pin to taskbar / Anclar a la barra de tareas**.

#### Anclar al menú Inicio
1. Buscar `RootCause` en Inicio.
2. Clic derecho.
3. Elegir **Pin to Start / Anclar a Inicio**.

### 10.4. Escenario corporativo o de despliegue masivo

Si el software se desplegara en flotas corporativas o imagen OEM:

- existen rutas administradas por políticas para taskbar,
- y customización de Start con `LayoutModification.json`.

Eso es una capa posterior y no debe mezclarse con el instalador estándar de usuario.

---

## 11. Ruta de branding visual mínima recomendada

No sobrediseñar al principio.

### Debe existir ya
- nombre: `RootCause`
- monograma: `RC`
- color principal: azul técnico
- uso del nombre en ventana, README, instalador y accesos directos

### Puede venir después
- logo final,
- sistema de color definitivo,
- guía de estilo,
- landing page,
- screenshots oficiales,
- branding para LinkedIn/GitHub/web.

---

## 12. Ruta paso a paso para implementación técnica en este repositorio

### Paso técnico 1. Confirmar el nombre interno
- `RootCause`

### Paso técnico 2. Confirmar nombres de archivos
- `rootcause.exe`
- `RootCause.iss`
- `assets/rootcause.ico`

### Paso técnico 3. Confirmar ventana y UI
- título de ventana = `RootCause`
- encabezado principal = `RootCause`
- badge visual = `RC`

### Paso técnico 4. Confirmar build.rs
- recurso Windows con icono y metadatos

### Paso técnico 5. Confirmar instalador
- acceso directo escritorio con nombre `RootCause`
- acceso menú inicio con nombre `RootCause`
- ícono `RC`

### Paso técnico 6. Confirmar documentación
Este documento debe estar linkeado desde:

- `README.md`
- `docs/INDEX.md`
- opcionalmente `docs/RECLUTADORES.md`

### Paso técnico 7. Confirmar pruebas manuales
En Windows real, revisar:

- nombre de la ventana,
- icono del `.exe`,
- icono del acceso directo,
- nombre en menú Inicio,
- anclaje manual correcto a taskbar,
- anclaje manual correcto a Start,
- ícono visible en búsqueda de Windows.

---

## 13. Ruta de crecimiento de marca si el proyecto escala

### Etapa 1. Producto local de escritorio
Proteger el nombre `RootCause`.

### Etapa 2. Distribución pública
Agregar:

- logo final,
- dominio,
- landing page,
- screenshots,
- release firmado.

### Etapa 3. Plataforma o SaaS
Evaluar protección adicional para:

- clases adicionales,
- branding de servicio web,
- panel cloud,
- agentes livianos,
- analítica remota.

### Etapa 4. Portafolio de productos
Definir familia de nombres, por ejemplo:

- RootCause Desktop
- RootCause Precision
- RootCause Insights
- RootCause Enterprise

**No hacer esto ahora** si todavía no existe una línea real de producto.

---

## 14. Checklist ejecutivo

### Legal
- [ ] Búsqueda previa en INAPI
- [ ] Definición de cobertura con clasificador oficial
- [ ] Decisión entre marca denominativa o mixta
- [ ] Ingreso con Clave Única / cuenta INAPI
- [ ] Solicitud presentada
- [ ] Pago inicial realizado
- [ ] Publicación en Diario Oficial realizada
- [ ] Seguimiento de oposiciones y observaciones
- [ ] Pago final acreditado
- [ ] Registro archivado y fecha de renovación anotada

### Técnica
- [ ] Nombre `RootCause` visible en UI
- [ ] Icono `RC` presente en assets
- [ ] build.rs incrusta icono en Windows
- [ ] instalador usa icono correcto
- [ ] escritorio crea acceso `RootCause`
- [ ] menú Inicio crea acceso `RootCause`
- [ ] prueba manual de anclaje completada en Windows 11

### Documental
- [ ] README actualizado
- [ ] índice documental actualizado
- [ ] branding documentado
- [ ] ruta legal documentada
- [ ] costos y plazos anotados con fecha

---

## 15. Resumen final recomendado

### Qué hacer ahora

1. Mantener el nombre **RootCause** dentro del repositorio.
2. Usar este documento como base de trabajo.
3. Hacer búsqueda oficial en INAPI antes de cualquier publicación pública fuerte.
4. Si la búsqueda es razonablemente limpia, preparar solicitud denominativa.
5. Mientras tanto, usar branding técnico mínimo:
   - nombre RootCause,
   - icono RC,
   - atajos RootCause,
   - documentación uniforme.

### Qué no hacer todavía

- prometer marca registrada antes de presentar solicitud,
- mezclar nombres alternativos en el software,
- vender la app con branding distinto al repositorio,
- asumir que un instalador normal puede fijar universalmente la app a la barra de tareas,
- gastar en un logo final complejo antes de validar el nombre.

---

## 16. Fuentes oficiales a revisar periódicamente

- INAPI marcas: https://www.inapi.cl/preguntas-frecuentes/marcas
- INAPI información de marcas: https://www.inapi.cl/marcas/para-informarse
- INAPI buscadores: https://www.inapi.cl/marcas/buscadores
- Buscador oficial de marcas: https://buscadormarcas.inapi.cl/
- Trámites INAPI: https://tramites.inapi.cl/
- Diario Oficial tarifas: https://www.diariooficial.interior.gob.cl/tarifas/
- UTM oficial 2026: https://www.sii.cl/valores_y_fechas/utm/utm2026.htm
- Soporte Microsoft Start: https://support.microsoft.com/en-us/windows/customize-the-windows-start-menu-fde6f576-0fc0-0813-6b0d-d3ec1d244c50
- Soporte Microsoft Taskbar: https://support.microsoft.com/en-us/windows/customize-the-taskbar-in-windows-0657a50f-0cc7-dbfd-ae6b-05020b195b07
- Microsoft app icon guidance: https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction
- Microsoft taskbar policy guidance: https://learn.microsoft.com/en-us/windows/configuration/taskbar/

