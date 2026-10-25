# Build e installazione correnti: Windows

Per preparare soltanto la build (Legion invariata):

```sh
python3 tools/physx/push_build.py --yes --no-wait
```

Quando la build è riuscita e vuoi installare, con Batman chiuso:

```sh
python3 tools/physx/push_build.py --install-run NUMERO_RUN
```

Il percorso sperimentale compila DLL a 32 bit e server Windows con rinvio delle
distruzioni texture e cache dei range attivi. Non compila LLVM/backend Linux.
I trasferimenti CPU→GPU e GPU→CPU usano nuovamente il percorso originale
con vettori temporanei. Entrambi gli esperimenti sulle copie sono stati rimossi.

Il pacchetto corrente ha configurazione `windows-original-transfers-v1`.
Non riutilizzare i run Linux 40020496411 o 40080854541.

L'installer verifica gioco/server chiusi e crea un backup. Se trova il wrapper
Linux, ripristina `proton.base` come launcher nello stesso aggiornamento di DLL e
impostazioni, e disabilita ZLUDA_NATIVE_BACKEND. Non cambia HIP/COMGR, il prefix
del gioco o i salvataggi. Il materiale Linux resta archiviato, non viene avviato.

Il checkout di sviluppo resta `/private/tmp/zluda-physx-diagnostics`; il launcher
nella cartella originale lo individua automaticamente. Nessun cambio di profilo.

## Branch e pulizia

Lo script continua a usare `experiment/physx-diagnostics`: è lo stesso ramo di
sviluppo, non un nuovo profilo. `master` resta invariato fino a una futura PR.
La pulizia mantiene il comportamento del default `e8dac2e`; rimuove backend Linux
inattivi e installer storici e conserva test, diagnostica e migrazione a Windows.

## Confronto originale / modificato prima della PR

La prossima build produce anche `physx-reference-*`: DLL e server compilati dal
codice upstream intatto `86ec0a9`, la stessa base da cui partono le nostre modifiche.
Il confronto riguarda il ponte 32→64 bit; Proton, HIP, ZLUDA a 64 bit, COMGR e
impostazioni del gioco restano gli stessi. Non è un confronto tra due release
complete di ZLUDA. Non aggiornare la base a metà confronto.

Build una volta, con il comando abituale sopra. Dopo il completamento, usando
sempre lo stesso numero di run e chiudendo Batman prima di ogni cambio:

```sh
# A: ponte upstream originale, ottimizzazioni e diagnostica spente
python3 tools/physx/push_build.py --install-run NUMERO_RUN --comparison reference
# B: nostro ponte, ottimizzazioni attive e diagnostica spenta
python3 tools/physx/push_build.py --install-run NUMERO_RUN --comparison modified
```

Fare A → B → A (meglio ripetere anche B), medesimo salvataggio, posizione e camera,
stesso TDP/risoluzione/impostazioni, alimentazione e limite FPS. In ogni esecuzione
attendere lo stesso tempo per il caricamento iniziale, poi misurare per 60 secondi.
Annotare media e, se disponibile, 1% low; riportare separatamente gli scatti al
primo ingresso. Il secondo A serve a riconoscere cambiamenti di cache/temperatura.

Il controllo gioco chiuso, verifica delle revisioni, backup e installazione restano
automatici. Senza `--comparison`, il comando installa nuovamente il default con
le impostazioni diagnostiche precedenti. Nessun secondo repository necessario.
