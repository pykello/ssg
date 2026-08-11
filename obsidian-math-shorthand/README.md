# Obsidian Math Shorthand

Render the SSG math shorthand language in Obsidian without changing the Markdown stored in your vault. The plugin supports inline and display math in Live Preview and Reading View and uses Obsidian's bundled MathJax renderer.

## Installation

### Build and install manually

1. Install [Node.js](https://nodejs.org/) 18 or newer.
2. From this directory, install dependencies and build the plugin:

   ```sh
   npm install
   npm run build
   ```

3. Create a directory named `math-shorthand` inside your vault's plugin directory:

   ```text
   <your-vault>/.obsidian/plugins/math-shorthand/
   ```

4. Copy these files into that directory:

   ```text
   main.js
   manifest.json
   ```

5. Restart Obsidian or choose **Settings → Community plugins → Reload plugins**.
6. In **Settings → Community plugins**, enable **Math Shorthand**.

> Obsidian may require you to turn off Restricted Mode before locally installed community plugins can be enabled.

When upgrading a manual installation, rebuild the plugin, replace both `main.js` and `manifest.json` in the vault plugin directory, and reload Obsidian. Merely rebuilding this source directory does not update the copy installed in a vault.

If Obsidian shows **Failed to open** after an earlier plugin version was installed, replace both files with version 0.1.3 or newer and restart Obsidian. If necessary, temporarily disable or remove `.obsidian/plugins/math-shorthand` before reopening the vault, then reinstall the new build.

## Using the plugin

Write shorthand inside ordinary Markdown math delimiters. For example:

```md
$norm(v{x}) <= eps$
$x in bb{R}^n$
$sum[i=1..n](a_i)$
```

Obsidian renders those expressions as if they contained:

```tex
\left\lVert \mathbf{x} \right\rVert \le \epsilon
x \in \mathbb{R}^n
\sum_{i=1}^{n} a_i
```

Display math works in the same way:

```md
$$
set(v{x} in bb{R}^n | norm(v{x}) <= 1)
$$
```

The source file continues to contain the shorthand. Expansion is virtual: the plugin does not replace or save the expanded LaTeX.

### Editing in Live Preview

When the cursor is outside a shorthand math expression, the plugin displays the rendered result. Move the cursor into the expression, or make a selection that overlaps it, to reveal and edit the original `$...$` or `$$...$$` source. Move the cursor away to render it again.

### Existing LaTeX

Shorthand and existing LaTeX can be mixed:

```md
$\gamma + norm(v{x})$
```

Text-like LaTeX commands are protected, so words inside commands such as `\text{...}`, `\mathrm{...}`, and `\operatorname{...}` are not expanded:

```md
$\text{alpha in beta}$
```

### Enable or disable shorthand

The plugin is enabled globally by default. Change it with **Settings → Math Shorthand → Enable math shorthand**.

A note can override the global setting with an SSG-compatible HTML comment:

```md
<!-- ssg-math-shorthand: off -->
```

To explicitly enable it for a note:

```md
<!-- ssg-math-shorthand: on -->
```

If a note contains more than one directive, the last valid directive wins. The aliases `true`, `yes`, `false`, and `no` are also accepted.

## Common shorthand

| Shorthand | Rendered LaTeX |
| --- | --- |
| `v{x}` | `\mathbf{x}` |
| `bb{R}` | `\mathbb{R}` |
| `cal{F}` | `\mathcal{F}` |
| `norm(x)` | `\left\lVert x \right\rVert` |
| `abs(x)` | `\left\lvert x \right\rvert` |
| `x in A` | `x \in A` |
| `a <= b` | `a \le b` |
| `lim[x -> 0](f(x))` | `\lim_{x \to 0} f(x)` |
| `sum[i=1..n](a_i)` | `\sum_{i=1}^{n} a_i` |
| `int[a..b](f(x), x)` | `\int_a^b f(x)\,dx` |
| `set(x in A \| P(x))` | `\left\{x \in A \;\middle\|\; P(x)\right\}` |

Shorthand calls can be nested, for example:

```md
$norm(v{x} - v{y}) <= eps$
$set(v{x} in bb{R}^n | norm(v{x}) <= 1)$
```

See [`../docs/math-shorthand.md`](../docs/math-shorthand.md) for the SSG shorthand language reference.

## Current limitations

- Only `$...$` and `$$...$$` math are supported in this MVP.
- SSG `:::math` blocks are not supported yet.
- The plugin does not provide an automatic command that converts shorthand into saved LaTeX.
- Source directives are supported; frontmatter settings are not.

## Development

```sh
npm test       # run the shorthand and scanner tests
npm run check  # type-check without emitting files
npm run build  # create main.js for Obsidian
npm run dev    # rebuild main.js when source files change
```
