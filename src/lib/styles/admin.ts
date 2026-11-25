/**
 * Shared admin UI styles.
 *
 * Extracted from repeated inline css() calls across admin components
 * to enforce consistency and reduce duplication.
 */
import { css, cva } from "styled-system/css";

/** Standard form field label (display:block + textStyle) */
export const labelClass = css({
  display: "block",
  textStyle: "admin.label",
  color: "admin.text",
});

/** Help text below form inputs */
export const helpTextClass = css({
  textStyle: "admin.helpText",
  color: "admin.textMuted",
});

/** Error message below form inputs */
export const errorTextClass = css({
  textStyle: "admin.helpText",
  color: "red.500",
});

/** Standard spacing wrapper for form fields */
export const fieldWrapperClass = css({ spaceY: "1.5" });

/** Admin page title */
export const pageTitleClass = css({
  textStyle: "admin.pageTitle",
  color: "admin.text",
});

/** Admin page subtitle / description */
export const pageDescriptionClass = css({
  textStyle: "admin.pageDescription",
  color: "admin.textMuted",
});

/** Section heading inside cards/panels */
export const sectionTitleClass = css({
  textStyle: "admin.sectionTitle",
  color: "admin.text",
});

/** Admin card / panel (rounded bordered surface with shadow) */
export const adminCardClass = css({
  rounded: "xl",
  borderWidth: "1px",
  borderColor: "admin.border",
  bg: "admin.surface",
  p: "6",
  shadow: "sm",
  shadowColor: "black/10",
  _dark: { shadowColor: "black/20" },
});

/** Dropdown / popover panel (shared by TagPicker, IconPicker, etc.) */
export const dropdownPanelClass = css({
  position: "absolute",
  zIndex: 10,
  mt: "1",
  w: "full",
  overflowY: "auto",
  rounded: "md",
  borderWidth: "1px",
  borderColor: "admin.border",
  bg: "admin.surface",
  shadow: "lg",
});

export const iconSm = css({ w: "4", h: "4" });
export const iconMd = css({ w: "5", h: "5" });
export const iconLg = css({ w: "6", h: "6" });

/** Admin input base styles (shared by Input, ColorPicker, IconPicker, etc.) */
export const adminInputBase = css({
  display: "block",
  w: "full",
  rounded: "md",
  borderWidth: "1px",
  borderColor: "admin.border",
  bg: "admin.bgSecondary",
  px: "3",
  py: "2",
  fontSize: "sm",
  color: "admin.text",
  _placeholder: { color: "admin.textMuted" },
  _focus: {
    borderColor: "admin.accent",
    outline: "none",
    ringWidth: "1px",
    ringColor: "admin.accent",
  },
  _disabled: { cursor: "not-allowed", opacity: "0.5" },
  transition: "colors",
});

/** Error border override for inputs */
export const adminInputError = css({
  borderColor: "red.500",
  _focus: { borderColor: "red.500", ringColor: "red.500" },
});

/** Section heading (used inside cards: "Create New Tag", "Site Identity", etc.) */
export const sectionHeadingClass = css({
  fontSize: "base",
  fontWeight: "medium",
  color: "admin.text",
  mb: "4",
});

/** Settings tab recipe */
export const settingsTab = cva({
  base: {
    display: "inline-block",
    textDecoration: "none",
    pb: "3",
    px: "1",
    fontSize: "sm",
    fontWeight: "medium",
    borderBottomWidth: "2px",
    transition: "colors",
  },
  variants: {
    state: {
      active: {
        borderColor: "admin.accent",
        color: "admin.text",
      },
      inactive: {
        borderColor: "transparent",
        color: "admin.textMuted",
        _hover: { color: "admin.text", borderColor: "admin.borderHover" },
      },
    },
  },
  defaultVariants: {
    state: "inactive",
  },
});
