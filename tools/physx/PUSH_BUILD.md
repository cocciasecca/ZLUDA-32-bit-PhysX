# Push e build senza assistente

Dal terminale del repository originale:

```sh
python3 tools/physx/push_build.py
```

Lo script trova automaticamente il worktree del ramo `experiment/physx-diagnostics`, verifica origin e autenticazione GitHub, mostra tutte le modifiche non ignorate e chiede conferma prima di includerle nel commit. Poi esegue push normale (mai force), avvia il workflow ZLUDA sul fork corretto, verifica la revisione della build e ne segue l’esito. Nessuna installazione o accesso alla Legion.

`--yes` include automaticamente tutte le modifiche del worktree sperimentale, comprese note e file nuovi non ignorati. `-m "messaggio"` personalizza il commit. `--no-wait` termina dopo l’avvio della build.

Servono Python 3, git, GitHub CLI (`gh`) autenticata e il worktree sperimentale ancora presente. In caso di push rifiutato lo script si ferma, senza fare rebase o modificare la cronologia. Interrompere il monitoraggio non annulla una build già avviata. Il link stampato permette di scaricare gli artifact da GitHub.
