# 🐜 Ant Simulator - Plan de Développement Itératif

**Règle pour l'Agent IA :**
1. Lis ce fichier avant chaque action.
2. Trouve la première tâche marquée `[ ] TODO`.
3. Change son statut en `[~] IN PROGRESS` (modifie ce fichier).
4. Exécute le code nécessaire.
5. Vérifie que ça compile (`cargo check` ou `npm run build`).
6. Si c'est bon, passe le statut à `[x] DONE` (modifie ce fichier) et arrête-toi en attendant la validation de l'utilisateur.

---

## Phase 1 : Stabilisation Backend (Rust ECS)
- [x] DONE: Tâche 1.1 - Corriger le warning `NYMPH_METABOLISM_INTERVAL`. Implémenter la logique de consommation dans le système ECS (1 protéine consommée toutes les 90 secondes in-game par les nymphes).
- [x] DONE: Tâche 1.2 - Modifier l'initialisation du `World` pour faire spawn 3 Reines (au lieu d'une seule) avec 10 ouvrières chacune.
- [ ] TODO: Tâche 1.3 - Espacer les 3 Reines géométriquement sur la grille (coordonnées x, y très éloignées) pour qu'elles ne se chevauchent pas au démarrage.
- [ ] TODO: Tâche 1.4 - Ajouter un composant `colony_id` (valeurs 0, 1, et 2) à toutes les entités pour les rattacher à leur faction respective.

## Phase 2 : Lisibilité Frontend (React / PixiJS)
- [ ] TODO: Tâche 2.1 - Modifier la couleur de fond du Canvas PixiJS (actuellement noir) vers une couleur "Terre" (ex: `#2b1d14`) pour améliorer le contraste.
- [ ] TODO: Tâche 2.2 - Adapter le `SimulationRenderer` pour utiliser le `colony_id` reçu. Assigner une couleur distincte par colonie (ex: 0 = Rouge, 1 = Bleu, 2 = Jaune).
- [ ] TODO: Tâche 2.3 - Augmenter drastiquement l'échelle (scale) visuelle de la Reine (4 à 5 fois plus grande qu'une ouvrière) pour la rendre repérable de loin.

## Phase 3 : Navigation et UX (Full-Stack)
- [ ] TODO: Tâche 3.1 - Ajouter un panneau React dans le HUD avec 3 boutons : "Focus Colonie 1", "Focus Colonie 2", "Focus Colonie 3".
- [ ] TODO: Tâche 3.2 - Câbler ces boutons : au clic, récupérer les coordonnées actuelles de la Reine correspondante dans le state React.
- [ ] TODO: Tâche 3.3 - Implémenter la logique de la caméra PixiJS (viewport ou conteneur) pour centrer la vue sur les coordonnées (x, y) de la Reine sélectionnée.

## Phase 4 : Écosystème (Déploiement cible : k3s)
- [ ] TODO: Tâche 4.1 - Backend : Créer un système de pop automatique de protéines végétales (100/min, max 1000 global sur la map).
- [ ] TODO: Tâche 4.2 - Gateway Go : Exposer les endpoints REST pour requêter l'état des ressources globales.
- [ ] TODO: Tâche 4.3 - Vérifier la conformité des ports et des Ingress Traefik dans les manifestes Kubernetes pour ces nouvelles routes.

## Phase 5 : Optimisation Spatiale (Performances Backend)
- [ ] TODO: Tâche 5.1 - Rust : Implémenter une structure de Partitionnement Spatial (ex: Spatial Hash Grid ou QuadTree) pour indexer la position de toutes les entités.
- [ ] TODO: Tâche 5.2 - Rust : Mettre à jour les systèmes ECS (vision, collisions, déplacement) pour qu'ils interrogent uniquement la grille spatiale au lieu de boucler sur toutes les entités du `World`.
- [ ] TODO: Tâche 5.3 - Rust : Ajuster la logique de vélocité pour qu'une fourmi parcoure exactement 3 fois sa propre taille par seconde (en tenant compte du tickrate de 100 TPS).

## Phase 6 : Architecture du Nid & Nouvelles Castes
- [ ] TODO: Tâche 6.1 - Protobuf : Mettre à jour `antsimulator.proto` pour ajouter la caste `SOLDAT` et les types de salles de nid (Chambre Royale, Grenier, Nurserie).
- [ ] TODO: Tâche 6.2 - Rust : Remplacer le point de spawn simple par une génération procédurale de nid (un graphe de 3 salles circulaires reliées par des galeries).
- [ ] TODO: Tâche 6.3 - PixiJS : Mettre à jour le rendu pour afficher visuellement ces salles souterraines (ex: des cercles de fond semi-transparents sous les fourmis).

## Phase 7 : Survie, Succession et Débordement de Ressources
- [ ] TODO: Tâche 7.1 - Rust : Implémenter le vieillissement. Durée de vie d'une Reine = 10 ans in-game. À sa mort, déclencher la mutation d'une ouvrière de la même faction en nouvelle Reine.
- [ ] TODO: Tâche 7.2 - Rust : Les cadavres. À la mort d'une fourmi (hors vieillesse absolue de la reine), transformer son entité en une ressource `ANIMALE` valant 20 protéines.
- [ ] TODO: Tâche 7.3 - Rust : Algorithme de débordement (Flood-fill). Fixer la limite d'une case de la grille spatiale à 30 protéines max. Si un pop de 100 protéines survient, répartir l'excédent sur les cases adjacentes.
Focus sur la Tâche 7.3 (Algorithme de débordement) :
Pour répartir la nourriture sur la grille spatiale avec un maximum de 30 unités par case, utilise un algorithme de parcours en largeur (BFS) itératif avec une VecDeque.

Règles strictes :

Ne fais jamais de récursion pure pour éviter d'exploser la pile d'appels (Stack Overflow).

Si un montant M doit être ajouté à une case ayant déjà C unités : l'espace disponible est S = 30 - C.

Si M > S, remplis la case à 30, et divise le reste (M - S) entre les 4 cases adjacentes (Nord, Sud, Est, Ouest).

Pousse ces voisins et leur part du reste dans la VecDeque.

N'oublie pas de gérer les restes de la division entière (modulo) pour ne perdre aucune protéine.
- [ ] TODO: Tâche 7.4 - PixiJS : Adapter le rendu de la nourriture pour que l'algorithme de débordement soit visible (plusieurs "tas" de 30 adjacents au lieu d'un seul point géant).

## Phase 8 : Interactivité "God Mode" (API & UI)
- [ ] TODO: Tâche 8.1 - Gateway Go : Ajouter une route REST `/api/spawn_animal` pour forcer l'apparition de protéines animales (max 1000 sur la map).
- [ ] TODO: Tâche 8.2 - React : Ajouter un bouton "Spawn Protéines Animales" dans le panneau de contrôle du HUD.
- [ ] TODO: Tâche 8.3 - Rust : Écouter cette nouvelle commande gRPC et exécuter le spawn aléatoire sur la map.