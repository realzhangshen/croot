# Sample Markdown Document

This file exercises **all Markdown syntax elements** for testing highlighting.

## Inline Formatting

This is *italic text* and _also italic_. This is **bold text** and __also bold__.
Combined ***bold italic*** and ~~strikethrough~~.
Inline `code spans` look like this, and so does `Vec<String>`.

## Links & Images

- [External link](https://example.com "Example Site")
- [Reference link][ref-1]
- Autolink: <https://example.com>
- Email: <user@example.com>
- ![Alt text for image](./images/logo.png)
- [![Clickable image](./badge.svg)](https://example.com)

[ref-1]: https://reference.example.com "Reference"

## Headings

### Third Level
#### Fourth Level
##### Fifth Level
###### Sixth Level

## Lists

### Unordered
- Item one
  - Nested item A
    - Deeply nested
  - Nested item B
- Item two
- Item three

### Ordered
1. First item
2. Second item
   1. Sub-item 2.1
   2. Sub-item 2.2
3. Third item

### Task List
- [x] Completed task
- [ ] Incomplete task
- [x] Another done task
- [ ] Yet to do

## Blockquotes

> Single line quote.

> Multi-line blockquote.
> This continues the quote.
>
> > Nested blockquote inside.
>
> Back to the first level.

## Code Blocks

Inline code: `let x = 42;`

Fenced code block with language:

```rust
fn main() {
    let greeting = "Hello, world!";
    println!("{greeting}");

    for i in 0..5 {
        if i % 2 == 0 {
            println!("{i} is even");
        }
    }
}
```

```typescript
interface User {
  id: number;
  name: string;
  email: string;
}

async function fetchUser(id: number): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  return response.json();
}
```

```json
{
  "name": "example",
  "version": "1.0.0",
  "dependencies": {
    "typescript": "^5.0.0"
  }
}
```

Indented code block (4 spaces):

    function hello() {
        console.log("indented code block");
    }

## Tables

| Column A | Column B | Column C | Alignment |
|----------|:--------:|---------:|-----------|
| Left     | Center   |    Right | Default   |
| `code`   | **bold** |    *em*  | mixed     |
| Cell 1   | Cell 2   |   Cell 3 | data      |
| Long content here | Short | 42 | numbers  |

## Horizontal Rules

---

***

___

## HTML (Raw)

<details>
<summary>Click to expand</summary>

This is hidden content with **markdown** inside.

- Item in details
- Another item

</details>

<div align="center">
  <strong>Centered HTML content</strong>
</div>

## Escaping

Literal asterisks: \*not italic\*
Literal backtick: \`not code\`
Literal brackets: \[not a link\]
Backslash: \\

## Footnotes

Here is a sentence with a footnote.[^1]
Another footnote reference.[^long-note]

[^1]: This is the footnote content.
[^long-note]: A longer footnote with multiple paragraphs.

    Second paragraph of the footnote.

## Definition Lists

Term 1
: Definition for term 1

Term 2
: Primary definition
: Alternate definition

## Math (if supported)

Inline math: $E = mc^2$

Block math:
$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

## Emoji (shortcodes)

:rocket: :star: :warning: :white_check_mark:

## Summary

This document covers: headings, emphasis, links, images, lists, blockquotes,
code (inline, fenced, indented), tables, horizontal rules, HTML, escaping,
footnotes, definition lists, and math blocks.
