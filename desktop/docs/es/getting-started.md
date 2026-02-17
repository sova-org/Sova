# Primeros pasos con Sova

Sova es un secuenciador de live coding. Escribes código que genera eventos
musicales en tiempo real — notas, controles, mensajes OSC — y Sova los
reproduce en una línea de tiempo compartida.

## Conceptos

- **Escena** — el contenedor principal. Una escena contiene líneas que se ejecutan en paralelo.
- **Línea** — una secuencia de eventos temporizados, escrita en uno de los lenguajes de Sova.
- **Celda** — un paso en la cuadrícula temporal. Cada celda tiene una duración (en beats) y un número de repeticiones.
- **Dispositivo** — un puerto MIDI, un punto de acceso OSC, o una salida de audio que recibe los eventos.

## Escribe tu primera secuencia

1. Conéctate al servidor Sova (o inicia el servidor integrado).
2. Selecciona una línea en la cuadrícula de escena.
3. Elige un lenguaje (Bob, Boinx, Forth o BaLi).
4. Escribe un programa corto y pulsa **Enter** para evaluar.

La línea comienza a producir eventos inmediatamente.

## Modos de ejecución

- **Free** — cada línea se repite independientemente a su propio ritmo.
- **AtQuantum** — las líneas se resincronizan en el límite del quantum global.
- **LongestLine** — todas las líneas esperan a la más larga antes de reiniciarse.
