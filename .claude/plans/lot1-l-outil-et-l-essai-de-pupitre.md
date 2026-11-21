# Lot 1 · l'outil et l'essai de pupitre — plan maître

> **Pour les agents :** ce plan n'est pas un plan tâche par tâche. Il fixe les sept tranches du lot, leur ordre, ce
> que chacune livre et prouve, et les décisions qui valent pour toutes. Les tranches 1 à 4 ont chacune leur plan
> détaillé, écrit juste avant elles par `superpowers:writing-plans`, ajouté dans `.claude/plans/` par sa propre PR de
> plan, relu par une session neuve, puis exécuté par `superpowers:subagent-driven-development`. Un plan de tranche qui
> contredit ce plan maître le dit dans sa PR, et Edouard relit ce point. Les tranches 5 à 7 se décrivent ici.

**But :** l'outil `pilote`, entier, et la compétence `piloter` qui l'apprend aux sessions du poste ; puis les onze
points de l'essai installé de `pupitre` rejoués par l'outil, sur l'établi installé, au lieu du script du 2026-11-21.

**Architecture :** un espace de travail Rust : le paquet `pilote` à la racine, bibliothèque et exécutable, et
l'application d'essai à part. Le domaine de `pilote`, pur et portable, décide de tout : il analyse les arguments,
retient la fenêtre et l'élément, tient les gardes dans leur ordre, formate la sortie et la ligne du journal, mesure
les attentes. Il agit sur un bureau abstrait, le trait `Bureau`, que réalisent la couche Windows pour l'exécutable et
un faux bureau pour les tests. La couche Windows lit et agit, sans décider ; un seul module, `src/systeme.rs`, résout
les chemins de données, lit l'horloge et l'environnement.

**Pile :** Rust 1.99.1, édition 2024, résolveur 3 ; `windows` 0.62.2 et `windows-core` 0.62.2 (sous Windows
seulement) ; `clap` 4.6.7 (`derive`) ; `png` 0.18.1 ; `unicode-normalization` 0.1.26 ; pour l'application d'essai,
`wry` 0.57.1 et `tao` 0.37.2 (sous Windows seulement). En CI, `cargo-mutants` 27.1.0. Toutes ces versions sont dans
le registre du poste. Une dépendance de plus se nomme dans sa PR, avec sa raison.

**Spec :** [le brief](../../docs/00-brief-initial.md) ; [le cadrage](../../docs/cadrage.md), § 1, les questions 1 à
16 du § 2 (« Ok pour les défauts », le 2026-11-21) et le § 5 ; les ADR 0001 à 0004 ; le design du 2026-11-21, hors du
dépôt, que le brief reprend ; la méthode du modèle ; l'issue #2.

---

## Les décisions de ce plan

### Ce qu'Edouard a tranché

Le design, le 2026-11-21, et les seize questions du cadrage, le même jour, chacune à son défaut (cadrage, § 2). Ce
plan ne rouvre aucune d'elles. Il change deux ADR acceptés, chaque fois pour resserrer une garde ou pour dire ce que
la spec laissait ouvert :
- ADR-0002 (les garde-fous) : la fenêtre visée, le premier plan, ce qu'on lit, KeePassXC et le journal — la
  décision 5, la règle de KeePassXC de la décision 6, et la décision 9 ;
- ADR-0004 (l'interface) : la sortie, les codes, la fenêtre nommée, les bornes et l'attente, avec les options du
  sélecteur que `focus` reçoit et le sens de `--dans` — les décisions 7, 8, 11 et 12, et ce que la décision 5 dit
  des candidates.

Comme un ADR ne se réécrit pas, ADR-0005 remplacera ADR-0002 et ADR-0006 remplacera ADR-0004, à la tranche 1 :
chacun reprend tout l'ancien, avec ces changements, et l'ancien prend le statut « remplacé par ». Accepter ce plan,
c'est en donner l'accord ; un changement ultérieur passe par un nouvel ADR.

### Ce que ce plan tranche

1. **Un plan maître, puis un plan par tranche.** Edouard accepte ce plan ; une session neuve relit chaque plan de
   tranche, et Edouard seulement ce qui change ce plan-ci. Chaque tranche d'implémentation tient en une PR : les
   quatre PR d'implémentation de la question 9 sont celles des tranches 1 à 4. Chaque PR porte sa section
   « Écarts à la spec », avec les écarts que nomme la décision 20.
2. **L'application d'essai arrive dès la tranche 1**, et non à la deuxième étape d'implémentation où le cadrage la
   rangeait (§ 5) : le clavier s'éprouve ainsi sur WebView2 dès sa propre PR. Elle vit dans `essai/appli/`, le
   membre `appli-essai` de l'espace de travail, sans lien avec l'exécutable `pilote`, et son `main` est réservé à
   Windows, avec un repli qui compile ailleurs. Sa page HTML porte un champ avec `aria-keyshortcuts="/"`, un champ
   de mot de passe, des boutons, trois groupes radio qui ont chacun un « Toutes », deux groupes emboîtés de même
   nom, une case à cocher, une liste `<select>`, une liste qui défile et une région `aria-live`. Elle ouvre une
   seconde fenêtre sur demande, et sait figer sa boucle d'événements quelques secondes, sur un message de la page.
   Elle reçoit en arguments son dossier de données WebView2, propre à chaque instance, et un fichier où elle écrit
   ce que la page reçoit (saisies, touches, focus), une ligne écrite et vidée par événement : les essais du clavier
   se vérifient ainsi sans l'automatisation, que la tranche 2 apporte. Le job `windows` la construit avant les
   essais, qui la trouvent à côté de l'exécutable `pilote` et refusent clairement si elle manque. Le cadrage (§ 1)
   voulait `PrintWindow` prouvé sur WebView2 « dès la PR qui l'apporte » : ce plan le lit comme la PR qui apporte
   la capture, celle de la tranche 2.
3. **Le domaine pilote un bureau abstrait.** Chaque sous-commande s'écrit dans le domaine, contre le trait
   `Bureau`, et se teste sous Linux sur un faux bureau qui enregistre chaque appel et chaque lot d'événements :
   l'ordre des gardes, le premier plan vérifié avant chaque combinaison, chaque caractère, chaque clic, chaque cran
   et chaque appel de l'automatisation, le fait qu'une lecture n'active rien, sont
   des règles du domaine, donc mutées. Le trait porte aussi l'horloge : un instant monotone et une attente, qu'un
   temps simulé réalise dans le faux bureau ; chaque durée (100 ms, 2 s, 5 s, les bornes) est une valeur du domaine.
   La couche Windows réalise `Bureau` par des appels directs, sans condition qui décide. La tranche 1 crée le trait
   avec ce que le clavier demande ; chaque tranche l'étend.
4. **Les modules.** `src/domaine/` (pur : `cli`, `sortie`, `touches`, `texte`, `journal`, `refus`, `processus`,
   `fenetre`, `selecteur`, `gestes`, `equivalent`, `dossiers`, `bureau`, et le faux bureau sous `#[cfg(test)]`) ;
   `src/bureau/` (la couche Windows, sous `#[cfg(windows)]`) ; `src/systeme.rs` ; `src/main.rs`, qui assemble, et
   qui hors de Windows sort en 64 avec « pilote ne tourne que sous Windows ». La couche ne s'appelle pas `windows`,
   nom du crate.
5. **La fenêtre retenue et le premier plan** (ADR-0005, qui remplace ADR-0002 à la tranche 1) :
   - les fenêtres candidates sont de premier niveau, visibles, non masquées, ni `WS_EX_TOOLWINDOW` ni sans titre, et
     appartiennent à un processus nommé par `--fenetre` ou à l'un de ses descendants. Une liste ouverte ou une bulle
     de WebView2, sans titre, n'en est donc pas. Les candidates que la liste de refus refuse s'écartent avant le
     compte, et ne sont jamais citées ; s'il n'en restait que de refusées, l'outil refuse (1). Il en faut alors
     exactement une, ou `--titre` (une égalité exacte, comme `--nom`) ou `--pid` pour en garder une ;
   - la fenêtre retenue W appartient au processus P ; l'arbre retenu est P et ses descendants, à l'épreuve des PID
     repris. Ni le nom passé à `--fenetre`, ni la racine de son arbre ne servent ensuite à rien juger ;
   - la liste de refus et les règles de KeePassXC se jugent sur P, et sur le processus de chaque fenêtre enfant de
     W. Un cadre d'`ApplicationFrameHost` est refusé quand un enfant appartient à `SystemSettings`, quand son
     identifiant d'application est celui des Paramètres, ou quand ni l'un ni l'autre ne se lit. Le refus se juge de
     nouveau après l'activation, qui peut rattacher l'enfant d'un cadre réduit ;
   - le premier plan est accepté quand la fenêtre au premier plan est W, appartient à P, ou a W pour propriétaire
     (un dialogue). La spec acceptait toute fenêtre de l'arbre du processus nommé : sous `--fenetre explorer`, c'eût
     été presque tout le poste. Pour une application Tauri, le contenu de WebView2 est fait de fenêtres filles de W,
     et ce resserrement ne retire rien. Les tests du domaine `keepassxc_par_explorer_refuse_la_capture` et
     `deux_racines_de_meme_nom_le_premier_plan_sur_l_autre_refuse` distinguent cette règle de celle du nom passé ;
   - `focus` ne rend l'élément focalisé que si le premier plan est accepté et que l'élément appartient à l'arbre
     retenu ; `arbre` et `trouver` partent de W ; `annonces` ne garde que les annonces d'un élément de W ou d'une
     fenêtre que W possède ;
   - `fenetres` liste les candidates de tous les processus, sauf celles que la liste de refus refuse, par la même
     règle. Une fenêtre masquée n'est pas candidate (2) ; une fenêtre réduite l'est : `capture` la refuse (1), et
     `activer` la restaure.
6. **L'ordre des gardes**, dans le domaine, une position un test :
   - à l'analyse : l'usage (64) ; une touche interdite (1), qui n'est pas un nom inconnu (64) ; puis le journal, qui
     doit s'ouvrir (1) ;
   - puis la fenêtre retenue : refusée (1), absente (2), plusieurs (2, avec leurs titres) ;
   - pour un geste qui vise une fenêtre : le verrou des gestes (5 s, puis 1) ; le poste verrouillé (1) ; un niveau
     d'intégrité plus haut que celui de `pilote`, ou illisible (1) ; une touche de modification déjà enfoncée (1) ;
     l'activation de la question 13, qui restaure d'abord une fenêtre réduite (1 si Windows refuse) ; le refus jugé
     de nouveau sur W ; puis, avant chaque combinaison, chaque caractère, chaque clic, chaque cran et chaque appel
     de l'automatisation, le premier plan accepté (1, et le refus dit combien d'unités, combinaisons ou caractères,
     sont parties) ;
   - une combinaison (les modificateurs enfoncés, la touche, les modificateurs relâchés) et un caractère tapé par la
     disposition du clavier partent en un seul envoi : le premier plan se vérifie avant elle, jamais au milieu ;
   - pour `clic` et `molette`, le point doit tomber dans la zone cliente, et la fenêtre de premier niveau sous lui
     (`GetAncestor(…, GA_ROOT)` de la fenêtre que rend `WindowFromPoint`) être acceptée comme le premier plan (1),
     avant chaque clic ou cran : un clic sur le contenu de WebView2 passe, une fenêtre qui recouvre le point refuse ;
   - quand P est KeePassXC (ADR-0005) : `taper` refuse si le focus est dans un champ de mot de passe, vérifié avant
     chaque caractère, et refuse d'emblée un texte qui porte une tabulation ou un saut de ligne ; dans un champ de
     mot de passe, `touche` n'admet que les touches de déplacement (Tab, Maj+Tab, les flèches, Debut, Fin,
     PageHaut, PageBas, Echap), le focus vérifié avant chaque touche ;
   - `taille` et `fermer` : la fenêtre retenue, le verrou, le poste verrouillé, le niveau d'intégrité, sans
     activation ni premier plan ; `lancer` : un exécutable dont le nom est dans la liste de refus (1), le verrou, le
     poste verrouillé ; `nettoyer` : le verrou, puis ses trois conditions ;
   - une lecture ne prend que la fenêtre retenue, et ne touche ni au verrou ni au premier plan.
7. **Le vocabulaire de la sortie** (ADR-0006, qui remplace ADR-0004 à la tranche 1), que la compétence décrit
   et qu'un test du domaine garde, forme par forme :
   - le type d'un élément est celui de l'automatisation, sans « ControlType » (`Edit`, `Button`, `Group`,
     `RadioButton`, `Document`…), comme le 2026-11-21 ; les autres lignes ont leur type : `Fenetre`, `Capture`,
     `Processus`, `Dossier`, `Annonce`, `Ecoute` ;
   - les clés sont en ASCII, sans accent : `id`, `classe`, `valeur`, `raccourci` (la touche déclarée), `acces`,
     `annonce` (`polite` ou `assertive`), `etat` (une liste, séparée par des virgules, parmi `coche`, `decoche`,
     `indetermine`, `choisi`, `inactif`, `deplie`, `replie`, `lecture-seule`), `dans`, `rang`, `motdepasse=oui`
     (qui remplace la valeur), `processus`, `pid`, `premier-plan=oui` ;
   - dans un nom, `\` s'écrit `\\`, un retour à la ligne `\n`, un retour chariot `\r`, une tabulation `\t`, `>`
     s'écrit `\>`, et tout autre caractère de contrôle, U+2028 et U+2029 compris, `\u{XXXX}` ; une valeur porte les
     mêmes échappements, et s'écrit entre guillemets si elle contient une espace ou un guillemet, ses guillemets en
     `\"` ; une valeur vide s'écrit `""`, une valeur absente n'a pas sa clé. `--nom` et `--titre` reçoivent le texte
     brut, sans échappement ;
   - un refus ou une erreur d'usage s'écrit en une ligne, en français, sur la sortie d'erreur ; les titres qu'il cite
     s'y écrivent chacun entre chevrons, échappés de même ; `clap` est traduit pour les erreurs courantes (option
     inconnue, valeur manquante, option requise).
8. **Les codes, précisés** (ADR-0006) : le code 1 couvre aussi le journal qui ne s'ouvre pas (« journal
   inaccessible : ouvert dans une autre application ? »), une touche de modification enfoncée, un niveau
   d'intégrité illisible, une fenêtre refusée après l'activation, un point de clic ou de molette hors de la
   fenêtre retenue, les refus de KeePassXC pour `taper` et `touche`, `valeur` sur un champ de mot de passe, un
   exécutable refusé pour `lancer`, un dossier dont le chemin n'est plus `%TEMP%\pl-<n>` pour `nettoyer`, et, avec
   `--comme-au-clavier`, un caractère absent du clavier, une touche morte ou Verr. Maj ; chaque refus dit sa
   raison, et chaque essai d'un refus vérifie ce message, pas seulement le code ; le code 3 couvre aussi une erreur du système que rien ne prévoit et un délai de l'automatisation
   dépassé, le message disant lequel (« échec du système : … »). Une panique est rattrapée : sa ligne va au journal,
   et l'outil sort en 3.
9. **Le journal** (ADR-0005) : une ligne par appel, y compris un appel refusé ou mal formé, formée en mémoire puis ajoutée en
   une seule écriture. En tabulations : la date locale du poste (`AAAA-MM-JJTHH:MM:SS`, lue dans `src/systeme.rs`),
   la session, la sous-commande, le processus visé, le code de sortie, la longueur du texte tapé. La sous-commande
   vient d'une liste fermée (`inconnue` sinon) ; `--fenetre` refuse un caractère de contrôle et plus de 260
   caractères (64) ; chaque champ est échappé. Aucun champ ne peut porter un texte tapé.
10. **Le verrou des gestes** (question 16) est un mutex nommé de la session Windows, `Local\pilote-gestes`. Un mutex
    abandonné par un processus mort compte comme pris par celui qui le reçoit.
11. **Les bornes**, au-delà desquelles l'usage refuse (64) : `--attendre` jusqu'à 30 000 ms, sondé toutes les
    100 ms ; `--fois` jusqu'à 100 ; `--max` à 400 par défaut, jusqu'à 10 000 ; un texte à taper jusqu'à 10 000
    caractères ; `--crans` de -50 à 50, sauf 0 ; `--pendant` de 1 à 300 s ; `--equivalent` de 100 à 400. `--crans`
    admet un nombre négatif, et les options de texte (`--texte`, `--nom`, `--contient`, `--titre`, `--id`,
    `--classe`, `--dans`, `--egale`) une valeur qui commence par un tiret. Toute sous-commande qui lit l'arbre ou
    le focus sonde jusqu'à 2 s un document encore vide, même sans `--attendre`, par une seule fonction du domaine.
12. **Les gestes de l'automatisation.** `invoquer` n'emploie que le motif Invoke. `choisir` bascule une fois
    (Toggle), sonde l'état (100 ms, au plus 2 s), et ne bascule une seconde fois que si l'état a changé sans être
    coché, au plus deux bascules, sinon 3 ; il choisit (SelectionItem), puis sonde de même ; il ne fait rien d'un
    élément déjà coché ou choisi. `valeur` lit le motif Value et refuse un champ de mot de passe (1). Un élément qui
    n'offre pas le motif fait sortir en 3. `--attendre` : `trouver` et `valeur` attendent l'élément de leur
    sélecteur, `valeur --egale <texte>` sa valeur ; `focus` accepte les options du sélecteur et attend que l'élément
    focalisé y réponde, ou, sans elles, que le focus soit dans W ; `arbre` attend le groupe de `--dans`, ou un
    document non vide. `arbre` montre tous les éléments de la vue de contrôle ; `--max` compte des lignes, et un
    arbre coupé le dit sur la sortie d'erreur. `--dans` garde les éléments dont un ancêtre, de tout type hors la
    fenêtre et le document, porte ce nom ; `dans=` nomme le plus proche ancêtre nommé de type `Group`, ou, à défaut,
    le plus proche ancêtre nommé hors la fenêtre et le document.
13. **Les essais de bout en bout ne tournent jamais sur le poste.** Ils sont marqués `#[ignore]`, avec la raison « il
    pilote le bureau : la CI le lance ». Ils ne lancent un processus que par les fonctions de leur module commun,
    qui paniquent si `GITHUB_ACTIONS` ne vaut pas `true`. `tests/statique.rs` vérifie, chaque règle avec sa
    plantation, que chaque test des fichiers d'essai porte cet attribut, qu'aucun fichier d'essai ne lance un
    processus hors de ce module, et qu'aucun test hors des fichiers d'essai n'appelle la couche Windows réelle
    (`src/bureau/` n'a aucun test). Chaque processus qu'un essai lance reçoit un profil jetable à lui
    (`USERPROFILE`, `APPDATA`, `LOCALAPPDATA`, leurs dossiers `AppData` créés, `TEMP` et `TMP` sur un chemin court).
    Les essais tournent une fois, dans l'étape de la sentinelle, un à la fois ; leurs captures et fichiers
    d'événements vont sous `%RUNNER_TEMP%\pilote-essais`, déposés en artefact quand un essai échoue. Aucune session
    ne lance `pilote` sur le poste avant la tranche 5, sauf `--version` et `--help` : lire les fenêtres d'Edouard ou
    agir sur son bureau attend le moment qu'il donne.
14. **Le refus d'une fenêtre plus élevée s'éprouve en CI** en lançant `pilote` par un jeton restreint comme celui
    du poste (le groupe Administrateurs en refus seul, le niveau moyen), contre l'application d'essai, qui reste au
    niveau haut du runner : `pilote` ne lit pas son jeton, et refuse (1), le message disant le niveau illisible
    (un journal fermé à ce jeton sortirait aussi en 1 : l'essai juge le message). Une seconde variante, au jeton
    seulement abaissé, éprouve la comparaison des deux niveaux lus. Ce code vit dans les essais, jamais dans l'exécutable. Si
    le runner refuse ce lancement, la tranche 2 demande l'accord d'Edouard avant de s'en remettre au seul test du
    domaine, et sa PR le déclare en écart.
15. **Des copies renommées de l'application d'essai** éprouvent ce qui vise un nom de processus, dans un dossier
    jetable : `KeePassXC.exe` (sa capture refusée, la frappe refusée dans son champ de mot de passe, une tabulation
    vers ce champ refusée ; question 14) et `SystemSettings.exe` (`activer` et `taper` refusés, absente de
    `fenetres`). Si le runner ouvre les Paramètres, l'essai les ouvre, les réduit, et vérifie qu'ils restent
    refusés ; sinon, la tranche 1 le dit, et le test du domaine en tient lieu.
16. **La CI** porte trois jobs, aux noms que la règle de protection exigera : `verifier` (Linux : `Cargo.lock` à
    jour, format, lint et tests de tout l'espace de travail), `windows` (Windows : lint, tests, la construction de
    l'application d'essai, la disposition française posée pour les essais, la sentinelle et les essais de bout en
    bout, puis `pilote` appelé depuis PowerShell 5.1 et depuis Git Bash) et `mutants` (Linux, sur le diff de la PR) ;
    `mutants-complet.yml` mute tout le domaine chaque lundi. Chaque délai vaut deux fois le plus long passage à
    froid mesuré, noté en commentaire, et ne baisse jamais ; celui de `mutants-complet.yml` part du job `mutants` de
    la tranche 1, puis se mesure par `gh workflow run mutants-complet.yml` après sa fusion. Si le runner ne prend
    pas la disposition française, les essais qui en dépendent (`@` par AltGr, `touche barre`) ne changent de statut
    qu'avec l'accord d'Edouard, déclaré en écart, et l'essai sur le poste les prouve.
17. **`pilote --version`** dit `pilote <version> (<commit>)`, le commit lu par un `build.rs` sans dépendance, qui
    écrit `inconnu` sans `.git` (cargo-mutants construit dans une copie).
18. **Les dossiers de données neufs** se nomment `pl-<n>`, `n` étant le plus petit nombre libre à la fois dans
    `dossiers.txt` et sous `%TEMP%`. `dossiers.txt` garde une ligne par dossier : son chemin complet, le PID et
    l'heure de création du processus lancé. `nettoyer` n'efface que si ce chemin est textuellement
    `%TEMP%\pl-<n>` au moment de l'effacement (sinon 1), et compte comme « lancés sur lui » le processus noté et
    tous ses descendants, à l'épreuve des PID repris, le processus noté comptant encore par son identité après sa
    mort : un WebView2 qui survit à l'établi fait refuser.
19. **Le relevé de l'essai sur le poste** va dans `docs/essai-sur-le-poste.md`, sans donnée personnelle, sans chemin
    de la machine, sans titre des autres fenêtres d'Edouard. Sa référence est le relevé du 2026-11-21 dans
    `pupitre` (« Les relevés » du § 70 · 7), la ligne de la reprise faisant foi pour les points qu'elle reprend ; la
    session note la version de l'établi installé avant l'essai. Les corrections arrivent avec la revue finale, et les
    points qu'elles touchent se rejouent sur le poste, avant sa fusion : pour cette reprise, l'outil s'installe
    depuis le worktree de la PR de la tranche 6, à la tête qui part, son SHA noté au relevé, et une compétence
    corrigée se donne par le texte de la branche.
20. **Les écarts connus**, chacun déclaré dans la PR qui le porte, sous la forme de la méthode, avec pour accord la
    date à laquelle Edouard accepte ce plan :
    - tranche 1 : l'application d'essai avancée (promesse : cadrage, § 5) ; ADR-0005, qui remplace ADR-0002 : la
      fenêtre retenue, le premier plan resserré, ce qu'on lit borné à W, la règle de KeePassXC étendue à `touche`,
      la ligne du journal (promesse : ADR-0002, « La fenêtre visée », « Ce qu'on lit reste dans la fenêtre visée »,
      « KeePassXC », « Le journal ») ; ADR-0006, qui remplace ADR-0004 : la sortie, les codes, les candidates, les
      bornes, l'attente et les options de `focus` (promesse : ADR-0004, « La sortie », « Les codes de sortie »,
      « La fenêtre », « Le sélecteur », « L'attente ») ;
    - chaque PR qui livre ensuite le code d'une de ces précisions (tranches 2 et 3) la déclare de nouveau ;
    - tranche 2 : le repli de la décision 14, seulement s'il sert, avec un accord daté d'Edouard (promesse : issue
      #2, critères d'acceptation) ;
    - tranche 1, ou celle où il sert : un essai du clavier qui change de statut faute de disposition française sur
      le runner, avec un accord daté d'Edouard (promesse : issue #2, critères d'acceptation) ;
    - tranche 3, et la clôture : `annonces` reportée, seulement si elle l'est (promesse : issue #2, critères
      d'acceptation ; accord : le 21/11/2026, question 2).

## Contraintes générales

- La chaîne vient de `rust-toolchain.toml` (1.99.1, `rustfmt`, `clippy`) ; `cargo fmt --all --check` et
  `cargo clippy --workspace --all-targets -- -D warnings` passent sur chaque PR, sans `#[allow]` qui ne dise sa
  raison. Chaque fichier d'essai qui appelle Windows porte `#![cfg(windows)]`.
- Le domaine ne nomme ni le crate `windows`, ni le système de fichiers, ni l'environnement, ni l'horloge : il reçoit
  des valeurs et rend des valeurs. Le contrôle statique (`tests/statique.rs`) refuse, dans `src/` du paquet `pilote`
  hors de `src/systeme.rs`, les chemins devinés, l'horloge (`std::time`, `GetLocalTime`, `GetTickCount64`,
  `QueryPerformanceCounter`), l'environnement (`std::env::var`, `GetEnvironmentVariableW`, `GetTempPath`) et le
  hasard ; ces règles ne visent ni `tests/` ni `essai/appli/`. Il refuse `SetValue` partout, et tient pour les seuls
  fichiers d'essai les règles de la décision 13. Il cherche des noms entiers, pour qu'un nom comme `Instantane` ne
  le trompe pas, et chaque mot refusé y a sa plantation.
- Chaque règle du domaine a son test, écrit avant elle ; `cargo-mutants` ne laisse aucun survivant, un mutant
  équivalent ou qui boucle (un sondage, par exemple) s'excluant par son nom dans `.cargo/mutants.toml`, avec sa
  raison.
- Chaque geste et chaque garde qui touche une fenêtre ont leur essai de bout en bout ; leurs attentes se font par
  sondage, jamais par une relance. Un essai instable se dit à Edouard avant d'en changer le statut.
- Les messages sont en français ; les clés de la sortie sont celles de la décision 7.
- Le dépôt est public : aucun chemin du poste, aucun nom de famille, aucun secret, dans le code, les tests, les
  commentaires, les plans, les PR ; `%LOCALAPPDATA%`, `%TEMP%` ou `~` à leur place. Les valeurs des champs de mot de
  passe de l'application d'essai sont des valeurs d'essai, de faible entropie.
- Conventional Commits en français, en-tête de 100 caractères au plus, titre de PR de 93 au plus ; `git add` par
  chemins ; chaque PR dit « Partie de #2 », sauf la clôture, et porte « ## Écarts à la spec » ; fusion par squash,
  contrôles tous verts, par `gh pr merge <n> --squash --match-head-commit <sha>`.
- L'outil s'installe par `cargo install --locked --path .`, depuis un clone à jour (`git fetch`, puis
  `main..origin/main` vide).

## Points de vigilance

Les cas que la spec implique sans les nommer, les plus probables d'abord ; chacun a son test dans sa tranche.

- Une fenêtre qui vole le premier plan au milieu d'une frappe, une notification par exemple : la frappe s'arrête, le
  code est 1, et le refus dit combien de caractères sont partis — `taper_s_arrete_quand_le_premier_plan_change`
  (domaine), puis `taper_refuse_quand_une_autre_fenetre_vole_le_premier_plan` (bout en bout : un autre processus
  vole le premier plan sur une minuterie, pendant un texte de plusieurs milliers de caractères), tranche 1.
- Une application qui ouvre un dialogue titré, et a donc deux fenêtres : sans `--titre`, le code 2 nomme les deux
  titres — `deux_fenetres_titrees_sans_option_titre_sortent_en_2`, tranche 1 ; une liste ouverte, elle, ne compte
  pas —
  `touche_avec_une_liste_ouverte_vise_la_fenetre_principale`, tranche 1.
- Un texte qui porte un CRLF, une tabulation, un caractère hors du plan de base ou une lettre décomposée : un seul
  Entrée par fin de ligne, la paire de substitution dans un seul envoi — `taper_crlf_donne_un_seul_entree`,
  `taper_hors_du_plan_de_base_envoie_la_paire`, tranche 1.
- Un élément qui disparaît entre la lecture et le geste, une liste qui se recharge par exemple : le code 2, jamais un
  plantage — `element_disparu_sort_en_2`, tranche 2.
- Une application dont l'hôte ne répond plus : les délais de l'automatisation la bornent, et l'outil sort en 3 ; une
  page dont seul le JavaScript est figé laisse lire l'arbre d'avant, en 0, sans planter —
  `arbre_d_un_hote_fige_sort_en_3` et `arbre_d_une_page_figee_rend_l_arbre_d_avant`, tranche 2.

## Mécanismes annoncés

- `.github/workflows/ci.yml:18` — `socle:` — le job du socle garde sa clé, qui forme les contrôles `socle / secrets` et `socle / confidentialite` ; la tranche 1 ajoute `verifier`, `windows` et `mutants` à côté de lui
- `.github/workflows/ci.yml:22` — `verifier` — le modèle réserve ce nom au job des vérifications du projet, que la tranche 1 écrit
- `.github/modele/ruleset-main.json:15` — `texte-pr` — la règle gérée n'exige que trois contrôles ; ce fichier géré ne se modifie pas, et la règle réelle du dépôt reçoit les trois autres par le geste d'Edouard
- `.gitignore:1-7` — `.superpowers/` — ni `target/` ni `mutants.out*/` n'y sont : la tranche 1 les ajoute
- `.github/workflows/modele-socle.yml:142` — `--neuf` — un plan ajouté se vérifie en entier, ses lieux ouverts : ce plan, puis chaque plan de tranche
- `CLAUDE.md:15` — `Aucune avant la première PR d'implémentation` — la tranche 1 y écrit les commandes
- `README.md:20` — `La commande qui lance le projet` — la tranche 4 remplit « Démarrer »
- `pupitre:.github/workflows/ci.yml:226-245` — `La sentinelle d'I30` — la sentinelle redirige le profil, crée `AppData\Roaming` et `AppData\Local`, relance les tests, puis échoue si un fichier y paraît ; la tranche 1 la reprend pour `pilote`
- `pupitre:app/src-tauri/src/systeme.rs:91-102` — `starts_with` — le garde-fou positif : le dossier résolu doit tomber dans la sentinelle ; la tranche 1 en écrit un pour le journal, la tranche 3 pour `dossiers.txt` et `pl-<n>`
- `pupitre:.github/workflows/ci.yml:298` — `--in-diff` — le job `mutants` ne mute que les lignes de la PR ; la tranche 1 le reprend sur le domaine
- `pupitre:.github/workflows/mutants-complet.yml:7` — `cron` — le passage complet tourne chaque lundi ; la tranche 1 le reprend
- `pupitre:app/src-tauri/tests/statique.rs:29` — `CHEMINS_DEVINES` — les mots des chemins devinés, refusés hors du module système ; la tranche 1 en reprend la forme, par noms entiers
- `pupitre:app/src-tauri/tests/statique.rs:66` — `HORLOGE` — l'horloge, refusée de même
- `pupitre:.cargo/mutants.toml:62` — `exclude_re` — un mutant équivalent ou qui boucle s'exclut par son nom, avec sa raison
- `pupitre:crates/stockage/tests/commun/mod.rs:54` — `fn dossier_jetable` — une seule fonction de dossier jetable par crate de tests ; les essais de bout en bout de `pilote` en ont une
- `pupitre:app/src-tauri/tauri.conf.json:15` — `maximized` — l'établi s'ouvre agrandi, à 1440 × 900 une fois restauré : `taille` restaure puis place la fenêtre (tranche 3)
- `pupitre:app/ui/src/entete.rs:110` — `aria-keyshortcuts` — la recherche déclare `/`, que `touche barre` doit déclencher à l'essai sur le poste (tranche 5)
- `pupitre:docs/70-qualite-et-verification.md:330` — `793d498` — le relevé de l'essai du 2026-11-21, référence des « mêmes constats » (tranche 5)
- `pupitre:docs/70-qualite-et-verification.md:331` — `0d6b265` — la reprise du même jour, qui fait foi pour les points qu'elle reprend, dont le journal du fonds (tranche 5)

## Outils et cas limites

- `clap`, par `try_parse` — une erreur d'usage sort en 2, et `--help` comme `--version` arrivent par une erreur — `usage_inconnu_sort_en_64` et `aide_et_version_sortent_en_0`, tranche 1
- `clap` — une valeur qui commence par un tiret est lue comme une option, sauf `allow_negative_numbers` ou `allow_hyphen_values` — `molette_accepte_des_crans_negatifs` et `taper_accepte_un_texte_qui_commence_par_un_tiret`, tranches 1 et 2
- `cargo`, dans un espace de travail dont la racine est un paquet — sans `--workspace`, seul ce paquet se construit, se teste et se lint — le lint et les tests de chaque job passent `--workspace`, et un essai refuse si l'application d'essai n'est pas construite, tranche 1
- `cargo mutants --in-diff` — un diff qui ne touche pas le domaine ne laisse rien à muter et sort en 0 ; un mutant qui boucle sort en 3 et fait échouer le job — le job `mutants` de la PR de la tranche 1, puis chaque mutant qui boucle exclu par son nom, tranches 1 à 3
- `git`, lu par `build.rs` — cargo-mutants construit dans une copie sans `.git` — `version_porte_le_commit_ou_inconnu`, et le job `mutants` qui construit, tranche 1
- `rustup` — depuis la version 1.28, `rustup show` n'installe plus la chaîne — l'étape « Installer la chaîne de rust-toolchain.toml » de chaque job, tranche 1
- `Get-ChildItem -Recurse -Force`, dans la sentinelle — un fichier caché ne se voit qu'avec `-Force` — l'essai positif de l'étape, qui plante un fichier caché avant de chercher, tranche 1
- `actions/upload-artifact` — un chemin sans fichier ne fait qu'avertir — `if-no-files-found: warn`, et le chemin pris de la même racine que les essais, `%RUNNER_TEMP%\pilote-essais`, tranche 1
- le contrôle statique, par sous-chaîne — un nom qui contient un mot refusé (`Instantane`) le déclenche à tort — `statique_cherche_des_noms_entiers`, tranche 1
- `SHGetKnownFolderPath` — il ignore `APPDATA` et `LOCALAPPDATA`, et échoue sous un `USERPROFILE` redirigé sans `AppData\Roaming` ni `AppData\Local` — `journal_tombe_dans_le_profil_redirige` (bout en bout), tranche 1 ; `donnees_neuves_portent_appdata` (domaine) et `lancer_sur_des_donnees_neuves` (bout en bout), tranche 3
- `std::env::temp_dir`, par `GetTempPath2W` — il lit `TMP` avant `TEMP` — le profil jetable de chaque essai redirige les deux, et `pl_tombe_dans_le_temp_redirige`, tranche 3
- `SetForegroundWindow` — Windows refuse sans dire pourquoi, et une frappe d'Alt seule ouvre le mode menu d'une fenêtre Win32 déjà au premier plan — `activer_ne_frappe_jamais_alt_seule` (domaine) et `activer_puis_taper_ne_perd_aucun_caractere` (bout en bout), tranche 1 ; le verrou du premier plan, à l'essai sur le poste, tranche 5
- `SetForegroundWindow` sur une fenêtre réduite — il l'active sans la restaurer — `activer_restaure_une_fenetre_reduite`, domaine et bout en bout, tranche 1
- `SendInput` — rend le nombre d'événements insérés, ne relâche aucune touche restée enfoncée, et UIPI le bloque sans le dire — `une_combinaison_part_en_un_seul_envoi` et `refuse_si_un_modificateur_est_enfonce` (domaine), tranche 1 ; `refuse_une_fenetre_plus_elevee` (bout en bout), tranche 2
- `VkKeyScanExW` — rend -1 pour un caractère absent du clavier, l'état 6 pour AltGr, une touche morte n'écrit rien seule, et il suit la disposition du fil au premier plan, américaine sur le runner — `comme_au_clavier_refuse_un_caractere_absent` et `comme_au_clavier_refuse_une_touche_morte` (domaine), `taper_arobase_par_altgr` (bout en bout, qui vérifie d'abord la disposition française), tranche 1
- `WaitForSingleObject`, sur `Local\pilote-gestes` — `WAIT_ABANDONED` donne la propriété — `verrou_abandonne_compte_comme_pris` (domaine) et un essai qui tue un `pilote` tenant le verrou, puis un geste qui passe, tranche 1
- `OpenProcessToken` — refusé sur une cible élevée, depuis le jeton filtré du poste — `niveau_illisible_refuse_en_1`, tranche 1 ; l'essai de la décision 14, tranche 2
- `CreateProcessAsUserW`, avec un jeton seulement abaissé — le groupe Administrateurs reste actif et le jeton de la cible se lit, quand le poste reçoit un refus — l'essai au jeton restreint de la décision 14, puis sa variante, tranche 2
- `WTSQuerySessionInformationW` et `OpenInputDesktop` — une session verrouillée, un bureau autre que `Default`, un accès refusé — `poste_verrouille_refuse_en_1`, un test du domaine par cas, tranche 1
- `CreateToolhelp32Snapshot` — un PID se réutilise après la mort de son processus — `un_pid_repris_n_est_pas_un_descendant`, tranche 1
- `EnumChildWindows`, sur un cadre d'`ApplicationFrameHost` — l'enfant de `SystemSettings` se détache d'un cadre réduit — `parametres_reduits_restent_refuses`, domaine, et bout en bout si le runner ouvre les Paramètres, tranche 1
- `GetFocusedElement` — rend le focus du système, même hors de la fenêtre retenue — `focus_hors_de_la_fenetre_visee_sort_en_1`, domaine et bout en bout, tranche 2
- l'automatisation, par `FindFirst` et `GetCurrentPatternAs` — une absence arrive en erreur de code `HRESULT(0)`, un élément disparu en `UIA_E_ELEMENTNOTAVAILABLE` — `invoquer_un_element_sans_le_geste_sort_en_3` et `element_disparu_sort_en_2`, tranche 2
- `Toggle` et `Select` de Chromium — ils rendent la main avant que l'état change — `choisir_attend_l_etat_avant_de_rebasculer` (domaine, faux bureau à état retardé), puis la case de l'application d'essai, tranche 2
- l'arbre d'accessibilité de Chromium — il ne se bâtit qu'à la première requête, et le navigateur le sert depuis sa copie quand le moteur de rendu est figé — `lecture_sonde_un_document_vide` (domaine) et `arbre_d_une_page_figee_rend_l_arbre_d_avant` (bout en bout), tranche 2
- `PrintWindow`, avec le rendu complet — une fenêtre réduite rend une image noire, la capture porte l'ombre du cadre, et `GetDIBits` rend un alpha nul — `capture_refuse_une_fenetre_reduite`, `capture_noire_sort_en_3` et `png_sans_alpha` (domaine), `capture_rogne_l_ombre` (bout en bout), tranche 2
- `WindowFromPoint` — rend la fenêtre fille sous le point, pas sa fenêtre de premier niveau — `clic_refuse_un_point_recouvert`, bout en bout, tranche 2
- `unicode-normalization`, la forme NFC — elle ne touche pas les espaces insécables — `nom_compte_les_insecables_pour_des_espaces`, tranche 2
- `symlink_metadata` et `remove_dir_all` — une jonction passe pour un lien, et l'effacement ne suit pas les liens intérieurs — `nettoyer_refuse_une_jonction` et `nettoyer_ne_suit_pas_une_jonction_interieure`, bout en bout, tranche 3
- l'événement `LiveRegionChanged` de Chromium — il ne porte aucun texte, et ne part que si un client écoute — `annonces_lisent_le_texte_de_la_region`, bout en bout, tranche 3
- PowerShell 5.1 — il lit la sortie d'un exécutable dans la page OEM et écrit son entrée standard en ASCII — `fichier_utf16_a_bom` et `fichier_utf8_a_bom` (domaine), puis l'étape du job `windows` qui appelle `pilote` depuis `shell: powershell`, tranche 1
- Git Bash — il réécrit un argument qui commence par `/` — l'étape du job `windows` qui appelle `pilote` depuis `shell: bash` (`touche barre`, l'entrée standard), tranche 1, puis l'essai sur le poste, tranche 5

## Effets cherchés

- `verifier`, `windows` et `mutants` deviennent des contrôles requis — la règle de protection de `pilote` sur GitHub, par le geste d'Edouard après la tranche 1 — la session lit `gh api repos/EdouardZemb/pilote/rules/branches/main` et y trouve les six contextes attendus
- ADR-0005 et ADR-0006 remplacent ADR-0002 et ADR-0004 — `docs/decisions/`, les invariants de `CLAUDE.md` qui citent ADR-0002, puis la compétence — la PR de la tranche 1 écrit les deux ADR, passe les anciens au statut « remplacé par », met à jour les renvois de `CLAUDE.md` et déclare ses écarts (décision 20) ; la revue de son plan les relit contre ce plan-ci
- le domaine se compile et se teste sous Linux comme sous Windows — la CI, `verifier` sous Linux et `windows` sous Windows Server — `cargo test --workspace` vert dans les deux jobs, à chaque PR
- les essais de bout en bout redirigent le profil de chaque processus qu'ils lancent, jamais celui du runner — le runner Windows de la CI — la sentinelle échoue si un fichier paraît dans le profil redirigé, et `journal_tombe_dans_le_profil_redirige` trouve le journal à sa place, tranche 1
- la disposition AZERTY du poste — le runner Windows, en disposition américaine — le job `windows` pose la disposition française avant les essais du clavier, qui la vérifient avant de juger ; si le runner ne la prend pas, la tranche 1 le dit à Edouard, et seul l'essai sur le poste prouve ce chemin
- aucun geste ni essai de bout en bout sur le bureau d'Edouard avant son accord — le poste, où un courrier, un coffre et des terminaux sont ouverts — la fonction commune des essais panique hors de la CI, `tests/statique.rs` refuse un essai sans `#[ignore]`, et `CLAUDE.md` écrit la règle, tranche 1
- `mutants-complet.yml` ne tourne que depuis la branche principale — GitHub, après la fusion de la tranche 1 — la session le lance par `gh workflow run mutants-complet.yml`, mesure son passage à froid et note le délai dans la PR suivante
- le binaire installé par `cargo install --locked --path .` — `~/.cargo/bin`, dans le PATH de toutes les sessions du poste — `pilote --version` dit la version et le commit de `origin/main`, à l'essai sur le poste, tranche 5
- la compétence `piloter` et son pointeur du poste — chaque session de Claude Code du poste, quel que soit son dossier — à la tranche 4, des sessions neuves reçoivent son texte ; après la fusion, le pointeur posé, une session neuve ouverte hors de `pilote` la trouve et la suit, tranche 5
- la sortie en UTF-8 sans BOM, et le texte par `--fichier` ou par l'entrée standard — PowerShell 5.1 et Git Bash, les deux shells des sessions — l'étape du job `windows` qui appelle `pilote` depuis chacun, tranche 1, puis l'essai sur le poste, tranche 5
- le journal `%LOCALAPPDATA%\pilote\journal.tsv` — le poste, où Edouard relit ce que les sessions ont fait — à l'essai sur le poste, chaque appel a sa ligne, et aucune ne porte un texte tapé, tranche 5
- la page de savoir sur le pilotage d'une application de bureau — le cerveau, par `/noter`, à la place de la proposition du 2026-11-21 sur le pilotage par PowerShell — le fichier déposé dans `entree/`, et la ligne « noté au cerveau », tranche 7

## Revues

- la revue de chaque plan de tranche, par une session neuve, avant son exécution : Tranche 1, Tranche 2, Tranche 3, Tranche 4
- la revue de chaque tâche, dans le plan de sa tranche, sur la tête qui part : Tranche 1, Tranche 2, Tranche 3, Tranche 4
- le jugement des écarts de forme de l'essai sur le poste, par Edouard : Tranche 5
- la revue finale du lot, par une session neuve, puis la relecture de ses corrections : Tranche 6

## Les tranches

### Tranche 1 · le socle, le clavier et l'application d'essai

**But :** un outil qui tape, sûr dès le premier jour : la fenêtre retenue, la liste de refus, le premier plan vérifié
avant chaque événement, le verrou, le journal ; et la CI qui le prouve sur WebView2.

**Elle livre :**
- L'espace de travail : `Cargo.toml` (le paquet `pilote` à la racine, le membre `essai/appli`),
  `rust-toolchain.toml`, `build.rs`, `.cargo/mutants.toml`, `target/` et `mutants.out*/` dans `.gitignore`.
- ADR-0005, qui remplace ADR-0002 (la décision 5, la règle de KeePassXC de la décision 6, la décision 9), et
  ADR-0006, qui remplace ADR-0004 (les décisions 7, 8, 11 et 12, et les candidates de la décision 5) ; chacun
  reprend tout l'ancien, et l'ancien prend le statut « remplacé par » ; les renvois de `CLAUDE.md` suivent, et son
  invariant 3 dit « hors de la fenêtre retenue » au lieu de « hors de son arbre de processus ».
- Le domaine : l'analyse des arguments de toutes les sous-commandes du lot, l'usage en 64 ; la sortie, ses
  échappements et ses codes ; les noms de touches (ceux du design, `Haut`, `Bas`, `Gauche`, `Droite`, `Inser`, et
  les caractères nommés `barre` pour `/` et `interrogation` pour `?`), leurs modificateurs et les touches
  interdites ; le décodage d'un texte (UTF-8 avec ou sans BOM, UTF-16LE à BOM, un saut de ligne final ignoré) ; la
  ligne du journal ; la liste de refus, fenêtres enfants et cadres d'`ApplicationFrameHost` compris ; l'arbre de
  processus, à l'épreuve des PID repris ; la fenêtre retenue ; l'ordre des gardes et l'activation de la question
  13 ; le trait `Bureau`, son horloge et le faux bureau.
- La couche Windows de ce que demande le clavier : les fenêtres et leurs propriétaires, l'identifiant d'application
  d'un cadre, l'instantané des processus, le premier plan, la restauration, le mouvement de souris nul, Alt
  entourant une touche sans effet, `SendInput`, `VkKeyScanExW` et l'état de Verr. Maj, les touches enfoncées, le
  poste verrouillé, le niveau d'intégrité, le mutex ; `src/systeme.rs` (le journal sous `%LOCALAPPDATA%\pilote\`,
  le dossier temporaire, l'horloge, la date locale, la session), la sensibilité au DPI par écran en tête de `main`.
- Les sous-commandes `fenetres`, `activer`, `touche` (avec `--fois`) et `taper` (`--texte`, `--fichier`, `-`,
  `--comme-au-clavier`), et `pilote --version`.
- L'application d'essai (décision 2).
- La CI de la décision 16, `tests/statique.rs` (décision 13 et contraintes), la sentinelle des chemins de données,
  le garde-fou positif du journal.
- Les essais de bout en bout du clavier et des gardes : des accents tapés en Unicode ; `@` par AltGr ; `touche
  barre`, qui déclenche le raccourci `/` de la page ; Tab qui déplace le focus ; l'activation d'une fenêtre
  d'arrière-plan, puis d'une fenêtre réduite ; une frappe sur une fenêtre Win32 déjà au premier plan qui ne perd
  aucun caractère (l'application d'essai, ou à défaut le Bloc-notes) ; une frappe arrêtée par un vol du premier
  plan ; deux fenêtres titrées sans `--titre`, et une liste ouverte qui ne compte pas ; la copie
  `SystemSettings.exe`, et les Paramètres réduits si le runner les ouvre ; deux gestes en même temps, le second
  refusé après 5 s, et un verrou abandonné repris.
- `CLAUDE.md` : les commandes, et la règle de la décision 13.
- 👤 La règle de protection préparée : un fichier de la règle réelle, avec les six contrôles, et la seule ligne
  qu'Edouard lance par `!`, une fois la PR fusionnée.

**Elle dépend de** ce plan. **Critères de sortie :** `verifier`, `windows` et `mutants` verts, `mutants` sans
survivant ; la sentinelle et le garde-fou positif verts ; la disposition française prise sur le runner, ou dite à
Edouard ; chaque délai de `ci.yml` mesuré à froid puis noté ; la règle de protection préparée. **Documents :**
ADR-0005 et ADR-0006 ; `CLAUDE.md` ; `.cargo/mutants.toml` ; le registre, si une leçon paraît.

### Tranche 2 · la lecture, les gestes et la capture

**But :** lire ce que lit un lecteur d'écran, et agir par l'accessibilité, sans qu'une lecture ne sorte de la
fenêtre retenue ni qu'une valeur secrète ne sorte.

**Elle livre :**
- La couche de l'automatisation : COM en MTA, `CUIAutomation8`, des délais de transaction et de connexion, la
  lecture d'un sous-arbre par le cache, l'absence distinguée de l'erreur.
- Le modèle d'un élément dans le domaine (type, nom, identifiant, classe, valeur, raccourci, touche d'accès, réglage
  d'annonce, état, mot de passe, processus, enfants), et le sélecteur : `--nom`, `--contient`, `--type`, `--id`,
  `--classe`, `--dans`, `--rang`, la normalisation NFC et les espaces insécables, le refus d'une ambiguïté.
- `focus`, `arbre` (`--max`, `--dans`), `trouver`, `focaliser`, `invoquer`, `choisir`, `valeur`, `defiler`,
  `defilables`, `capture`, `clic` (`--double`, `--droit`) et `molette`, et `--attendre` (décisions 11 et 12).
- Les gardes des lectures : le focus hors de la fenêtre retenue (1), sans nom ni valeur ; aucune valeur d'un élément
  `IsPassword` ; une capture qui refuse une fenêtre réduite (1) et une fenêtre de KeePassXC (1), et qui sort en 3
  sur une image toute noire ; le point de `clic` et de `molette` ; les règles de KeePassXC pour `taper` et
  `touche`.
- Les essais de bout en bout de chaque sous-commande et de chaque garde, sur l'application d'essai : les trois
  « Toutes » départagés par `--dans` ; les groupes emboîtés de même nom ; le champ de mot de passe ; la copie
  `KeePassXC.exe`, une tabulation vers son champ de mot de passe comprise ; la fenêtre plus élevée (décision 14) ;
  un point recouvert par une autre fenêtre ; l'hôte figé et la page figée ; la capture d'une fenêtre réduite ;
  `arbre` et `capture` d'une fenêtre d'arrière-plan, qui laissent le premier plan inchangé ; une capture relue, ni
  noire ni d'une autre fenêtre.

**Elle dépend de** la tranche 1 et de la règle de protection posée. **Critères de sortie :** les essais de bout en
bout verts dans `windows` ; `mutants` sans survivant ; `PrintWindow` prouvé sur WebView2 lui-même ; le refus d'une
fenêtre plus élevée prouvé en CI, ou son repli accepté par Edouard et déclaré ; les délais remesurés.

### Tranche 3 · la fenêtre, les dossiers, puis les annonces

**But :** changer les conditions de l'essai, et écouter ce qu'un lecteur d'écran entendrait.

**Elle livre :**
- `taille` : `--largeur` et `--hauteur`, `--agrandir`, `--restaurer`, et `--equivalent`, un calcul du domaine que
  la mesure du poste du 2026-11-21 garde (une zone cliente agrandie de 1366 × 705, un cadre de 16 × 38 : 699 × 390).
- `lancer` (attendre la fenêtre, puis un document qui a des enfants), avec `--donnees-neuves` : le dossier `pl-<n>`
  sous `%TEMP%`, ses `AppData\Roaming` et `AppData\Local` créés d'abord (question 7), `USERPROFILE`, `APPDATA` et
  `LOCALAPPDATA` redirigés pour le seul processus lancé, la ligne de `dossiers.txt` (décision 18).
- `fermer` (`WM_CLOSE`) et `nettoyer`, ses trois conditions, un dossier encore utilisé refusé (1), sans rien tuer.
- En dernier, `annonces --pendant <s>` : « écoute prête » une fois abonnée, puis une ligne horodatée par annonce de
  la fenêtre retenue, région annoncée ou notification, avec son réglage et son texte. Si elle ne tient pas dans la
  tranche, elle va dans `docs/pistes.md`, et la PR le déclare en écart (décision 20).
- Leurs essais de bout en bout : la taille, l'équivalent calculé sur l'écran du runner, un lancement sur des données
  neuves, puis `nettoyer` refusé aussitôt après `fermer` tant que WebView2 survit, puis accepté ; une jonction
  refusée, une jonction intérieure qui n'est pas suivie ; le garde-fou positif de `dossiers.txt` et de `pl-<n>` ;
  une annonce de la région `aria-live` de l'application d'essai.

**Elle dépend de** la tranche 2. **Critères de sortie :** les essais verts ; `mutants` sans survivant ; les délais
remesurés ; `annonces` livrée, ou reportée et déclarée.

### Tranche 4 · la compétence

**But :** que chaque session du poste sache quand et comment se servir de `pilote`, et ce qui reste interdit.

**Elle livre :**
- `.claude/skills/piloter/SKILL.md`, écrite par `superpowers:writing-skills` : quand s'en servir ; toutes les règles
  qu'ADR-0005 lui confie ; l'enchaînement d'un essai ; la sortie, les codes et les bornes (ADR-0006) ; les pièges du
  cadrage et de l'essai : les deux shells et leurs encodages, l'arbre vide à la première requête, les processus
  `msedgewebview2` qui survivent à l'établi, les listes fenêtrées, la valeur d'un champ de date qui se lit dans ses
  parties, une seconde instance sur les mêmes données que l'établi refuse, l'équivalent de 200 %, `annonces` en
  arrière-plan, l'écart entre la vérification du premier plan et l'envoi, ce qui reste à Edouard ; les refus de
  KeePassXC ne sont éprouvés que sur une copie : que Qt marque `IsPassword` reste à prouver, et aucun geste dans
  le vrai coffre ne compte sur eux.
- Le README : « Démarrer » (`cargo install --locked --path .`) et la carte des documents ; `CLAUDE.md` : l'état.
- Ses épreuves : des scénarios donnés à des sessions neuves, avec le texte de la compétence, sans lancer `pilote`
  (taper un vrai mot de passe ; taper dans un terminal une commande que le garde de Claude Code refuserait ; taper
  dans le terminal d'une autre session Claude ; valider un paiement ; piloter une page web que Claude in Chrome
  sert ; fermer une application qu'Edouard a ouverte), avant puis après la compétence.
- Après la fusion, le pointeur du poste, `~/.claude/skills/piloter/SKILL.md`, qui dit de lire et d'appliquer celle
  du clone principal, une fois ce clone à jour.

**Elle dépend des** tranches 1 à 3. **Critères de sortie :** chaque scénario suivi, après la compétence ; le
pointeur posé après la fusion ; `documentation.py` vert. **Documents :** `docs/pistes.md`, l'épreuve
d'`IsPassword` sous Qt sur un coffre jetable, si Edouard la demande.

### Tranche 5 · l'essai sur le poste

**But :** le critère du lot. Les onze points de la liste de l'essai installé de `pupitre`, du 2026-11-18, hors du
dépôt, dans la boîte de réception d'Edouard, rejoués par l'outil, sur l'établi installé, avec les mêmes constats que
le relevé du 2026-11-21 (décision 19).

**Elle se fait** 👤 au moment qu'Edouard donne, lui présent ou non (question 10) :
- `git fetch`, `main..origin/main` vide, puis `cargo install --locked --path .` et `pilote --version` ; la version
  de l'établi installé, notée ;
- une session neuve, ouverte hors de `pilote`, trouve la compétence par son pointeur et la suit ;
- les onze points, sur le fonds d'essai de l'établi, depuis PowerShell 5.1 et depuis Git Bash ; `--donnees-neuves`
  pour le fonds neuf, puis `nettoyer` ;
- ce qui reste à prouver au cadrage (§ 1) : le verrou du premier plan sur le poste, `activer` sur une fenêtre
  d'arrière-plan ; `fenetres` ne liste aucune fenêtre des Paramètres ; `@` par AltGr et `touche barre` sur le
  clavier du poste, que la CI n'éprouve qu'avec la disposition française ;
- ni KeePassXC, ni le courrier, ni aucune autre application d'Edouard ; avant de fermer une fenêtre qu'il a
  ouverte, la session dit pourquoi ;
- les captures relues par la session ; les lignes du journal comptées, aucune ne portant un texte.

Chaque point est jugé sur sa forme, non sur ses nombres (question 15) ; un écart de forme revient à Edouard, qui le
juge. Le relevé et les corrections arrivent dans la PR de la tranche 6.

**Elle dépend des** tranches 1 à 4. **Critères de sortie :** les onze points relevés, chacun « même constat », ou un
écart rapporté à Edouard, qu'il juge étranger à l'outil ou qui part en correction à la tranche 6 ; le verrou du
premier plan prouvé sur le poste, ou son échec parti en correction.

### Tranche 6 · la revue finale

**But :** relire le lot entier, d'un regard qui ne l'a pas écrit.

**Elle livre :**
- La PR de revue finale : une revue indépendante du lot, par lentilles (les garde-fous d'ADR-0002 et d'ADR-0005,
  l'interface d'ADR-0004 et d'ADR-0006, le domaine et la mutation d'ADR-0003, la CI, la compétence), chaque constat
  contre-vérifié ; la recherche, dans tout le dépôt, des mots que le lot a périmés ; les corrections, et celles de
  l'essai sur le poste ; le relevé de l'essai ; les délais remesurés.
- 👤 La reprise sur le poste des points de l'essai que touche une correction, au moment qu'Edouard donne, relevée
  dans `docs/essai-sur-le-poste.md`.
- La relecture des corrections, sur la tête qui part.

**Elle dépend de** la tranche 5. **Critères de sortie :** chaque constat confirmé corrigé, ou reporté dans les
pistes avec sa raison ; chaque écart de l'essai parti en correction, corrigé puis repris sur le poste, verrou du
premier plan compris ; la relecture faite ; la CI verte.

### Tranche 7 · la clôture

**But :** clore le lot.

**Elle livre :**
- La PR de clôture, qui supprime ce plan maître et les plans des tranches 1 à 4, une fois chaque ligne de « Ce qui
  doit survivre au plan » écrite ailleurs ; met à jour `CLAUDE.md` (l'état, les commandes) et, s'il le faut, un ADR ;
  relit le README d'après `docs/modele/documentation.md` ; porte au registre les leçons du lot encore sans ligne, par
  `/consigner` ; déclare de nouveau un écart du lot qui demeure ; dit « Closes #2 » ; nomme la tête relue
  (`Tête relue : <SHA>`).
- 👤 L'étiquette `v0.1.0`, par Edouard, après la fusion.
- La page de savoir, par `/noter`, et l'annonce « noté au cerveau ».
- 👤 La suppression de la branche jetable `essai/runner`, jamais fusionnée, avec l'accord d'Edouard.

**Elle dépend de** la tranche 6. **Critères de sortie :** l'issue #2 close par la PR de clôture ; la page de savoir
déposée.

---

## Le déroulé

1. **Ce plan.** La PR l'ajoute dans `.claude/plans/lot1-l-outil-et-l-essai-de-pupitre.md` et dit « Partie de #2 ».
   👤 Edouard l'accepte, et avec lui les précisions des ADR 0005 et 0006.
2. **Chaque tranche, dans l'ordre.** `git fetch`, puis un worktree dans `pilote.worktrees/<sujet>`. Le plan de la
   tranche s'écrit par `superpowers:writing-plans`, une session neuve relit sa PR, puis l'exécution suit
   `superpowers:subagent-driven-development` et les contraintes générales.
3. **Chaque PR.** Elle nomme toute dépendance nouvelle et déclare ses écarts ; une tranche qui alourdit les tests
   remesure ses délais ; les captures d'un essai qui échoue restent en artefact.
4. **Les étapes 👤 :** accepter ce plan ; poser la règle de protection après la tranche 1 ; accepter, s'il sert, le
   repli de la décision 14 ; donner le moment de l'essai sur le poste, puis celui de sa reprise ; juger un écart de
   forme de l'essai ; poser l'étiquette `v0.1.0` ; accepter la suppression de la branche `essai/runner`.

## Ce qui doit survivre au plan

Le plan se supprime à la clôture. Ce qu'il décide s'écrit avant, par la tranche qui le livre :

| Quoi | Où | Tranche |
|---|---|---|
| La fenêtre retenue, le premier plan resserré, ce qu'on lit, la règle de KeePassXC étendue, la ligne du journal | ADR-0005, qui remplace ADR-0002 | 1 |
| La sortie, les échappements, les codes précisés, les candidates, les bornes, l'attente, les options de `focus` | ADR-0006, qui remplace ADR-0004, puis la compétence `piloter` | 1, 4 |
| Les commandes ; la règle des essais de bout en bout, jamais sur le poste | `CLAUDE.md` | 1 |
| Les modules, le trait `Bureau`, l'ordre des gardes | `CLAUDE.md` (ce qui fait foi), et les commentaires du domaine | 1 |
| Les six contrôles requis | `CLAUDE.md` | 1 |
| Les délais de la CI et leurs mesures | les commentaires de `ci.yml` et de `mutants-complet.yml` | 1 à 3 |
| Les mutants exclus | `.cargo/mutants.toml` | 1 à 3 |
| Le relevé de l'essai sur le poste, et sa reprise | `docs/essai-sur-le-poste.md` | 6 |
| Ce que le lot reporte | `docs/pistes.md` | 3, 4, 6 |
