# 🚀 Resource Collection Simulation — TODO

## ✅ Déjà fait
- [x] Structure du projet Rust
- [x] model.rs (Position, Robot, Resource, Tile, Base)
- [x] Map avec grille Vec<Vec<Tile>>
- [x] Génération obstacles (random)
- [x] Génération ressources (E / C)
- [x] Placement de la base (#)
- [x] Affichage console de la map

---

## 🧱 Phase 1 — Simulation simple (sans Ratatui)

### 🤖 Robots (PRIORITÉ)
- [ ] Créer une liste de robots (Vec<Robot>)
- [ ] Spawn robots à la base
- [ ] Afficher robots (x scout, o collector)

### 🧠 Comportement Scout
- [ ] Déplacement aléatoire
- [ ] Éviter obstacles
- [ ] Découvrir ressources
- [ ] Mémoriser ressources trouvées

### 📦 Comportement Collector
- [ ] Aller vers ressource connue
- [ ] Collecter 1 unité à la fois
- [ ] Stocker temporairement (carrying)
- [ ] Retourner à la base
- [ ] Décharger ressources

### 🔄 Boucle de simulation
- [ ] Boucle principale (loop)
- [ ] Mise à jour des robots à chaque tick
- [ ] Rafraîchir affichage
- [ ] Ajouter délai (sleep)

---

## 🧠 Phase 2 — Intelligence & Communication

### 📡 Communication
- [ ] Système de messages (enum Message)
- [ ] Scouts → envoient ressources découvertes
- [ ] Collectors → envoient collecte
- [ ] Base → centralise infos

### 🗺️ Connaissance
- [ ] Robots ont connaissance locale
- [ ] Base a connaissance globale
- [ ] Synchronisation infos

---

## ⚙️ Phase 3 — Améliorations techniques

### 🧭 Pathfinding
- [ ] Remplacer random par déplacement intelligent
- [ ] BFS ou A* (optionnel mais propre)

### 🪨 Perlin Noise
- [ ] Remplacer obstacles random par Perlin noise
- [ ] Génération plus naturelle

### 🧱 Robustesse
- [ ] Éviter collisions robots
- [ ] Gérer ressources épuisées
- [ ] Empêcher blocage total

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
