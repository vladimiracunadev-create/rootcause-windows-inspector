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
