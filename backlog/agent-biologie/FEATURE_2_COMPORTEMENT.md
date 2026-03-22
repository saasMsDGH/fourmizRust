# Axe 2 : Simulation Éducative et Stigmergie (Phéromones)

**Rôle :** Expert Rust ECS & Myrmécologie. Le but est de simuler l'intelligence en essaim (Stigmergie). Les fourmis n'ont pas de mémoire globale, elles réagissent à leur environnement immédiat.

## Tâches :
- [x] DONE: Task 2.1 - Implémenter l'état `WANDER` (Recherche). LIRE LE FICHIER `agent-biologie/ALGO_WANDER.md` avant de coder. Implémenter la marche aléatoire avec persistance de direction (Correlated Random Walk).
- [x] DONE: Task 2.2 - Implémenter le dépôt de "Phéromone d'Exploration" (faible) dans la grille spatiale à chaque tick lors du `WANDER`.
- [x] DONE: Task 2.3 - Implémenter l'état `RETURN_TO_NEST`. Lorsqu'une fourmi trouve une ressource, elle fait demi-tour vers les coordonnées du nid et dépose une "Phéromone de Nourriture" (très forte).
- [x] DONE: Task 2.4 - Modifier l'état `WANDER` pour détecter les pistes. LIRE LE FICHIER `agent-biologie/ALGO_GRADIENT.md`. Implémenter le système des 3 capteurs frontaux pour ajuster l'angle de la fourmi vers la source de phéromones.
- [x] DONE: Task 2.5 - Implémenter un système d'évaporation : chaque case de phéromone perd X% de sa valeur à chaque seconde.