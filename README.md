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
- ✅ **Phase 2** — Système de messages (enum `Message`) et connaissance globale de la base
- ✅ **Phase 3** — Pathfinding BFS, Perlin noise
- ✅ **Phase 4** — UI Ratatui (couleurs, compteurs, quitter au clavier)
- ✅ **Phase 5** — Concurrence : 1 robot = 1 thread, communication par canal `mpsc`

Voir `step.md` pour le détail.

### Architecture concurrente (Phase 5)

Chaque robot tourne dans **son propre thread**. L'état du monde (carte + positions)
est partagé via un `Arc<Mutex<World>>` ; les découvertes et collectes sont transmises
à la base — le **thread principal**, consommateur unique — par un canal `std::sync::mpsc`.
La base est ainsi le seul à écrire la connaissance globale (*« share memory by
communicating »*). Le verrou est relâché avant chaque pause des robots : les opérations
sont **non-bloquantes**. À l'appui d'une touche, un `AtomicBool` arrête tous les threads,
qui sont rejoints (`join`) avant restauration du terminal.

## Installation

```bash
cargo build
```

## Utilisation

```bash
cargo run --release
```

Chaque robot avance d'un pas toutes les 200 ms ; l'UI se rafraîchit toutes les 50 ms.
**Toute touche** quitte proprement la simulation.

## Structure du projet

```
src/
├── main.rs         # Point d'entrée : câblage threads + canal + UI
├── config.rs       # Constantes de la simulation (carte, robots, timings)
├── logging.rs      # Initialisation de tracing (fichier logs/simulation.log)
├── model.rs        # Types du domaine : Robot, Base, Resource, Message…
├── map.rs          # Carte : génération Perlin (obstacles) et ressources
├── pathfinding.rs  # BFS, déplacement aléatoire, ciblage de ressources
├── robot.rs        # Boucle de thread et comportements scout / collecteur
├── world.rs        # État partagé (Arc<Mutex<World>>) et spawn des robots
└── ui.rs           # Rendu Ratatui et boucle d'événements terminal
```

## Logs

Les événements de la simulation (découvertes, collectes, dépôts) sont écrits via
[`tracing`](https://docs.rs/tracing) dans `logs/simulation.log` — le terminal reste
réservé à l'interface Ratatui.

```bash
tail -f logs/simulation.log
```

## Tests

```bash
cargo test
```

Les comportements aléatoires sont testés de façon déterministe grâce à un
générateur seedé (`StdRng::seed_from_u64`), injecté en paramètre des fonctions.
