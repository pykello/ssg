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

## Cards

Use `:::card` to wrap Markdown in a card container:

```markdown
:::card[example]
**Example.** Card body.
:::
```

The optional bracket value is appended as a CSS class. For example,
`:::card[example]` generates a card with class `example`.

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
