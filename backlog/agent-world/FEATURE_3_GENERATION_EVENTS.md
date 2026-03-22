# Axe 3 : Génération de la Carte et Événements Globaux

**Rôle :** Expert Backend Rust/Go. Le monde doit être organique (obstacles naturels) et dynamique (météo), sans saturer la bande passante du réseau.

## Tâches :
- [x] DONE: Task 3.1 - Rust : LIRE LE FICHIER `agent-world/ALGO_MAP_GEN.md`. Intégrer la crate `noise` pour générer un terrain procédural basé sur un `seed` unique généré au démarrage du serveur.
- [x] DONE: Task 3.2 - Rust : Mettre à jour le système de déplacement (ECS). Avant de valider la nouvelle position d'une fourmi, interroger la fonction de bruit : si la destination est de l'eau ou un rocher, la fourmi rebondit ou contourne.
- [x] DONE: Task 3.3 - Go / Protobuf : Ajouter le `map_seed` (uint64) au payload initial de connexion WebSocket pour que le Frontend puisse dessiner la même carte.
- [x] DONE: Task 3.4 - Rust : Créer un composant global `WeatherState` (ex: `CLEAR`, `RAIN`). 
- [x] DONE: Task 3.5 - Rust : Modifier le système d'évaporation des phéromones : si `WeatherState == RAIN`, multiplier la vitesse d'évaporation par 10.