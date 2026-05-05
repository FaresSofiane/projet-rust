# Projet Rust

Bienvenue dans ce projet Rust réalisé dans le cadre du cursus à l'EFREI.

## Auteurs

- [Sofiane Fares](https://github.com/FaresSofiane)
- [Julien Esnault](https://github.com/julienESN)
- [Galaad Filâtre](https://github.com/Ga1aad)
- [Clément Suire](https://github.com/cleluke)

## Introduction

**Resource Collection Simulation** — une simulation graphique en terminal où des robots
autonomes collectent des ressources sur une carte générée procéduralement.

Deux types de robots :
- **Scouts** (`x`) : explorent la carte et partagent les positions des ressources qu'ils découvrent.
- **Collecteurs** (`o`) : vont chercher les ressources connues, les ramènent à la base (`#`).

La carte contient des obstacles (`O`), des sources d'énergie (`E`) et des gisements de
cristaux (`C`). La base centrale stocke les ressources collectées.

## État d'avancement

- ✅ **Phase 1** — Simulation simple en console (spawn, scout, collector, boucle)
- ⏳ **Phase 2** — Communication par channels (à faire)
- ⏳ **Phase 3** — Pathfinding A*, Perlin noise (à faire)
- ⏳ **Phase 4** — UI Ratatui (à faire)
- ⏳ **Phase 5** — Concurrence (1 robot = 1 thread)

Voir `step.md` pour le détail.

## Installation

```bash
cargo build
```

## Utilisation

```bash
cargo run --release
```

L'animation se rafraîchit toutes les 150 ms. `Ctrl+C` pour quitter.
