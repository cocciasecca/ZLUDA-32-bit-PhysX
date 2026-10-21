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

## Confronto con la stessa build

Profili: `baseline` = control e log OFF; `diagnostic` = server diagnostico, log ON, pre-sync OFF; `unbind-probe` = stesso server diagnostico, log ON e pre-sync ON. Cache intervalli ON, texture deferral OFF in tutti i profili. Il cambio profilo crea un backup distinto; log nuovo a ogni avvio. Ripetere un profilo dopo un altro preserva anche i backup precedenti.

Prossima prova, Batman chiuso, senza ricompilare:

```sh
python3 tools/physx/push_build.py --install-run 39524047539 --profile diagnostic
```

Per le future modifiche al codice, push/build/install:

```sh
python3 tools/physx/push_build.py --yes --install --profile diagnostic
```

Confrontare stessa scena, impostazioni e riscaldamento; FPS da soli non bastano. Probe ON è una perturbazione, non un'ottimizzazione. Lasciare all'utente push/build/install come concordato.

## Sonda global texture

`--profile texture-global` attiva fino a otto letture di confronto host/device, con diagnostica ON e pre-sync OFF. Nessuna scrittura GPU o pool. Questa prova verifica accessibilità/layout, non gli FPS. Comando: `python3 tools/physx/push_build.py --yes --install --profile texture-global`.

## Raccolta unica per il pool

`python3 tools/physx/push_build.py --yes --install --profile pool-capture` prepara la raccolta unica. Dopo l’avvio attendi nella scena e scrivi «ci sono»: la cattura viene armata tramite il file `diagnostics/pool-capture.start` sulla Legion, senza ricompilare. Durata60secondi, log LRU e sequenza limitati; non attiva un pool né modifica le chiamate GPU.

## Prototipo pool reale

`python3 tools/physx/push_build.py --yes --install --profile texture-pool` attiva il pool sperimentale64elementi; le sonde e la simulazione sono spente. Verifica effetti PhysX e contatori prima di attribuire variazioni FPS. `--install-run RUN --profile diagnostic` usa la stessa build con pool spento.
