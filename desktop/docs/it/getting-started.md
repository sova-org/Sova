# Per iniziare con Sova

Sova è un sequencer di live coding. Scrivi codice che genera eventi musicali
in tempo reale — note, control change, messaggi OSC — e Sova li riproduce
su una timeline condivisa.

## Concetti

- **Scena** — il contenitore principale. Una scena contiene linee che vengono eseguite in parallelo.
- **Linea** — una sequenza di eventi temporizzati, scritta in uno dei linguaggi di Sova.
- **Cella** — un passo nella griglia temporale. Ogni cella ha una durata (in beat) e un numero di ripetizioni.
- **Dispositivo** — una porta MIDI, un endpoint OSC, o un'uscita audio che riceve gli eventi.

## Scrivi la tua prima sequenza

1. Connettiti al server Sova (o avvia il server integrato).
2. Seleziona una linea nella griglia della scena.
3. Scegli un linguaggio (Bob, Boinx, Cagire o BaLi).
4. Scrivi un breve programma e premi **Invio** per valutare.

La linea inizia a produrre eventi immediatamente.

## Modalità di esecuzione

- **Free** — ogni linea si ripete indipendentemente al proprio ritmo.
- **AtQuantum** — le linee si risincronizzano al confine del quantum globale.
- **LongestLine** — tutte le linee attendono la più lunga prima di ripartire.
