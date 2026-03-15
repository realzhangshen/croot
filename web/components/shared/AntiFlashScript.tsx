// Anti-flash script prevents FOUC by setting data-theme before paint.
// The script content is a compile-time constant string with no user input —
// this is a well-established Next.js pattern for theme initialization.
// SECURITY: Safe — hardcoded string literal, no dynamic content, no XSS vector.
const THEME_SCRIPT = `(function(){var t=localStorage.getItem('theme');if(!t)t=window.matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light';document.documentElement.setAttribute('data-theme',t)})()`;

export function AntiFlashScript() {
  return <script dangerouslySetInnerHTML={{ __html: THEME_SCRIPT }} />;
}
