# NodeStract (NS)

Benvenuto in **NodeStract**, un linguaggio di programmazione didattico, procedurale e funzionale a tipizzazione dinamica, progettato come prototipo nell'ambito del progetto d'esame universitario (Tipologia 2). 

La caratteristica distintiva di NodeStract è il supporto dinamico e concorrente a **7 lingue diverse** (`inglese`, `italiano`, `spagnolo`, `francese`, `tedesco`, `portoghese`, `rumeno`). Le parole chiave (keywords) di una lingua e le funzioni di sistema diventano disponibili all'interno del programma solo se esplicitamente importate in testa al sorgente.

---

## Struttura della Documentazione

Tutta la documentazione dettagliata del progetto è organizzata nella cartella `docs/`. Di seguito trovi i link diretti ai documenti principali:

1. [**Manuale d'Uso (docs/ManualeUso.md)**](file:///w:/University/3o%20anno/ICDD/Esame/Progetto/NodeStract/docs/ManualeUso.md)
   Una guida dettagliata per l'utente che spiega l'installazione del compilatore, la sintassi del linguaggio, la gestione del vocabolario multi-lingua e l'utilizzo dei moduli standard (`nio`, `nfs`, `nmath`, `nnet`).
   
2. [**Documentazione Tecnica (docs/DocumentazioneTecnica.md)**](file:///w:/University/3o%20anno/ICDD/Esame/Progetto/NodeStract/docs/DocumentazioneTecnica.md)
   L'analisi dell'architettura interna dell'interprete scritto in Rust. Spiega nel dettaglio le fasi di importazione, lexing (analisi lessicale), parsing (analisi sintattica), AST (Abstract Syntax Tree) e l'esecuzione del codice.
   
3. [**Analisi Critica (docs/AnalisiCritica.md)**](file:///w:/University/3o%20anno/ICDD/Esame/Progetto/NodeStract/docs/AnalisiCritica.md)
   Un'analisi onesta dei punti di forza, dei limiti strutturali dell'interprete attuale, delle scelte progettuali e delle sessioni di test effettuate per validarne il funzionamento.

---

## Analisi Strutturale ed Errori Rilevati

Se sei un manutentore o desideri consultare le note di sviluppo relative ai bug o alle incoerenze logiche e architetturali riscontrate nell'attuale codice sorgente, fai riferimento al file di sviluppo nella root del progetto:
* [**NextStep.md**](file:///w:/University/3o%20anno/ICDD/Esame/Progetto/NodeStract/NextStep.md)

---

## Avvio Rapido

Il compilatore/interprete di NodeStract è scritto in Rust ed è eseguibile tramite Cargo.

### Prerequisiti
* [Rust e Cargo](https://rustup.rs/) installati sul sistema.

### Comandi Principali
Dalla cartella principale del progetto, puoi eseguire i seguenti comandi:

* **Eseguire un sorgente NodeStract (`.ns`):**
  ```bash
  cargo run -- build <percorso_file.ns>
  ```
  *Esempio:*
  ```bash
  cargo run -- build examples/it.ns
  ```

* **Visualizzare la versione del compilatore:**
  ```bash
  cargo run -- version
  ```
