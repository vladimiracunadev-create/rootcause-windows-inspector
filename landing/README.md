# RootCause — Landing Page

Landing page oficial del producto **RootCause Windows Inspector**.

---

## Cómo publicarla en GitHub Pages (repo público separado)

### Paso 1 — Crear el repo público

En GitHub, crea un nuevo repositorio **público**:

```
Nombre sugerido: rootcause-landing
Visibilidad:     Public
```

### Paso 2 — Copiar archivos al nuevo repo

Copia el **contenido de esta carpeta** (`landing/`) a la raíz del nuevo repo:

```
rootcause-landing/
├── index.html                        ← landing page
├── README.md                         ← este archivo
└── .github/
    └── workflows/
        └── deploy.yml                ← auto-deploy en cada push
```

> El código fuente del producto se queda en el repo privado.
> Solo `index.html` es público.

### Paso 3 — Activar GitHub Pages

En el repo público:

```
Settings → Pages → Source → GitHub Actions
```

### Paso 4 — Hacer push

```bash
git push origin main
```

El workflow `deploy.yml` se ejecuta y publica la página automáticamente.

### URL resultante

```
https://<tu-usuario>.github.io/rootcause-landing/
```

O si usas el repo especial `<usuario>.github.io`:

```
https://<tu-usuario>.github.io/
```

---

## Dominio personalizado (opcional)

Si tienes un dominio propio:

1. En el repo público → Settings → Pages → Custom domain → escribe tu dominio
2. Agrega un registro `CNAME` en tu DNS apuntando a `<usuario>.github.io`
3. Activa "Enforce HTTPS"

---

## Actualizar la versión en la landing

Busca y reemplaza en `index.html`:

```
v0.6.0  →  v0.7.0   (o la versión que corresponda)
```

Luego `git push` y GitHub Actions despliega automáticamente.

---

## Bloque publico de seguridad y resiliencia

La landing debe mantener un bloque breve y estrategico sobre dos lineas documentadas del producto:

- deteccion de actividad anomala compatible con problemas de seguridad;
- resiliencia del propio agente ante detencion, manipulacion o corrupcion.

Reglas de redaccion para ese bloque:

- no publicar heuristicas internas detalladas ni material que facilite evasion;
- no vender RootCause como antivirus, proteccion total o EDR completo;
- aclarar que se trata de evolucion del producto y no de una promesa de deteccion perfecta;
- mantener el tono serio, tecnico y honesto.
