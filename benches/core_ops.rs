use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

use croot::config::TreeConfig;
use croot::preview::highlight::highlight_code;
use croot::preview::render_md::render_markdown;
use croot::tree::forest::FileTree;
use croot::tree::node::{NodeKind, TreeNode};
use croot::tree::sorter::sort_nodes;

/// Create a temporary directory with `n` files and `n_dirs` subdirectories.
fn create_test_tree(n_files: usize, n_dirs: usize) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    for i in 0..n_dirs {
        std::fs::create_dir(tmp.path().join(format!("dir_{i:04}"))).unwrap();
        // Put some files in each directory
        for j in 0..3 {
            std::fs::write(
                tmp.path().join(format!("dir_{i:04}/file_{j}.txt")),
                format!("content {i}-{j}"),
            )
            .unwrap();
        }
    }
    for i in 0..n_files {
        std::fs::write(
            tmp.path().join(format!("file_{i:04}.txt")),
            format!("content {i}"),
        )
        .unwrap();
    }
    tmp
}

fn bench_tree_loading(c: &mut Criterion) {
    let tmp = create_test_tree(500, 100);

    c.bench_function("tree_load_500files_100dirs", |b| {
        b.iter(|| {
            let tree = FileTree::new(black_box(tmp.path().to_path_buf()), TreeConfig::default());
            black_box(tree.len());
        });
    });
}

fn bench_tree_loading_large(c: &mut Criterion) {
    let tmp = create_test_tree(1000, 200);

    c.bench_function("tree_load_1600files_200dirs", |b| {
        b.iter(|| {
            let tree = FileTree::new(black_box(tmp.path().to_path_buf()), TreeConfig::default());
            black_box(tree.len());
        });
    });
}

fn bench_natural_sort(c: &mut Criterion) {
    let names: Vec<String> = (0..1000)
        .map(|i| format!("file_{}.txt", 1000 - i))
        .collect();

    c.bench_function("natural_sort_1000_items", |b| {
        b.iter(|| {
            let mut nodes: Vec<TreeNode> = names
                .iter()
                .map(|name| TreeNode::new(PathBuf::from(name), NodeKind::File, 0))
                .collect();
            sort_nodes(black_box(&mut nodes), true);
        });
    });
}

fn bench_natural_sort_mixed(c: &mut Criterion) {
    let mut names: Vec<(String, NodeKind)> = Vec::new();
    for i in 0..500 {
        names.push((format!("dir_{}", 500 - i), NodeKind::Directory));
        names.push((format!("file_{}.rs", 500 - i), NodeKind::File));
    }

    c.bench_function("natural_sort_1000_mixed_dirs_files", |b| {
        b.iter(|| {
            let mut nodes: Vec<TreeNode> = names
                .iter()
                .map(|(name, kind)| TreeNode::new(PathBuf::from(name), kind.clone(), 0))
                .collect();
            sort_nodes(black_box(&mut nodes), true);
        });
    });
}

fn bench_syntax_highlight(c: &mut Criterion) {
    let rust_code = r#"
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");

    for (k, v) in &map {
        println!("{}: {}", k, v);
    }

    let result: Result<i32, String> = Ok(42);
    match result {
        Ok(n) => println!("Got: {}", n),
        Err(e) => eprintln!("Error: {}", e),
    }
}
"#;
    // Repeat to simulate a larger file
    let large_code = rust_code.repeat(20);

    c.bench_function("highlight_rust_400_lines", |b| {
        b.iter(|| {
            let result = highlight_code(black_box("rs"), black_box(&large_code), 500);
            black_box(result.len());
        });
    });
}

fn bench_syntax_highlight_short(c: &mut Criterion) {
    let code = "fn hello() { println!(\"Hello, world!\"); }\n".repeat(10);

    c.bench_function("highlight_rust_10_lines", |b| {
        b.iter(|| {
            let result = highlight_code(black_box("rs"), black_box(&code), 100);
            black_box(result.len());
        });
    });
}

fn bench_markdown_render(c: &mut Criterion) {
    let markdown = r#"
# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

- Item 1
- Item 2
  - Nested item
- Item 3

```rust
fn main() {
    println!("Hello!");
}
```

> A blockquote with some text.

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

1. First
2. Second
3. Third
"#;
    let large_md = markdown.repeat(10);

    c.bench_function("render_markdown_300_lines_w80", |b| {
        b.iter(|| {
            let result = render_markdown(black_box(&large_md), black_box(80));
            black_box(result.len());
        });
    });
}

fn bench_markdown_render_narrow(c: &mut Criterion) {
    let markdown = "This is a paragraph. ".repeat(50)
        + "\n\n"
        + &"Another paragraph with more text. ".repeat(30);

    c.bench_function("render_markdown_wrapping_w40", |b| {
        b.iter(|| {
            let result = render_markdown(black_box(&markdown), black_box(40));
            black_box(result.len());
        });
    });
}

criterion_group!(
    benches,
    bench_tree_loading,
    bench_tree_loading_large,
    bench_natural_sort,
    bench_natural_sort_mixed,
    bench_syntax_highlight,
    bench_syntax_highlight_short,
    bench_markdown_render,
    bench_markdown_render_narrow,
);
criterion_main!(benches);
