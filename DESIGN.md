---
name: "CoquiPCCleaner"
description: "Native macOS analysis with clear, inspectable evidence."
colors:
  dark-canvas: "#211f23"
  dark-sidebar: "#28272a"
  dark-surface: "#302e33"
  dark-elevated: "#3a373d"
  dark-border: "#434047"
  dark-text: "#f4f2f5"
  dark-secondary: "#b3b0b7"
  dark-accent: "#68d9c0"
  dark-on-accent: "#10352c"
  dark-accent-text: "#83e1cc"
  dark-accent-wash: "#263f39"
  dark-input: "#28272a"
  dark-hover: "#3e3b42"
  dark-selected: "#3b4140"
  dark-tag: "#29282d"
  dark-focus: "#83e1cc"
  dark-warning: "#f2c483"
  dark-warning-wash: "#493b2b"
  dark-danger: "#ee9399"
  dark-category-0: "#275f56"
  dark-category-1: "#3b5c7b"
  dark-category-2: "#625580"
  dark-category-3: "#7d5848"
  dark-category-4: "#52683d"
  dark-category-5: "#7d455e"
  dark-category-6: "#4e5964"
  light-canvas: "#f7f7f8"
  light-sidebar: "#eeeef1"
  light-surface: "#fff"
  light-elevated: "#f5f5f7"
  light-border: "#d8d8de"
  light-text: "#25232a"
  light-secondary: "#62616b"
  light-accent: "#006b59"
  light-on-accent: "#fff"
  light-accent-text: "#006b59"
  light-accent-wash: "#e3f2ed"
  light-input: "#fff"
  light-hover: "#e8e8ed"
  light-selected: "#dceae5"
  light-tag: "#f0f0f3"
  light-focus: "#00745f"
  light-warning: "#83540b"
  light-warning-wash: "#fff0d8"
  light-danger: "#ad2f3b"
typography:
  display:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: "32px"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  headline:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: "28px"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  title:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: "17px"
    fontWeight: 650
    lineHeight: 1.4
  subtitle:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: "14px"
    fontWeight: 600
    lineHeight: 1.5
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
  caption:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  tag: "5px"
  control: "6px"
  navigation: "7px"
  choice: "8px"
  callout: "10px"
  surface: "12px"
  dialog: "14px"
spacing:
  "4": "4px"
  "8": "8px"
  "12": "12px"
  "16": "16px"
  "20": "20px"
  "24": "24px"
  "32": "32px"
components:
  button-primary:
    backgroundColor: "{colors.dark-accent}"
    textColor: "{colors.dark-on-accent}"
    rounded: "{rounded.control}"
    padding: "6px 12px"
    typography: "{typography.body}"
  button-secondary:
    backgroundColor: "{colors.dark-elevated}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.control}"
    padding: "6px 12px"
    typography: "{typography.body}"
  button-text:
    textColor: "{colors.dark-accent-text}"
    padding: "4px 0"
    typography: "{typography.body}"
  search-field:
    backgroundColor: "{colors.dark-input}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.control}"
    padding: "6px 10px"
    typography: "{typography.body}"
  navigation-selected:
    backgroundColor: "{colors.dark-selected}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.navigation}"
    padding: "8px 10px"
    typography: "{typography.label}"
  status-neutral:
    backgroundColor: "{colors.dark-tag}"
    textColor: "{colors.dark-secondary}"
    rounded: "{rounded.tag}"
    padding: "4px 8px"
    typography: "{typography.caption}"
  status-attention:
    backgroundColor: "{colors.dark-warning-wash}"
    textColor: "{colors.dark-warning}"
    rounded: "{rounded.tag}"
    padding: "4px 8px"
    typography: "{typography.caption}"
  panel:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-text}"
    rounded: "{rounded.surface}"
    padding: "20px"
  table:
    textColor: "{colors.dark-text}"
    typography: "{typography.label}"
    width: "100%"
  storage-tile:
    backgroundColor: "{colors.dark-category-0}"
    textColor: "#fff"
    rounded: "{rounded.control}"
    padding: "16px"
---

# Design System: CoquiPCCleaner

## Overview

**Creative North Star: "Native macOS clarity"**

Coqui adapts the supplied Apple Music macOS kit into a quiet diagnostic workspace. Neutral window chrome, a persistent grouped sidebar, solid surfaces and compact system typography make evidence easy to inspect. Mint identifies the main action and active details; it never substitutes for a status label.

Mode is Operate. The visual authority remains Figma file Gz7WEiN8Y3Vvivvae0ZPlP; this reference records the implemented Tauri + Svelte/Vite interface rather than creating a new visual direction. Native macOS composition and honest unavailable states are durable commitments from PRODUCT.md.

**Key Characteristics:**

- Neutral dark and light appearances with restrained Coqui mint.
- Compact, resizable desktop composition with visible keyboard focus.
- Text-first evidence, explicit capability states and accessible chart alternatives.

The implementation reference is `src/tokens.css`, `src/styles.css`, the reusable Svelte components and `src-tauri/tauri.conf.json`. Captured native evidence lives in `.impeccable/review/native-dashboard-dark.png`, `native-dashboard-minimum.png`, `native-storage-minimum.png` and `native-settings-light.png`; the labeled failure fixture is `monitoring-failure.png`. These files record a review point, not permanent machine measurements or blanket runtime acceptance.

Kit reuse follows the established mapping: Music for shell/navigation, shared foundations for controls/settings, Numbers for tables/graphs, Notes for evidence/reports, Mail for finding reading, App Store for application inventory, and iMovie for scan stages/history. Messages, Reminders and Calendar inform later troubleshooting, action queues and maintenance. Original kit content stays intact. Existing ledgers are `docs/figma-foundations.json`, `docs/figma-components.json`, `docs/figma-icons.json`, `docs/figma-screens.json`, `docs/figma-prototype.json` and `docs/figma-handoff.json`.

## Colors

The palette uses warm charcoal or soft gray neutrals with one mint action accent. The frontmatter records the exact implemented values; `dark-*` and `light-*` map to the same semantic CSS variable in each appearance. The default CSS root is dark, an explicit appearance overrides it, and system mode follows `prefers-color-scheme`.

### Primary

- **Coqui Mint / Coqui Deep Green** (`dark-accent` / `light-accent`): primary button and brand fill, paired with the corresponding `on-accent` foreground.
- **Readable Mint / Readable Green** (`accent-text`): text actions, selected navigation icons, chart lines and positive status text.
- **Mint Wash** (`accent-wash`): selected workload choices, scan banners and positive status backgrounds.

### Neutral

- **Window Canvas** (`canvas`) and **Sidebar** (`sidebar`): persistent app regions.
- **Diagnostic Surface** (`surface`) and **Raised Control** (`elevated`): panels and default controls.
- **Separator** (`border`): single-pixel dividers, panel outlines and table rules.
- **Primary Ink** (`text`) and **Secondary Ink** (`secondary`): main content and supporting labels. The dark secondary token is the implemented value, replacing the provisional seed's older value.
- **Input**, **Hover**, **Selected** and **Tag**: field fill, transient hover, active selection and quiet metadata backgrounds.

Warning and warning wash are semantic attention colors. Danger is reserved for errors and local-history purge controls. Focus uses its own high-visibility token. `category-0` through `category-6` form the categorical storage palette; they retain their root values in both appearances. They encode categories, never disk health, and storage tiles use white text.

**The Semantic Color Rule.** Use accent for actions, selection and supported/complete labels; use warning for attention. Keep readable status text beside every status color.

## Typography

**Display and Body Font:** macOS system stack (`-apple-system`, `BlinkMacSystemFont`, `Segoe UI`, sans-serif). Figma uses the kit's editable Inter fallback; the runtime does not load Inter as a web font.

**Character:** Compact and familiar, with strong page headings, modest section titles and quiet evidence metadata. Numeric readings use tabular figures; exact paths are selectable and wrap in detail views.

### Hierarchy

- **Display:** onboarding headline; use the `display` token.
- **Headline:** page heading; use `headline` with tight tracking.
- **Title:** section heading; use `title`. At the narrower desktop breakpoint, section headings reduce to (15px).
- **Subtitle:** evidence and card subheadings; use `subtitle`.
- **Body:** default reading and controls; use `body`.
- **Label / Caption:** compact labels, timestamps and capability details; use `label` and `caption`. Dense metadata also uses observed (10px) text; chart axes use (9px). Do not promote these dense metadata sizes to body copy.

Metric summaries use (25px), reducing to (22px) at the minimum-width breakpoint. Evidence and explanation blocks commonly cap text at (80–95ch), while modal paragraphs cap at (70ch).

**The Evidence Before Decoration Rule.** A missing, failed or stale reading must remain visibly qualified. Never draw a reassuring measurement or health gauge in its place.

## Layout

The native window opens at (1440×960) with a supported minimum of (1100×720). The shell is a two-column grid with a persistent sidebar (232px), a toolbar (52px) and an independently scrolling content area. Native traffic lights occupy the reserved sidebar titlebar; the Tauri overlay titlebar uses real window controls.

At full width, content padding is (30px 24px 18px). The core spacing rhythm is (4/8/12/16/20/24/32px), with (16px) summary gaps, (20px) panel and two-column gaps, and (20px) panel interiors. The dashboard uses three summary columns and a lower (1.15fr / 1fr) pair; hardware and performance use two columns.

At (1250px) and below, content padding becomes (24px 20px 16px), dashboard columns become equal and secondary details tighten. At (1100px) and below, the sidebar narrows to (205px), dashboard/performance detail columns stack, decorative device-side metadata hides, and the summary row remains three columns. This is a desktop interface; no mobile navigation is implemented.

Tables scroll horizontally within their container. Long names may truncate in a row, while evidence and detail views preserve full paths. Storage inspection uses dialogs; the provisional fixed inspector/sheet is not implemented. Keep controls and disclosure text accessible when the window reaches its minimum size.

## Elevation & Depth

Tonal layering and thin borders carry nearly all depth. Solid panels sit on the canvas without shadows. Floating search and notification surfaces use a small ambient shadow; modal dialogs use the stronger modal shadow and a dimmed, blurred backdrop. The exact shadow and motion values are recorded in the sidecar because they are outside the frontmatter schema.

### Shadow Vocabulary

- **Floating surface:** search/notification popovers (`0 10px 35px #0003`).
- **Modal:** inspection and confirmation dialogs (`0 20px 80px #0005`).
- **Switch thumb:** small local lift (`0 1px 3px #0004`).

**The Flat Workspace Rule.** Keep panels flat at rest. Reserve shadows for floating controls, modal dialogs and the small switch thumb.

## Shapes

Use gently rounded rectangles: small tags, compact controls, slightly softer navigation rows, rounded diagnostic panels and modal dialogs. The exact reusable radius steps live in `rounded`. Panels, fields and dialogs use a (1px) separator-colored border. Navigation selection is a filled row, tabs use a (2px) underline, and status dots remain circular (5px).

Avoid decorative gauges and ornamental containers. The device illustration is a restrained line drawing; the storage map uses proportionate, clipped rectangular category tiles with (2px) surface-colored separation.

## Components

### Buttons

Compact, explicit actions. Primary buttons use the accent/on-accent pair, semibold text (600), control radius and standard padding; default secondary buttons use elevated fill with a separator border. Both have a minimum height of (32px). The primary hover brightens by (1.06); secondary hover/press uses hover/selected fills. Disabled controls use opacity (0.5). Text actions use readable accent text, transparent fill and a (28px) minimum height. Icon buttons are (30×30px) with accessible names.

Every keyboard-focusable control receives a (3px) focus outline offset by (3px). Background/border changes use (0.13s ease); reduced motion removes animation, transitions and smooth scrolling. Active scan indicators rotate over (1.3s) only when motion is allowed.

### Inputs / Fields

Fields and selects use the input fill, separator border and control radius, with padding (6px 10px) and minimum height (32px). Placeholders use secondary ink. Toolbar search is a compact shell with a search icon, clear accessible label and visible ⌘K shortcut; its width changes from (260px) to (230px) and (200px) with desktop density. Checkbox/radio controls use readable accent; settings switches use a (34×20px) track and (14px) thumb. Workload choices add both a checked input and accent outline/wash.

### Navigation

Grouped, icon-led sidebar rows use label typography and a minimum height of (36px), with padding (8px 10px). Rest is transparent; hover uses hover fill. Active rows use selected fill, primary ink, accent icon and `aria-current="page"`. The sidebar persists at the minimum size. Page-local tabs keep secondary text at rest and accent text plus underline when selected. Tab navigation supports keyboard focus and selection.

### Chips / Status

Quiet tags identify workload and context. Status badges combine a dot and a readable label: neutral for unknown/other states, accent wash for supported/complete, warning wash for partial, permission, temporary unavailability, failure, interruption and attention. Status appearance never converts an unknown state into a successful one.

### Cards / Containers

Diagnostic panels use surface fill, separator border, surface radius and standard panel padding; metric summaries use a compact (17px 20px) interior. They stay flat at rest. Clickable summary cards include a directional affordance. Finding/activity rows use horizontal separators and compact icon or status treatments rather than nested card shadows. Empty states explain absent records and offer an applicable action.

### Evidence / Tables

Evidence rows keep label, source, time and qualification together, with the value aligned at the opposite edge. Non-supported readings retain their status badge. File/process/history tables use compact headers, row separators, tabular numeric columns and horizontal scrolling. A row action opens inspection or reveals a location; it does not imply deletion. Detail dialogs use native modal focus behavior, an accessible title, close control, Escape handling and full-path disclosure.

### Charts / Storage Map

Live charts use quiet separator grids, a readable accent line and labeled percentages. Fewer than two samples show a waiting message rather than a synthetic trace; an accessible chart label describes the recorded sample count and latest value. Monitoring failure displays an explicit unavailable message and retry action, and current values remain marked unrecorded or stale as appropriate. Monitoring only runs while the app is open.

Storage tiles reflect inspected allocated bytes, not promised recoverable storage. Category names, values and equivalent legend/table controls remain available; narrow tiles may omit their interior label. Capacity qualifications and skipped/protected scope disclosures stay visible.

## Do's and Don'ts

### Do:
- **Do** use the semantic CSS variables so dark, light and system appearances stay aligned.
- **Do** retain source, observation time, qualification and units when presenting evidence.
- **Do** preserve the minimum desktop layout, scrolling tables and wrapping full paths in details.
- **Do** show a visible focus outline and honor reduced motion.
- **Do** label design specimens and component previews as illustrative; production measurements come from the engine.

### Don't:
- **Don't** replace the supplied native kit direction with a new aesthetic.
- **Don't** invent health scores, fake progress percentages or reassuring values for unavailable collections.
- **Don't** use color as the only indicator of capability, scan state or selection.
- **Don't** present future cleanup, security or firmware actions as working Phase 1 controls.

Keep general read-only/privacy explanations in onboarding and Settings. Avoid repeated shield reassurance badges in the workspace, sidebar, footer or device summary.
