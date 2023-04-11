# ctags-to-scip

Self-explanatory.

Notes:
- ctags input must be JSON formatted

## Take it for a spin

```bash
cd path/to/sourcegraph/docker-images
ctags --fields=\* --output-format=json syntax-highlighter/**/*.rs > syntax-highlighter_tags.txt
cargo run -- --project-root path/to/sourcegraph/docker-images --input syntax-highlighter.tag --output syntax-highlighter.scip
```

