# DESIGN.md

> 把 Steam 商店式的沉浸陈列感，翻译成一页暖色、非黑底、适合产品叙事的桌面产品介绍页。

## 1. Visual Theme & Atmosphere

**Style**: Warm Storefront Cinematic  
**Keywords**: layered, editorial, collectible, warm glow, dense highlights, premium utility, amber glass, showcase rhythm  
**Tone**: 沉浸但不压抑，像高级游戏商店陈列区而不是深夜网吧界面 — NOT cyberpunk black, neon overload, cold SaaS template  
**Feel**: 像傍晚灯光打在木质展柜和金属卡匣上，信息很多，但层次始终清楚。

**Interaction Tier**: L2 流畅交互  
**Dependencies**: CSS + 原生 JS

## 2. Color Palette & Roles

```css
:root {
  --bg: #f4eadc;
  --surface: #fbf4ea;
  --surface-alt: #ead8c3;
  --surface-hover: #fff8f0;

  --border: #cfb79a;
  --border-hover: #9c6a3d;

  --text: #3d2415;
  --text-secondary: #684c39;
  --text-tertiary: #866a53;

  --accent: #b85c2f;
  --accent-hover: #9f4b21;
  --accent-2: #d38d3f;
  --accent-soft: #f0c18a;

  --bg-rgb: 244, 234, 220;
  --accent-rgb: 184, 92, 47;
  --surface-rgb: 251, 244, 234;

  --success: #4d8657;
  --error: #a6462a;
  --warning: #cc8a2e;
}
```

**Color Rules:**
- 所有颜色通过变量引用，组件里禁止直接硬编码新的十六进制色值。
- 暖色优先来自陶土、蜂蜜、铜、羊皮纸，不使用纯黑和近黑背景。
- 强调色一次只用一个主导来源，金色用于点亮，砖红用于 CTA，不混成彩虹。

## 3. Typography Rules

**Font Stack:**
```css
@import url('https://fonts.googleapis.com/css2?family=Cinzel:wght@500;600;700&family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@500;600&display=swap');
```

| Role | Font | Size | Weight | Line Height | Letter Spacing |
|------|------|------|--------|-------------|----------------|
| Hero H1 | Cinzel, Georgia, serif | clamp(3.4rem, 8vw, 6.6rem) | 700 | 0.95 | -0.03em |
| Section H2 | Cinzel, Georgia, serif | clamp(2rem, 4vw, 3rem) | 600 | 1.05 | -0.02em |
| H3 | Inter, Arial, sans-serif | 1.15rem | 700 | 1.2 | -0.01em |
| Body | Inter, Arial, sans-serif | 1rem | 500 | 1.7 | 0 |
| Label | Inter, Arial, sans-serif | 0.78rem | 700 | 1.2 | 0.14em |
| Mono/Code | JetBrains Mono, Consolas, monospace | 0.84rem | 600 | 1.5 | 0 |

**Typography Rules:**
- 大标题只用于 Hero 与章节标题，正文一律保持清晰无衬线。
- 数字、状态、版本信息优先用 `JetBrains Mono`，增强“系统装备感”。
- Heading weight 保持在 600 以上，正文避免低于 500。
- **NEVER use**: Inter Tight, Poppins, Orbitron, Arial Black, 任意过度未来感 display font

**Text Decoration:**
- Hero H1: 允许微弱暖金渐变与柔和文字阴影，但不做高饱和霓虹。
- Section H2: 不使用渐变，靠字重和留白建立权重。
- 数据数字与标签：禁止投影，保持像商店价签一样利落。

## 4. Component Stylings

### Buttons
```css
.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.65rem;
  min-height: 3.25rem;
  padding: 0 1.4rem;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: linear-gradient(135deg, rgba(var(--surface-rgb), 0.96), rgba(240, 193, 138, 0.55));
  color: var(--text);
  font: 700 0.92rem/1 Inter, Arial, sans-serif;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  transition: transform 0.24s ease, border-color 0.24s ease, background 0.24s ease, box-shadow 0.24s ease;
}

.button:hover {
  transform: translateY(-2px);
  border-color: var(--border-hover);
  box-shadow: 0 18px 34px rgba(var(--accent-rgb), 0.18);
}

.button:active {
  transform: translateY(0);
  background: linear-gradient(135deg, rgba(240, 193, 138, 0.92), rgba(184, 92, 47, 0.2));
}

.button:focus-visible {
  outline: 2px solid rgba(var(--accent-rgb), 0.45);
  outline-offset: 3px;
}

.button[disabled] {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.button.is-primary {
  background: linear-gradient(135deg, var(--accent), var(--accent-2));
  color: #fff6ee;
  border-color: rgba(var(--accent-rgb), 0.4);
}
```

### Cards
```css
.card {
  position: relative;
  border: 1px solid rgba(156, 106, 61, 0.18);
  border-radius: 28px;
  background: linear-gradient(180deg, rgba(255, 248, 240, 0.92), rgba(234, 216, 195, 0.92));
  box-shadow: 0 16px 40px rgba(114, 70, 32, 0.08);
  overflow: hidden;
  transition: transform 0.3s ease, border-color 0.3s ease, box-shadow 0.3s ease;
}

.card:hover {
  transform: translateY(-5px);
  border-color: rgba(var(--accent-rgb), 0.42);
  box-shadow: 0 24px 50px rgba(114, 70, 32, 0.14);
}

.card:focus-within {
  border-color: rgba(var(--accent-rgb), 0.48);
  box-shadow: 0 0 0 4px rgba(var(--accent-rgb), 0.12);
}
```

### Navigation
```css
.site-nav {
  position: sticky;
  top: 0;
  z-index: 30;
  backdrop-filter: blur(12px);
  background: rgba(var(--bg-rgb), 0.68);
  border-bottom: 1px solid transparent;
  transition: background 0.25s ease, border-color 0.25s ease, box-shadow 0.25s ease;
}

.site-nav.is-scrolled {
  background: rgba(251, 244, 234, 0.88);
  border-color: rgba(156, 106, 61, 0.18);
  box-shadow: 0 14px 30px rgba(101, 61, 28, 0.08);
}
```

### Links
```css
.nav-link,
.text-link {
  position: relative;
  color: var(--text-secondary);
  text-decoration: none;
  transition: color 0.2s ease;
}

.nav-link::after,
.text-link::after {
  content: "";
  position: absolute;
  left: 0;
  bottom: -0.2rem;
  width: 100%;
  height: 1px;
  transform: scaleX(0);
  transform-origin: left;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  transition: transform 0.22s ease;
}

.nav-link:hover,
.text-link:hover {
  color: var(--text);
}

.nav-link:hover::after,
.text-link:hover::after {
  transform: scaleX(1);
}
```

### Tags / Badges
```css
.badge {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  min-height: 2rem;
  padding: 0.3rem 0.72rem;
  border-radius: 999px;
  border: 1px solid rgba(156, 106, 61, 0.2);
  background: rgba(255, 248, 240, 0.72);
  color: var(--text-secondary);
  font: 700 0.74rem/1 Inter, Arial, sans-serif;
  text-transform: uppercase;
  letter-spacing: 0.12em;
}
```

### Feature Panels
```css
.feature-panel {
  display: grid;
  gap: 1rem;
  padding: 1.4rem;
  min-height: 15rem;
}

.feature-panel::before {
  content: "";
  position: absolute;
  inset: auto -20% -32% auto;
  width: 12rem;
  height: 12rem;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(211, 141, 63, 0.32), transparent 68%);
  pointer-events: none;
}
```

## 5. Layout Principles

**Container:**
- Max width: 1240px
- Padding: 0 24px desktop, 0 18px mobile
- Narrow variant (text-heavy): 760px

**Spacing Scale:**
- Section padding: clamp(4.5rem, 9vw, 8rem)
- Component gap: 1rem / 1.5rem / 2rem / 3rem
- Card internal padding: 1.25rem to 2rem

**Grid:**
```css
.container {
  width: min(1240px, calc(100% - 48px));
  margin: 0 auto;
}

.bento-grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: 1rem;
}

.span-7 { grid-column: span 7; }
.span-5 { grid-column: span 5; }
.span-4 { grid-column: span 4; }
.span-8 { grid-column: span 8; }
```

## 6. Depth & Elevation

| Level | Treatment | Use |
|-------|-----------|-----|
| Flat | 无阴影，仅靠浅边框和色差 | 标签、辅助面板 |
| Subtle | `0 12px 26px rgba(114, 70, 32, 0.08)` | 常规卡片、导航 |
| Elevated | `0 20px 48px rgba(114, 70, 32, 0.14)` | Hero 视觉框、Bento 主卡 |
| Glow | 暖色径向辉光，不代替阴影 | CTA、重点数据、展示框 |

## 7. Animation & Interaction

**Motion Philosophy**: 像橱窗灯光与卡片抽屉轻轻滑开，只用透明度、位移、缩放和局部光感，不做重型滚动劫持。  
**Tier**: L2

### Dependencies
```html
<script defer src="./app.js"></script>
```

### Base Setup
```js
function initScrollReveal(selector = ".reveal") {
  const obs = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (!entry.isIntersecting) return;
      entry.target.classList.add("in-view");
      obs.unobserve(entry.target);
    });
  }, { threshold: 0.16 });

  document.querySelectorAll(selector).forEach((node) => obs.observe(node));
}
```

### Entrance Animation
```css
.reveal {
  opacity: 0;
  transform: translateY(28px);
  transition: opacity 0.72s cubic-bezier(0.16, 1, 0.3, 1),
              transform 0.72s cubic-bezier(0.16, 1, 0.3, 1);
}

.reveal.in-view {
  opacity: 1;
  transform: translateY(0);
}
```

### Scroll Behavior
```js
window.addEventListener("scroll", () => {
  document.querySelector(".site-nav")?.classList.toggle("is-scrolled", window.scrollY > 16);
}, { passive: true });
```

### Hover & Focus States
```css
.card:hover .eyebrow,
.button:hover .button-arrow {
  transform: translateX(2px);
}

:focus-visible {
  outline: 2px solid rgba(var(--accent-rgb), 0.42);
  outline-offset: 3px;
}
```

### Special Effects
- Hero 背景使用鼠标驱动的暖色 `radial-gradient`，只更新 CSS 变量，不做 canvas。
- 首次滑动后的宣言带采用纯 CSS 横向流动字带，模拟 Steam 首页的陈列节奏。
- Bento 阵列中的大卡片使用局部高光跟随，限制在单区域内，避免全页重绘。

### Reduced Motion
```css
@media (prefers-reduced-motion: reduce) {
  html { scroll-behavior: auto; }
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
  .hero-orb,
  .ticker-track {
    animation: none !important;
  }
}
```

## 8. Do's and Don'ts

### Do
- 用大块分区和卡片尺寸对比制造“商店首页”的陈列节奏。
- 让每个 section 都像一个上架中的专题页，有自己的标题、标签和数据句子。
- 让暖色来自材质和灯光，而不是把整页刷成橙色。
- 保持信息密度高，但用留白和层级保证首屏可扫读。
- 用真实产品语言写卖点，避免空洞营销短句。

### Don't
- ❌ 不要出现纯黑、近黑或蓝黑背景。
- ❌ 不要套标准白底 SaaS 模板然后只把按钮改橙色。
- ❌ 不要在一个 section 内同时出现两种以上主强调色。
- ❌ 不要使用模糊过大的玻璃拟态遮住正文。
- ❌ 不要把所有卡片做成完全等大等宽，失去“商店货架”节奏。
- ❌ 不要引入重型轮播、自动切换大 Banner 或 scroll-jacking。
- ❌ 不要把标题都做成渐变字，首屏之外必须收敛。
- ❌ 不要写假大空文案，比如“革命性赋能未来生态”。
- ❌ 不要让 hover 只改颜色不改层次，卡片需要有轻微抬升。
- ❌ 不要在移动端保留桌面级并列布局导致内容挤压。

## 9. Responsive Behavior

**Breakpoints:**

| Name | Width | Key Changes |
|------|-------|-------------|
| Desktop | > 1080px | 12 列 Bento，Hero 左右分栏，Ticker 保持整条 |
| Tablet | 720px-1080px | Hero 改单列，Bento 压缩为 6 列，导航简化 |
| Mobile | < 720px | 全部单列堆叠，指标卡横向滚动取消，CTA 全宽 |

**Touch Targets:** minimum 44px  
**Collapsing Strategy:** 导航链接折成紧凑行，Bento 卡片全部 `grid-column: 1 / -1`，数据条目改为 2 列或单列。

```css
@media (max-width: 1080px) {
  .hero-layout,
  .story-grid,
  .footer-grid {
    grid-template-columns: 1fr;
  }

  .span-7,
  .span-5,
  .span-4,
  .span-8 {
    grid-column: span 6;
  }
}

@media (max-width: 720px) {
  .container {
    width: min(100% - 36px, 1240px);
  }

  .span-7,
  .span-5,
  .span-4,
  .span-8,
  .metric-strip,
  .nav-links {
    grid-column: 1 / -1;
  }

  .button {
    width: 100%;
  }
}
```
