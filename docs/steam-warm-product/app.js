function initScrollReveal(selector = ".reveal") {
  const nodes = document.querySelectorAll(selector);
  if (!nodes.length) return;

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("in-view");
        observer.unobserve(entry.target);
      });
    },
    { threshold: 0.16 }
  );

  nodes.forEach((node) => observer.observe(node));
}

function initScrolledNav() {
  const nav = document.querySelector(".site-nav");
  if (!nav) return;

  const sync = () => {
    nav.classList.toggle("is-scrolled", window.scrollY > 16);
  };

  sync();
  window.addEventListener("scroll", sync, { passive: true });
}

function initHeroPointerGlow() {
  const shell = document.querySelector(".hero-section");
  if (!shell || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  let frame = null;

  const update = (event) => {
    const rect = shell.getBoundingClientRect();
    const x = ((event.clientX - rect.left) / rect.width) * 100;
    const y = ((event.clientY - rect.top) / rect.height) * 100;

    if (frame) cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      document.documentElement.style.setProperty("--mouse-x", `${x}%`);
      document.documentElement.style.setProperty("--mouse-y", `${y}%`);
    });
  };

  shell.addEventListener("pointermove", update, { passive: true });
}

function initTiltSurfaces() {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  document.querySelectorAll("[data-tilt-surface]").forEach((surface) => {
    let frame = null;

    const reset = () => {
      surface.style.transform = "";
    };

    surface.addEventListener("pointermove", (event) => {
      const rect = surface.getBoundingClientRect();
      const offsetX = (event.clientX - rect.left) / rect.width - 0.5;
      const offsetY = (event.clientY - rect.top) / rect.height - 0.5;

      if (frame) cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        surface.style.transform = `rotateX(${(-offsetY * 4).toFixed(2)}deg) rotateY(${(offsetX * 5).toFixed(2)}deg)`;
      });
    });

    surface.addEventListener("pointerleave", reset);
  });
}

function initWarmToggle() {
  const trigger = document.querySelector("[data-warm-toggle]");
  if (!trigger) return;

  trigger.addEventListener("click", () => {
    document.body.classList.toggle("is-warmer");
    const active = document.body.classList.contains("is-warmer");
    trigger.textContent = active ? "Show default warmth" : "Toggle showcase warmth";
  });
}

document.addEventListener("DOMContentLoaded", () => {
  initScrollReveal();
  initScrolledNav();
  initHeroPointerGlow();
  initTiltSurfaces();
  initWarmToggle();
});
