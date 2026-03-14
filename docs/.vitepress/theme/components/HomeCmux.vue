<template>
  <section class="cmux-section">
    <div class="section-inner">
      <p class="section-label">Workflow</p>
      <h2 class="section-title">Pair with cmux</h2>

      <div class="cmux-content">
        <div class="cmux-text">
          <p>croot works great alongside <a href="https://github.com/manaflow-ai/cmux">cmux</a> for a full vibe coding setup in the terminal — file tree on one side, editor and shell on the other.</p>
          <p>Together they give you a VS Code-like workspace that lives entirely in your terminal. No Electron, no GUI, just fast tools that compose well.</p>
          <a href="https://github.com/manaflow-ai/cmux" class="btn btn-outline" target="_blank" rel="noopener">
            Learn about cmux
          </a>
        </div>
        <div class="cmux-visual">
<pre><span class="dim">&#x250C;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x252C;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2510;</span>
<span class="dim">&#x2502;</span> <span class="label">croot</span>        <span class="dim">&#x2502;</span> <span class="label">$EDITOR</span>                <span class="dim">&#x2502;</span>
<span class="dim">&#x2502;</span>              <span class="dim">&#x2502;</span>                        <span class="dim">&#x2502;</span>
<span class="dim">&#x2502;</span>  src/        <span class="dim">&#x2502;</span>  fn main() {           <span class="dim">&#x2502;</span>
<span class="dim">&#x2502;</span>    main.rs   <span class="dim">&#x2502;</span>      println!("hello");<span class="dim">&#x2502;</span>
<span class="dim">&#x2502;</span>    lib.rs    <span class="dim">&#x2502;</span>  }                     <span class="dim">&#x2502;</span>
<span class="dim">&#x2502;</span>  Cargo.toml  <span class="dim">&#x2502;</span>                        <span class="dim">&#x2502;</span>
<span class="dim">&#x2502;</span>  README.md   <span class="dim">&#x2502;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2524;</span>
<span class="dim">&#x2502;</span>              <span class="dim">&#x2502;</span> <span class="label">$ </span>cargo run              <span class="dim">&#x2502;</span>
<span class="dim">&#x2502;</span>              <span class="dim">&#x2502;</span>                        <span class="dim">&#x2502;</span>
<span class="dim">&#x2514;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2534;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2500;&#x2518;</span></pre>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.cmux-section {
  border-top: 1px solid var(--croot-border);
  padding: 80px 24px;
  background: var(--croot-bg);
  font-family: var(--croot-font-sans);
}

.section-inner {
  max-width: var(--croot-max-width);
  margin: 0 auto;
}

.section-label {
  font-family: var(--croot-font-mono);
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--croot-accent);
  margin-bottom: 8px;
}

.section-title {
  font-size: clamp(1.5rem, 3vw, 2rem);
  font-weight: 700;
  margin-bottom: 24px;
  color: var(--croot-text);
}

.cmux-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 48px;
  align-items: center;
}

.cmux-text p {
  color: var(--croot-text-muted);
  margin-bottom: 16px;
  font-size: 0.95rem;
  line-height: 1.7;
}

.cmux-text a:not(.btn) {
  color: var(--croot-link);
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  border-radius: 6px;
  font-size: 0.95rem;
  font-weight: 500;
  text-decoration: none;
  transition: all 0.2s;
  margin-top: 8px;
}

.btn-outline {
  background: transparent;
  color: var(--croot-text);
  border: 1px solid var(--croot-border);
}

.btn-outline:hover {
  border-color: var(--croot-text-muted);
  text-decoration: none;
}

.cmux-visual {
  background: var(--croot-bg-surface);
  border: 1px solid var(--croot-border);
  border-radius: var(--croot-radius);
  padding: 24px;
  font-family: var(--croot-font-mono);
  font-size: 0.8rem;
  color: var(--croot-text-muted);
  line-height: 1.6;
}

.cmux-visual pre {
  margin: 0;
}

.label {
  color: var(--croot-accent);
}

.dim {
  color: var(--croot-border);
}

@media (max-width: 768px) {
  .cmux-content {
    grid-template-columns: 1fr;
    gap: 32px;
  }
  .cmux-section {
    padding: 60px 16px;
  }
}
</style>
