# Analisi Critica dello Strumento: NodeStract (NS)

Questo documento presenta una valutazione critica del prototipo del linguaggio **NodeStract (NS)**, evidenziandone i punti di forza, le scelte di progettazione, le caratteristiche architetturali e i test effettuati per convalidare il compilatore/interprete.

---

## 1. Punti di Forza e Innovazione Didattica

* **Isolamento e Attivazione Dinamica delle Parole Chiave**: L'idea di caricare e abilitare le parole chiave in base alle lingue importate dimostra in modo eccellente il funzionamento teorico di un analizzatore lessicale flessibile. Consente di osservare empiricamente come un termine acquisisca o perda il suo status di "parola riservata" a seconda della configurazione.
* **Separazione Netta delle Fasi della Pipeline**: L'architettura modulare (Import Validator -> Lexer -> Parser -> AST -> Interpreter) rispetta i pattern classici di progettazione dei compilatori, rendendo la base di codice Rust facile da leggere, estendere e spiegare in un contesto didattico.
* **Scoping Lessicale Rigido**: L'ambiente di esecuzione isola correttamente le variabili locali delle funzioni garantendo lo scoping lessicale ed evitando la visibilità involontaria di variabili appartenenti allo stack dei chiamanti intermedi.
* **Gestione Protetta del File System e della Rete**: Il caricamento controllato tramite i moduli di sistema (`nio`, `nfs`, `nmath`, `nnet`) costringe a rispettare il principio del minor privilegio (least privilege), rendendo le capacità di I/O e di rete sicure e isolate.

---

## 2. Caratteristiche Architetturali e Scelte di Progetto

L'attuale implementazione, focalizzata sulle finalità didattiche del progetto, adotta alcune soluzioni specifiche che ne definiscono il perimetro d'uso:

### 2.1 Tipizzazione Dinamica e Flessibile
* **Descrizione**: NodeStract non richiede la dichiarazione statica dei tipi per le variabili, preferendo un approccio dinamico simile a JavaScript e Python.
* **Valutazione**: Questa scelta semplifica notevolmente la scrittura del codice sorgente da parte degli studenti e snellisce la struttura dell'AST, sebbene sposti i controlli di compatibilità dei tipi interamente a runtime.

### 2.2 Sincronicità delle Operazioni I/O e Network
* **Descrizione**: Tutte le operazioni di rete (richieste HTTP GET/POST) e di lettura/scrittura su File System vengono eseguite in modo bloccante e sincrono.
* **Valutazione**: L'assenza di asincronia (come `async/await` reali o thread concorrenti nell'esecutore) è ottimale per mantenere lineare l'esecuzione e comprensibile il codice dell'interprete, senza introdurre la complessità dei cicli di eventi (event loops).

### 2.3 Serializzazione e Deserializzazione JSON Automatica
* **Descrizione**: La libreria `nfs` integra un supporto nativo che rileva l'estensione `.json` ed esegue il parsing o la formattazione dei dati in modo trasparente.
* **Valutazione**: Consente la persistenza e lo scambio strutturato dei dati a runtime senza costringere a implementare parser o serializzatori personalizzati nel codice sorgente.

---

## 3. Test Effettuati e Validazione dello Strumento

Il prototipo è stato validato attraverso una suite di test strutturata:

### 3.1 Suite di Test Automatizzati (Unit Test)
Il codice sorgente Rust include unit test interni per verificare i moduli critici:
* **Normalizzazione Lessicale**: Verifica del funzionamento di `normalize` in `translate.rs` con caratteri accentati ed estesi (es. `SÉ` -> `se`).
* **Importazione Gerarchica**: Controllo dell'efficacia di `ImportManager` nella gestione dei permessi (es. verifica che l'importazione di una singola funzione escluda le altre dello stesso modulo).
* **Integrità del Conteggio Righe**: Verifica che la rimozione degli import non alteri la corrispondenza dei numeri di riga per la segnalazione degli errori.

Per eseguire i test automatizzati Rust:
```bash
cargo test
```

### 3.2 Test di Integrazione e Categorie di Test
La suite di test di integrazione (`examples/test/`) copre oltre 50 scenari divisi per aree tematiche:
1. **languages**: Verifica l'attivazione dei singoli vocabolari e le importazioni multiple.
2. **typing**: Controllo della corretta gestione di costanti, riassegnazioni protette, tipi booleani, stringhe e null.
3. **data**: Operazioni su array, range, mappe e coercizione automatica dei tipi.
4. **net**: Simulazione di richieste GET e POST su server locale e gestione dei tentativi di riconnessione (retry).
5. **function**: Test di ricorsione, funzioni nidificate, scoping e chiamate dinamiche.
6. **file**: Suite di 10 test che convalidano la scrittura, lettura, sovrascrittura ed eliminazione sicura di file `.txt` e `.json`.

Per eseguire la test suite completa di integrazione:
```bash
cargo run --example test_suite
```
