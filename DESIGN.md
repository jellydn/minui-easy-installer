---
name: MinUI Easy Installer
version: alpha
description: Desktop app for installing and managing MinUI on retro handheld SD cards
colors:
  primary: "#ffffff"
  secondary: "#000000"
  surface: "#000000"
  text: "#ffffff"
  text-muted: "#999999"
  text-dim: "#888888"
  text-placeholder: "#777777"
  border: "#444444"
  border-hover: "#666666"
  success: "#ffffff"
  error: "#ffffff"
  warning: "#ffffff"
  overlay: "rgba(0, 0, 0, 0.9)"
  on-primary: "#000000"
  on-success: "#000000"
  on-error: "#000000"
  on-warning: "#000000"
  disabled-bg: "#111111"
  disabled-text: "#777777"
typography:
  h1:
    fontFamily: Inter, system-ui, -apple-system, sans-serif
    fontSize: 2rem
    fontWeight: 600
    color: "{colors.primary}"
  h2:
    fontFamily: Inter, system-ui, -apple-system, sans-serif
    fontSize: 1.125rem
    fontWeight: 600
    color: "{colors.primary}"
  body:
    fontFamily: Inter, system-ui, -apple-system, sans-serif
    fontSize: 0.9375rem
    lineHeight: 1.5
    fontWeight: 400
    color: "{colors.text}"
  subtitle:
    fontFamily: Inter, system-ui, -apple-system, sans-serif
    fontSize: 0.9375rem
    color: "{colors.text-muted}"
  label:
    fontFamily: Inter, system-ui, -apple-system, sans-serif
    fontSize: 0.8125rem
    fontWeight: 600
    textTransform: uppercase
    letterSpacing: 0.08em
    color: "{colors.text-muted}"
  small:
    fontFamily: Inter, system-ui, -apple-system, sans-serif
    fontSize: 0.8125rem
    color: "{colors.text-muted}"
  caption:
    fontFamily: Inter, system-ui, -apple-system, sans-serif
    fontSize: 0.75rem
    color: "{colors.text-dim}"
rounded:
  sm: 4px
  md: 6px
  lg: 8px
spacing:
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
components:
  nav-btn:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 8px 0
    width: 120px
    typography: "{typography.body}"
  nav-btn-active:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.sm}"
    padding: 8px 0
    width: 120px
    typography: "{typography.body}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.sm}"
    padding: 10px 20px
    typography: "{typography.body}"
  button-secondary:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 10px 20px
    typography: "{typography.body}"
  card:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.lg}"
    padding: 1.5rem
  card-ready:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.lg}"
    padding: 1.5rem
  select:
    backgroundColor: "{colors.secondary}"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 10px 12px
    typography: "{typography.body}"
  select-option:
    backgroundColor: "{colors.secondary}"
    textColor: "{colors.text}"
  input:
    backgroundColor: "{colors.secondary}"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 10px 12px
    typography: "{typography.body}"
  device-card:
    backgroundColor: "{colors.secondary}"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 12px
  device-card-selected:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.sm}"
    padding: 12px
  package-card:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.lg}"
    padding: 1rem
  package-category:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 2px 6px
  confirm-dialog:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.lg}"
    padding: 1.5rem
  confirm-overlay:
    backgroundColor: "{colors.overlay}"
  confirm-warning:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 12px
  confirm-cancel:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 10px 20px
  install-btn:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.sm}"
    padding: 8px 16px
  installed-badge:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 8px 16px
  error-card:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 12px
  success-card:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 12px
  validation-check-passed:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 6px 10px
  validation-check-failed:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 6px 10px
  update-all-btn:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.sm}"
    padding: 10px 20px
  health-check-passed:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: 6px
  health-check-warning:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 6px
  wifi-cancel:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 10px 20px
  wifi-save:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.sm}"
    padding: 10px 20px
  rescan-btn:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 6px 12px
  scan-failed:
    backgroundColor: "transparent"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.sm}"
    padding: 4px 0
---

## Overview

A minimal, focused desktop installer that mirrors MinUI's monochrome aesthetic.
Pure black and white — no color, no distraction. The interface prioritizes
clarity and safety, requiring explicit confirmation before any write operation.
Every element earns its place. The visual language is one of restraint: high
contrast for readability, generous whitespace for breathing room, and consistent
typography for hierarchy.

## Colors

The palette is strictly monochrome — black, white, and functional grays only.

- **Primary (#ffffff):** Pure white — used for text, interactive elements, and active states.
- **Secondary (#000000):** Pure black — page background, input backgrounds.
- **Surface (#000000):** Card backgrounds blend with the page — depth is conveyed by borders and spacing, not color.
- **Text (#ffffff):** White for all readable content.
- **Text Muted (#999999):** Gray for secondary information, labels, and inactive states. 7.15:1 contrast — AAA.
- **Text Dim (#888888):** Darker gray for captions and metadata. 8.68:1 contrast — AAA.
- **Text Placeholder (#777777):** Input placeholder text. 10.5:1 contrast — AAA.
- **Border (#444444):** Subtle borders for cards, inputs, and containers.
- **Border Hover (#666666):** Brighter borders on hover and focus states.
- **Overlay:** 90% black backdrop for modal dialogs.
- **Disabled (#111111 bg, #777 text):** Muted states for unavailable actions.

An agent that reads this file will produce a stark, monochrome UI with white
text on black backgrounds, subtle gray borders, and no color accents.

## Typography

A single typeface with weight variations provides all hierarchy:

- **Inter/system-ui** — clean, legible, no custom font loading.
- **H1 (2rem, semibold):** App title and major headings.
- **H2 (1.125rem, semibold):** Section headers within cards.
- **Body (0.9375rem, 1.5 line-height):** Default text for content and buttons.
- **Subtitle (0.9375rem):** Description text, muted gray.
- **Label (0.8125rem, semibold, uppercase, 0.08em tracking):** Section labels in dialogs and reports.
- **Small (0.8125rem):** Metadata like drive details, package versions.
- **Caption (0.75rem):** Package category badges, tertiary metadata.

## Layout

Centered single-column layout with tab navigation:

- **Container:** Full viewport, centered, with 10vh top padding.
- **Screen:** All tabs share a `.screen` container — 600px max-width, centered horizontally with 20px horizontal padding.
- **Navigation:** Horizontal tab bar with three text buttons. Active tab inverts to white background with black text. Inactive tabs are muted gray.
- **Cards:** Borderless containers with subtle 1px borders. Cards stack vertically with consistent spacing.
- **Device Grid:** Two-column responsive grid (auto-fill, minmax 200px) for device selection.
- **Package Grid:** Three-column responsive grid (auto-fill, minmax 280px) for package cards.

## Elevation & Depth

Depth is communicated through border contrast and spacing — no shadows, no gradients:

- **Base:** #000000 (page and card backgrounds merge)
- **Border:** #333333 (subtle separation between elements)
- **Border Hover:** #555555 (interactive feedback)
- **Active:** White background with black text (selected states)

## Shapes

- **Border Radius:** Three scales — sm (4px) for buttons and small elements, md (6px) for medium elements, lg (8px) for cards and dialogs.
- **No decorative elements:** No shadows, gradients, or ornamentation.
- **Spinner:** 32px circle with 2px white border, animated rotation for loading states.

## Components

### Navigation Tabs

Horizontal text button row. Active tab inverts to white background with black
text. Inactive tabs are muted gray. Tabs switch between Home, Package Store,
and WiFi Setup screens.

### Device Cards

Grid of selectable cards. Each shows device name (semibold) and platform in
muted text. Selected state inverts to white background with black text. Cards
have fixed minimum height (80px) for visual consistency.

### Drive Cards

List of removable drives. Each shows drive name (semibold) and size/filesystem
in muted text. Selected state mirrors device card inversion.

### Button - Primary

White background (#ffffff) with black text (#000000). Used for main CTAs:
Install, Save, Proceed. No hover animation — state change is immediate.

### Button - Secondary

Transparent background with muted text. Used for Cancel, Done, and Copy actions.
Hover brightens text to white.

### Card

Black background with subtle 1px border (#333333), 8px border radius, 1.5rem
padding. Cards blend into the page — separation comes from border and spacing
alone.

### Select / Input

Black background with white text and 4px border radius. Focus state shows
brighter border. Placeholder text uses dim gray.

### Confirmation Dialog

Modal overlay with 90% black backdrop. Dialog card contains warning text,
device/drive details in labeled sections, and Cancel/Proceed action buttons.
Cancel uses secondary styling, Proceed uses primary inversion.

### Package Cards

Three-column grid items with package name, version, author, category label,
description, and install button. Category uses muted text. Installed packages
show muted "Installed" label instead of install button.

### Install Button

Primary action button. Full-width on package cards. Disabled state uses muted
background with dim text. "Update All" uses same primary styling.

### WiFi Wizard

Form-based flow with SSID selector (dropdown or manual input), password field,
and Cancel/Save actions. Scanning state shows spinner and "Scanning networks..."
hint in muted italic. Failed scan shows muted warning text.

### Install Progress

Centered layout with rotating spinner, phase message, and file copy counters.
Success state shows white text summary. Error state shows muted error text with
retry hint.

### Validation Report

Post-install checklist with pass/fail indicators per check item. Passed items
show white text. Failed items show muted text. Includes disk space summary and
Copy Report/Done actions.

### Health Check

Section showing system checks. Passed checks show white text. Warnings show
muted text. Includes filesystem and disk space info.

## Do's and Don'ts

- **Do** use pure black (#000000) and pure white (#ffffff) as primary colors.
- **Do** use muted gray (#999999) for secondary information and inactive states.
- **Do** require explicit confirmation before any write operation to the SD card.
- **Do** show device and drive information at all times when selected.
- **Do** keep the interface minimal — every element must earn its place.
- **Don't** use any color — MinUI is strictly monochrome.
- **Don't** add shadows, gradients, or decorative elements.
- **Don't** use hover animations — state changes should be immediate.
- **Don't** auto-proceed with installation — always wait for user confirmation.
- **Don't** add unnecessary visual complexity — simplicity is the goal.
