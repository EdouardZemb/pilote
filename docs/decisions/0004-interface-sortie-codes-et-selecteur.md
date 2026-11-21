# ADR-0004 — L'interface : une ligne par résultat, des codes de sortie fixes, un sélecteur exact

- **Statut** : accepté le 2026-11-21
- **Contexte amont** : [le brief](../00-brief-initial.md) (§ 6, les contraintes du poste) ; le design du 2026-11-21, hors du dépôt, ses sections « Ce que fait l'outil » et « Les règles fines » ; [le cadrage](../cadrage.md) (§ 1 et 3, et § 2, questions 1 et 3 à 6) ; [ADR-0001](0001-executable-rust-a-sous-commandes.md)

## Contexte

Une session lit ce que rend `pilote`, enchaîne ses appels selon leurs codes de sortie, et désigne
les éléments d'une fenêtre par un sélecteur. Elle l'appelle depuis PowerShell 5.1 comme depuis Git
Bash. Un format flou, un code qui change de sens ou un sélecteur qui choisit au hasard ferait agir
la session sur le mauvais élément.

## Options

1. **Du JSON seulement.** Précis, mais lourd à lire pour une session dans un terminal, et le
   design le laisse optionnel.
2. **Du texte libre, comme le script de l'essai.** Lisible, mais sans forme stable.
3. **Une ligne de texte par résultat, d'une forme fixe, et des codes de sortie qui ont chacun un
   sens.**

## Décision

L'option 3.

**La sortie.** Une ligne par résultat, sur la sortie standard : le type entre crochets, le nom entre
chevrons, puis des champs `clé=valeur`, par exemple
`[Edit] <Chercher dans le texte des idées> id=recherche raccourci=/`. `arbre` indente ses lignes
dans l'ordre du document, et dit le réglage d'annonce d'un élément qui en porte un
(`annonce=polite`). `focus` et `trouver` disent le groupe qui contient l'élément (`dans=<nom>`) :
trois boutons « Toutes » de trois groupes s'y distinguent. La sortie est en UTF-8, sans BOM. Un
refus se dit en une ligne, en français, sur la sortie d'erreur.

**Les codes de sortie.**

| Code | Sens |
|---|---|
| 0 | fait |
| 1 | refus : fenêtre refusée, fenêtre pas au premier plan, poste verrouillé, touche interdite, fenêtre plus élevée, focus hors de la fenêtre visée, fenêtre réduite pour une capture, dossier encore utilisé pour `nettoyer`, un autre geste en cours après 5 s d'attente |
| 2 | introuvable : fenêtre ou élément, ou plusieurs candidats sans `--rang` ; avec `--attendre`, rien de trouvé dans le délai |
| 3 | le geste a échoué : l'élément n'offre pas le geste demandé, ou une capture toute noire |
| 64 | usage : une option inconnue, manquante ou mal formée |

Une erreur d'usage que l'analyse des arguments rendrait en 2 sort en 64.

**La fenêtre.** Chaque sous-commande qui touche une fenêtre la nomme par `--fenetre <processus>`,
le nom du processus sans `.exe` ; `--titre <texte>` départage les fenêtres d'un même processus, et
vise un dialogue précis ; `--pid <nombre>` départage deux processus de même nom, comme deux établis
titrés « pupitre ».

**Le sélecteur.**
- `--nom` compare en égalité exacte, après normalisation NFC, les espaces insécables comptées pour
  des espaces ; `--contient` compare en sous-chaîne ; `--type` filtre sur le type ; `--id` sur
  l'identifiant d'automatisation ; `--classe` sur la classe.
- `--dans <nom>` garde les éléments dont un groupe ancêtre porte ce nom, à toute profondeur : deux
  groupes emboîtés de même nom comptent tous deux.
- Plusieurs éléments trouvés sans `--rang` font refuser, avec le code 2 et le nombre de candidats.
- `--rang` compte à partir de 1.
- `trouver` liste tous les candidats, avec leur rang.

Le choix d'un élément se fait dans le domaine pur, sur l'arbre que la couche Windows a lu
([ADR-0003](0003-domaine-pur-et-couche-windows.md)).

**L'attente.** `--attendre <ms>`, sur `focus`, `trouver`, `valeur` et `arbre`, sonde jusqu'à
trouver l'élément ou la valeur voulus, au plus ce délai : une session ne peut pas dormir entre deux
appels.

**Les textes à taper** passent par `--texte`, par `--fichier`, qui lit l'UTF-8 avec ou sans BOM et
l'UTF-16 à BOM, ou par l'entrée standard, un seul saut de ligne final ignoré. Les touches ont des
noms (`barre` pour `/`), qu'aucun shell ne réécrit ; un caractère nommé se tape par la disposition
du clavier, comme `taper --comme-au-clavier`, et déclenche ainsi le raccourci que la page écoute.

## Conséquences

- La compétence décrit ce format et ces codes ; un test du domaine garde chaque forme de ligne.
- **Difficile à changer** : tout ce que les sessions lisent, le format des lignes et le sens des
  codes. Un changement passe par un nouvel ADR et par la compétence, dans la même PR.

## Révision

Si un usage par un programme, et non par une session, demande une sortie JSON (une piste).
