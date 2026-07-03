# Risultati della Test Suite di NodeStract (NS)

Questo documento riassume l'esito dell'esecuzione della suite di test automatizzata e fornisce indicazioni dettagliate su quali anomalie interne (avvisi, errori di tipo a runtime) si sono verificate, perché si verificano e su cosa lavorare per correggerle prima della consegna.

---

## 1. Esito dell'Esecuzione

Tutti i **43 test** suddivisi nelle 8 categorie sono stati eseguiti con successo dal punto di vista logico del test runner (ossia, le asserzioni scritte all'interno degli script `.ns` sul comportamento atteso sono state tutte verificate e soddisfatte, ed il compilatore/interprete non è andato in crash).

### Riepilogo Numerico
* **Test Totali**: 43
* **Superati**: 43
* **Falliti**: 0

---

## 2. Anomalie Rilevate e Su Cosa Lavorare

Nonostante i test abbiano superato le asserzioni di controllo, l'analisi dell'output a basso livello (stdout/stderr dei singoli processi) rivela comportamenti e messaggi diagnostici che denotano problemi architetturali e logici del motore di esecuzione. 

Di seguito vengono analizzati i singoli comportamenti anomali rilevati e le relative cause.

### 2.1 Assegnazione a Costante non Bloccante
* **Test Coinvolto**: `examples/test/typing/5_constants.ns`
* **Errore Stampato**: 
  ```text
  Runtime Error: Cannot assign to lock (constant) 'COSTANTE'.
  ```
* **Cosa succede e perché**: Quando l'interprete esegue un'istruzione di assegnazione su una variabile dichiarata tramite `fissa` (`const`), la funzione `set_var` in `interpreter.rs` rileva che la variabile non è mutabile, stampa il messaggio di errore a terminale e ritorna (`return;`). Tuttavia, l'interprete **non interrompe** l'esecuzione dello script. Il flusso prosegue con il vecchio valore.
* **Su cosa lavorare**: L'interprete dovrebbe sollevare un errore fatale o salvare lo stato di errore nel flag `exception` per bloccare l'esecuzione dello script immediatamente o propagare l'eccezione, anziché limitarsi a un semplice `println`.

### 2.2 Errori Critici sui Tipi Silenti (Coercizione Invalida)
* **Test Coinvolti**: 
  * `examples/test/data/5_coercion_mul.ns` (`1 * "pesci"`)
  * `examples/test/function/3_wrong_types.ns` (`"cane" * 2`)
  * `examples/test/function/4_return_mismatch.ns` (`"Luca" * 3`)
* **Errore Stampato**:
  ```text
  CRITICAL TYPE ERROR: Incompatible types for '*': Integer(1) and String("pesci")
  ```
* **Cosa succede e perché**: Nel file `src/interpreter/ops.rs`, le operazioni binarie tra tipi incompatibili (come la moltiplicazione tra stringhe e interi) ricadono nel blocco catch-all `(l, r) => match operator`. Questo blocco stampa un messaggio di errore a console e restituisce `Value::Null`. Di conseguenza lo script continua a girare trattando il risultato come valore nullo.
* **Su cosa lavorare**: Nel design di un linguaggio didattico robusto, le operazioni tra tipi incompatibili dovrebbero generare un'eccezione a runtime che interrompe l'esecuzione del codice (o viene intercettata da un eventuale blocco `try/catch`). Stampare un messaggio a schermo senza interrompere l'esecuzione rende il comportamento del linguaggio imprevedibile.

### 2.3 Gestione dei Parametri Mancanti nelle Funzioni
* **Test Coinvolto**: `examples/test/function/5_arg_mismatch.ns`
* **Errore Stampato**:
  ```text
  CRITICAL TYPE ERROR: Incompatible types for '+': Integer(30) and Null
  ```
* **Cosa succede e perché**: La funzione `somma_tre(a, b, c)` si aspetta tre parametri, ma viene chiamata con `somma_tre(10, 20)`. In `functions.rs`, la chiamata assegna `Value::Null` a `c`. L'espressione all'interno della funzione esegue `a + b + c` (ovvero `(10 + 20) + nullo`). L'operatore `+` tra l'intero `30` e il tipo `Null` genera un errore di tipo critico e restituisce `nullo`.
* **Su cosa lavorare**: L'interprete dovrebbe convalidare il numero di argomenti prima di avviare l'esecuzione della funzione (arity check) e lanciare un errore di sintassi/runtime se i parametri passati non coincidono con quelli dichiarati nella firma della funzione.

### 2.4 Correttezza della Funzione `stampa` (`print`)
* **Test Coinvolti**: Tutti i test che stampano parametri multipli intervallati da virgole o punteggiatura.
* **Cosa succede e perché**: In `functions.rs`, la logica deputata a inserire spazi tra gli argomenti di `print` è basata su euristiche (ad esempio, controlla se l'argomento successivo inizia con punteggiatura per omettere lo spazio). Questa euristica è fragile e porta a spaziature incoerenti o doppie spaziature quando l'utente passa stringhe già formattate.
* **Su cosa lavorare**: Sostituire l'euristica con un comportamento semplice ed esplicito (ad esempio, unire sempre gli argomenti con un singolo spazio fisso, in stile Python o JavaScript, lasciando all'utente la responsabilità di formattare le stringhe come desidera).
