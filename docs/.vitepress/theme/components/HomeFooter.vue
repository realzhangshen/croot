<template>
  <footer class="site-footer">
    <div class="footer-inner">
      <div class="footer-left">
        Built with <span class="accent">Rust</span> &amp; <a href="https://ratatui.rs">Ratatui</a>
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
  border-top: 1px solid var(--croot-border);
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
  font-family: var(--croot-font-mono);
  font-size: 0.8rem;
  color: var(--croot-text-muted);
}

.footer-left .accent {
  color: var(--croot-accent);
}

.footer-left a {
  color: var(--croot-accent);
  text-decoration: none;
}

.footer-left a:hover {
  text-decoration: underline;
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
  transition: color 0.2s;
}

.footer-links a:hover {
  color: var(--croot-text);
}
</style>
