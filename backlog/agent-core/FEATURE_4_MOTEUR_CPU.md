# Axe 4 : Moteur Headless et Optimisation CPU

**Rôle :** Ingénieur Système Rust. Le code doit tourner sur un cluster CPU (k3s) sans jamais saturer la RAM ni le CPU, même avec des dizaines de milliers d'entités.

## Tâches :
- [x] DONE: Task 4.1 - Implémenter la structure `SpatialHashGrid` en Rust. LIRE LE FICHIER `agent-core/ALGO_SPATIAL_GRID.md` avant de coder.
- [x] DONE: Task 4.2 - Créer un système ECS (`update_spatial_grid_system`) qui tourne au tout début de chaque tick pour vider la grille et y réinsérer toutes les entités possédant un composant `Position`.
- [x] DONE: Task 4.3 - Mettre à jour les systèmes de vision (ex: recherche de nourriture) pour utiliser la méthode `query_nearby` de la grille spatiale au lieu de boucler sur tout le `World` hecs.
- [x] DONE: Task 4.4 - Optimisation mémoire : Utiliser des types primitifs stricts (ex: `f32` au lieu de `f64` pour les positions, `u16` pour les quantités de ressources) pour minimiser l'empreinte cache du CPU.