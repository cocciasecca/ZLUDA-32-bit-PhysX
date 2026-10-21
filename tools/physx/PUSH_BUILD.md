# Push, build e installazione Legion

Dal terminale del repository:

```sh
python3 tools/physx/push_build.py --yes --install
```

Include tutte le modifiche non ignorate del worktree sperimentale, crea il commit, fa push, riutilizza una build riuscita o in corso dello stesso commit (se disponibile), altrimenti avvia ZLUDA e aspetta l’esito. Poi verifica hash, architetture e revisione di server/proxy e installa nella copia sperimentale con backup. Batman deve essere chiuso; in caso contrario l’installazione si ferma senza chiuderlo. La Legion deve essere raggiungibile via SSH con la chiave configurata.

Senza `--yes` mostra le modifiche e chiede conferma prima del commit. Senza `--install` esegue solo push/build. `--no-wait` non è compatibile con installazione.

Per installare una build già riuscita senza commit o nuove compilazioni:

```sh
python3 tools/physx/push_build.py --install-run 39521873732
```

Opzioni: `--host deck@192.168.10.67`, `--key ~/.ssh/zluda_legion_ed25519`, `-m "messaggio commit"`. Richiede Python 3, git, gh autenticato e SSH. Interrompere il monitoraggio non cancella Actions. Nessun force-push, nessun riavvio del gioco, nessuna modifica a Proton originale o HIP. Una reinstallazione dello stesso run verifica i file già presenti e termina.

La ricerca di build esistenti copre gli ultimi 30 run del workflow per il commit; eseguire una sola istanza dello script alla volta. Gli artifact scaduti richiedono una nuova build da GitHub Actions.

## Profilo predefinito dopo questa indagine

Installa il server control con proxy della stessa revisione: cache degli intervalli ON, rinvio texture OFF, diagnostica OFF. Il guadagno FPS della cache non è dimostrato. Gli esperimenti restano nel codice, ma non sono attivati dall’installer normale. Backup e manifest usano il suffisso `-baseline`.
