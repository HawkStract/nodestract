# Manuale d'Uso: NodeStract (NS)

Questo manuale descrive la sintassi, le caratteristiche e le modalità di utilizzo del linguaggio didattico **NodeStract (NS)**. Il linguaggio è progettato per dimostrare come sia possibile gestire dinamicamente il vocabolario delle parole chiave e l'accesso alle risorse di sistema tramite un meccanismo controllato di importazione.

---

## 1. Installazione e Requisiti

Il compilatore ed interprete di NodeStract è implementato in Rust. Per poterlo utilizzare, occorre disporre di:
* **Rust** (versione 1.70 o superiore consigliata) e il toolchain **Cargo**. Puoi installarli da [rustup.rs](https://rustup.rs/).

### Compilazione del Progetto
Per compilare il compilatore (chiamato `nsc` o `ns`), posizionati nella cartella principale del progetto ed esegui:
```bash
cargo build --release
```
L'eseguibile compilato sarà disponibile all'interno della cartella `target/release/`.

---

## 2. Concetto Chiave: Attivazione Dinamica dei Vocabolari

A differenza dei linguaggi tradizionali che possiedono un elenco rigido di parole chiave riservate, NodeStract avvia la lettura del file in uno stato di **bootstrap minimale**.
* Le uniche parole chiave attive all'inizio sono `import` (e le sue traduzioni come `importa`) e `from` (e `da`).
* Per poter scrivere codice, è obbligatorio importare almeno una lingua dal modulo fittizio `translate`.
* Se si tenta di scrivere del codice senza prima importare una lingua, il compilatore rifiuterà l'esecuzione segnalando un errore.

### Esempio di Bootstrap
```ns
importa italiano da translate
// Ora tutte le parole chiave della grammatica italiana (crea, fissa, se, mentre, ecc.) sono attive!
```

Se una lingua non viene importata, i suoi termini non sono riservati. Ad esempio, se non importi l'inglese, puoi tranquillamente creare una variabile di nome `let` o `if`. Se invece importi l'inglese, l'uso di `let` o `if` come identificatori genererà un errore di sintassi.

---

## 3. Sintassi del Linguaggio (Grammatica Italiana)

Di seguito viene illustrata la sintassi utilizzando il vocabolario **italiano**.

### 3.1 Variabili e Costanti
Le variabili si dichiarano con `crea`, mentre le costanti (valori non riassegnabili) si dichiarano con `fissa`.

```ns
crea raggio = 15          // Variabile modificabile
fissa PI_GRECO = 3.14159  // Costante protetta (non modificabile)

raggio = 10               // Consentito
// PI_GRECO = 3.14        // Genera un errore di runtime (riassegnazione di costante)
```

### 3.2 Tipi di Dato Supportati
NodeStract è a tipizzazione dinamica. I tipi supportati a runtime sono:
* **Null**: Rappresentato da `nullo` (o `null`).
* **Booleani**: `vero` e `falso` (o `true` e `false`).
* **Numeri**: Interi e decimali a virgola mobile a 64 bit (es. `42`, `3.14`).
* **Stringhe**: Racchiuse tra doppie virgolette (es. `"Ciao Mondo"`). Supportano le sequenze di escape comuni come `\n` e `\t`.
* **Array (Vettori)**: Dichiarati con parentesi quadre (es. `[1, 2, 3]`).
* **Map (Dizionari)**: Coppie chiave-valore racchiuse tra graffe (es. `{ "nome": "Mario", "eta": 21 }`).

### 3.3 Strutture di Controllo
#### Istruzione Condizionale (`se` / `altrimenti`)
Risolve bivi logici. Le parentesi tonde intorno alla condizione sono facoltative.
```ns
se (voto >= 18) {
    stampa("Esame superato")
} altrimenti se (voto == 17) {
    stampa("Ammesso all'orale con riserva")
} altrimenti {
    stampa("Bocciato")
}
```

#### Selezione Multipla (`scelta` / `caso` / `predefinito`)
Corrisponde allo `switch` di JavaScript o C.
```ns
scelta (colore) {
    caso "rosso":
        stampa("Alt")
    caso "verde":
        stampa("Avanti")
    predefinito:
        stampa("Colore non riconosciuto")
}
```

### 3.4 Cicli ed Iterazioni
#### Ciclo Condizionale (`mentre`)
Esegue un blocco di istruzioni finché la condizione è vera.
```ns
crea contatore = 0
mentre (contatore < 5) {
    stampa("Giro numero:", contatore)
    contatore = contatore + 1
}
```

#### Ciclo a Intervallo (`per` / `in`)
Scorre un intervallo di interi definito dall'operatore `..`.
```ns
// Esegue il ciclo per i valori da 0 a 4 inclusi (5 escluso)
per i in 0 .. 5 {
    stampa("Valore corrente:", i)
}
```

### 3.5 Funzioni (`funzione`)
Le funzioni sono dichiarate con la parola chiave `funzione` e possono restituire valori tramite `ritorna`.
```ns
funzione calcola_area(raggio) {
    crea area = 3.14159 * raggio * raggio
    ritorna area
}

crea ris = calcola_area(5)
stampa("L'area è:", ris)
```

---

## 4. Moduli e Funzioni Built-in (Librerie)

Per proteggere l'ambiente di esecuzione, le funzioni di base (come la stampa a schermo o il calcolo matematico) non sono accessibili globalmente di default. Devono essere importate dal loro rispettivo modulo di sistema.

### 4.1 Input/Output: Modulo `nio`
Fornisce l'accesso alla console.
* **`stampa` (canonical: `print`)**: Stampa a schermo uno o più argomenti.
* **`inserisci` (canonical: `input`)**: Legge una stringa inserita dall'utente da terminale (accetta un messaggio di prompt opzionale).

```ns
importa stampa, inserisci da nio

crea nome = inserisci("Come ti chiami? ")
stampa("Ciao", nome)
```

### 4.2 File System: Modulo `nfs`
Consente l'interazione con i file locali (limitata a file `.txt` e `.json` per motivi di sicurezza).
* **`leggi` (canonical: `read`)**: Legge il contenuto testuale di un file.
* **`scrivi` (canonical: `write`)**: Scrive o sovrascrive un file con il testo fornito.
* **`elimina` (canonical: `delete`)**: Elimina il file specificato dal disco.

```ns
importa * da nfs
scrivi("log.txt", "Esecuzione completata con successo")
crea dati = leggi("log.txt")
```

### 4.3 Matematica: Modulo `nmath`
Funzioni trigonometriche e aritmetiche.
* **`sen` (sin)**, **`cos` (cos)**, **`radq` (sqrt)**, **`casuale` (random)**, **`arrotonda` (round)**, **`minimo` (min)**, **`massimo` (max)**, **`assoluto` (abs)**, **`logaritmo` (log)**, **`potenza` (pow)**.

```ns
importa * da nmath
crea valore = radq(64) // restituisce 8
```

### 4.4 Rete: Modulo `nnet`
Consente di effettuare richieste HTTP.
* **`richiedi` (canonical: `fetch`)**: Esegue una richiesta HTTP GET e restituisce il corpo come stringa.
* **`invia` (canonical: `send`)**: Esegue una richiesta HTTP POST inviando un payload JSON.

---

## 5. Esempio Completo di Esecuzione

Crea un file chiamato `test.ns` con il seguente codice:

```ns
importa italiano da translate
importa * da nio
importa * da nmath

stampa("=== Calcolatore Didattico ===")
crea n = 9
crea radice = radq(n)
stampa("La radice quadrata di", n, "è", radice)
```

Esegui il file con il comando:
```bash
cargo run -- build test.ns
```
L'output a schermo mostrerà il risultato dei calcoli.
