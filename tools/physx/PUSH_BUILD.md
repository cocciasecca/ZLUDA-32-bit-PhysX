# Build e installazione correnti

```sh
python3 tools/physx/push_build.py --yes --install
```

Un comando: controlli locali, commit, push, build Windows/Linux, download verificato
e installazione sulla Legion. Non occorre scegliere profili. Il checkout attivo resta
`/private/tmp/zluda-physx-diagnostics`; il launcher nella cartella originale rimanda lì.

La configurazione corrente è il backend Linux con trasporto in RAM condivisa,
rinvio delle distruzioni e cache attivi. Serve una nuova build per questa modifica;
il vecchio run 40020496411 contiene ancora il trasporto TCP. La prima build Linux compila anche LLVM e può richiedere parecchio tempo;
le successive riutilizzano la cache. Il server Windows viene conservato per ripristino.

L'installer controlla sempre che Batman e i server siano chiusi. Prima di attivare il
nuovo percorso esegue la DLL a 32 bit e un kernel PTX sul backend Linux in un prefix di
prova separato. Se il test fallisce, la configurazione funzionante resta intatta.
Non vengono cambiati HIP/COMGR, il prefix del gioco o i salvataggi.

Per riprendere soltanto l'installazione di una build già riuscita:

```sh
python3 tools/physx/push_build.py --install-run NUMERO_RUN
```

Il pacchetto deve contenere DLL e backend Linux della stessa revisione. Vecchi artifact
di esperimenti diversi vengono rifiutati. La directory diagnostics contiene gli errori
del preflight. Dopo un'installazione riuscita viene stampato il comando di rollback.
