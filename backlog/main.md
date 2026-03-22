# 🐜 Ant Simulator - Orchestrateur de Projet

**Règles strictes pour l'Agent IA :**
1. Tu es un agent autonome. Ton but est de compléter les tâches du projet étape par étape.
2. Ne modifie JAMAIS plusieurs fichiers de features en même temps.
3. Cherche la première feature marquée `[~] IN PROGRESS` ci-dessous. S'il n'y en a pas, prends la première marquée `[ ] TODO` et passe-la en `[~] IN PROGRESS`.
4. Ouvre le fichier `.md` correspondant dans le sous-dossier, lis ses instructions, et exécute UNIQUEMENT la première tâche marquée `[ ] TODO` à l'intérieur de ce sous-fichier.
5. Une fois la tâche codée et le compilateur au vert (`cargo check` ou `npm run build`), mets à jour le statut dans le sous-fichier (`[x] DONE`), explique brièvement ce que tu as fait à l'utilisateur, et ARRÊTE-TOI.

---

## 🗺️ Index des Features

- [x] DONE: `agent-ui-ux/FEATURE_1_VISIBILITE.md` - Refonte visuelle et accessibilité (PixiJS/React)
- [x] DONE: `agent-biologie/FEATURE_2_COMPORTEMENT.md` - IA individuelle, aléatoire et phéromones (Rust)
- [x] DONE: `agent-world/FEATURE_3_GENERATION_EVENTS.md` - Carte procédurale et événements (Rust/Go)
- [x] DONE: `agent-core/FEATURE_4_MOTEUR_CPU.md` - Optimisation mathématique pure (Rust)