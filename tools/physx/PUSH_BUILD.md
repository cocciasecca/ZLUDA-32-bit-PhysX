# Build e installazione correnti

```sh
python3 tools/physx/push_build.py --yes --install
```

Il comando verifica Python e diff, esegue i test Rust portabili disponibili
(riutilizza l'esito se sorgenti e compilatore non cambiano), committa le modifiche,
pusha e attende GitHub Actions. La CI esegue anche compilazione e test Windows.
Installa soltanto una build riuscita, verificando revisione, hash e architettura.
Controlla automaticamente che Batman e il server siano chiusi prima del download
e nuovamente prima della sostituzione. Se sono aperti, si ferma senza chiuderli.

Non occorre scegliere un profilo: la configurazione corrente ripristina il rinvio delle distruzioni di `3e9ea16`,
con riuso attraverso i trasferimenti, cache e diagnostica originali attivi.
Il pool e le sue sonde sono rimossi. I vecchi pacchetti sono rifiutati.
Le opzioni storiche `--profile` non sono più necessarie né accettate.

Per riprendere soltanto un'installazione interrotta (per esempio gioco aperto):

```sh
python3 tools/physx/push_build.py --install-run NUMERO_RUN
```

Il checkout attivo esistente è `/private/tmp/zluda-physx-diagnostics`, ramo
`experiment/physx-diagnostics`. Il launcher nella cartella originale lo trova
automaticamente: non crea altri checkout. Modificare il codice nel checkout
attivo; il vecchio master è soltanto la reference storica, non una seconda versione
da aggiornare. Prima di commit vengono elencate tutte le modifiche incluse.

Senza Rust locale i controlli Python/diff vengono comunque eseguiti e i test Rust
restano obbligatori in CI. Nessun risultato locale sostituisce i test Windows.
Il confronto prestazionale della configurazione ripristinata resta da fare sulla GPU.
