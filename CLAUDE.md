@docs/modele/methode.md

# pilote

Piloter une application de bureau de Windows depuis une session Claude : le clavier, l'automatisation de Windows et la capture de la seule fenêtre visée

Pour les sessions de Claude Code du poste d'Edouard : faire les essais installés et les contrôles d'accessibilité de ses projets sans lui, au clavier et par ce que lit un lecteur d'écran.

## État

Né le 2026-11-21 de `modele-projet` v0.14.0. Phase 0 : le brief, le cadrage et les ADR 0001 à 0003, en PR. Ensuite : l'issue du lot 1 et sa PR de plan.

## Commandes

*Aucune avant la première PR d'implémentation.*

## Ce qui fait foi

*Pour chaque sujet, l'endroit où la réponse se trouve. En cas de désaccord, c'est cet endroit qui a raison. Un fichier de la racine s'y cite par un lien.*

- Le comportement, c'est le code et les tests.
- Le besoin de départ, c'est [le brief](docs/00-brief-initial.md), et les choix de départ, [le cadrage](docs/cadrage.md), figés depuis la phase 0.
- Les décisions, ce sont [celles du projet](docs/decisions/) et [celles du modèle](docs/modele/decisions/).
- La feuille de route, ce sont les issues (une milestone par étape) et [les pistes](docs/pistes.md).
- Les constats, c'est [le registre](docs/registre.md).

## Invariants — ne jamais violer

Les invariants communs sont dans `docs/modele/methode.md`. Ceux-ci sont propres au projet.

1. Rien ne se tape hors de la fenêtre visée : le premier plan se vérifie avant chaque touche et chaque caractère. Taper dans la mauvaise fenêtre, même une fois, est un échec (ADR-0002).
2. Une capture ne prend que la fenêtre visée, jamais l'écran entier, et l'outil n'en garde aucune copie (ADR-0002).
3. Une valeur que l'automatisation marque comme mot de passe n'est jamais rendue, et le journal ne garde jamais un texte tapé (ADR-0002).
4. Rien ne configure les refus : aucun fichier qu'une session lirait ne peut élargir ni réduire ce que l'outil refuse (ADR-0002).
5. Les règles vivent dans le domaine pur, testé sans fenêtre et éprouvé par la mutation ; la couche Windows ne fait que lire et agir (ADR-0003).
6. Les chemins de données se résolvent dans le seul `src/systeme.rs` ; aucun test ne touche les vrais (ADR-0003).
