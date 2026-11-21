# ADR-0003 — L'architecture : un domaine pur et portable, une couche Windows mince

- **Statut** : accepté le 2026-11-21
- **Contexte amont** : le design du 2026-11-21, hors du dépôt, sa section « Les tests et la CI » ; la méthode du modèle, « Les familles de garde-fous de l'écosystème » ; [le cadrage](../cadrage.md) (§ 1) ; [ADR-0002](0002-garde-fous-dans-le-code.md)

## Contexte

La mutation et le test d'isolation des chemins de données sont des obligations du projet. La
mutation de `pupitre` tourne sous Linux, où elle est plus rapide et moins chère que sous Windows.
Or presque tout `pilote` appelle Windows.

## Options

1. **Tout dans un seul module, testé sous Windows seulement.** Simple, mais la mutation tournerait
   sous Windows, et la plupart des règles ne se testeraient qu'avec une fenêtre.
2. **Un domaine pur, sans dépendance à `windows`, et une couche Windows mince.** Les règles se
   testent sans fenêtre, sur les deux systèmes ; la couche Windows ne fait que lire et agir.

## Décision

L'option 2.

- **Le domaine** porte les règles : la liste de refus, la valeur d'un champ de mot de passe jamais
  rendue, les règles de KeePassXC, les noms de touches et leurs modificateurs, les touches
  interdites, le choix d'un élément par un sélecteur sur un arbre factice, le format de la sortie,
  l'équivalent de 200 %, une ligne du journal, le décodage d'un fichier à taper, les trois conditions
  de `nettoyer` ([ADR-0002](0002-garde-fous-dans-le-code.md)). Il se compile et se teste sous Linux comme sous Windows ; `cargo-mutants` s'y
  applique, sous Linux, zéro survivant, les mutants équivalents exclus avec leur raison.
- **La couche Windows** (`windows` en dépendance du seul Windows) lit et agit, puis rend au domaine
  des valeurs simples ; elle est éprouvée par les essais de bout en bout du runner Windows. Hors de
  Windows, l'exécutable sort en 64.
- **Les chemins de données** (le journal, la liste des dossiers créés, le dossier temporaire) se
  résolvent dans un seul module, `src/systeme.rs`. Un contrôle statique refuse ailleurs les chemins
  devinés, l'horloge et le hasard. En CI, l'étape « La sentinelle » redirige le profil, et échoue
  si un fichier y paraît. Le binaire livré ne lit aucune variable de surcharge.

## Conséquences

- Chaque règle a un test sans fenêtre ; chaque geste, un essai de bout en bout.
- Une règle glissée dans la couche Windows échapperait à la mutation : la revue de chaque tâche le
  regarde.
- **Difficile à changer** : la frontière entre le domaine et la couche Windows, sur laquelle
  reposent la mutation et la CI.

## Révision

Si la mutation sous Linux manque des mutants que seuls les essais de bout en bout tueraient, ou si
la couche Windows grossit au point de porter des règles.
