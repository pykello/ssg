# Math Shorthand

Set both options below to use shorthand in Markdown math:

```yaml
escape_markdown_in_math: false
math_shorthand: true
```

Shorthand is expanded only inside `$...$`, `$$...$$`, and the shorthand math blocks
below. Existing LaTeX remains valid.

## Inline Forms

```text
$norm(v{x} - v{y}) <= eps$
$lim[x -> 0] (f(x) + 1) != inf$
```

Generated LaTeX:

```tex
$\left\lVert \mathbf{x} - \mathbf{y} \right\rVert \le \epsilon$
$\lim_{x \to 0} \left(f\left(x\right) + 1\right) \ne \infty$
```

Supported compact forms:

| Shorthand | LaTeX |
| --- | --- |
| `v{x}` | `\mathbf{x}` |
| `bb{R}` | `\mathbb{R}` |
| `cal{F}` | `\mathcal{F}` |
| `hat{n}` | `\hat{n}` |
| `norm(x)` | `\left\lVert x \right\rVert` |
| `abs(x)` | `\left\lvert x \right\rvert` |
| `(x)` | `\left(x\right)` |
| `lim[x -> 0]` | `\lim_{x \to 0}` |
| `eps`, `del`, `inf` | `\epsilon`, `\delta`, `\infty` |
| `=>`, `<=>`, `->`, `!=`, `<=`, `>=` | implication, equivalence, arrow, not equal, less/greater-or-equal |

## Block Forms

Use `:::align` for aligned derivations:

```text
:::align
v{x} &= v{y}
&=> norm(v{x}) <= eps
:::
```

Generated LaTeX:

```tex
$$
\begin{align*}
\mathbf{x} &= \mathbf{y} \\
&\implies \left\lVert \mathbf{x} \right\rVert \le \epsilon
\end{align*}
$$
```

Use `:::cases` for piecewise definitions:

```text
:::cases f(x)
1 | x != 0
0 | x = 0
:::
```

Generated LaTeX:

```tex
$$
f(x) = \begin{cases}
1 & x \ne 0 \\
0 & x = 0
\end{cases}
$$
```
