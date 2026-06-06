# Authoring Shortcuts

These shortcuts are available in Markdown content without enabling
`math_shorthand`.

## Includes

Use `#include "file.md"` to insert another Markdown file before rendering:

```markdown
#include "proof.md"
```

Include paths are resolved relative to the current Markdown file. Absolute paths
and parent-directory traversal are rejected. Includes are not expanded inside
code fences and are not processed recursively.

## Alerts

GitHub-style Markdown alerts are enabled:

```markdown
> [!NOTE]
> This is a note.
```

The supported alert syntax is provided by the Markdown renderer.

## Expandable Blocks

Use `:::expandable` for collapsible sections:

```markdown
:::expandable
**Proof.** [Click to Expand]

Proof text.
:::
```

The second line is the visible heading. Text inside square brackets on that line
becomes the collapse toggle link.

Use `:::proof` for the common collapsible proof form:

```markdown
:::proof
Proof text.
:::
```

This is equivalent to an expandable block titled `Proof.` with a `Click to
Expand` toggle. Use a bracketed title for variants:

```markdown
:::proof[Proof of Lemma]
Proof text.
:::
```

## Cards

Use `:::card` to wrap Markdown in a card container:

```markdown
:::card[example]
**Example.** Card body.
:::
```

The optional bracket value is appended as a CSS class. For example,
`:::card[example]` generates a card with class `example`.

Use semantic card aliases for common note-taking blocks:

```markdown
:::aside
Side note.
:::

:::remark
Remark text.
:::
```

These generate card containers with `aside` or `remark` classes.

## Figures

Use `:::figure` for centered figures. For images, put the source on the opening
line and optional metadata in the body:

```markdown
:::figure diagram.png
alt: Force diagram
caption: Geometry of the surface patch.
width: 360
:::
```

For JavaScript-rendered figures, use an `id` instead of an image source:

```markdown
:::figure id=fig12 width=360 ratio=1/1
:::
```

The generated container uses the same centered sizing style as the handwritten
HTML used by the notes. Add `class=...` when a JavaScript library expects one:

```markdown
:::figure id=board class=jxgbox width=480
:::
```

## GeomDSL Figures

Use `:::geomdsl` to render a GeomDSL diagram into a generated image:

```markdown
:::geomdsl width=620 alt="Perpendicular bisector"
scene(min=(-2,-2), max=(2,2), grid=false, axes=false)

include "diagram-styles.geom"

A = pt(-1, 0)
B = pt(1, 0)
draw LineSegment(A, B)
draw marker(A)
draw marker(B)
:::
```

The block is rendered through `geomdsl` from `~/projects/geomdsl2` by default.
Set `geomdsl_dir`, `geomdsl_python`, `geomdsl_timeout_seconds`, or
`geomdsl_dpi` in the site config to override the runner. GeomDSL `include`
paths are resolved relative to the Markdown file that contains the block.

The opening line accepts `format=svg` or `format=png`, `dpi=...`, `width=...`,
`alt="..."`, `caption="..."`, `class=...`, and `id=...`. Generated assets are
written under `build/static/assets/<content path>/.geomdsl/`.

## LaTeX Environments

Configured theorem-like LaTeX environments are converted during LaTeX rendering:

```yaml
theorems:
  - name: theorem
    label: Theorem
    numbered: true
```

```tex
\begin{theorem}\label{thm:one}
Statement.
\end{theorem}

See \ref{thm:one}.
```

Numbered theorem references are converted to links. Equation labels are preserved
so MathJax can handle equation references.
