# Audit des capacités réelles

Audit réalisé sur cette machine le 14 octobre 2026, avant le choix d'architecture. Les sondes d'exécution sont rapportées à part : une présence de binaire n'est pas une intégration vérifiée.

## Environnement

| Élément | Constat |
| --- | --- |
| Système | Windows 10.0.26200 |
| Python | 3.14.5, `C:\Python314\python.exe` |
| Git | 2.52.0.windows.1 |
| Node / npm | 26.1.0 / 11.13.0 |
| Cursor IDE | 3.24.12, commande `cursor` |
| VS Code | 1.141.1, commande `code` |
| GitHub CLI `gh` | absent |
| Dépôt au départ | vide, hors le fichier d'espace de travail |

## Matrice

| Solution | Confirmé | Non confirmé au moment de l'audit | Limite |
| --- | --- | --- | --- |
| `cursor` (IDE) | Ouvre l'éditeur, le chat graphique, les diffs et les extensions | Aucun mode `--print` ni exécution d'agent | Ce n'est pas un pilotage d'agent |
| CLI `agent` | Documentée par Cursor (`-p`, `--output-format json`, `--trust`, `--workspace`, `--mode ask`, `--force`) | Absente du PATH et des emplacements locaux inspectés. `CURSOR_API_KEY` absente | Inutilisable tant qu'elle n'est pas installée et authentifiée |
| SDK `cursor-sdk` | API Python documentée (`Agent.prompt`, `Agent.create`) | Paquet non installé, clé absente, Python 3.14 non garanti par l'éditeur | Non branché : un agent local peut écrire sans le contrôle d'outils retenu |
| Extension Cursor / MCP | Extensions remote, SSH, CSV, PDF, Xdebug. Aucun serveur MCP utilisateur listé | Le MCP ne lance pas l'agent | MCP n'est pas l'orchestrateur |
| VS Code + Copilot | VS Code est installé | L'extension GitHub Copilot n'est pas installée | L'interface graphique n'est pas pilotée |
| Copilot CLI 1.0.47 | Binaire `copilot`. Aide locale : `-p`, `-s`, `--output-format json`, `--resume`, `--continue`, `--no-ask-user`, `--allow-tool`, `--deny-tool`, `-C`, `--add-dir` | Aucun `GH_TOKEN`, `GITHUB_TOKEN` ni `COPILOT_GITHUB_TOKEN` dans l'environnement. Le dossier `%USERPROFILE%\.copilot` existe, donc une session a déjà été utilisée. Le jeton, s'il existe, est dans le magasin d'identifiants | L'authentification n'est vraie qu'après une sonde. Pas de `--allow-all` |
| Git | `status`, `diff`, `commit`, `worktree add` utilisables avec des arguments séparés |  | `reset --hard`, `clean` et `push --force` sont refusés par l'application |

## Décision

L'orchestrateur parle à des adaptateurs. Le mode simulé est complet et testé. Copilot est piloté par son CLI officiel, invoqué via Node et `npm-loader.js` pour éviter `cmd.exe`. Cursor est piloté par le `node.exe` et le `index.js` livrés avec le CLI `agent`, sans passer par `cmd.exe`. Aucune saisie clavier simulée n'est utilisée.

## Sondes exécutées après implémentation

- Installation officielle du CLI Cursor (`irm https://cursor.com/install?win32=true | iex`) réussie. Le binaire utile est `%LOCALAPPDATA%\cursor-agent\versions\<version>\node.exe` avec `index.js`. `agent status` a répondu : `Not logged in`. Aucune sonde `-p` n'a été lancée, car elle exigerait une session. Étape restante : `agent login` ou `CURSOR_API_KEY`.
- Copilot CLI a été appelé en non-interactif (`-p`, `-s`, `--no-ask-user`, `--output-format json`, `--deny-tool`). Code de sortie 1. Message : `No authentication information found`. Étape restante : `copilot login`, ou un jeton fin `COPILOT_GITHUB_TOKEN` avec la permission Copilot Requests. Les jetons classiques `ghp_` ne sont pas acceptés par ce CLI.
- Le mode simulé ne dépend pas de ces authentifications.
