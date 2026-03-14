<template>
  <footer class="site-footer">
    <div class="footer-inner">
      <div class="footer-left">
        Built with Rust &amp; <a href="https://ratatui.rs">Ratatui</a>
      </div>
      <ul class="footer-links">
        <li><a href="https://github.com/realzhangshen/croot">GitHub</a></li>
        <li><a href="https://github.com/realzhangshen/croot/blob/main/CHANGELOG.md">Changelog</a></li>
        <li><a href="https://github.com/realzhangshen/croot/blob/main/LICENSE">MIT License</a></li>
      </ul>
    </div>
  </footer>
</template>

<style scoped>
.site-footer {
  padding: 40px 24px;
  background: var(--croot-bg);
  font-family: var(--croot-font-sans);
}

.footer-inner {
  max-width: var(--croot-max-width);
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 16px;
}

.footer-left {
  font-size: 0.85rem;
  color: var(--croot-text-muted);
}

.footer-left a {
  color: var(--croot-text-secondary);
  text-decoration: none;
  transition: color var(--croot-dur-fast) var(--croot-ease);
}

.footer-left a:hover {
  color: var(--croot-text);
}

.footer-links {
  display: flex;
  gap: 20px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.footer-links a {
  color: var(--croot-text-muted);
  font-size: 0.85rem;
  text-decoration: none;
  transition: color var(--croot-dur-fast) var(--croot-ease);
}

.footer-links a:hover {
  color: var(--croot-text);
}
</style>
