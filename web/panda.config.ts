import { defineConfig, defineRecipe } from "@pandacss/dev";

export default defineConfig({
  preflight: true,

  include: ["./src/**/*.{js,ts,svelte}"],

  exclude: [],

  // Runtime-dynamic theming color (author-set per project, unknown at build
  // time, so not a token). Set via `style="--accent: …"` on a container and
  // consumed with `var(--accent)` / `color-mix(… var(--accent) …)` in css().
  // Registered as a typed custom property so it animates and autocompletes.
  globalVars: {
    "--accent": {
      syntax: "<color>",
      inherits: true,
      initialValue: "#71717a",
    },
    // Legible black/white ink for text painted on top of a solid --accent fill.
    "--accent-ink": {
      syntax: "<color>",
      inherits: true,
      initialValue: "#ffffff",
    },
  },

  // Class-based dark mode using .dark on <html>
  conditions: {
    extend: {
      dark: ".dark &, [data-theme=dark] &",
    },
  },

  theme: {
    extend: {
      tokens: {
        colors: {
          // Extra zinc shade not in default palette
          "zinc.850": { value: "#1d1d20" },
          // Controls layered over media (lightbox, gallery tiles). Theme-
          // independent — they always sit on imagery, not the page surface.
          overlay: {
            scrim: { value: "rgba(9, 9, 11, 0.9)" },
            control: { value: "rgba(255, 255, 255, 0.12)" },
            controlHover: { value: "rgba(255, 255, 255, 0.22)" },
            badge: { value: "rgba(255, 255, 255, 0.86)" },
          },
          // CLI-hero terminal palette. Always-dark (the cast doesn't follow the
          // site theme), so these are flat values, not semantic light/dark pairs.
          terminal: {
            bg: { value: "#1b1a18" },
            head: { value: "#211f1c" },
            border: { value: "#2e2b27" },
            dim: { value: "#6f6a62" },
            text: { value: "#d8d4cc" },
            cmd: { value: "#f2efe9" },
            muted: { value: "#8a857c" },
            err: { value: "#e0664f" },
          },
        },
        fonts: {
          inter: { value: '"Inter Variable", sans-serif' },
          hanken: { value: '"Hanken Grotesk", sans-serif' },
          schibsted: { value: '"Schibsted Grotesk Variable", sans-serif' },
          geist: { value: '"Geist Mono", ui-monospace, monospace' },
        },
        fontSizes: {
          "10xl": { value: "10rem" },
          // Named ramp for the redesign's recurring (3+×) sizes. One-offs (the
          // 38px hero h1, 28px mobile h1, 18.5px lede) stay arbitrary px.
          meta: { value: "11px" }, // mono meta: ages, slugs, related type
          metaLg: { value: "11.5px" }, // status label, mobile pill
          caption: { value: "12.5px" }, // figcaptions, terminal body
          bodySm: { value: "13.5px" }, // rail facts, card/row blurb, buttons
          body: { value: "15.5px" }, // prose paragraph
          title: { value: "16.5px" }, // card + row project title
        },
      },
      semanticTokens: {
        colors: {
          bg: {
            primary: {
              value: { base: "#ffffff", _dark: "#000000" },
            },
            secondary: {
              value: { base: "#f4f4f5", _dark: "#09090b" },
            },
          },
          surface: {
            DEFAULT: {
              value: { base: "#ffffff", _dark: "#18181b" },
            },
            secondary: {
              value: { base: "#fafafa", _dark: "#27272a" },
            },
          },
          border: {
            DEFAULT: {
              value: { base: "#e4e4e7", _dark: "#27272a" },
            },
            subtle: {
              value: { base: "#f4f4f5", _dark: "#18181b" },
            },
            // Faint divider/rule hairline (section underlines, rail edges).
            hairline: {
              value: { base: "#ececee", _dark: "#27272a" },
            },
          },
          text: {
            primary: {
              value: { base: "#18181b", _dark: "#fafafa" },
            },
            secondary: {
              value: { base: "#52525b", _dark: "#d4d4d8" },
            },
            tertiary: {
              value: { base: "#71717a", _dark: "#a1a1aa" },
            },
          },
          admin: {
            bg: {
              value: { base: "#f9fafb", _dark: "#0a0a0b" },
            },
            bgSecondary: {
              value: { base: "#ffffff", _dark: "#18181b" },
            },
            surface: {
              value: { base: "#ffffff", _dark: "#27272a" },
            },
            surfaceHover: {
              value: { base: "#f3f4f6", _dark: "#3f3f46" },
            },
            border: {
              value: { base: "#e5e7eb", _dark: "#27272a" },
            },
            borderHover: {
              value: { base: "#d1d5db", _dark: "#3f3f46" },
            },
            text: {
              value: { base: "#111827", _dark: "#fafafa" },
            },
            textSecondary: {
              value: { base: "#4b5563", _dark: "#a1a1aa" },
            },
            textMuted: {
              value: { base: "#6b7280", _dark: "#71717a" },
            },
            accent: {
              value: { base: "#6366f1", _dark: "#6366f1" },
            },
            accentHover: {
              value: { base: "#818cf8", _dark: "#818cf8" },
            },
          },
        },
      },
      textStyles: {
        // Uppercase mono micro-label (rail fact labels, "Built with").
        "label.micro": {
          value: {
            fontFamily: "geist",
            fontSize: "10px",
            fontWeight: "500",
            letterSpacing: "0.07em",
            textTransform: "uppercase",
            color: "zinc.400",
          },
        },
        // Section heading (Gallery head + prose <h2>s).
        "heading.section": {
          value: {
            fontSize: "19px",
            fontWeight: "700",
            letterSpacing: "-0.01em",
            color: "zinc.900",
            _dark: { color: "white" },
          },
        },
        // Uppercase mono section eyebrow ("Related work").
        "label.eyebrow": {
          value: {
            fontFamily: "geist",
            fontSize: "13px",
            fontWeight: "500",
            letterSpacing: "0.06em",
            textTransform: "uppercase",
            color: "zinc.400",
          },
        },
        // Uppercase mono status label (rail + mobile summary). Caller sets the
        // color (status hue); the rest of the recipe is shared so the three
        // former copies of this rule can't drift.
        "label.status": {
          value: {
            fontFamily: "geist",
            fontSize: "metaLg",
            fontWeight: "500",
            letterSpacing: "0.06em",
            textTransform: "uppercase",
          },
        },
        "admin.pageTitle": {
          value: {
            fontSize: "xl",
            fontWeight: "semibold",
          },
        },
        "admin.pageDescription": {
          value: {
            fontSize: "sm",
          },
        },
        "admin.label": {
          value: {
            fontSize: "sm",
            fontWeight: "medium",
          },
        },
        "admin.helpText": {
          value: {
            fontSize: "xs",
          },
        },
        "admin.sectionTitle": {
          value: {
            fontSize: "lg",
            fontWeight: "semibold",
          },
        },
      },
      recipes: {
        button: defineRecipe({
          className: "button",
          description: "Admin button with multiple variants and sizes",
          base: {
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            fontWeight: "medium",
            transition: "all",
            _focusVisible: {
              outline: "none",
              ringWidth: "2px",
              ringOffset: "2px",
            },
            _disabled: {
              pointerEvents: "none",
              opacity: "0.5",
            },
          },
          variants: {
            variant: {
              primary: {
                bg: "admin.accent",
                color: "white",
                shadow: "sm",
                _hover: { bg: "admin.accentHover", shadow: "sm" },
                _focusVisible: { ringColor: "admin.accent" },
              },
              secondary: {
                bg: { base: "zinc.200/60", _dark: "zinc.600/50" },
                color: "admin.text",
                borderWidth: "1px",
                borderColor: { base: "zinc.400/50", _dark: "zinc.700" },
                _hover: {
                  borderColor: { base: "zinc.400", _dark: "zinc.500" },
                  bg: { base: "zinc.300/70", _dark: "zinc.500/60" },
                },
                _focusVisible: { ringColor: "admin.accent" },
              },
              danger: {
                bg: "red.600",
                color: "white",
                shadow: "sm",
                _hover: { bg: "red.500", shadow: "sm" },
                _focusVisible: { ringColor: "red.500" },
              },
              ghost: {
                color: "admin.text",
                _hover: { bg: "admin.surfaceHover" },
                _focusVisible: { ringColor: "admin.accent" },
              },
            },
            size: {
              sm: { h: "8", px: "3", fontSize: "sm", rounded: "sm" },
              md: { h: "9", px: "4", fontSize: "sm", rounded: "md" },
              lg: { h: "11", px: "6", fontSize: "base", rounded: "md" },
            },
          },
          defaultVariants: {
            variant: "primary",
            size: "md",
          },
        }),
        badge: defineRecipe({
          className: "badge",
          description: "Status badge for projects and events",
          base: {
            display: "inline-flex",
            alignItems: "center",
            rounded: "full",
            px: "2.5",
            py: "0.5",
            fontSize: "xs",
            fontWeight: "medium",
          },
          variants: {
            variant: {
              active: { bg: "emerald.500/10", color: "emerald.400" },
              maintained: { bg: "indigo.500/10", color: "indigo.400" },
              archived: { bg: "zinc.500/10", color: "zinc.400" },
              hidden: { bg: "zinc.500/10", color: "zinc.500" },
              info: { bg: "teal.500/10", color: "teal.400" },
              warning: { bg: "amber.500/10", color: "amber.400" },
              error: { bg: "red.500/10", color: "red.400" },
              default: { bg: "zinc.500/10", color: "zinc.400" },
            },
          },
          defaultVariants: {
            variant: "default",
          },
        }),
        iconButton: defineRecipe({
          className: "icon-button",
          description: "Icon-only button for theme toggle and admin actions",
          base: {
            position: "relative",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            rounded: "md",
            borderWidth: "1px",
            cursor: "pointer",
            transition: "all",
            transitionDuration: "200ms",
          },
          variants: {
            variant: {
              surface: {
                borderColor: "zinc.300",
                bg: "zinc.100",
                _hover: { bg: "zinc.200" },
                _dark: {
                  borderColor: "zinc.700",
                  bg: "zinc.900/50",
                  _hover: { bg: "zinc.800/70" },
                },
              },
            },
            size: {
              md: { w: "9", h: "9" },
            },
          },
          defaultVariants: {
            variant: "surface",
            size: "md",
          },
        }),
        prose: defineRecipe({
          className: "prose",
          description:
            "Long-form reading rhythm over sanitized TipTap HTML. §NN markers are CSS counters on <h2>; var(--accent) tints markers, list squares, inline code and the pull-quote rule. Reused by project detail + (future) blog.",
          base: {
            color: "zinc.700",
            counterReset: "rd-section",
            _dark: { color: "zinc.300" },
            "& h2": {
              textStyle: "heading.section",
              display: "flex",
              alignItems: "center",
              gap: "12px",
              counterIncrement: "rd-section",
              m: "30px 0 12px",
            },
            "& h2::before": {
              content: '"\\00a7" counter(rd-section, decimal-leading-zero)',
              fontFamily: "geist",
              fontSize: "caption",
              fontWeight: "600",
              color: "var(--accent)",
              flexShrink: "0",
              whiteSpace: "nowrap",
            },
            "& h2::after": {
              content: '""',
              flex: "1",
              h: "1px",
              bg: "border.hairline",
            },
            "& h3": {
              fontSize: "17px",
              fontWeight: "700",
              color: "zinc.900",
              m: "24px 0 8px",
              _dark: { color: "zinc.100" },
            },
            "& h4": {
              fontSize: "15px",
              fontWeight: "600",
              color: "zinc.900",
              m: "20px 0 6px",
              _dark: { color: "zinc.100" },
            },
            "& p": {
              fontSize: "body",
              lineHeight: "1.72",
              color: "zinc.700",
              m: "0 0 14px",
              textWrap: "pretty",
              _dark: { color: "zinc.300" },
            },
            "& ul": {
              listStyle: "none",
              p: "0",
              m: "0 0 16px",
              display: "flex",
              flexDirection: "column",
              gap: "5px",
            },
            "& ul > li": {
              position: "relative",
              pl: "20px",
              fontSize: "15px",
              lineHeight: "1.6",
              color: "zinc.700",
              _dark: { color: "zinc.300" },
            },
            "& ul > li::before": {
              content: '""',
              position: "absolute",
              left: "2px",
              top: "9px",
              w: "5px",
              h: "5px",
              rounded: "1px",
              bg: "var(--accent)",
            },
            "& ol": {
              listStyle: "decimal",
              pl: "22px",
              m: "0 0 16px",
              display: "flex",
              flexDirection: "column",
              gap: "5px",
            },
            "& ol > li": {
              fontSize: "15px",
              lineHeight: "1.6",
              color: "zinc.700",
              _dark: { color: "zinc.300" },
            },
            "& a": {
              color: "blue.600",
              textDecoration: "underline",
              _dark: { color: "blue.400" },
            },
            "& code": {
              fontFamily: "geist",
              fontSize: "0.86em",
              bg: "zinc.100",
              borderWidth: "1px",
              borderColor: "zinc.200",
              rounded: "4px",
              px: "5px",
              py: "1px",
              color: "var(--accent)",
              whiteSpace: "nowrap",
              _dark: { bg: "zinc.800", borderColor: "zinc.700" },
            },
            "& kbd": {
              fontFamily: "geist",
              fontSize: "metaLg",
              display: "inline-flex",
              alignItems: "center",
              minH: "18px",
              px: "6px",
              py: "1px",
              borderWidth: "1px",
              borderBottomWidth: "2px",
              borderColor: "zinc.300",
              rounded: "5px",
              bg: "surface",
              color: "zinc.700",
              shadow: "0 1px 0 rgba(24,24,27,.04)",
              whiteSpace: "nowrap",
              _dark: { borderColor: "zinc.600", color: "zinc.300" },
            },
            "& blockquote": {
              position: "relative",
              m: "22px 0",
              p: "2px 0 2px 22px",
              borderLeftWidth: "3px",
              borderColor: "var(--accent)",
              fontSize: "17.5px",
              lineHeight: "1.55",
              color: "zinc.800",
              textWrap: "pretty",
              _dark: { color: "zinc.200" },
            },
            "& hr": {
              borderColor: "zinc.200",
              my: "6",
              _dark: { borderColor: "zinc.700" },
            },
            "& img": { maxW: "full", rounded: "md", my: "4" },
            "& .rd-figure": { m: "24px 0" },
            "& .rd-figure-media": {
              display: "block",
              w: "full",
              rounded: "lg",
              borderWidth: "1px",
              borderColor: "border.hairline",
              bg: "zinc.50",
              _dark: { bg: "zinc.900" },
            },
            "& .rd-figure-cap": {
              mt: "8px",
              fontSize: "caption",
              lineHeight: "1.5",
              color: "zinc.500",
              textAlign: "center",
              textWrap: "pretty",
              _dark: { color: "zinc.400" },
            },
            "& .gloss": {
              borderBottomWidth: "1px",
              borderBottomStyle: "dotted",
              borderColor: "color-mix(in srgb, var(--accent) 60%, transparent)",
              cursor: "help",
              position: "relative",
              outline: "none",
            },
            "& .gloss::after": {
              content: "attr(data-note)",
              position: "absolute",
              left: "0",
              top: "calc(100% + 7px)",
              zIndex: "20",
              w: "max-content",
              maxW: "260px",
              p: "8px 10px",
              rounded: "7px",
              bg: "zinc.900",
              color: "zinc.50",
              fontSize: "metaLg",
              lineHeight: "1.45",
              whiteSpace: "normal",
              textWrap: "pretty",
              boxShadow: "0 8px 20px -8px rgba(0,0,0,.4)",
              opacity: "0",
              visibility: "hidden",
              transform: "translateY(-2px)",
              transition: "opacity .12s, transform .12s, visibility .12s",
              pointerEvents: "none",
              _dark: { bg: "zinc.100", color: "zinc.900" },
            },
            "& .gloss:hover::after, & .gloss:focus::after, & .gloss:focus-visible::after":
              {
                opacity: "1",
                visibility: "visible",
                transform: "translateY(0)",
              },
            "& :where(h2, h3, h4)": { clear: "right" },
            "& .rd-sidenote": {
              borderLeftWidth: "2px",
              borderColor: "var(--accent)",
              bg: "color-mix(in srgb, var(--accent) 5%, transparent)",
              rounded: "0 6px 6px 0",
              p: "10px 14px",
              m: "8px 0 16px",
              "& p": {
                fontSize: "caption",
                lineHeight: "1.55",
                m: "0",
                color: "zinc.600",
                _dark: { color: "zinc.400" },
              },
              "& p + p": { mt: "8px" },
              "@media (min-width: 880px)": {
                float: "right",
                clear: "right",
                w: "210px",
                ml: "26px",
                my: "6px",
              },
            },
          },
        }),
      },
      keyframes: {
        "scrollbar-fade-in": {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        "vt-slide-to-left": {
          from: { transform: "translateX(0)", opacity: "1" },
          to: { transform: "translateX(-20px)", opacity: "0" },
        },
        "vt-slide-from-right": {
          from: { transform: "translateX(20px)", opacity: "0" },
          to: { transform: "translateX(0)", opacity: "1" },
        },
      },
    },
  },

  // Global styles for html/body and non-component CSS
  globalCss: {
    "html, body": {
      fontFamily: "inter",
      overflowX: "hidden",
      color: "text.primary",
    },
    body: {
      height: "100%",
      backgroundColor: "bg.primary",
    },
    // OverlayScrollbars theming
    "html:not(.dark) .os-scrollbar": {
      "--os-handle-bg": "rgba(0, 0, 0, 0.25) !important",
      "--os-handle-bg-hover": "rgba(0, 0, 0, 0.35) !important",
      "--os-handle-bg-active": "rgba(0, 0, 0, 0.45) !important",
    },
    "html.dark .os-scrollbar": {
      "--os-handle-bg": "rgba(255, 255, 255, 0.35) !important",
      "--os-handle-bg-hover": "rgba(255, 255, 255, 0.45) !important",
      "--os-handle-bg-active": "rgba(255, 255, 255, 0.55) !important",
    },
    ".os-scrollbar-handle": {
      borderRadius: "4px",
    },
    // Body scrollbar view-transition-names
    "body > .os-scrollbar-horizontal": {
      viewTransitionName: "scrollbar-h",
    },
    "body > .os-scrollbar-vertical": {
      viewTransitionName: "scrollbar-v",
    },
    ".os-scrollbar": {
      animation: "scrollbar-fade-in 300ms ease-out",
    },
    // Native scrollbars (Webkit) - light mode
    "html:not(.dark) ::-webkit-scrollbar": {
      width: "10px",
      height: "10px",
    },
    "html:not(.dark) ::-webkit-scrollbar-track": {
      background: "rgba(0, 0, 0, 0.05)",
      borderRadius: "4px",
    },
    "html:not(.dark) ::-webkit-scrollbar-thumb": {
      background: "rgba(0, 0, 0, 0.25)",
      borderRadius: "4px",
    },
    "html:not(.dark) ::-webkit-scrollbar-thumb:hover": {
      background: "rgba(0, 0, 0, 0.35)",
    },
    "html:not(.dark) ::-webkit-scrollbar-thumb:active": {
      background: "rgba(0, 0, 0, 0.45)",
    },
    // Native scrollbars (Webkit) - dark mode
    "html.dark ::-webkit-scrollbar": {
      width: "10px",
      height: "10px",
    },
    "html.dark ::-webkit-scrollbar-track": {
      background: "rgba(255, 255, 255, 0.05)",
      borderRadius: "4px",
    },
    "html.dark ::-webkit-scrollbar-thumb": {
      background: "rgba(255, 255, 255, 0.35)",
      borderRadius: "4px",
    },
    "html.dark ::-webkit-scrollbar-thumb:hover": {
      background: "rgba(255, 255, 255, 0.45)",
    },
    "html.dark ::-webkit-scrollbar-thumb:active": {
      background: "rgba(255, 255, 255, 0.55)",
    },
    // Native scrollbars (Firefox)
    ".native-scrollbar": {
      scrollbarWidth: "thin",
    },
    "html:not(.dark) .native-scrollbar": {
      scrollbarColor: "rgba(0, 0, 0, 0.25) rgba(0, 0, 0, 0.05)",
    },
    "html.dark .native-scrollbar": {
      scrollbarColor: "rgba(255, 255, 255, 0.35) rgba(255, 255, 255, 0.05)",
    },
    // Code-block chrome (github canvas + scrollbar) shared by the in-editor block
    // (lowlight, ContentEditor.svelte) and the public detail page block (Shiki,
    // projects/[slug]). Token colors are NOT shared — the editor approximates via
    // local `--cb-*` vars; the public page uses Shiki's inline per-token colors.
    html: {
      "--code-canvas": "#f6f8fa",
      "--code-scrollbar": "#afb8c1",
    },
    "html.dark": {
      "--code-canvas": "#161b22",
      "--code-scrollbar": "#30363d",
    },
    // Hide native scrollbar on html/body - OverlayScrollbars handles body scrolling
    "html, body ": {
      scrollbarWidth: "none",
      msOverflowStyle: "none",
    },
    "html::-webkit-scrollbar, body::-webkit-scrollbar": {
      display: "none",
    },
    // Page main utility
    ".page-main": {
      position: "relative",
      minHeight: "100vh",
      color: "{colors.zinc.900}",
      ".dark &": {
        color: "{colors.zinc.50}",
      },
    },
    // View Transitions API - theme toggle animation
    "::view-transition-old(root), ::view-transition-new(root)": {
      animation: "none",
      mixBlendMode: "normal",
    },
    // Persistent elements excluded from page transitions
    "::view-transition-old(background), ::view-transition-old(theme-toggle), ::view-transition-old(scrim), ::view-transition-old(scrollbar-h), ::view-transition-old(scrollbar-v)":
      {
        display: "none",
      },
    "::view-transition-new(background), ::view-transition-new(theme-toggle), ::view-transition-new(scrim), ::view-transition-new(scrollbar-h), ::view-transition-new(scrollbar-v)":
      {
        animation: "none",
      },
    // Page content transition
    "::view-transition-old(page-content)": {
      animation: "vt-slide-to-left 250ms cubic-bezier(0.4, 0, 0.2, 1) both",
    },
    "::view-transition-new(page-content)": {
      animation: "vt-slide-from-right 250ms cubic-bezier(0.4, 0, 0.2, 1) both",
    },
    // Index ↔ detail "morph": the opening card's cover + title fly into the
    // detail hero + h1. Each named element forms its own transition group that
    // animates independently of the page-content slide.
    "@media (prefers-reduced-motion: no-preference)": {
      "::view-transition-group(project-cover), ::view-transition-group(project-title)":
        {
          animationDuration: "520ms",
          animationTimingFunction: "cubic-bezier(.32, .72, 0, 1)",
        },
      "::view-transition-old(project-cover), ::view-transition-new(project-cover)":
        {
          mixBlendMode: "normal",
        },
    },
    // Media mask for project card backgrounds
    ".media-mask-fade-left": {
      maskImage:
        "linear-gradient(to right, transparent 0%, rgba(0,0,0,0.08) 12%, rgba(0,0,0,0.2) 22%, rgba(0,0,0,0.4) 32%, rgba(0,0,0,0.65) 42%, rgba(0,0,0,0.85) 52%, black 65%)",
    },
  },

  outdir: "styled-system",
});
