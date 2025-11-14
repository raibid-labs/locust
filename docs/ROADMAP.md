# Locust Roadmap (Initial Draft)

## Phase 0: Scaffold (this repo)
- [x] Core types: `Locust`, `LocustPlugin`, `LocustContext`.
- [x] Event pipeline and overlay pass.
- [x] Basic `NavPlugin` stub with simple hint-mode banner.
- [x] Example ratatui app wiring Locust in.

## Phase 1: Real Navigation
- [ ] Introduce `NavTarget` actions (select, activate, scroll).
- [ ] Add ratatui adapters for `List`, `Table`, `Tabs`.
- [ ] Implement hint generation and input decoding.
- [ ] Draw per-target hints instead of the simple banner.

## Phase 2: Omnibar / Command Palette
- [ ] Define an Omnibar plugin with:
    - [ ] input capture when active,
    - [ ] filtering commands and items,
    - [ ] result action dispatch.
- [ ] Add example integrating Omnibar with navigation.

## Phase 3: Overlay Ecosystem
- [ ] Tooltip plugin.
- [ ] "Highlight region" plugin (tours, onboarding).
- [ ] Configuration layer for keymaps and themes.

## Phase 4: Integration & Docs
- [ ] Document integration patterns for existing ratatui apps.
- [ ] Provide reference examples:
    - [ ] Multi-pane dashboard.
    - [ ] Log viewer with jump navigation.
    - [ ] File browser with hint-based navigation.
