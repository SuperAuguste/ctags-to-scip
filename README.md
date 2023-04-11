# ctags-to-scip

Convert ctags JSON files into [SCIP indices](https://github.com/sourcegraph/scip).

Notes:
- ctags input must be JSON formatted

## Take it for a spin

```bash
cd path/to/sourcegraph/docker-images
ctags --fields=\* --output-format=json syntax-highlighter/**/*.rs > this/repo/syntax-highlighter_tags.txt
cd this/repo
cargo run -- --project-root path/to/sourcegraph/docker-images --input syntax-highlighter.tag --output syntax-highlighter.scip
```

