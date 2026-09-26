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

TRANSLATION (MULTI-ENGINE & ON-DEMAND OFFLINE):
- Translate Selection  : Highlight text -> click "Translate" pill or press Ctrl+Alt+T
- Translate Document   : Click "Translate" in top bar or press Ctrl+Shift+T to dock document translation bar
- Multiple Engines     :
  1. LibreTranslate    : Free cloud instance (https://translate.adminforge.de) or local (http://127.0.0.1:5000) instance
  2. Free Web Engine   : Zero-setup web translation with automatic cloud failover
  3. Offline Packs     : Download on-demand language packs (~20-25MB each) in Settings for 100% offline, private translation

SEARCH & NAVIGATION:
- Ctrl+F               : Open floating search bar with next/prev match and count
- Vim Mode /           : Search forward directly from Vim Normal mode (no floating box)
- Middle Click Tab     : Quickly close tab with middle mouse click
- Right Click Tab      : Context menu to "Open Containing Folder" or "Close Tab"

SETTINGS & CUSTOMIZATION:
- Click the "Keys" button in the top bar to customize:
  - Keybindings, scroll speed, physics & max FPS
  - Outline / Table of Contents layout (Side Tab vs Floating Window)
  - Color Theme (syntax highlights, accents, markdown colors)
  - Multilingual Fonts (UI & Monospace fonts with custom fallback support)
  - Translation Provider & Offline Language Pack Downloader
- Settings are automatically saved to keybindings.json.

================================================================================
