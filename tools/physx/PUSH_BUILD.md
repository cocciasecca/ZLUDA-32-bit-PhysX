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
