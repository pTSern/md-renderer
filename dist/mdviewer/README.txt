================================================================================
 MDViewer - Fast, Modern Markdown & Diagram Viewer / Editor
================================================================================

HOW TO USE:
1. Double-click mdviewer.exe to launch.
2. Open files by:
   - Clicking "Open" in the top toolbar.
   - Dragging and dropping any .md file directly onto the window ("Drag-to-Read").
   - Right-clicking any .md file in Windows -> "Open With" -> choose mdviewer.exe.
   - Running from terminal: mdviewer.exe path\to\document.md
   - Single-instance support: opening another file opens a new tab in the running app.

KEYBOARD SHORTCUTS:
- Shift+Tab            : Cycle view modes (Preview -> Split -> Edit -> Preview)
- Alt+O / Ctrl+Shift+O : Toggle Table of Contents / Outline
- Ctrl+Alt+V           : Toggle Vim Mode for editor on / off
- Ctrl+S               : Quick save
- Ctrl+Shift+S         : Save As
- Ctrl+O               : Open file dialog
- Ctrl+N               : New document
- Ctrl+Q               : Close application

TABLE OF CONTENTS (OUTLINE):
- Toggle Outline       : Alt+O (or click "Outline" button in toolbar)
- 2 Display Modes      : Side Tab (Docked) or Floating Window (Overlay)
- Side Tab Config      : Position (Left / Right), Width (%), Navigator keys (j / k)
- Floating Window      : Position (Center / Top / Bottom), Width/Height (%), Navigator keys (j / k)
- Navigation           : j / k (or Down/Up arrow) moves between headings, Enter jumps, Esc closes

PREVIEW MODE (NVim Navigation):
- j / k                : Smooth scroll down / up (Neovide fluid physics)
- h / l                : Smooth scroll left / right
- Ctrl+d / Ctrl+u      : Smooth scroll half page down / up
- gg / G               : Jump to top / jump to bottom
- Alt+Shift+H          : Switch to previous open file tab
- Alt+Shift+L          : Switch to next open file tab
- w                    : Quick save active document

EDITOR VIM MODE (Edit & Split Modes):
- NORMAL Mode          : Full modal motions (h/j/k/l, w/b/e, 0/^/$, dd, cc, yy, diw, ciw, u, Ctrl+r, etc.)
- INSERT Mode          : Standard editing mode (press Esc to return to Normal mode)
- VISUAL Mode          : Character (v) and line (V) selection with y, d, c, ~
- COMMAND Mode (:)     : :w (save), :q (close), :wq (save and close), :<line> (jump to line), :%s/find/replace/g, :help
- Animated Caret       : Neovide-style floating smooth spring cursor

SETTINGS:
- Click the "Keys" button in the top bar to customize keybindings, scroll speed, TOC modes, and Max FPS.
- Settings are automatically saved to keybindings.json.

================================================================================
