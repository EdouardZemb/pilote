# ADR-0001 — L'outil : un exécutable Rust à sous-commandes, sur le crate `windows`, avec une compétence qui pointe vers lui

- **Statut** : accepté le 2026-11-21
- **Contexte amont** : les décisions 2 à 4, dans [le brief](../00-brief-initial.md) (§ 6) ; le design du 2026-11-21, hors du dépôt, sa section « Les approches » ; [le cadrage](../cadrage.md) (§ 1)

## Contexte

Une session de Claude Code en ligne de commande n'a aucun outil pour piloter une application de
bureau. L'essai installé de `pupitre`, le 2026-11-21, s'est fait par un script PowerShell écrit pour
l'occasion. Edouard veut que toutes les sessions du poste aient cet outil, et Rust est son langage
par défaut pour tout outil neuf.

## Options

1. **Un exécutable Rust dans son dépôt, et une compétence qui pointe vers lui.** Un seul
   exécutable, rapide à chaque appel ; le client COM de l'automatisation lit `aria-keyshortcuts` et
   `aria-live` ; le code est versionné, testé en CI et soumis à la mutation. Il faut le construire.
2. **Le script PowerShell, rangé en compétence du poste.** Prêt en une heure, mais il contredit la
   préférence pour Rust, recompile son C# à chaque appel (environ 2 s), son client .NET ne voit pas
   `aria-live`, PowerShell 5.1 lit en ANSI un script sans BOM, et rien ne le versionne.
3. **Le pilotage d'Anthropic, dans l'application de bureau de Claude.** Officiel, mais absent des
   sessions en ligne de commande ; il clique sur des pixels et ne lit ni le focus, ni les noms, ni
   les raccourcis.

## Décision

L'option 1. L'outil s'appelle `pilote`. Chaque appel est un processus, et rien ne persiste entre
deux appels, sauf le journal et la liste des dossiers de données neufs. Chaque sous-commande qui
touche une fenêtre la nomme par le nom de son processus.

Les crates : `windows` pour l'automatisation, le clavier, les fenêtres et GDI ; `clap` pour les
arguments ; `png` pour l'image ; `unicode-normalization` pour le sélecteur. L'outil s'installe par
`cargo install --path .` depuis le clone.

La compétence `piloter` vit dans le dépôt, à `.claude/skills/piloter/SKILL.md`, versionnée avec
l'outil qu'elle décrit. Le pointeur du poste, `~/.claude/skills/piloter/SKILL.md`, dit seulement de
la lire et de l'appliquer.

## Conséquences

- Une session appelle `pilote <sous-commande>` depuis PowerShell 5.1 comme depuis Git Bash ; le
  texte à taper passe aussi par un fichier ou l'entrée standard, et les touches ont des noms.
- Le pilotage d'Anthropic reste disponible à côté, pour une session de l'application de bureau.
- **Difficile à changer** : les noms des sous-commandes, de leurs options et le format de la
  sortie, que les sessions et la compétence apprennent. Un changement passe par la compétence dans
  la même PR.

## Révision

Si le pilotage d'Anthropic arrive dans les sessions en ligne de commande et lit l'accessibilité,
ou si une session ne peut pas lancer d'exécutable natif.
