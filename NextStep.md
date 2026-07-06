# Prossimi Passi (NextStep) - NodeStract

Questo documento raccoglie tutte le anomalie, i bug, le omissioni rispetto alle specifiche e le debolezze architetturali riscontrate durante l'analisi statica e dinamica del codice di NodeStract. Trattandosi di un progetto a scopo didattico, non è necessario che sia pronto per la produzione di massa, ma è fondamentale che le funzionalità presenti siano corrette e coerenti in vista della consegna.

---

## 1. Difetti Gravi e Incoerenze di Esecuzione (Runtime / Semantica)

---

## 2. Omissioni rispetto alle Specifiche (`spec.md`)

### 2.1 Funzioni Built-in Dichiarate ma Non Implementate
Le seguenti funzioni sono definite in `spec.md` (e mappate nei dizionari di traduzione come `it.json`), ma sono **completamente assenti** nel motore di esecuzione (`src/interpreter/functions.rs`):
* **Libreria `nfs` (File System)**:
  * `open` (apri): Dichiarata nelle specifiche ma non gestita dall'interprete.
  * `delete` (elimina): Dichiarata nelle specifiche ma non gestita dall'interprete.
* **Libreria `nnet` (Network)**:
  * `connect` (connetti): Dichiarata ma non implementata.
  * `receive` (ricevi): Dichiarata ma non implementata.

* **Soluzione consigliata**: Implementare queste funzioni in `src/interpreter/fs.rs` e `src/interpreter/net.rs`, registrandole poi nel blocco di match di `handle_function_call`.


