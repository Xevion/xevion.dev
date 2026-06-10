import { defineConfig, defineRecipe } from "@pandacss/dev";

export default defineConfig({
  preflight: true,

  include: ["./src/**/*.{js,ts,svelte}"],

  exclude: [],

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
        },
        fonts: {
          inter: { value: '"Inter Variable", sans-serif' },
          hanken: { value: '"Hanken Grotesk", sans-serif' },
          schibsted: { value: '"Schibsted Grotesk Variable", sans-serif' },
        },
        fontSizes: {
          "10xl": { value: "10rem" },
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
    "::view-transition-old(background), ::view-transition-old(theme-toggle), ::view-transition-old(scrollbar-h), ::view-transition-old(scrollbar-v)":
      {
        display: "none",
      },
    "::view-transition-new(background), ::view-transition-new(theme-toggle), ::view-transition-new(scrollbar-h), ::view-transition-new(scrollbar-v)":
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
    // Media mask for project card backgrounds
    ".media-mask-fade-left": {
      maskImage:
        "linear-gradient(to right, transparent 0%, rgba(0,0,0,0.08) 12%, rgba(0,0,0,0.2) 22%, rgba(0,0,0,0.4) 32%, rgba(0,0,0,0.65) 42%, rgba(0,0,0,0.85) 52%, black 65%)",
    },
  },

  outdir: "styled-system",
});
