# VHS tape file for generating croot demo GIF
# https://github.com/charmbracelet/vhs

Output docs/public/demo.gif

Set Shell zsh
Set FontFamily "JetBrains Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "GitHub Dark"
Set Padding 20
Set Framerate 30
Set TypingSpeed 60ms

# Start croot in the sample project
Type "croot demo-project"
Enter
Sleep 2s

# Navigate down through the tree
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Expand a directory
Type "l"
Sleep 600ms

# Navigate into expanded dir
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Open preview panel
Type "p"
Sleep 1.5s

# Navigate to show different files
Type "j"
Sleep 800ms
Type "j"
Sleep 800ms
Type "k"
Sleep 800ms

# Show search
Type "/"
Sleep 500ms
Type "main"
Sleep 1s
Enter
Sleep 1s

# Clear and navigate
Escape
Sleep 500ms

# Move around more
Type "g"
Sleep 600ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "l"
Sleep 600ms
Type "j"
Sleep 400ms

# Show preview of another file
Type "p"
Sleep 2s

# Quit
Type "q"
Sleep 500ms
