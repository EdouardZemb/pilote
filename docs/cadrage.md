# Cadrage — pilote

- **Statut** : validé à la fusion, puis figé
- **Date** : 2026-11-21
- **Brief** : [`00-brief-initial.md`](00-brief-initial.md)

*Livrable de la phase 0. Une fois la PR fusionnée, ce document ne bouge plus : ce qui change ensuite passe par un ADR ou par `docs/pistes.md`.*

## 1. Ce que la lecture change

La phase 0 a lu le design et le script de l'essai du 2026-11-21, tous deux hors du dépôt, la
méthode du modèle, la CI de `pupitre` (le seul projet en Rust né du modèle), la documentation des
API de Windows, et le déroulé de l'essai installé de `pupitre`, point par point. Puis trois sondes
ont tourné sur les runners Windows de GitHub, sur une branche jetable (`essai/runner`), jamais
fusionnée.

**Ce qui est établi par la lecture.**

- Tout ce que demande le design se fait avec le crate `windows` (0.62), déjà dans le registre local
  du poste : UI Automation par COM, `SendInput`, les fenêtres, GDI. Le client COM lit le réglage
  d'annonce (`aria-live`, dans `LiveSetting`) et les événements d'annonce, que le client .NET du
  script ne voyait pas. Les raccourcis déclarés (`aria-keyshortcuts`) arrivent dans
  `AcceleratorKey`, que les deux clients lisent.
- L'arbre d'accessibilité de Chromium se bâtit à la première requête : une lecture sonde jusqu'à
  2 s avant de conclure à un arbre vide.
- Une application élevée ne reçoit pas les frappes d'un processus moins élevé, et `SendInput` ne le
  dit pas : l'outil compare le niveau d'intégrité de la fenêtre visée au sien avant tout geste, et
  refuse une fenêtre plus élevée que lui.
- Les Paramètres de Windows tournent dans une fenêtre d'`ApplicationFrameHost`, dont un enfant
  appartient à `SystemSettings` : la liste de refus regarde aussi les fenêtres enfants, sans quoi
  elle se contournerait.
- `--donnees-neuves` ne peut pas partir d'un dossier vide : `SHGetKnownFolderPath`, par où une
  application Tauri trouve son dossier de données, échoue si `AppData\Roaming` et `AppData\Local`
  n'existent pas sous le profil redirigé. L'outil les crée avant le lancement (question 7).
- L'analyse des arguments (`clap`) sort en code 2 sur une erreur d'usage, le code d'« introuvable » :
  l'outil l'intercepte et sort en 64.
- PowerShell 5.1 lit la sortie d'un exécutable natif dans la page de code OEM, et passe son entrée
  standard en ASCII : la compétence règle l'encodage de la console, et le texte accentué passe par
  `--fichier`, qui lit l'UTF-8 avec ou sans BOM et l'UTF-16 à BOM.
- La mutation se lance sous Linux, comme dans `pupitre` : les règles pures vivent dans un module
  qui ne dépend pas de `windows`, compilé et testé sur les deux systèmes.

**Ce qu'ont établi les sondes du runner**, le 2026-11-21, sous Windows Server 2022 et 2025, les
mêmes résultats sur les deux :

- le runner a un bureau interactif (`Default`, session 2, processus interactif), un écran de
  1024 × 768, le délai de verrouillage du premier plan à sa valeur par défaut, comme le poste, le
  runtime WebView2 et Edge ;
- son compte est élevé (niveau d'intégrité haut), quand la session du poste ne l'est pas : en CI,
  `pilote` et l'application d'essai tournent au même niveau ;
- `SendInput` tape en Unicode, accents compris, et le premier plan se donne à une autre fenêtre ;
- l'automatisation lit l'arbre de Chromium : un bouton par son nom, un champ par son identifiant,
  son raccourci déclaré ;
- `PrintWindow` avec le rendu complet capture le contenu d'une fenêtre Chromium, même en
  arrière-plan ; la capture porte l'ombre du cadre, à rogner ;
- **une frappe d'Alt seule ouvre le mode menu** d'une fenêtre Win32 qui a déjà le premier plan, et
  le caractère suivant s'y perd : le Bloc-notes a reçu « onjour » pour « Bonjour ». Un mouvement de
  souris nul, ou Alt entourant une touche sans effet, ne perd rien (question 13).

**Ce qui reste à prouver.**

- Le verrou du premier plan : sur le runner, même un processus neuf, qui n'a envoyé aucune frappe,
  obtient le premier plan sans aucune ruse ; sur le poste, où Edouard tape, cela se prouve à l'essai
  sur le poste.
- `PrintWindow` sur WebView2 lui-même, et non sur Edge : prouvé par l'application d'essai, dès la
  PR qui l'apporte.
- Le refus d'une fenêtre plus élevée : en CI, il faut lancer `pilote` à un niveau plus bas que
  l'application d'essai, ce que le plan dit.
- Que l'automatisation marque `IsPassword` sur les champs de mot de passe de KeePassXC (Qt)
  (question 14).

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
- les options d'une liste déroulante fermée ne sont pas exposées ;
- l'essai n'est pas rejouable à l'identique : le fonds d'essai a changé depuis (son journal est
  passé de 53 à 56 entrées), et « les mêmes constats » doit se définir.

**Les pièges de l'établi**, à tenir dans les essais et la compétence : l'arbre vide à la première
requête ; les processus `msedgewebview2` qui survivent quelques secondes à l'établi ; les listes
fenêtrées, dont les éléments hors de vue n'existent pas ; la valeur vide d'un champ de date, qui se
lit dans ses parties ; une seconde instance sur les mêmes données, refusée par l'établi ;
l'équivalent de 200 %, qui donne 699 × 390 sur l'écran du poste.

## 2. Les réponses d'Edouard

**Le 2026-11-21, sur le design** : « Ok pour le design », puis « Ok pour les défauts », en deux
tours. Les douze décisions sont dans le brief (§ 6).

**Le 2026-11-21, sur la naissance** : « Fais la toi même la naissance ». Sur cet ordre, la session
a lancé `poste/Nouveau-Projet.ps1` de `modele-projet` (11 étapes sur 11), dépôt, réglages, secret et
règle de protection compris, en levant pour ce seul lancement la serrure qui le refuse sous Claude
Code.

**Le 2026-11-21, les questions de la session de construction**, chacune avec son défaut entre
parenthèses. Edouard a répondu le même jour « Ok pour les défauts » : chaque défaut est retenu, et
les ADR 0002 et 0004 portent ceux qui touchent leurs décisions (questions 1, 3 à 6, 13 et 16).

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
   Tauri), dans un crate à part, sans lien avec l'exécutable `pilote`. Les sondes ont trouvé
   WebView2 sur les deux runners : le repli sur une application Win32 n'est plus prévu. *(oui)*
9. Un seul lot, avec toutes les sous-commandes ; quatre PR d'implémentation (le socle et le
   clavier ; la lecture, les gestes et la capture ; la fenêtre et les dossiers, puis les annonces ;
   la compétence), puis l'essai sur le poste, la revue finale et la clôture. *(oui)*
10. L'essai sur le poste : les onze points, sur l'établi installé ; l'outil tape sur le bureau
    d'Edouard une vingtaine de minutes. *(Edouard en donne le moment ; il peut être absent)*
11. *Sans objet* : la PR #48 de `pupitre` est fusionnée depuis le 2026-11-21, avant la naissance
    du dépôt, et le relevé de son essai est dans `pupitre`, `docs/70-qualite-et-verification.md`,
    « Les relevés ».
12. Une liste déroulante fermée : `choisir` n'en ouvre pas les options dans ce lot ; le clavier
    puis `valeur` suffisent, comme le 2026-11-21. *(oui)*
13. `activer` ne frappe jamais Alt seule : rien si la fenêtre a déjà le premier plan ; sinon un
    mouvement de souris nul, puis, à défaut, Alt entourant une touche sans effet ; puis la
    vérification. Un écart à « la frappe d'Alt » du design, que les sondes ont vue perdre un
    caractère. *(oui)*
14. `IsPassword` et les deux règles de KeePassXC s'éprouvent en CI, sur le champ de mot de passe de
    l'application d'essai, et sur une copie de l'application d'essai nommée `KeePassXC.exe`.
    L'essai sur le poste ne touche pas KeePassXC : aucun geste hors des deux de la décision 10. Que
    Qt marque `IsPassword` reste à prouver, plus tard, sur un coffre jetable, si Edouard le
    demande. *(oui)*
15. « Les mêmes constats », critère du lot : chacun des onze points montre le même comportement que
    le 2026-11-21, jugé sur sa forme et non sur ses nombres, sur le fonds d'essai de l'établi ; un
    écart de forme se rapporte, et Edouard le juge. *(oui)*
16. Un seul geste à la fois sur le poste : un verrou que prennent les gestes qui tapent, cliquent,
    activent, défilent, changent la taille, lancent, ferment ou effacent ; les lectures et
    `annonces` ne le prennent jamais. Un geste qui le trouve pris attend 5 s, puis refuse (code 1).
    Une serrure de plus que le design. *(oui)*

## 3. Les décisions

- **Un exécutable Rust à sous-commandes, sur le crate `windows`** ; une compétence du dépôt,
  `piloter`, et un pointeur du poste vers elle ([ADR-0001](decisions/0001-executable-rust-a-sous-commandes.md)).
  *Difficile à changer* : la forme des appels, que les sessions et la compétence apprennent.
- **Les garde-fous sont dans le code, sans aucun fichier de réglage** : la liste de refus, les
  lectures bornées à la fenêtre visée, les valeurs secrètes jamais rendues, les deux règles de
  KeePassXC, le premier plan vérifié avant chaque événement, le poste verrouillé et la fenêtre plus
  élevée refusés, les touches interdites, un seul geste à la fois, `nettoyer` borné ; la compétence
  porte le reste
  ([ADR-0002](decisions/0002-garde-fous-dans-le-code.md)). *Difficile à changer* : une frontière de
  sécurité, qui ne s'élargit qu'avec l'accord d'Edouard.
- **Le domaine est pur et portable** : les règles se testent sans fenêtre, sous Linux comme sous
  Windows, et la mutation les éprouve sous Linux ; la couche Windows est mince. Un seul module
  résout les chemins de données, et aucun test ne touche les vrais
  ([ADR-0003](decisions/0003-domaine-pur-et-couche-windows.md)). *Difficile à changer* : la
  frontière entre les deux, sur laquelle reposent la mutation et la CI.
- **L'interface** : une ligne par résultat, `[Type] <nom> clé=valeur`, en UTF-8 sans BOM ; les codes
  0, 1, 2, 3 et 64, chacun avec son sens ; un refus en une ligne de français sur la sortie d'erreur ;
  un sélecteur exact, qui refuse s'il est ambigu ([ADR-0004](decisions/0004-interface-sortie-codes-et-selecteur.md)).
  *Difficile à changer* : tout ce que les sessions lisent. Pas de sortie JSON dans le premier lot
  (une piste).
- **La CI et `pilote --version`** relèvent du plan du lot : les jobs `verifier`, `windows` et
  `mutants`, la mutation complète chaque semaine, les essais de bout en bout qui bloquent, rendus
  stables par des attentes avec sondage et jamais par des relances, et des délais qui valent deux
  fois le plus long passage à froid.

## 4. Le calendrier

Aucune échéance. L'essai sur le poste se fait au moment que donne Edouard (question 10).

## 5. Le plan par étapes et par lots

**Étape 1, milestone « Lot 1 : l'outil et l'essai de pupitre »** (questions 2 et 9) :

1. la PR de plan ;
2. le socle et le clavier : le crate, le domaine pur, la CI, la sentinelle, `fenetres`, `activer`,
   `touche`, `taper` ;
3. la lecture, les gestes et la capture : `focus`, `arbre`, `trouver`, `focaliser`, `invoquer`,
   `choisir`, `valeur`, `defiler`, `defilables`, `capture`, `clic`, `molette`, et l'application
   d'essai ;
4. la fenêtre et les dossiers, puis les annonces : `taille`, `lancer`, `fermer`, `nettoyer`,
   `annonces` ;
5. la compétence `piloter` et son pointeur du poste ;
6. l'essai sur le poste : les onze points de l'essai de `pupitre`, sur l'établi installé ; ses
   corrections passent par la revue finale ;
7. la revue finale, puis la relecture de ses corrections ;
8. la clôture, puis la page de savoir au cerveau, par `/noter`.

**Ensuite**, au choix d'Edouard : ce que le premier lot laisse dans `docs/pistes.md`.

## 6. Ce qu'Edouard seul peut faire

- Accepter le plan du lot.
- Rendre requis les jobs `verifier`, `windows` et `mutants`, une fois qu'ils existent : la session
  prépare la règle, Edouard lance la seule ligne qui l'écrit, par `!`.
- Donner le moment de l'essai sur le poste.
- Déverrouiller KeePassXC et confirmer sa saisie automatique : aucun geste du premier lot ne le
  demande.
- Poser l'étiquette `v0.1.0`.

La naissance du dépôt, de son secret et de sa règle de protection lui revenait aussi ; il l'a
confiée à la session le 2026-11-21 (section 2).

## 7. Ce qui reste ouvert

- Le moment de l'essai sur le poste, qu'Edouard donne (question 10).
- Ce qui reste à prouver, à la section 1 : le verrou du premier plan sur le poste, `PrintWindow` sur
  WebView2, le refus d'une fenêtre plus élevée en CI, `IsPassword` dans KeePassXC. Si un essai de
  bout en bout se montre instable, la session le dit à Edouard avant d'en changer le statut.
