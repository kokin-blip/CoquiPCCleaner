CoquiPCCleaner is a smart system analysis and optimization app that scans the hardware of the machine in order to choose the most optimal settings based on what the user uses the computer for on the daily. It analyzes software settings, BIOS / UEFI settings and more in order to come to a conclusion about how the system should be optimized. It also is able to scan for bottlenecks, necessary upgrades, etc. The user will be able to describe their problem, and the app will be able to provide a solution. E.g., “My computer has flagship parts, yet I am lagging on startup..." Then the app provides a solution.

The app will also be able to scan for storage clearing solutions and provide solutions for future organization and how to keep storage levels low based on what the user does on the daily or uses the machine for. It should be able to clear cache, identify duplicate files, identify duplicate photos, analyze stale git trees that are taking up storage, and other development artifacts that may be taking up storage and may not necessarily be needed anymore. It should also be able to delete these things on its own but obviously with a prior review process by the user. In the event that the app does not have sufficient permissions to delete a file or anything of the sort it should guide the user through the process. It should also have a clickable visualization of the storage. With recommended first deletions.

Since it can scan the hardware the app should be able to provide an estimate of how much longer this machine can be used for specific things (e.g gaming, coding etc.) and if an upgrade is warranted or not, and if its fine or not.

Avoid making a generic "Optimize Everything" button that silently alters BIOS settings, disables Windows services, changes security features, or performs aggressive registry cleaning.

Instead, consider three optimization profiles:
Balanced: Prioritizes stability, reasonable power consumption, and responsiveness.
Performance: Prioritizes sustained performance within supported operating-system controls.
Efficiency: Prioritizes battery life, reduced background activity, and energy savings.
Make the more serious hardware optimization features optional.


Other general planned features:
Storage dashboard
Junk file scanner
Large file explorer
Safe cleanup with confirmation
Disk usage visualization
Duplicate file detector
Application leftover finder
Startup item management
Scheduled cleaning
Storage, junk files, caches
Startup applications
Background processes
CPU, RAM, GPU utilization
Windows power settings
Driver versions
BIOS/UEFI version
BIOS configuration
RAM XMP/EXPO profiles
Secure Boot / TPM status
SSD health and temperatures

Stack: 
Dart + Flutter + Figma

Rust + Flutter Rust Bridge 

SQLite

Planned Builds: Mac first, then Windows (deferred until the first completed Mac build)

Working Repo: https://github.com/kokin-blip/CoquiPCCleaner
Keep context via markdowns in the repo.

You will use every aspect of this Figma UI kit to build out the UI where possible: https://www.figma.com/design/Gz7WEiN8Y3Vvivvae0ZPlP/Apple-Music-%E2%80%94-macOS-UI-Kit?node-id=0-1&p=f
If new UI assets are needed you may make them in Figma in the same style. Make a new tab in the UI kit.


Phase 1 — System Scanner
Build hardware inventory, CPU/RAM/storage monitoring, startup application detection, temporary file scanning, disk health checks, and read-only BIOS version detection. 
Phase 2 — Optimization Engine
Add user-confirmed cleanup, reversible startup adjustments, supported power profiles, performance baselines, and recommendations grounded in actual measurements.
Phase 3 — Advanced Firmware Diagnostics
Add manufacturer detection, BIOS configuration audits on supported models, firmware update notifications, and guided manual configuration. Any supported firmware modifications should be separate, explicitly authorized operations.


CoquiPCCleaner — UI/UX Master Plan
Version: 2.0
Product: CoquiPCCleaner
Platform: macOS first; Windows after the first completed macOS build
Frontend: Flutter + Dart
Design: Figma
System Engine: Rust + Flutter Rust Bridge
Local Storage: SQLite
Design Reference: Apple Music — macOS UI Kit

1. Product Vision
CoquiPCCleaner is an intelligent desktop application that analyzes a computer's hardware, software, storage, startup behavior, performance, and supported firmware information to recommend configurations suited to its owner's actual workloads.
Rather than assuming that every computer requires identical optimizations, the application develops a contextual understanding of the machine and how the user intends to use it.
Users can describe problems in natural language, receive diagnostic explanations supported by system measurements, and safely apply approved solutions.
The platform has six primary responsibilities:
Understand: Identify system hardware, operating-system configuration, installed software, usage goals, and relevant performance characteristics.
Diagnose: Investigate performance complaints, resource bottlenecks, configuration issues, storage consumption, and component health.
Recommend: Generate workload-specific optimization plans with supporting evidence, confidence levels, and safety classifications.
Maintain: Identify unnecessary files, duplicates, caches, unused application data, and development artifacts while recommending sustainable storage organization.
Forecast: Assess whether the current hardware remains suitable for the user's intended workloads and whether upgrading specific components would be worthwhile.
Act safely: Apply only user-approved changes, report results, preserve recoverability where possible, and provide manual instructions when automated actions are unsupported.
Product principle
Understand the machine. Diagnose the problem. Explain the solution. Obtain consent. Verify the outcome.
The interface must never claim that a device is unhealthy simply because it has temporary files, uses substantial RAM, or runs background processes.

2. Core UX Strategy
Primary experience: Diagnose first
The application should prioritize problem resolution over exposing every available diagnostic tool immediately.
The main workflow is:
Describe a problem → Gather relevant information → Investigate → Present findings → Recommend actions → Review → Apply → Verify
A technically experienced user may navigate directly to individual metrics, storage locations, firmware properties, and diagnostic reports.
A less experienced user should be able to solve common problems without understanding firmware settings, CPU scheduling, filesystem structures, or memory management.
Progressive disclosure
Use three information depths:
Level
Audience
Information
Overview
All users
System status, plain-language findings, clear actions
Details
Interested users
Measurements, evidence, affected applications, comparisons
Advanced
Technical users
Raw diagnostics, supported firmware attributes, logs, detailed configurations

Advanced hardware and firmware functionality must be optional and disabled in the default simplified experience.
Diagnostic integrity
The interface must distinguish between:
Observed facts
Possible causes
Confirmed causes
Recommendations
Unsupported or unverified assumptions
No recommendation should be represented as a guaranteed performance improvement without supporting measurements.

3. Visual Design and Figma Kit Integration
Overall direction
Build a native-feeling macOS desktop application using the supplied Figma kit's visual language.
The design should emphasize:
Clean desktop navigation
Native-inspired window composition
Dark and light appearance
Strong information hierarchy
Compact but readable data presentation
Subtle animations
Contextual inspectors and side panels
Consistent controls
Clear distinctions between informational and destructive actions
Avoid excessive neon styling, artificial performance scores, oversized gauges, and decorative animations that interfere with usability.
Actual Figma kit component mapping
The referenced Figma file contains multiple macOS application families. Reuse their applicable layout structures, variants, primitives, and interaction patterns as follows.
Figma kit family
CoquiPCCleaner application
Apple Music
Main shell, sidebar navigation, search, content cards, toolbar controls
Shared Foundations
Appearance modes, buttons, toggles, tabs, badges, settings rows
Messages
Natural-language troubleshooting interface, message history, guided follow-up
Notes
Detailed diagnostic reports, recommendation explanations, inspector panels
Numbers
Performance charts, component comparisons, storage tables, measurement views
Reminders
Recommended-action checklists, optimization queues, maintenance tasks
Calendar
Scheduled scans, cleaning schedules, maintenance history
Mail
Findings inbox, notification lists, diagnostic report reader
App Store
Installed-application inventory, application detail cards
iMovie
Scan progress, staged operations, history timelines, before/after comparisons

Reuse all applicable states: default, selected, hover, pressed, disabled, expanded, loading, light/dark, and supported solid/translucent appearances.
Do not copy music artwork, playback controls, fictional application content, or Apple branding into unrelated system-management functionality merely for the sake of using every component. Preserve useful structure while adapting semantics and product identity.
Foundation colors
The supplied kit's Music foundations include these dark-mode values:
Token
Source color
Canvas
#211F23
Sidebar
#28272A
Surface
#302E33
Elevated surface
#3A373D
Border
#434047
Primary text
#F4F2F5
Secondary text
#A5A2A7
Music accent
#FA2D48

Retain the underlying neutral color relationships as the starting theme.
Create a CoquiPCCleaner-specific accent token instead of making the Apple Music accent a permanent branding requirement. A restrained blue-green accent is a suitable initial direction, subject to contrast testing.
Support light and dark mode from the design-system stage.
Typography and geometry
Use the kit's typography hierarchy and macOS-inspired geometry as a starting point.
Suggested sizes:
Large page title: 26–28 px
Section heading: 20 px
Card heading: 15–17 px
Body: 13–14 px
Caption: 11–12 px
Standard spacing: 4 / 8 / 12 / 16 / 24 / 32
Card corner radius: 8–16 px
Use the kit's documented SF Pro styling where licensing and platform support permit, with appropriate Flutter font fallbacks.
Accessibility requirements include keyboard navigation, visible focus indicators, screen-reader labels, readable contrast, reduced-motion support, and controls large enough to operate reliably.

4. Navigation Architecture
Primary sidebar
The desktop application should contain:
Main
Dashboard
Ask Coqui
System Analysis
System
Performance
Hardware
Storage
Firmware & Security
Optimization
Optimization Center
Maintenance
History & Recovery
Application
Settings
The exact sidebar structure may consolidate secondary screens into expandable groups.
Global header
Include:
Current device name
Operating system
Most recent scan
Current scan activity
Global search
Notifications
Scan button
Global state indicators
The interface should consistently communicate:
Scan running
Scan complete
Partial scan
Permission required
Information unavailable
Action awaiting approval
Operation failed
Reboot required
A partial or unsupported scan must never be represented as a completely healthy system.

5. First-Launch Experience
Onboarding is a central component of CoquiPCCleaner's intelligence.
Screen 1 — Welcome
Introduce the application's purpose and its local-system access model.
Primary CTA: Set up my computer
Secondary CTA: Explore without scanning
Screen 2 — Define daily usage
Ask users what they primarily do on the machine.
Allow multiple selections:
Gaming
Software development
Video editing
Graphic design
Music production
General productivity
School or research
Virtual machines and containers
Other / custom
Follow with optional context-specific questions.
For example, a gaming user may identify typical games and performance targets. A developer may identify Docker, Xcode, Flutter, Android Studio, local AI models, or relevant build systems.
Users must be able to edit their workload profile later.
Screen 3 — Select optimization preference
Offer three profiles:
Balanced
Prioritizes stability, responsiveness, and reasonable energy use.
Performance
Prioritizes sustained performance using supported system controls.
Efficiency
Prioritizes lower power consumption and reduced unnecessary background activity.
These are optimization preferences, not promises that a fixed set of settings is optimal for every computer.
Show a summary of proposed changes before applying any profile.
Screen 4 — Privacy and permissions
Provide individual permission explanations for:
Accessible storage scanning
Protected folder access
Application inventory
Background monitoring
Optional usage-pattern analysis
Optional advanced hardware diagnostics
Use separate consent controls for optional capabilities.
The user should retain the ability to diagnose specific issues without enabling continuous activity monitoring.
Screen 5 — Initial system analysis
Display progress across:
Hardware inventory
Storage analysis
Startup applications
Operating-system configuration
Available hardware health information
Supported firmware information
End with a personalized dashboard and clearly disclose incomplete scan areas.

6. Dashboard
Purpose
Provide a practical summary of machine condition, workload suitability, unresolved issues, and recommended next steps.
Desktop layout
Use the following hierarchy:
Top section — Greeting and system context
Display:
Computer name and model
Current workload profile
Last scan time
Primary action: Analyze System
Secondary action: Describe a Problem
Section A — Ask Coqui
A prominent but compact natural-language problem entry field.
Placeholder:
“What problem are you experiencing with your computer?”
Suggested prompts:
My computer is slow at startup
Why does my game keep stuttering?
What is consuming my storage?
Is my machine still good for development?
Should I upgrade my computer?
This is a primary product feature and must remain discoverable throughout the application.
Section B — System summary
Cards for:
CPU and memory utilization
Available storage
Storage health, if supported
Startup analysis
Outstanding findings
Workload suitability
Avoid a single arbitrary health score unless its methodology is transparent. Individual component conditions are more actionable.
Section C — Recommended next steps
List up to five prioritized recommendations.
Each recommendation displays:
Problem or opportunity
Supporting evidence
Expected benefit
Confidence
Risk
Action availability
Review CTA
Section D — Workload suitability
A comparative view showing how well the current system meets the user's identified workloads.
Section E — Recent activity
Show completed scans, approved changes, unresolved issues, and scheduled operations.

7. Ask Coqui — Intelligent Troubleshooting
Purpose
Enable natural-language problem solving using machine-specific diagnostic information.
This should function as a dedicated troubleshooting workspace, not merely a decorative chatbot.
Layout
Use a three-part desktop layout:
Left: Conversation history and saved investigations.
Center: Conversation, findings, progress, and generated diagnostic explanations.
Right: Context panel showing relevant live measurements, selected evidence, and proposed actions.
The inspector can collapse on narrower windows.
Use the Figma kit's Messages conversation patterns, Notes detail views, and Numbers chart elements.
Core interaction
Example user message:
“I have a powerful computer, but startup takes forever.”
Stage 1 — Clarification
Coqui requests only information necessary to investigate the complaint.
It should distinguish between slow power-on, slow operating-system login, and applications being unresponsive after login.
Stage 2 — Diagnostic investigation
Show individual diagnostic tasks with live status.
Examples:
Inspect startup applications
Evaluate login items
Review available startup timing data
Check storage pressure
Examine background activity
Stage 3 — Findings
Provide an evidence-oriented report.
Each finding includes:
Observation
Supporting measurement
Potential cause
Confidence classification
Whether more testing is necessary
Stage 4 — Solutions
Generate an action plan.
Potential actions include changing supported login-item settings, removing an identified unnecessary startup item, or investigating a particular application.
Stage 5 — Verification
Measure relevant conditions again where possible and compare before-and-after results.
AI interface requirements
Never allow the conversational layer to invent capabilities, system readings, or completed actions.
The conversation must use structured information supplied by the diagnostic engine.
Separate conversational explanations from authorized system-modification commands.
If a cause cannot be established, provide further diagnostic steps rather than fabricating certainty.

8. System Analysis
Purpose
Offer a complete machine analysis accessible independently from the conversational troubleshooting system.
Main tabs
Overview
Findings
Bottlenecks
Workload Suitability
Upgrade Assessment
Reports
Bottleneck analysis
Provide subsystem comparisons involving:
CPU utilization and saturation
Memory pressure
Storage capacity and I/O behavior
GPU load where measurable
Thermal or power constraints where exposed
Startup and background activity
A bottleneck should be diagnosed in relation to a specific workload, rather than by component age or headline specifications alone.
Bottleneck detail view
Each finding includes:
Affected workload
Observed behavior
Evidence
Possible impact
Confidence
Recommended diagnostic test
Potential corrective actions

9. Hardware Diagnostics
Purpose
Present the current hardware configuration and available component-health information.
Screen composition
Display overview cards for:
CPU
Memory
GPU
Storage
Motherboard or system platform
Battery, where applicable
Available thermal sensors
Selecting a component opens a technical inspector.
Information architecture
CPU
Display model, architecture, core configuration, utilization, and supported performance information.
Memory
Display installed capacity, pressure, swap activity, and available configuration details.
GPU
Display GPU model, memory information where supported, and supported utilization measurements.
Storage
Display disks, volumes, filesystem information, available capacity, and drive-health measurements when available.
System platform
Display manufacturer, model identifier, and available firmware information.
Unsupported sensor values must display Unavailable rather than invented estimates.
Hardware suitability
Introduce a dedicated workload suitability card:
“How suitable is this hardware for your current work?”
Evaluate the user's selected workloads against:
Actual hardware characteristics
Measured performance
Representative workload requirements
Software support requirements
User-defined performance expectations

10. Hardware Longevity & Upgrade Advisor
Purpose
Help users decide whether to keep using their computer, modify its configuration, or replace or upgrade particular hardware.
Main screen
Use workload-specific assessments:
Workload
Assessment
Everyday productivity
Suitable
Software development
Suitable with possible constraints
Gaming
Depends on target games and settings
Local AI workloads
Depends on memory and model requirements

The values above are illustrative only.
Longevity estimates
Where evidence permits, provide approximate suitability horizons expressed as ranges or scenarios—not guaranteed years of remaining life.
The forecast must disclose assumptions about software requirements, target performance, and the availability of measurements.
Distinguish between:
Hardware wear or failure risk
Workload performance sufficiency
Operating-system support
Upgradeability
Upgrade recommendations
For each suggested upgrade, show:
Current component
Identified limitation
Proposed upgrade
Compatibility confidence
Expected workload benefit
Estimated cost, if sourced
Upgrade feasibility
Alternative software optimization
Recommendation to retain existing hardware where appropriate
The system should be fully comfortable concluding that no upgrade is warranted.
For non-upgradable computers, distinguish configuration improvements from replacement recommendations.

11. Storage Intelligence
Purpose
Move beyond removing temporary files toward personalized storage management and prevention.
Main interface
Use a large storage visualization showing available space and allocation categories.
Categories may include:
Applications
Documents
Photos and videos
System-accessible caches
Downloads
Development artifacts
Archives
Duplicate content
Other / uncategorized
Allow users to select a category and inspect contributing folders or files.
Main tabs
Overview
Cleanup Opportunities
Large Files
Duplicates
Developer Storage
Organization
Scheduled Maintenance
Cleanup opportunities
Each item must include:
File or folder
Size
Origin
Last modification date
Reason identified
Safety classification
Potential consequences
Selection control
The application must not automatically select uncertain or high-risk items for deletion.
Duplicate files and photos
Provide grouped duplicate candidates.
Distinguish:
Byte-identical files
Visually similar photos
Modified versions
Potentially intentional copies
Visually similar photos must not be labeled exact duplicates without verification.
Offer previews and preserve at least one confirmed copy.
Development artifact analysis
This is a major product differentiator.
Recognize common development environments and potential storage consumers:
Git repositories and linked worktrees
Unused repository clones
Xcode DerivedData
Flutter and Dart build outputs
Rust target directories
JavaScript dependency directories
Python virtual environments
Build caches
Docker images and other Docker resources
Emulator images
Abandoned generated artifacts
Local development logs
Every recognized item must have a technology-specific explanation.
Do not interpret a development artifact as disposable solely because it is old or large.
For Git worktrees, inspect repository relationships, cleanliness, and worktree state before offering removal. Stale Git administrative metadata is distinct from an actual worktree directory containing project files.
Developer storage actions require explicit review.
Storage organization advisor
Provide suggestions based on identified usage patterns.
Examples:
Review old archives in Downloads
Introduce a dedicated development-cache cleanup routine
Consolidate redundant personal file copies
Establish separate working and archive directories
Schedule non-destructive storage analysis
Each suggestion should explain the expected organizational benefit.
Do not move the user's documents automatically based on assumptions about their intended location.

12. Cleanup Review & Execution
Safety-oriented workflow
Scan → Select → Preview → Confirm → Execute → Verify
Review screen
Use a two-pane layout.
Left panel: Selected cleanup categories and items.
Right panel: Preview, exact target path, risk details, and recovery information.
Persistent footer
Display:
Number of selected items
Estimated recoverable storage
Items requiring permission
Items excluded for safety
Primary CTA: Review Cleanup
Confirmation
Distinguish between:
Moving files to Trash
Removing safely regenerable caches
Permanently deleting files
Removing application-managed resources through their native tools
Prefer reversible operations when possible.
Permission-denied workflow
If an operation cannot proceed:
Identify the affected file or resource.
Explain why access is unavailable.
Offer a supported permission-granting path.
Provide a contextual button to open the relevant system interface where possible.
Allow the user to retry or skip.
Preserve the failure reason in the operation report.
Never bypass operating-system protections or silently attempt broader privileges.

13. Performance Center
Purpose
Help users understand system performance and compare the effects of configuration changes.
Main tabs
Overview
CPU & Memory
Processes
Startup
Power
Performance History
Overview
Display compact graphs for:
CPU utilization
Memory pressure
Disk activity
Available GPU activity
Supported thermal information
Provide selectable measurement intervals and clearly indicate when historical data was not recorded.
Startup management
Use a table with:
Application
Publisher
Startup mechanism
Current state
Measured or estimated impact
Recommendation
Available action
Do not claim an exact boot-time improvement without a valid measurement.
Power profiles
Display the three workload preferences:
Balanced
Performance
Efficiency
Selecting a profile opens a comparison panel describing supported changes, tradeoffs, and reversibility.
Do not silently alter protected security settings, disable essential system services, or apply unrelated performance tweaks.

14. Firmware & Security
Purpose
Present supported firmware details and security-related system configuration without implying that every computer exposes conventional BIOS controls.
macOS-first design
For macOS, organize around:
Mac model and platform
Available firmware information
Boot and system security information
Supported configuration status
Firmware-related update guidance
Available system management actions
Conventional PC BIOS tuning controls must not be displayed for Apple Silicon devices.
Clearly label unavailable or restricted functionality.
Future Windows extension
When Windows support is implemented, extend the interface to include supported:
BIOS/UEFI inventory
Firmware version
Motherboard manufacturer
Secure Boot status
TPM information
CPU virtualization configuration
RAM XMP/EXPO information
Manufacturer-specific firmware capabilities
Vendor firmware update guidance
Advanced mode
Advanced Firmware Diagnostics must be explicitly enabled in Settings.
Use an additional warning and authorization step for any supported firmware modification.
Firmware and security changes should not be included in standard batch optimizations.
The default behavior remains read-only analysis and guided recommendations.

15. Optimization Center
Purpose
Convert diagnostic findings into understandable, actionable plans.
Main sections
Recommended Plan
A machine-specific collection of proposed changes.
Optimization Profiles
Balanced, Performance, and Efficiency preferences.
Individual Actions
User-selectable supported changes.
Advanced Actions
Available only after explicit activation, with additional review requirements.
Recommendation card anatomy
Every recommendation includes:
Clear action title
Explanation
Supporting evidence
Workload relevance
Expected benefit
Confidence
Risk level
Reversibility
Permission requirement
Restart requirement
Review CTA
Action classification
Use four categories:
Informational
Low-risk and supported
Requires careful review
Advanced / manual
Approval flow
Plan → Inspect actions → Check eligibility → Confirm → Apply → Verify → Report
Allow partial plan execution when individual actions are unsupported or the user rejects them.
No generic Optimize Everything operation may modify unrelated operating-system or firmware settings.

16. Maintenance Center
Purpose
Reduce recurring clutter and performance issues through user-controlled maintenance.
Features
Scheduled storage scans
Storage-capacity alerts
Developer-artifact review reminders
Startup configuration review
Diagnostic comparison history
Maintenance recommendations
User-defined excluded directories
Scheduled operations
Separate scheduled analysis from scheduled deletion.
Scheduled scans may run with the user's permission.
Scheduled destructive operations should require explicit per-rule authorization, clear scope restrictions, and appropriate review. Initially, use scheduled scanning with manual cleanup approval.
Use Calendar and Reminders component patterns from the supplied Figma kit.

17. History, Recovery & Audit Trail
Purpose
Ensure users can inspect every system modification performed by CoquiPCCleaner.
Main interface
Show a chronological activity timeline containing:
Diagnostic runs
Recommended plans
Approved actions
Completed actions
Failed operations
Permissions granted or denied
Configuration changes
Recovery operations
Action detail
Include:
Previous state
Requested modification
Actual result
Execution timestamp
Relevant logs
Verification status
Recovery availability
Only display a Rollback action when a supported recovery procedure exists.
Moving a file to Trash does not guarantee that every modification can be reversed, and a completed deletion must not be falsely represented as reversible.

18. Application Settings
Organize Settings into:
General
Appearance
Workload Profile
Optimization Preferences
Storage Scanning
Scan Scheduling
Privacy & Permissions
Protected Locations
Advanced Diagnostics
Notifications
Data Retention
About
Advanced Diagnostics
Include individually configurable controls for:
Expanded hardware analysis
Advanced logs
Experimental diagnostics
Supported firmware inspection
Additional diagnostic permissions
Do not use a single toggle that grants all sensitive permissions at once.
Privacy design
The default operating model is local-first.
System measurements, diagnostic history, user preferences, and cleanup records should remain local in SQLite unless the user explicitly enables a future synchronization or external AI capability.
The user must be able to inspect and remove locally retained diagnostic data.

19. Figma Component Architecture
Recommended Figma file organization
Create a CoquiPCCleaner design file containing:
00 — Reference & Attribution
Reference kit links, reuse rules, component inventory, attribution and licensing notes.
01 — Foundations
Typography, colors, spacing, radii, iconography, appearance modes.
02 — Imported Components
Applicable reusable components from the supplied kit, preserving their original identities.
03 — Coqui Components
Domain-specific cards, system status indicators, diagnostic panels, optimization controls.
04 — Navigation & Shell
Sidebar, toolbar, global search, window layouts.
05 — Onboarding
06 — Dashboard
07 — Ask Coqui
08 — System Analysis
09 — Hardware & Upgrades
10 — Storage Intelligence
11 — Performance
12 — Firmware & Security
13 — Optimization Center
14 — Maintenance
15 — History & Recovery
16 — Settings
17 — Prototypes & Flows
18 — Developer Handoff
New components required
Create reusable application-specific components:
Coqui/DeviceSummary
Coqui/WorkloadProfile
Coqui/DiagnosticFinding
Coqui/EvidencePanel
Coqui/DiagnosticProgress
Coqui/RecommendationCard
Coqui/RiskIndicator
Coqui/ConfidenceIndicator
Coqui/CapabilityStatus
Coqui/HardwareCard
Coqui/WorkloadSuitability
Coqui/UpgradeRecommendation
Coqui/StorageCategory
Coqui/DuplicateGroup
Coqui/DeveloperArtifact
Coqui/CleanupCandidate
Coqui/PermissionGuide
Coqui/OptimizationProfile
Coqui/ActionApproval
Coqui/ExecutionProgress
Coqui/RecoveryStatus
Use consistent component properties for interaction state, severity, enabled status, selection, appearance, and operation availability.

20. Desktop Layout Specifications
Primary reference frame
1440 × 960
Additional design frames:
1280 × 800
1600 × 1000
1920 × 1080
Layout measurements
Element
Initial specification
Sidebar
232 px
Window toolbar region
52 px
Main content padding
24 px
Standard card gap
16 px
Main inspector panel
320 px
Major section spacing
24–32 px
Minimum practical window target
Approximately 1100 × 720

The interface must support resizing, scrolling, keyboard interaction, and overflow management.
Use Figma Auto Layout and constraints rather than absolute-positioning every screen element.
Responsive behavior
At narrower widths:
Collapse or reduce sidebar width.
Move the right inspector into a sheet or dedicated detail page.
Reduce summary-card column count.
Preserve diagnostic explanations and action controls.
Avoid horizontal clipping of paths, tables, and risk disclosures.
Use the supplied kit's solid materials for dense diagnostic surfaces. Reserve glass effects for appropriate window chrome, transient overlays, and selected navigation regions.

21. Flutter Implementation Contract
The design must map to reusable Flutter components rather than a collection of isolated screenshots.
UI architecture
Flutter application shell
Riverpod state management
Declarative routing
Reusable design tokens
Feature-based widgets
Responsive desktop layouts
Native platform integration where needed
Core domain models
Design the UI around typed entities:
MachineSnapshot
WorkloadProfile
SystemCapability
DiagnosticSession
DiagnosticEvidence
DiagnosticFinding
Recommendation
OptimizationPlan
OptimizationAction
CleanupCandidate
PermissionRequirement
UpgradeAssessment
ExecutionRecord
RecoveryRecord
Capability states
Every OS-dependent feature should be able to present:
Supported
Requires permission
Manual only
Unsupported
Unknown
Temporarily unavailable
This is necessary because macOS and Windows expose different diagnostic capabilities.
Backend boundary
Flutter displays findings and collects approvals.
Rust performs scanning, analysis, validation, and supported system operations.
Flutter Rust Bridge provides typed communication between the two.
SQLite stores local configurations, diagnostic records, findings, user preferences, and activity history.
The conversational assistant may explain or request supported diagnostic commands, but must not bypass the action validation and approval system.

22. Core Prototype Flows
The Figma prototype must demonstrate the following complete user journeys.
Flow A — Initial system analysis
Welcome → Workload selection → Permissions → Scan → Dashboard → Findings
Flow B — Natural-language troubleshooting
Describe issue → Diagnostic investigation → Evidence → Proposed fix → Approval → Verification
Flow C — Storage cleanup
Storage overview → Scan → Category → File preview → Select → Confirm → Execute → Results
Flow D — Developer storage cleanup
Developer Storage → Detect environments → Inspect artifacts → Validate project state → Approve → Clean safely → Verify
Flow E — Hardware upgrade decision
System Analysis → Workload Suitability → Bottleneck → Upgrade Assessment → Recommendation
Flow F — Permissions failure
Attempt supported operation → Permission denied → Explanation → Permission guidance → Retry or skip
Flow G — Profile adjustment
Performance → Select profile → Compare proposed changes → Confirm → Apply → Verify
Flow H — Firmware review
Firmware & Security → Read supported properties → Review finding → Guided manual action
Advanced firmware modification, where ever supported, must use a separate high-risk workflow.

23. Release-Aligned Design Roadmap
Phase 1 — System Scanner
Platform: macOS
Design and implement:
Application shell
Onboarding
Workload profiles
Dashboard
Hardware inventory
CPU/RAM/storage monitoring
Startup-item detection
Storage scanner
Read-only firmware/system information
Diagnostics results
Permission explanations
Early bottleneck reporting
The MVP should be useful without conversational AI or destructive automation.
The first complete macOS build must provide reliable scan results and accurate capability disclosures.
Phase 2 — Optimization Engine
Platform: macOS
Add:
Ask Coqui troubleshooting
Evidence-based recommendations
Reviewable optimization plans
Safe cleanup execution
Duplicate file review
Developer artifact analysis
Startup adjustments where supported
Power optimization profiles where supported
Hardware suitability assessments
Upgrade advice
Scheduled maintenance
Execution history
Recovery workflows
Before-and-after measurements
Phase 3 — Advanced Firmware Diagnostics
Platform: macOS capabilities first; expanded Windows functionality later
Add:
Enhanced platform and firmware inventory
Manufacturer-specific compatibility information
Supported configuration auditing
Firmware update guidance
Advanced diagnostic workflows
Explicit advanced-action authorization
Conventional BIOS/UEFI configuration analysis should expand when Windows support is developed and relevant vendor APIs are available.
Post-macOS milestone — Windows implementation
After the first completed macOS build:
Implement the Windows system adapter.
Introduce Windows startup and power management.
Expand BIOS/UEFI analysis where supported.
Add Windows-specific cleanup rules.
Adapt window chrome and platform interactions.
Test Windows-specific permissions and privileged actions.
Maintain shared Flutter design components and Rust interfaces.
Do not assume that a feature available on Windows must appear enabled on macOS.

24. UX Acceptance Criteria
A screen is not complete merely because it matches the Figma design.
Each major workflow must demonstrate:
Accurate device and operation states
Working keyboard navigation
Appropriate loading and empty states
Partial-result handling
Permission-denied handling
Clear risk explanations
Explicit approval before modification
Real progress reporting
Cancellation where safe
Error recovery
Measured or clearly qualified results
No fabricated hardware measurements
No unsupported optimization promises
Product completion principle
CoquiPCCleaner should make the user more informed about the computer regardless of whether any optimization is applied.
A valid successful outcome may be:
“Your computer is performing normally for your workloads. No changes are recommended.”
That is a stronger result than manufacturing problems to justify a cleaning operation.

25. Final Design Direction
CoquiPCCleaner will combine the familiarity and polish of a native macOS application with the depth of a system diagnostics tool and the accessibility of an intelligent troubleshooting assistant.
The application should present actionable explanations rather than overwhelming users with unexplained metrics.
Its defining characteristics are:
Personalized analysis. Evidence-based recommendations. Transparent actions. Safe system maintenance. Realistic hardware guidance.
Every user-facing optimization must support the central promise:
CoquiPCCleaner understands what the user needs from the machine before recommending how to change it.

