# 🚀 Resource Collection Simulation — TODO

## ✅ Déjà fait

- [x] Structure du projet Rust
- [x] model.rs (Position, Robot, Resource, Tile, Base)
- [x] Map avec grille Vec<Vec<Tile>>
- [x] Génération obstacles (random)
- [x] Génération ressources (E / C)
- [x] Placement de la base (#)
- [x] Affichage console de la map
- [x] Renommage `Ressource`/`RessouceKind` → `Resource`/`ResourceKind`
- [x] Dérives `Clone`/`Debug`/`Copy`/`PartialEq` sur les structs
- [x] Champ `carrying: Option<ResourceKind>` ajouté sur `Robot`

---

## 🧱 Phase 1 — Simulation simple (sans Ratatui) ✅

### 🤖 Robots (PRIORITÉ)

- [x] Créer une liste de robots (Vec<Robot>)
- [x] Spawn robots à la base
- [x] Afficher robots (x scout, o collector)

### 🧠 Comportement Scout

- [x] Déplacement aléatoire
- [x] Éviter obstacles
- [x] Découvrir ressources
- [x] Mémoriser ressources trouvées (liste partagée — sera remplacée par channels en Phase 2)

### 📦 Comportement Collector

- [x] Aller vers ressource connue
- [x] Collecter 1 unité à la fois
- [x] Stocker temporairement (carrying)
- [x] Retourner à la base
- [x] Décharger ressources

### 🔄 Boucle de simulation

- [x] Boucle principale (loop)
- [x] Mise à jour des robots à chaque tick
- [x] Rafraîchir affichage (clear screen ANSI)
- [x] Ajouter délai (sleep 150ms)

---

## 🧠 Phase 2 — Intelligence & Communication ✅

### 📡 Communication

- [x] Système de messages (enum Message)
- [x] Scouts → envoient ressources découvertes
- [x] Collectors → envoient collecte
- [x] Base → centralise infos

### 🗺️ Connaissance

- [x] Robots ont connaissance locale
- [x] Base a connaissance globale
- [x] Synchronisation infos

---

## ⚙️ Phase 3 — Améliorations techniques

### 🧭 Pathfinding

- [x] Remplacer random par déplacement intelligent
- [x] BFS ou A\* (optionnel mais propre)

### 🪨 Perlin Noise

- [x] Remplacer obstacles random par Perlin noise
- [x] Génération plus naturelle

### 🧱 Robustesse

- [x] Éviter collisions robots
- [x] Gérer ressources épuisées
- [x] Empêcher blocage total

---

## 🎨 Phase 4 — Ratatui (UI)

- [ ] Remplacer print() par Ratatui
- [ ] Affichage couleurs (O, E, C, #, x, o)
- [ ] UI avec compteur ressources
- [ ] Input clavier (quit)

---

## ⚡ Phase 5 — Concurrence (Rust threads)

- [ ] 1 robot = 1 thread
- [ ] Channels pour communication
- [ ] Pas de blocage (async / message passing)

---

## 🏁 Bonus (si t’es chaud)

- [ ] Optimisation déplacements
- [ ] Multi-bases
- [ ] Simulation plus rapide
- [ ] Stats en temps réel

---

## 🧠 Rappel important

> Toujours faire dans cet ordre :
>
> 1. Faire marcher
> 2. Améliorer
> 3. Optimiser
> 4. Rendre joli

---

## 🔥 Objectif final

- Robots autonomes ✔️
- Exploration + découverte ✔️
- Collecte efficace ✔️
- Simulation fluide ✔️
- UI propre Ratatui ✔️

---

💪 Si tu suis ça étape par étape, t’as un projet solide qui peut taper une très bonne
