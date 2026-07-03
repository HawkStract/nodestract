# Analisi Critica dello Strumento: NodeStract (NS)

Questo documento presenta un'analisi critica del prototipo di **NodeStract (NS)**, evidenziandone i punti di forza, i limiti strutturali attuali e i test effettuati per convalidare il funzionamento del compilatore/interprete.

---

## 1. Punti di Forza e Innovazione Didattica

* **Isolamento delle Parole Chiave**: L'idea di caricare dinamicamente le parole chiave in base alle lingue importate dimostra in modo eccellente il funzionamento teorico di un analizzatore lessicale flessibile. Gli studenti possono constatare empiricamente come un termine acquisisca o perda il suo status di "parola riservata" a seconda del contesto di configurazione.
* **Separazione Netta delle Fasi**: L'architettura modulare (Lexer -> Parser -> AST -> Interpreter) rispetta rigorosamente i pattern canonici di progettazione dei compilatori, rendendo la base di codice Rust facile da leggere, estendere e spiegare in un contesto accademico.
* **Robustezza dei Pre-controlli**: I controlli sul bilanciamento preventivo dei delimitatori e l'impossibilità di usare parole chiave come identificatori riducono drasticamente i crash dell'interprete a runtime, intercettando gli errori logici comuni già nella fase di compilazione iniziale.

---

## 2. Limitazioni Strutturali e Aree di Miglioramento

Trattandosi di un prototipo didattico (Tipologia 2), l'attuale implementazione presenta alcune limitazioni di rilievo che influiscono sulla correttezza formale del linguaggio. Questi punti sono documentati anche nel file di sviluppo [`NextStep.md`](file:///w:/University/3o%20anno/ICDD/Esame/Progetto/NodeStract/NextStep.md) per future correzioni:

### 2.1 Modello di Scoping Dinamico (Anomalia di Esecuzione)
* **Descrizione**: L'interprete gestisce gli ambienti delle funzioni inserendo lo scope locale sopra lo stack di quelli correnti. Questo fa sì che le variabili locali della funzione chiamante siano visibili alla funzione chiamata (scoping dinamico).
* **Impatto**: Il comportamento devia dallo standard dei moderni linguaggi a cui NodeStract si ispira (JavaScript/Python), i quali adottano lo scoping lessicale.

### 2.2 Omissione di Funzioni Standard della Specifica
* **Descrizione**: Alcune funzioni built-in indicate nella documentazione dei requisiti del progetto (`spec.md`) e inserite nei dizionari di traduzione, non sono state collegate nel codice dell'interprete:
  * `open` (apri) e `delete` (elimina) per il filesystem (`nfs`).
  * `connect` (connetti) e `receive` (ricevi) per la rete (`nnet`).
* **Impatto**: Gli studenti o gli utenti che provano a utilizzare queste funzioni riscontreranno errori di compilazione/esecuzione, nonostante la documentazione ufficiale le indichi come presenti.

### 2.3 Perdita del Tracciamento Corretto delle Righe negli Errori
* **Descrizione**: La rimozione fisica delle righe contenenti gli `import` all'inizio del file (eseguita da `check.rs`) fa sì che le righe successive vengano spostate verso l'alto nel codice passato al compilatore.
* **Impatto**: Di conseguenza, se viene rilevato un errore sintattico alla riga 10 del file originale, il compilatore potrebbe segnalarlo erroneamente alla riga 8, rendendo difficile il debug per l'utente finale.

### 2.4 Assenza di Controllo Semantico su Istruzioni di Loop
* **Descrizione**: Le istruzioni `break` e `continue` sono interpretate sintatticamente ovunque nel codice.
* **Impatto**: Se inserite fuori da un ciclo `mentre`/`per` (ad esempio in cima a una funzione), l'interprete non genera un errore di compilazione, ma altera i flag interni, causando comportamenti imprevedibili all'uscita delle chiamate.

---

## 3. Test Effettuati e Validazione dello Strumento

Il prototipo è stato convalidato attraverso due modalità di test:

### 3.1 Suite di Test Automatizzati (Unit Test)
Il codice sorgente Rust include una suite di unit test integrata per verificare la correttezza del comportamento dei singoli moduli isolati.
* **Test di Normalizzazione**: Validazione del corretto funzionamento di `normalize` in `translate.rs` con caratteri accentati ed estesi (es. verifica che `SÉ` diventi correttamente `se`).
* **Test di Importazione Gerarchica**: Validazione dell'efficacia di `ImportManager` nella gestione dei permessi (es. verifica che l'importazione di una singola funzione come `sin` non consenta l'utilizzo di `cos`, e che le importazioni wildcard `*` funzionino come previsto).

Per eseguire i test automatizzati della suite Rust:
```bash
cargo test
```

### 3.2 Test di Integrazione e Script di Esempio
Sono stati scritti ed eseguiti diversi file di esempio (situati in `examples/`) per verificare il comportamento dell'interprete su script completi:
1. **`examples/it.ns`**: Verifica il funzionamento della sintassi di base italiana (dichiarazione di variabili, costanti, commenti su più righe e funzioni di I/O). Ha permesso di individuare l'errore di assegnazione a costanti (`PI_GRECO`).
2. **`examples/it_test.ns`**: Script complesso che testa cicli annidati, funzioni ricorsive (fattoriale), logica booleana complessa, array, dizionari e accesso a file system locale.
3. **`examples/lessons/`**: Una serie di mini-lezioni didattiche progressive ideate per guidare l'utente finale nell'apprendimento passo-passo dei costrutti.
