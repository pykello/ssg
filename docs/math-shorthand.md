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
$set(v{x} in bb{R}^n | norm(v{x}) <= 1)$
$abs(x) = cases(x | x >= 0; -x | x < 0)$
```

Generated LaTeX:

```tex
$\left\lVert \mathbf{x} - \mathbf{y} \right\rVert \le \epsilon$
$\lim_{x \to 0} \left(f\left(x\right) + 1\right) \ne \infty$
$\left\{\mathbf{x} in \mathbb{R}^n \;\middle|\; \left\lVert \mathbf{x} \right\rVert \le 1\right\}$
$\left\lvert x \right\rvert = \begin{cases}x & x \ge 0 \\ -x & x < 0\end{cases}$
```

Supported compact forms:

| Shorthand | LaTeX |
| --- | --- |
| `v{x}` | `\mathbf{x}` |
| `bb{R}` | `\mathbb{R}` |
| `cal{F}` | `\mathcal{F}` |
| `hat{n}` | `\hat{n}` |
| `unit{n}` | `\hat{\mathbf{n}}` |
| `norm(x)` | `\left\lVert x \right\rVert` |
| `abs(x)` | `\left\lvert x \right\rvert` |
| `(x)` | `\left(x\right)` |
| `[x]` | `\left[x\right]` |
| `set(x | x > 0)` | `\left\{x \;\middle|\; x > 0\right\}` |
| `cases(x | x >= 0; -x | x < 0)` | `\begin{cases}x & x \ge 0 \\ -x & x < 0\end{cases}` |
| `lim[x -> 0]` | `\lim_{x \to 0}` |
| `eps`, `del`, `inf` | `\epsilon`, `\delta`, `\infty` |
| `eps_0`, `del_a`, `inf_n` | `\epsilon_0`, `\delta_a`, `\infty_n` |
| `=>`, `<=>`, `->`, `!=`, `<=`, `>=` | implication, equivalence, arrow, not equal, less/greater-or-equal |

Plain square brackets are not expanded after LaTeX commands or line breaks, so
forms like `\sqrt[n]` and `\\[1em]` are preserved. Mixed interval delimiters
like `[0, 1)` and `(0, 1]` are also left unchanged.

## Block Forms

Use `:::math` for display math:

```text
:::math
x^2 + y^2 = r^2
:::
```

Generated LaTeX:

```tex
$$
x^2 + y^2 = r^2
$$
```

When a `:::math` block has multiple rows with top-level relation operators,
rows are aligned automatically:

```text
:::math
v{x} = v{y}
=> norm(v{x}) <= eps
:::
```

Generated LaTeX:

```tex
$$
\begin{aligned}
\mathbf{x} &= \mathbf{y} \\
&\implies \left\lVert \mathbf{x} \right\rVert \le \epsilon
\end{aligned}
$$
```

Explicit `&` alignment markers are preserved. Use `:::math plain` to disable
auto alignment, or `:::math align` to force an aligned block.

Use `cases:` inside `:::math` for piecewise definitions:

```text
:::math
f(x) = cases:
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
