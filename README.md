# ssg

A tiny static site generator for technical sites.

## Quick start

```bash
git clone https://github.com/pykello/ssg.git
cd ssg
cargo build --release
export PATH="$PWD/target/release:$PATH"

./init.sh my-site
cd my-site
make
```

Open `build/index.html` in your browser.

## Adding content

- Create `content/en/about.md` for a simple page.
- Or create a directory with `metadata.yaml` + `body.md`.
- Use `content/en/index.yaml` to generate list pages.

Example `metadata.yaml`:

```yaml
title: My Page
type: page
```

## Bilingual sites

```bash
./init.sh my-site --bilingual
```

This creates `config.en.yaml` + `config.fa.yaml` and a stub translation file.

## Configuration

See the generated `config.yaml`. Common fields:

- `theorems` — custom LaTeX environments
- `escape_markdown_in_math: false`
- `translations_csv`
- `context` — extra values available in templates

## Templates

Put your HTML templates in `templates/`. Use Tera syntax.

The `init.sh` gives you a minimal working set.

## Building manually

```bash
ssg-content content/en/about --config config.yaml
ssg-list content/en/index.yaml --config config.yaml
```

## License

MIT
