# Cadrage — pilote

- **Statut** : validé à la fusion, puis figé
- **Date** : 2026-11-21
- **Brief** : [`00-brief-initial.md`](00-brief-initial.md)

*Livrable de la phase 0. Une fois la PR fusionnée, ce document ne bouge plus : ce qui change ensuite passe par un ADR ou par `docs/pistes.md`.*

## 1. Ce que la lecture change

La phase 0 a lu le design et le script de l'essai du 2026-11-21, la méthode du modèle, la CI de
`pupitre` (le seul projet en Rust né du modèle), la documentation des API de Windows, et le
déroulé de l'essai installé de `pupitre`, point par point.

**Ce qui est établi.**

- Tout ce que demande le design se fait avec le crate `windows` (0.62), déjà dans le registre local
  du poste : UI Automation par COM, `SendInput`, les fenêtres, GDI. Le client COM lit ce que le
  client .NET du script ne voyait pas : les raccourcis déclarés (`aria-keyshortcuts`, dans la
  propriété `AcceleratorKey`) et le réglage d'annonce (`aria-live`, dans `LiveSetting`).
- L'arbre d'accessibilité de Chromium se bâtit à la première requête : une lecture sonde jusqu'à
  2 s avant de conclure à un arbre vide.
- Le premier plan ne se donne pas à un processus d'arrière-plan sans la frappe d'Alt, et le poste
  a le délai de verrouillage du premier plan à sa valeur par défaut : l'outil restaure une fenêtre
  réduite, frappe Alt, demande le premier plan, puis vérifie qu'il l'a, en sondant.
- Une application élevée ne reçoit pas les frappes d'un processus non élevé, et `SendInput` ne le
  dit pas : l'outil lit le niveau d'intégrité de la fenêtre visée avant tout geste, et refuse.
- Les Paramètres de Windows tournent dans une fenêtre d'`ApplicationFrameHost`, dont un enfant
  appartient à `SystemSettings` : la liste de refus regarde aussi les fenêtres enfants, sans quoi
  elle se contournerait.
- `--donnees-neuves` ne peut pas partir d'un dossier vide : WebView2 échoue si
  `AppData\Roaming` et `AppData\Local` n'existent pas sous le profil redirigé. L'outil les crée
  avant le lancement (un écart au mot « vide » du design, à la question 7).
- L'analyse des arguments (`clap`) sort en code 2 sur une erreur d'usage, le code d'« introuvable »
  du design : l'outil l'intercepte et sort en 64.
- PowerShell 5.1 lit la sortie d'un exécutable natif dans la page de code OEM, et passe son entrée
  standard en ASCII : la compétence règle l'encodage de la console, et le texte accentué passe par
  `--fichier`, qui lit l'UTF-8 avec ou sans BOM et l'UTF-16 à BOM.
- La mutation se lance sous Linux, comme dans `pupitre` : les règles pures vivent dans un module
  qui ne dépend pas de `windows`, compilé et testé sur les deux systèmes.

**Ce qui n'est pas prouvé**, et que la première PR d'implémentation éprouve avant tout le reste :

- que le runner Windows de GitHub a un bureau interactif où `SendInput` et le premier plan
  marchent : aucun projet n'y a encore ouvert de fenêtre ;
- que `PrintWindow` avec le rendu complet capture le contenu de WebView2 : le script de l'essai
  copiait l'écran ;
- que le runner a le runtime WebView2.

**Ce que l'essai de `pupitre` du 2026-11-21 révèle dans le design**, chacun tranché par une
question de la section 2 :

- une session ne peut pas dormir entre deux appels, et le script dormait jusqu'à 1,5 s après un
  geste : il manque une attente par sondage ;
- les annonces ne s'écoutent que pendant les gestes : la sous-commande doit pouvoir tourner en
  arrière-plan, et dire quand elle écoute ;
- deux établis peuvent avoir chacun une fenêtre titrée « pupitre » (le fonds habituel et un fonds
  neuf) : le nom du processus et le titre ne les départagent pas ;
- les trois « Toutes » de trois groupes sont indiscernables au Tab : `focus` et `trouver` doivent
  dire le groupe ;
- `--dans` doit dire quoi faire de deux groupes emboîtés de même nom ;
- `touche barre` doit taper `/` par la disposition du clavier ;
- les options d'une liste déroulante fermée ne sont pas exposées.

**Les pièges de l'établi**, à tenir dans les essais et la compétence : l'arbre vide à la première
requête ; les processus `msedgewebview2` qui survivent quelques secondes à l'établi ; les listes
fenêtrées, dont les éléments hors de vue n'existent pas ; la valeur vide d'un champ de date, qui se
lit dans ses parties ; une seconde instance sur les mêmes données, refusée par l'établi ;
l'équivalent de 200 %, qui donne 699 × 390 sur l'écran du poste.

## 2. Les réponses d'Edouard

**Le 2026-11-21, sur le design** : « Ok pour le design », puis « Ok pour les défauts », en deux
tours. Les douze décisions sont dans le brief (§ 6).

**Le 2026-11-21, sur la naissance** : « Fais la toi même la naissance ». La session a lancé
`poste/Nouveau-Projet.ps1` de `modele-projet` (11 étapes sur 11), en levant pour ce seul lancement
la serrure qui le refuse sous Claude Code.

**Le 2026-11-21, les questions de la session de construction**, chacune avec son défaut entre
parenthèses. *Réponse attendue.*

1. Une option `--attendre <ms>` sur `focus`, `trouver`, `valeur` et `arbre`, qui sonde jusqu'à
   trouver l'élément ou la valeur voulus. *(oui)*
2. `annonces` dans le premier lot, en dernière tâche, retirable ; elle écrit « écoute prête » une
   fois abonnée, et la session la lance en arrière-plan pendant les gestes. *(oui)*
3. Les sélecteurs gagnent `--id` (l'identifiant d'automatisation) et `--classe`, et la fenêtre
   `--pid`, pour départager deux processus de même nom. *(oui)*
4. `focus` et `trouver` disent le groupe qui contient l'élément (`dans=<nom>`) ; `arbre` dit le
   réglage d'annonce (`annonce=polite`). *(oui)*
5. `--dans <nom>` : il suffit qu'un groupe ancêtre porte ce nom ; deux groupes emboîtés de même nom
   comptent tous deux. *(oui)*
6. `touche barre` et les autres caractères nommés passent par la disposition du clavier, comme
   `--comme-au-clavier`. *(oui)*
7. Le dossier de `--donnees-neuves` porte d'emblée `AppData\Roaming` et `AppData\Local` : un écart
   au mot « vide » du design. *(oui)*
8. L'application d'essai de la CI : une page HTML dans WebView2, par `wry` et `tao` (les briques de
   Tauri), dans un crate à part, sans lien avec l'exécutable `pilote` ; repli sur une application
   Win32 si le runner n'a pas WebView2. *(oui)*
9. Un seul lot, avec toutes les sous-commandes ; quatre PR d'implémentation (le socle et le
   clavier ; la lecture, les gestes et la capture ; la fenêtre et les dossiers, puis les annonces ;
   la compétence), puis la revue finale, l'essai sur le poste et la clôture. *(oui)*
10. L'essai sur le poste : les onze points, sur l'établi installé ; l'outil tape sur le bureau
    d'Edouard une vingtaine de minutes. *(Edouard en donne le moment ; il peut être absent)*
11. La PR #48 de `pupitre` : la construction ne l'attend pas ; seul l'essai sur le poste attend sa
    fusion, pour s'appuyer sur son relevé. *(oui)*
12. Une liste déroulante fermée : `choisir` n'en ouvre pas les options dans ce lot ; le clavier
    puis `valeur` suffisent, comme le 2026-11-21. *(oui)*

## 3. Les décisions

- **Un exécutable Rust à sous-commandes, sur le crate `windows`** ; une compétence du dépôt,
  `piloter`, et un pointeur du poste vers elle ([ADR-0001](decisions/0001-executable-rust-a-sous-commandes.md)).
  *Difficile à changer* : la forme des appels, que les sessions et la compétence apprennent.
- **Les garde-fous sont dans le code, sans aucun fichier de réglage** : la liste de refus, les
  valeurs secrètes jamais rendues, les deux règles de KeePassXC, le premier plan vérifié avant
  chaque événement, le poste verrouillé et l'application élevée refusés, les touches interdites,
  un seul appel à la fois sur le poste ([ADR-0002](decisions/0002-garde-fous-dans-le-code.md)).
- **Le domaine est pur et portable** : les règles se testent sans fenêtre, sous Linux comme sous
  Windows, et la mutation les éprouve sous Linux ; la couche Windows est mince. Un seul module
  résout les chemins de données, et aucun test ne touche les vrais
  ([ADR-0003](decisions/0003-domaine-pur-et-couche-windows.md)).
- **La sortie** est du texte, une ligne par résultat, `[Type] <nom> clé=valeur`, en UTF-8 sans BOM ;
  les codes sont ceux du design (0, 1, 2, 3, 64) ; un refus se dit en une ligne de français sur la
  sortie d'erreur. Pas de sortie JSON dans le premier lot : le texte est fait pour une session
  (une piste).
- **La CI** ajoute au socle trois jobs : `verifier` (le formatage, `clippy`, les tests du domaine,
  le `Cargo.lock` à jour), `windows` (la construction, les tests, la sentinelle des chemins de
  données, les essais de bout en bout) et `mutants` (sur chaque PR, les mutants du diff, zéro
  survivant), plus une mutation complète chaque semaine. Les essais de bout en bout bloquent ;
  leur stabilité vient d'attentes par sondage, jamais de relances. Le délai de chaque job vaut deux
  fois son plus long passage à froid, et ne baisse jamais.
- **`pilote --version`** dit la version et le commit, lu à la construction, sans `git` exigé.

## 4. Le calendrier

Aucune échéance. L'essai sur le poste attend la fusion de la PR #48 de `pupitre`, puis le moment
que donne Edouard (question 10).

## 5. Le plan par étapes et par lots

**Étape 1, milestone « Lot 1 : l'outil et l'essai de pupitre »** (sous réserve des questions 2 et
9) :

1. la PR de plan ;
2. le socle et le clavier, après la preuve sur le runner Windows (bureau interactif, `SendInput`,
   premier plan, `PrintWindow` sur WebView2) : le crate, le domaine pur, la CI, la sentinelle,
   `fenetres`, `activer`, `touche`, `taper` ;
3. la lecture, les gestes et la capture : `focus`, `arbre`, `trouver`, `focaliser`, `invoquer`,
   `choisir`, `valeur`, `defiler`, `defilables`, `capture`, `clic`, `molette`, et l'application
   d'essai ;
4. la fenêtre et les dossiers, puis les annonces : `taille`, `lancer`, `fermer`, `nettoyer`,
   `annonces` ;
5. la compétence `piloter` et son pointeur du poste ;
6. la revue finale ;
7. l'essai sur le poste : les onze points de l'essai de `pupitre`, sur l'établi installé ;
8. la clôture, puis la page de savoir au cerveau, par `/noter`.

**Ensuite**, au choix d'Edouard : ce que le premier lot laisse dans `docs/pistes.md`.

## 6. Ce qu'Edouard seul peut faire

- Accepter le plan du lot.
- Rendre requis les jobs `verifier`, `windows` et `mutants`, une fois qu'ils existent : la session
  prépare la règle, Edouard lance la seule ligne qui l'écrit, par `!`.
- Donner le moment de l'essai sur le poste.
- Déverrouiller KeePassXC et confirmer sa saisie automatique : aucun geste du premier lot ne le
  demande ; le refus de frappe se vérifie sur le champ du mot de passe de l'écran de déverrouillage.
- Poser l'étiquette `v0.1.0`.

## 7. Ce qui reste ouvert

- Les douze questions de la section 2, à Edouard, avant le plan.
- Les trois points non prouvés de la section 1, tranchés par la première PR d'implémentation ; s'ils
  échouent, la session le dit à Edouard avant de changer le statut des essais de bout en bout.
- Si l'automatisation marque bien `IsPassword` sur les champs de mot de passe de KeePassXC (Qt) :
  éprouvé à l'essai sur le poste.
