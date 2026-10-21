# Figma implementation handoff

Existing kit: [Coqui design workspace](https://www.figma.com/design/Gz7WEiN8Y3Vvivvae0ZPlP). Original pages/components were preserved. Nineteen Coqui-prefixed pages were added, covering reference, foundations, imports, components, shell, onboarding, all Phase 1 destinations, future journeys, prototypes and developer handoff.

Start at [the linked workspace prototype](https://www.figma.com/design/Gz7WEiN8Y3Vvivvae0ZPlP?node-id=163-2). Flow starts also include first analysis (`163:816`) and storage inspection (`163:376`). Prototype frames share a page so native Figma navigation reactions work. The setup sequence progresses through purpose, workload, preference, permissions/scope and initial analysis. Product navigation contains only Phase 1 destinations.

## Kit inventory and intended use

| Source pattern | Source node | Coqui use |
| --- | --- | --- |
| Standard button | `3:118` | Adapted button variants: default, hover, pressed, focused, disabled, loading. |
| Navigation row | `3:140` | Reusable sidebar rows with Coqui labels/icons. |
| Selected navigation row | `3:141` | Current destination and keyboard-focused state. |
| Search field | `3:171` | Global page/finding/application search pattern. |

The kit's neutral relationships and native control semantics guide the implementation. No media-library content or Apple brand assets were imported as product content. Eight Lucide navigation icons correspond to the shipped icon library. The Coqui mark is an authored vector in `assets/coqui.svg`.

| Named application family | Phase 1 or future feature mapping |
| --- | --- |
| Music | Main window, sidebar, toolbar and grouped navigation. |
| Shared foundations | Appearance, controls, focus and preferences. |
| Numbers | Storage tables, metric charts, comparison and future upgrade analysis. |
| Notes | Evidence, diagnostic reports and inspector content. |
| Mail | Findings list/detail reading and contextual inspection. |
| App Store | Installed application inventory and future supported upgrade information. |
| iMovie | Scan stages and chronological analysis/history presentation. |
| Messages | Future Ask Coqui troubleshooting conversation (Phase 2). |
| Reminders | Future approved action queues and follow-up verification (Phase 2). |
| Calendar | Future maintenance planning and scheduling (Phase 2). |

## Created system

Foundations include linked primitive and semantic variables, Dark/Light modes, typography, spacing and radius references. Application typography uses the macOS system family; editable Figma text uses Inter. Component sets cover buttons, navigation and six capability states. Domain components cover device summaries, workload profiles, findings, evidence, progress, hardware, storage categories, cleanup candidates, permissions and future recovery.

The reference layout is 1440 × 960 with a 232 px sidebar, 52 px toolbar and 24 px content padding. Additional dashboard frames cover light appearance and 1280/1600/1920 widths. The app enforces an approximately 1100 × 720 minimum and adapts inspector presentation. The shipped app carries real readings, richer tables and scrollable results; the Figma frames are structural, illustrative handoff compositions. Remaining future journeys and exhaustive state variants are handoff work, not inactive app navigation.

## Durable object ledgers

- `figma-foundations.json`: page IDs, variable collections/modes, typography and source references.
- `figma-components.json`: component IDs and source-node mapping.
- `figma-icons.json`: icon components.
- `figma-screens.json`: core screen/frame IDs and responsive references.
- `figma-handoff.json`: attribution, family mapping, navigation and future-work guides.
- `figma-prototype.json`: linked same-page frames and navigation/setup flows.

Scripts in `scripts/figma/` record the construction approach. Ledgers are the authoritative identifiers for future edits; preserve original kit nodes and extend the Coqui pages.

## Phase 2.1 cleanup workflow

The existing Optimization Center page now includes linked, editable selection, exact-path approval and verification/recovery handoff frames. Start at [cache review](https://www.figma.com/design/Gz7WEiN8Y3Vvivvae0ZPlP?node-id=165-64). The original future-work guide is preserved. `figma-cleanup.json` records the new frame/button IDs, and `scripts/figma/cleanup.js` records construction using the Coqui variables, text styles and component instances. Example outcomes are illustrative rather than claims that device files were changed.
