# Context Menus

Right-click any file or directory to open a context menu with common actions.

## Available Actions

The context menu provides quick access to:

- **New File** / **New Directory** — create entries from a directory or workspace menu
- **Open in editor** — opens the file in your configured editor
- **Open in cmux Tab** — opens the file in a new cmux tab when croot detects cmux
- **Open externally** — opens with the system default application
- **Copy relative path** — copies the relative path to clipboard
- **Copy absolute path** — copies the full absolute path to clipboard
- **Reveal in Finder** — reveals the file on macOS or opens its containing directory on Linux
- **Refresh** / **Collapse All** / **Toggle Preview** / **Find** — workspace and directory helpers
- **Rename** — rename the file or directory
- **Delete** — delete with confirmation

## Keyboard Shortcuts

Right-click works with mouse enabled. Direct shortcuts for actions such as create, rename, delete, open in editor, and toggle preview are opt-in via `[keybindings]`.

## Navigation

Once the context menu is open:

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate menu items |
| `Enter` | Select action |
| `Esc` | Close menu |
