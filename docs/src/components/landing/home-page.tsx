import { useState } from 'react';
import { ThemeSwitch } from 'fumadocs-ui/layouts/shared/slots/theme-switch';
import { ArrowRight, ArrowUpRight, ChevronRight } from 'lucide-react';
import { CodeWindow } from './code-window';
import { errors, examples, heroCode, heroOutput, keywords, logoSrc, ribbonItems } from './content';
import { SectionHeading } from './section-heading';
import { StickerButton } from './sticker-button';

const runtimeStages = ['lexer', 'parser', 'AST', 'bytecode', 'VM'] as const;

const manifestoCards = [
  {
    title: '¿Qué es?',
    body:
      'Un lenguaje dinámico y de tipado débil, escrito en Rust. Por dentro ya recorre lexer, parser, AST, bytecode y una VM de pila funcionando de verdad.',
    tone: 'blue',
  },
  {
    title: "¿Pa' quién?",
    body:
      "Pa' estudiantes y curiosos que quieren cachar cómo se construye un lenguaje, y pa' cualquiera que quiera codear en su propia jerga.",
    tone: 'rose',
  },
  {
    title: '¿Por qué existe?',
    body:
      'Porque aprender lenguajes se hace mejor cuando la sintaxis te habla en chileno, y porque un error también puede tener personalidad.',
    tone: 'blue',
  },
] as const;

const navLinks: Array<{ href: string; label: string; external?: boolean }> = [
  { href: '#manifiesto', label: 'Qué es' },
  { href: '#codigo', label: 'Código' },
  { href: '#errores', label: 'Errores' },
  { href: 'https://github.com/cuervolu/wn', label: 'GitHub', external: true },
];

function DecorativeStar({
  size,
  color,
  left,
  top,
  rotate,
  delay,
}: {
  size: number;
  color: string;
  left: string;
  top: string;
  rotate: number;
  delay: number;
}) {
  return (
    <span
      aria-hidden="true"
      className="wn-star"
      style={{
        width: `${size}px`,
        height: `${size}px`,
        left,
        top,
        background: color,
        rotate: `${rotate}deg`,
        animationDelay: `${delay}s`,
      }}
    />
  );
}

function LandingNav() {
  return (
    <header className="wn-nav">
      <div className="wn-nav__inner">
        <a className="wn-nav__brand" href="#top" aria-label="WN++ home">
          <img className="wn-nav__logo" src={logoSrc} alt="WN++" />
        </a>

        <nav className="wn-nav__links" aria-label="Landing">
          {navLinks.map((link) => (
            <a
              key={link.label}
              href={link.href}
              target={link.external ? '_blank' : undefined}
              rel={link.external ? 'noreferrer' : undefined}
            >
              {link.label}
            </a>
          ))}
        </nav>

        <div className="wn-nav__actions">
          <ThemeSwitch mode="light-dark" className="wn-theme-switch" />
          <StickerButton href="docs/" kind="ghost" size="sm">
            Docs
          </StickerButton>
          <StickerButton href="tour/" size="sm">
            Tour <ArrowUpRight size={16} strokeWidth={2.25} aria-hidden="true" />
          </StickerButton>
        </div>
      </div>
    </header>
  );
}

function RuntimePipeline() {
  return (
    <span className="wn-hero__tag-mono" aria-label="lexer, parser, AST, bytecode y VM">
      {runtimeStages.map((stage, index) => (
        <span key={stage} className="wn-hero__tag-stage">
          {index > 0 ? <ChevronRight size={14} strokeWidth={2.2} aria-hidden="true" /> : null}
          <span>{stage}</span>
        </span>
      ))}
    </span>
  );
}

function Hero() {
  return (
    <section className="wn-hero" id="top">
      <div className="wn-hero__decor">
        <DecorativeStar size={34} color="var(--wn-blue)" left="8%" top="22%" rotate={8} delay={0} />
        <DecorativeStar size={20} color="var(--wn-rose)" left="18%" top="62%" rotate={-12} delay={0.6} />
        <DecorativeStar size={26} color="var(--wn-rose)" left="86%" top="16%" rotate={14} delay={1.1} />
        <DecorativeStar size={16} color="var(--wn-blue)" left="78%" top="68%" rotate={0} delay={0.3} />
        <DecorativeStar size={22} color="var(--wn-blue)" left="48%" top="9%" rotate={20} delay={0.9} />
        <span className="wn-hero__glow wn-hero__glow--blue" />
        <span className="wn-hero__glow wn-hero__glow--rose" />
      </div>

      <div className="wn-hero__grid">
        <div className="wn-hero__lead">
          <a className="wn-hero__pill" href="https://github.com/cuervolu/wn" target="_blank" rel="noreferrer">
            <span className="wn-hero__pill-dot" />
            hecho en Chile · open source
          </a>

          <img className="wn-hero__logo" src={logoSrc} alt="Wn++ — Wea X = Wena, wn!" />

          <p className="wn-hero__tag">
            Un lenguaje de juguete que habla <em>como tú</em>. Aprende cómo funcionan los lenguajes por dentro{' '}
            <RuntimePipeline /> sin dejar de webear.
          </p>

          <div className="wn-hero__actions">
            <StickerButton href="tour/" size="lg">
              Hacer el tour
            </StickerButton>
            <StickerButton href="docs/" kind="ghost" size="lg">
              Leer los docs
            </StickerButton>
          </div>

          <div className="wn-hero__meta">
            <span>
              <b>dinámico</b> · tipado débil
            </span>
            <span className="wn-hero__meta-separator" />
            <span>
              archivos <code>.cl</code>
            </span>
            <span className="wn-hero__meta-separator" />
            <span>open source</span>
          </div>
        </div>

        <div className="wn-hero__code">
          <CodeWindow filename="wena.cl" code={heroCode} output={heroOutput} />
        </div>
      </div>
    </section>
  );
}

function Ribbon() {
  const items = [...ribbonItems, ...ribbonItems];

  return (
    <div className="wn-ribbon" aria-hidden="true">
      <div className="wn-ribbon__track">
        {items.map((item, index) => (
          <span key={`${item}-${index}`} className="wn-ribbon__item">
            {item}
            <span className="wn-ribbon__star">✦</span>
          </span>
        ))}
      </div>
    </div>
  );
}

function Manifesto() {
  return (
    <section className="wn-manifesto" id="manifiesto">
      <SectionHeading kicker="manifiesto" title="¿Qué es esta wea?" highlight="wea" />

      <div className="wn-manifesto__cards">
        {manifestoCards.map((card) => (
          <article key={card.title} className={`wn-manifesto-card wn-manifesto-card--${card.tone}`}>
            <h3>{card.title}</h3>
            <p>{card.body}</p>
          </article>
        ))}
      </div>

      <div className="wn-dictionary">
        <div className="wn-dictionary__head">
          <span className="wn-kicker wn-kicker--rose">diccionario</span>
          <h3>Tu jerga, tus keywords</h3>
          <p>Las palabras clave son chilenas. Esto es todo lo que necesitai pa&apos; empezar.</p>
        </div>

        <div className="wn-dictionary__grid">
          {keywords.map((keyword) => (
            <div key={keyword.kw} className="wn-dictionary__chip">
              <code>{keyword.kw}</code>
              <span>{keyword.en}</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function ExamplesSection() {
  const [activeExample, setActiveExample] = useState(examples[0]);

  return (
    <section className="wn-examples" id="codigo">
      <SectionHeading kicker="en acción" title="Código que corre de verdad" highlight="corre" />

      <div className="wn-examples__tabs" role="tablist" aria-label="Ejemplos de WN++">
        {examples.map((example) => (
          <button
            key={example.id}
            type="button"
            role="tab"
            aria-selected={example.id === activeExample.id}
            className={example.id === activeExample.id ? 'is-active' : undefined}
            onClick={() => setActiveExample(example)}
          >
            {example.label}
          </button>
        ))}
      </div>

      <div className="wn-examples__body">
        <div className="wn-examples__code">
          <CodeWindow
            filename={activeExample.filename}
            code={activeExample.code}
            output={activeExample.output}
            compact
          />
        </div>

        <aside className="wn-examples__note">
          <span className="wn-examples__note-label">{`// ${activeExample.label.toLowerCase()}`}</span>
          <p>{activeExample.blurb}</p>
          <a href="tour/">
            Probarlo en el tour <ArrowRight size={16} strokeWidth={2.25} aria-hidden="true" />
          </a>
        </aside>
      </div>
    </section>
  );
}

function ErrorsSection() {
  return (
    <section className="wn-errors" id="errores">
      <SectionHeading
        kicker="personalidad"
        title="Cuando la cagai, te funa"
        highlight="cagai"
        highlightTone="rose"
        subtitle="Los mensajes de error tienen humor local. Con cariño, eso sí po."
      />

      <div className="wn-errors__grid">
        {errors.map((error) => (
          <article key={error.code} className="wn-error-card">
            <div className="wn-error-card__top">
              <span className="wn-error-card__code">{error.code}</span>
              <span className="wn-error-card__bang">✗</span>
            </div>
            <p className="wn-error-card__when">Cuando {error.when}…</p>
            <div className="wn-error-card__message">
              <span className="wn-error-card__prompt">wn++ ✗</span>
              <span className="wn-error-card__text">{error.msg}</span>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function FooterCta() {
  return (
    <footer className="wn-footer">
      <div className="wn-footer__cta">
        <h2>¿Le entrai o no le entrai?</h2>
        <div className="wn-footer__actions">
          <StickerButton href="tour/" size="lg">
            Hacer el tour
          </StickerButton>
          <StickerButton href="docs/" kind="ghost" size="lg">
            Leer los docs
          </StickerButton>
        </div>
      </div>

      <div className="wn-footer__bar">
        <div className="wn-footer__brand">
          <img src={logoSrc} alt="WN++" />
          <code>Wea X = &quot;Wena, wn!&quot;</code>
        </div>

        <nav className="wn-footer__links" aria-label="Footer">
          <a href="docs/">Documentación</a>
          <a href="tour/">Tour interactivo</a>
          <a href="https://github.com/cuervolu/wn" target="_blank" rel="noreferrer">
            GitHub
          </a>
        </nav>

        <p className="wn-footer__fine">
          Hecho con ❤️ por <a href="https://cuervolu.dev" target="_blank" rel="noreferrer">cuervolu</a> ·{' '}
          Logo por <a href="https://instagram.com/iriata18" target="_blank" rel="noreferrer">@iriata18</a>
        </p>
      </div>
    </footer>
  );
}

export function LandingHomePage() {
  return (
    <div className="wn-landing">
      <LandingNav />
      <main>
        <Hero />
        <Ribbon />
        <Manifesto />
        <ExamplesSection />
        <ErrorsSection />
      </main>
      <FooterCta />
    </div>
  );
}
