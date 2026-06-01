#!/usr/bin/env bash
set -e

show_help() {
    cat << 'EOF'
Usage: ./init.sh <directory> [options]

Scaffold a new ssg project.

Options:
  --bilingual      Create English + Farsi (fa) setup with separate configs
  --no-makefile    Skip generating a Makefile
  -h, --help       Show this help message
EOF
    exit 0
}

# Check for help anywhere in the arguments first
for arg in "$@"; do
    case "$arg" in
        -h|--help)
            show_help
            ;;
    esac
done

if [ $# -lt 1 ]; then
    echo "Error: missing target directory" >&2
    echo "Usage: $0 <directory> [--bilingual] [--no-makefile]" >&2
    exit 1
fi

TARGET_DIR=""
BILINGUAL=false
MAKEFILE=true

while [ $# -gt 0 ]; do
    case "$1" in
        --bilingual)
            BILINGUAL=true
            shift
            ;;
        --no-makefile)
            MAKEFILE=false
            shift
            ;;
        -*)
            echo "Unknown option: $1" >&2
            echo "Try '$0 --help' for more information." >&2
            exit 1
            ;;
        *)
            if [ -z "$TARGET_DIR" ]; then
                TARGET_DIR="$1"
            else
                echo "Error: only one directory name is allowed" >&2
                exit 1
            fi
            shift
            ;;
    esac
done

if [ -z "$TARGET_DIR" ]; then
    echo "Error: no directory specified" >&2
    exit 1
fi

if [ -e "$TARGET_DIR" ]; then
    echo "Error: $TARGET_DIR already exists" >&2
    exit 1
fi

mkdir -p "$TARGET_DIR"
cd "$TARGET_DIR"

echo "Creating skeleton in $TARGET_DIR..."

# --- config ---
if [ "$BILINGUAL" = true ]; then
    cat > config.en.yaml << 'EOF'
build_dir: build
content_dir: content
template_dir: templates

syntax_highlighter_theme: "InspiredGitHub"
escape_markdown_in_math: false

context:
  site_name: "My Site"
EOF

    cat > config.fa.yaml << 'EOF'
build_dir: build
content_dir: content
template_dir: templates
translations_csv: translate.fa.csv

language: fa
text_direction: rtl

syntax_highlighter_theme: "InspiredGitHub"
escape_markdown_in_math: false

context:
  site_name: "سایت من"
EOF

    cat > translate.fa.csv << 'EOF'
Home,خانه
About,درباره
Blog,وبلاگ
EOF
else
    cat > config.yaml << 'EOF'
build_dir: build
content_dir: content
template_dir: templates

syntax_highlighter_theme: "InspiredGitHub"
escape_markdown_in_math: false

context:
  site_name: "My Site"
EOF
fi

# --- content ---
mkdir -p content/en

if [ "$BILINGUAL" = true ]; then
    mkdir -p content/fa
fi

# Root index
cat > content/index.yaml << 'EOF'
content-type: blog
template: blog-list.html
title: "Home"
EOF

# English content
cat > content/en/about.md << 'EOF'
# About

This is a sample about page.

You can edit this file or turn it into a directory with metadata.yaml.
EOF

mkdir -p content/en/blog/hello-world
cat > content/en/blog/hello-world/metadata.yaml << 'EOF'
title: "Hello World"
type: blog
EOF

cat > content/en/blog/hello-world/body.md << 'EOF'
# Hello World

This is your first post.

You can use **markdown**, math like $E=mc^2$, and `:::expandable` blocks.
EOF

cat > content/en/index.yaml << 'EOF'
content-type: blog
template: blog-list.html
title: "Blog"
EOF

# Templates (very minimal but functional)
mkdir -p templates

cat > templates/layout.html << 'EOF'
<!DOCTYPE html>
<html lang="{{ language | default(value="en") }}" dir="{{ text_direction | default(value="ltr") }}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{ title | default(value=site_name) }}</title>
  <style>
    body { font-family: system-ui, sans-serif; max-width: 720px; margin: 40px auto; padding: 0 20px; line-height: 1.6; }
    pre { background: #f4f4f4; padding: 12px; overflow: auto; }
  </style>
</head>
<body>
  <nav>
    <a href="/">Home</a>
    {% if context.about %}<a href="/about.html">About</a>{% endif %}
  </nav>
  <main>
    {{ content | default(value=body) }}
  </main>
</body>
</html>
EOF

cat > templates/page.html << 'EOF'
{% extends "layout.html" %}
{% block content %}
<h1>{{ page.title }}</h1>
{{ page.body | safe }}
{% endblock %}
EOF

cat > templates/blog-list.html << 'EOF'
{% extends "layout.html" %}
{% block content %}
<h1>{{ title }}</h1>
<ul>
{% for item in content_items %}
  <li><a href="{{ item.url }}">{{ item.title }}</a></li>
{% endfor %}
</ul>
{% endblock %}
EOF

# Static (optional placeholder)
mkdir -p static

# Makefile
if [ "$MAKEFILE" = true ]; then
    cat > Makefile << 'EOF'
CONTENT_METADATA := $(shell find content -name "metadata.yaml")
CONTENT_TARGETS := $(patsubst content/%/metadata.yaml,build/%.html,$(CONTENT_METADATA))

BARE_PAGES := $(shell find content -type f \( -name '*.md' -o -name '*.html' \) ! -execdir test -e metadata.yaml \; -print)
BARE_TARGETS := $(patsubst content/%.md,build/%.html,$(patsubst content/%.html,build/%.html,$(BARE_PAGES)))

INDEXES := $(shell find content -name "index.yaml")
INDEX_TARGETS := $(patsubst content/%/index.yaml,build/%/index.html,$(INDEXES))

all: $(CONTENT_TARGETS) $(BARE_TARGETS) $(INDEX_TARGETS) build/index.html

CONFIG = $(if $(findstring /fa/,$(1)),config.fa.yaml,config.yaml)

build/%.html: content/%/metadata.yaml
	@mkdir -p $(dir $@)
	ssg-content $(dir $<) --config $(call CONFIG,$<)

build/%.html: content/%.md
	@mkdir -p $(dir $@)
	ssg-content $< --config $(call CONFIG,$<)

build/%.html: content/%.html
	@mkdir -p $(dir $@)
	ssg-content $< --config $(call CONFIG,$<)

build/%/index.html: content/%/index.yaml
	@mkdir -p $(dir $@)
	ssg-list $< --config $(call CONFIG,$<)

build/index.html: content/index.yaml
	@mkdir -p $(dir $@)
	ssg-list $< --config $(call CONFIG,$<)

clean:
	rm -rf build

serve:
	python3 -m http.server --directory build 8000
EOF
fi

# Basic .gitignore
cat > .gitignore << 'EOF'
build/
*.swp
EOF

echo ""
echo "Done! cd into $TARGET_DIR and run:"
echo "  make"
echo ""
if [ "$MAKEFILE" = true ]; then
    echo "Or use the generated Makefile for convenience."
fi
