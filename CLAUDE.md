@docs/modele/methode.md

# pilote

Piloter une application de bureau de Windows depuis une session Claude : le clavier, l'automatisation de Windows et la capture de la seule fenêtre visée

Pour les sessions de Claude Code du poste d'Edouard : faire les essais installés et les contrôles d'accessibilité de ses projets sans lui, au clavier et par ce que lit un lecteur d'écran.

## État

Né le 2026-11-21 de `modele-projet` v0.14.0. Phase 0 faite : le brief, le cadrage et les ADR 0001 à 0004. Tranche 1 du lot 1 faite : le socle, le clavier, l'application d'essai. Prochaine : la revue finale de la tranche 1, puis la tranche 2 du lot 1.

## Commandes

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                  # le domaine ; les essais de bout en bout restent ignorés
cargo build -p pilote -p appli-essai
cargo install --locked --path .         # depuis un clone à jour : git fetch, puis main..origin/main vide
```

Ne jamais lancer `cargo test -- --ignored` ni `--include-ignored` sur le poste, ni `pilote` ou `appli-essai` hors de `--version` et `--help` avant l'essai sur le poste : les essais de bout en bout pilotent le bureau, et la CI seule les lance.

## Ce qui fait foi

*Pour chaque sujet, l'endroit où la réponse se trouve. En cas de désaccord, c'est cet endroit qui a raison. Un fichier de la racine s'y cite par un lien.*

- Le comportement, c'est le code et les tests.
- Les modules : le domaine pur, `src/domaine/`, qui décide de tout ; la couche Windows, `src/bureau/`, qui lit et agit sans décider ; `src/systeme.rs`, seul à résoudre les chemins de données et à lire l'horloge, la date et l'environnement ([ADR-0003](docs/decisions/0003-domaine-pur-et-couche-windows.md)).
- Ce que le domaine lit du poste et y fait, c'est le trait `Bureau`, dans `src/domaine/bureau.rs`.
- L'ordre des gardes, c'est `src/domaine/gestes.rs`, et ses raisons, [ADR-0005](docs/decisions/0005-garde-fous-fenetre-retenue-et-premier-plan.md).
- La sortie, les codes de sortie et les messages, c'est [ADR-0006](docs/decisions/0006-interface-sortie-codes-journal-et-bornes.md).
- Les contrôles requis avant toute fusion, ce sont les six de la règle de protection de la branche principale : `socle / secrets`, `socle / confidentialite`, `texte-pr`, `verifier`, `windows` et `mutants`.
- Le besoin de départ, c'est [le brief](docs/00-brief-initial.md), et les choix de départ, [le cadrage](docs/cadrage.md), figés depuis la phase 0.
- Les décisions, ce sont [celles du projet](docs/decisions/) et [celles du modèle](docs/modele/decisions/).
- La feuille de route, ce sont les issues (une milestone par étape) et [les pistes](docs/pistes.md).
- Les constats, c'est [le registre](docs/registre.md).

## Invariants — ne jamais violer

Les invariants communs sont dans `docs/modele/methode.md`. Ceux-ci sont propres au projet.

1. Rien ne se tape hors de la fenêtre visée : le premier plan se vérifie avant chaque touche et chaque caractère. Taper dans la mauvaise fenêtre, même une fois, est un échec (ADR-0005).
2. Une capture ne prend que la fenêtre visée, jamais l'écran entier, et l'outil n'en garde aucune copie (ADR-0005).
3. Ce qu'on lit reste dans la fenêtre visée : un focus hors de la fenêtre retenue ne rend ni nom ni valeur (code 1), et lire n'active rien (ADR-0005).
4. Une valeur que l'automatisation marque comme mot de passe n'est jamais rendue, et le journal ne garde jamais un texte tapé (ADR-0005).
5. `nettoyer` n'efface qu'un dossier `pl-<nombre>` de sa liste, directement sous `%TEMP%`, ni jonction ni lien, sur lequel rien ne tourne plus ; il ne tue jamais rien (ADR-0005).
6. Rien ne configure les refus : aucun fichier qu'une session lirait ne peut élargir ni réduire ce que l'outil refuse (ADR-0005).
7. Les règles vivent dans le domaine pur, testé sans fenêtre et éprouvé par la mutation ; la couche Windows ne fait que lire et agir (ADR-0003).
8. Les chemins de données se résolvent dans le seul `src/systeme.rs` ; aucun test ne touche les vrais (ADR-0003).
