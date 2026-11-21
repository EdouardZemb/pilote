# Brief initial — pilote

*La demande de départ, écrite par Edouard avant tout code. Elle sert de point d'appui à la phase 0 ; le cadrage la dépasse, ce brief ne se réécrit pas.*

Ce brief reprend le design de `pilote` du 2026-11-21, écrit par la session de `pupitre` qui venait
de faire l'essai installé de sa PR #48 avec un script PowerShell, et approuvé par Edouard le même
jour (« Ok pour le design », puis « Ok pour les défauts », en deux tours). Le design et le script de
l'essai sont hors du dépôt, dans la boîte de réception d'Edouard ; ce qui en fait foi ici est ce
brief, puis le cadrage.

## 1. D'où l'on part

Une session de Claude Code en ligne de commande, sur le PC, n'a aucun outil pour piloter une
application de bureau. Le pilotage d'Anthropic (« computer use ») n'existe que dans l'application
de bureau de Claude, et il travaille sur des captures d'écran : il clique sur des pixels, et ne lit
ni le focus, ni les noms qu'entend un lecteur d'écran, ni les raccourcis.

Le 2026-11-21, l'essai installé de `pupitre` devait pourtant se faire sans Edouard, loin de son PC.
Il s'est fait par un script PowerShell écrit pour l'occasion : le clavier par `SendInput`, le
focus, les noms et les valeurs lus par l'automatisation de Windows (UI Automation), des captures de
la seule fenêtre de l'application. Le script recompile son C# à chaque appel, son client .NET ne
voit pas `aria-live`, et rien ne le versionne.

## 2. Ce qu'on veut obtenir

Que toutes les sessions du poste aient cet outil. Les essais installés des projets d'Edouard, et les
contrôles d'accessibilité qui vont avec, se font alors par une session, à sa demande. Ces contrôles
comptent : `pupitre` a pour invariant qu'aucune information n'est portée par la seule couleur, et
chaque écran s'y vérifie au clavier.

- **Un pilotage sûr**, au clavier, des applications des projets d'Edouard, installées ou lancées en
  développement (une application Tauri, donc WebView2, ou une autre application Windows qui expose
  l'automatisation) ; aussi un navigateur, un terminal, et KeePassXC pour deux gestes.
- **Lire ce que lit un lecteur d'écran :** le focus ; le type, le nom, la valeur et l'état de chaque
  élément, et l'arbre de la fenêtre ; les raccourcis déclarés (`aria-keyshortcuts`) et le réglage
  d'annonce (`aria-live`).
- **Agir sans souris :** donner le focus, invoquer, cocher ou choisir, faire défiler ; taper du
  texte, accents compris, ou des touches comme au clavier du poste (AZERTY). Le clic à des
  coordonnées est permis, mais l'outil préfère l'accessibilité.
- **Voir :** capturer la fenêtre visée en PNG, jamais l'écran entier.
- **Changer les conditions de l'essai :** la taille de la fenêtre, l'équivalent d'un zoom de 200 %
  sans toucher à l'échelle de Windows ; lancer l'application sur un dossier de données neuf, puis
  l'effacer.
- **Souhaité :** écouter, quelques secondes, les annonces qu'un lecteur d'écran recevrait. Cela
  peut attendre une seconde version si cela double le travail.

**C'est réussi** quand les onze points de la liste de l'essai installé de `pupitre` se rejouent par
l'outil, sur l'établi installé, au lieu du script, avec les mêmes constats.

## 3. Ce qui existe déjà

- Le script de l'essai du 2026-11-21 : la spécification de ce qui marche sur ce poste, pas du code à
  reprendre.
- Claude in Chrome, qui lit une page web de l'intérieur : il reste le premier choix pour une page
  web.
- Le pilotage d'Anthropic, dans l'application de bureau de Claude : il reste disponible à côté.
- L'établi de `pupitre`, installé sur le poste : l'application de l'essai qui fait le critère.
- `modele-projet`, d'où naît le dépôt, avec sa méthode et ses garde-fous ; Rust, déjà sur le poste.

## 4. Méthode : la phase 0

*Une phase de cadrage sans code. Elle produit `docs/cadrage.md` : ce que la lecture change, les questions posées à Edouard, les décisions, le plan. Rien ne se construit avant sa fusion.*

## 5. Plus tard

- L'écoute des annonces, si elle ne tient pas dans le premier lot.
- Une page de savoir au cerveau, par `/noter` puis `/digerer`, qui remplacera la proposition du
  2026-11-21 sur le pilotage par PowerShell.

## 6. Points d'attention

**Les décisions d'Edouard du 2026-11-21**, en deux tours :

1. L'usage : l'essai installé et l'accessibilité des applications de ses projets ; s'y ajoutent les
   navigateurs, les terminaux et deux gestes de KeePassXC. Ses autres applications (Outlook, Word)
   ne sont pas bloquées par l'outil, mais restent hors de l'usage, sauf à sa demande.
2. Rust, un exécutable à sous-commandes.
3. Un dépôt neuf et public, né de `modele-projet`, nommé `pilote`.
4. Les sessions le trouvent par une compétence du poste, `piloter`, qui pointe vers le dépôt, et
   par une page de savoir.
5. Chaque appel nomme sa fenêtre ; rien ne se tape hors du premier plan ; une capture ne prend que
   la fenêtre visée ; un journal garde les gestes, sans le texte tapé.
6. Le clic à des coordonnées est permis ; l'outil préfère l'accessibilité.
7. Une session dédiée crée le dépôt et construit l'outil.
8. Les navigateurs et les terminaux sont permis ; pour une page web, Claude in Chrome d'abord.
9. Dans un terminal, la compétence interdit de taper ce que le garde de Claude Code refuserait, et
   de taper dans une autre session Claude ; l'outil ne le bloque pas.
10. KeePassXC est permis pour deux gestes : ajouter une entrée, le mot de passe créé par son
    générateur et jamais vu ; laisser sa saisie automatique taper un secret, confirmée par Edouard,
    présent. La session ne lit, ne copie et ne tape jamais une valeur, et ne capture pas le coffre
    ouvert. Le déverrouillage reste à Edouard.
11. L'outil ne refuse que les invites sécurisées de Windows et les Paramètres de Windows.
12. Les captures ne prennent que la fenêtre visée.

**Les anti-buts : un outil qui marche, mais un échec.**

- Il tape dans la mauvaise fenêtre, même une fois.
- Une capture montre une autre fenêtre que la fenêtre visée : un courrier, un coffre, un terminal.
- Le journal garde un texte tapé.
- Une valeur secrète passe par une session : rendue par `valeur` ou par `arbre`, capturée, tapée.
- Il devient un moyen de contourner les règles de sécurité des sessions : taper un mot de passe
  réel, valider un paiement, changer un réglage du système, accepter une invite d'administrateur,
  taper dans un terminal une commande que le garde de Claude Code refuserait, donner des ordres à
  une autre session Claude.

**Les non-buts :** remplacer Claude in Chrome ; piloter Outlook ou Word, sauf à la demande
d'Edouard ; toucher à un secret ; reconnaître une image ou lire un texte dans une capture ;
enregistrer et rejouer des scénarios ; tourner sur le serveur, ou hors de Windows 10 et 11 ; écrire
une valeur dans un champ sans taper (`SetValue`).

**Les contraintes du poste :** Windows 10 Pro, un écran de 1366 × 768 à 100 %, un clavier AZERTY,
mais l'outil marche à une autre échelle (sensible au DPI de chaque écran, coordonnées en pixels
physiques de la zone cliente) ; la session n'est pas élevée, et une application élevée ne reçoit
pas ses frappes : l'outil le dit ; les sessions l'appellent depuis PowerShell 5.1 comme depuis Git
Bash, qui abîment un argument commençant par `/` ou portant des guillemets : le texte passe aussi
par un fichier ou l'entrée standard, et les touches ont des noms ; la sortie est en UTF-8 sans BOM ;
le dossier temporaire d'une session est trop long pour WebView2 : un dossier de données neuf prend
un chemin court sous `%TEMP%`.
