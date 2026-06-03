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
$lim[x -> 0](f(x) + 1) != inf$
$sum[i=1..n](a_i) <= norm[2](v{x})$
$set(v{x} in bb{R}^n | norm(v{x}) <= 1)$
$abs(x) = cases(x | x >= 0; -x | x < 0)$
```

Generated LaTeX:

```tex
$\left\lVert \mathbf{x} - \mathbf{y} \right\rVert \le \epsilon$
$\lim_{x \to 0} \left(f\left(x\right) + 1\right) \ne \infty$
$\sum_{i=1}^{n} a_i \le \left\lVert \mathbf{x} \right\rVert_2$
$\left\{\mathbf{x} \in \mathbb{R}^n \;\middle|\; \left\lVert \mathbf{x} \right\rVert \le 1\right\}$
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
| `norm[2](x)` | `\left\lVert x \right\rVert_{2}` |
| `abs(x)` | `\left\lvert x \right\rvert` |
| `ip(x, y)` | `\left\langle x, y \right\rangle` |
| `dot(x, y)` | `x \cdot y` |
| `cross(x, y)` | `x \times y` |
| `dist(x, y)` | `d(x,y)` |
| `tuple(x_1, ..., x_n)` | `(x_1,\ldots,x_n)` |
| `seq(x_n)` | `\{x_n\}_{n \ge 1}` |
| `(x)` | `\left(x\right)` |
| `[x]` | `\left[x\right]` |
| `set(x | x > 0)` | `\left\{x \;\middle|\; x > 0\right\}` |
| `set(x : x > 0)` | `\left\{x \;\middle|\; x > 0\right\}` |
| `img(f, A)` | `f(A)` |
| `pre(f, B)` | `f^{-1}(B)` |
| `comp(A)` | `A^c` |
| `cl(A)` | `\overline{A}` |
| `interior(A)` | `A^\circ` |
| `bd(A)` | `\partial A` |
| `ball(x, r)` | `B_r(x)` |
| `closedball(x, r)` | `\overline{B}_r(x)` |
| `cases(x | x >= 0; -x | x < 0)` | `\begin{cases}x & x \ge 0 \\ -x & x < 0\end{cases}` |
| `lim[x -> 0]` | `\lim_{x \to 0}` |
| `lim[x -> 0](f(x))` | `\lim_{x \to 0} f(x)` |
| `sum[i=1..n](a_i)` | `\sum_{i=1}^{n} a_i` |
| `prod[i=1..n](a_i)` | `\prod_{i=1}^{n} a_i` |
| `sup[x in A](f(x))` | `\sup_{x \in A} f(x)` |
| `inf[x in A](f(x))` | `\inf_{x \in A} f(x)` |
| `min[x in A](f(x))` | `\min_{x \in A} f(x)` |
| `max[x in A](f(x))` | `\max_{x \in A} f(x)` |
| `union[a in A](X_a)` | `\bigcup_{a \in A} X_a` |
| `inter[a in A](X_a)` | `\bigcap_{a \in A} X_a` |
| `int[a..b](f(x), x)` | `\int_a^b f(x)\,dx` |
| `int[D](f, A)` | `\int_D f\,dA` |
| `iint[D](f(x,y), x, y)` | `\iint_D f(x,y)\,dx\,dy` |
| `iiint[E](f(x,y,z), x, y, z)` | `\iiint_E f(x,y,z)\,dx\,dy\,dz` |
| `oint[C](F dot t, s)` | `\oint_C F \cdot t\,ds` |
| `dd(f, x)` | `\frac{d f}{d x}` |
| `dd[n](f, x)` | `\frac{d^n f}{d x^n}` |
| `pd(f, x)` | `\frac{\partial f}{\partial x}` |
| `pd[n](f, x)` | `\frac{\partial^n f}{\partial x^n}` |
| `pd2(f, x, y)` | `\frac{\partial^2 f}{\partial x \partial y}` |
| `grad(f)` | `\nabla f` |
| `div(F)` | `\operatorname{div} F` |
| `curl(F)` | `\operatorname{curl} F` |
| `jac(f)` | `J_f` |
| `hess(f)` | `H_f` |
| `mat(a,b; c,d)` | `\begin{bmatrix}a & b \\ c & d\end{bmatrix}` |
| `pmat(a,b; c,d)` | `\begin{pmatrix}a & b \\ c & d\end{pmatrix}` |
| `detmat(a,b; c,d)` | `\begin{vmatrix}a & b \\ c & d\end{vmatrix}` |
| `wedge(dx, dy)` | `dx \wedge dy` |
| `ext(omega)` | `d\omega` |
| `pull(T, omega)` | `T^*\omega` |
| `form(F)` | `\lambda_F` |
| `boundary(Phi)` | `\partial \Phi` |
| `eps`, `del`, `inf` | `\epsilon`, `\delta`, `\infty` |
| `eps_0`, `del_a`, `inf_n` | `\epsilon_0`, `\delta_a`, `\infty_n` |
| `in`, `notin` | `\in`, `\notin` |
| `subset`, `supset`, `subseteq`, `supseteq` | `\subset`, `\supset`, `\subseteq`, `\supseteq` |
| `union`, `inter` | `\cup`, `\cap` |
| `alpha`, `beta`, `gamma`, `theta`, `lambda`, `omega`, `Phi`, ... | Greek letters |
| `=>`, `<=>`, `->`, `!=`, `<=`, `>=`, `...` | implication, equivalence, arrow, not equal, less/greater-or-equal, ellipsis |

Plain square brackets are not expanded after LaTeX commands or line breaks, so
forms like `\sqrt[n]` and `\\[1em]` are preserved. Mixed interval delimiters
like `[0, 1)` and `(0, 1]` are also left unchanged.

Text-like LaTeX commands such as `\text{...}`, `\mathrm{...}`, and
`\operatorname{...}` are preserved literally, so ordinary words inside them are
not treated as shorthand.

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
auto alignment, or `:::math align` to force an aligned block. Add `tag=...` on
the opening line, or `#tag ...` inside a row, to emit `\tag{...}`.

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

Use `:::math system` for systems of equations:

```text
:::math system tag=2.1
2x + 3y = 1
x - y = 0
:::
```

Generated LaTeX:

```tex
$$
\left\{\begin{aligned}
2x + 3y &= 1 \\
x - y &= 0 \tag{2.1}
\end{aligned}\right.
$$
```

Use `:::math matrix` for matrices:

```text
:::math matrix
a, b
c, d #tag A
:::
```

Generated LaTeX:

```tex
$$
\begin{bmatrix}
a & b \\
c & d
\end{bmatrix} \tag{A}
$$
```
